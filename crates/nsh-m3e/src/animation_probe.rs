//! The animation measurements of ADR 0010, shared by the tests of every animated component.
//!
//! A component runs on NSH's headless engine (`nsh_testing::Headless`) with a manual clock, and a [`Recording`]
//! keeps every mount item it sends. Its animation operations are then replayed in [`Replay`]: NSH's own animation
//! graph (`nsh_core::animation::Graph`, the one every NSH platform plays animations with), ticked at [`REPLAY_HZ`] on
//! a clock of its own. No wall-clock time is involved, so a measurement gives the same numbers on every run and
//! every machine.
//!
//! [`spring_position`] is the closed-form solution a replayed spring is compared with.

use std::sync::{Arc, Mutex, PoisonError};

use nsh_core::MountItem;
use nsh_core::animation::{AnimationOp, AnimationValue, Driver, Finished, Graph, NodeId};
use nsh_testing::Headless;

/// The rate the replay samples at: the reference devices' display, 120 Hz (`docs/app-requirements.md`, Motion).
pub(crate) const REPLAY_HZ: f64 = 120.0;

/// Every mount item an app sends from the moment the recording is attached, with the step it came in.
pub(crate) struct Recording(Arc<Mutex<Vec<(String, MountItem)>>>);

impl Recording {
    /// Starts recording `app`'s batches. Attach it before the first step, so the values the component creates are
    /// recorded too.
    pub(crate) fn attach(app: &mut Headless) -> Self {
        let items: Arc<Mutex<Vec<(String, MountItem)>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |step, batch| {
            let mut sink = sink.lock().unwrap_or_else(PoisonError::into_inner);
            sink.extend(
                batch
                    .items
                    .iter()
                    .map(|item| (step.to_owned(), item.clone())),
            );
        });
        Self(items)
    }

    /// Every item recorded so far, in order.
    pub(crate) fn items(&self) -> Vec<MountItem> {
        self.lock().iter().map(|(_, item)| item.clone()).collect()
    }

    /// The items that came in step `step` (`Headless::step` names a step; `press(Go)` is the step of a press).
    pub(crate) fn in_step(&self, step: &str) -> Vec<MountItem> {
        self.lock()
            .iter()
            .filter(|(name, _)| name == step)
            .map(|(_, item)| item.clone())
            .collect()
    }

    /// Runs `action` and returns what it gave back with the items the engine sent meanwhile, whatever step they came
    /// in. Asking a step by its name for "nothing was sent" is not safe: `Headless::run_for` renames the current
    /// step to `clock.runFor(<ms>)`, so a step named before it stays empty whatever the engine sends.
    pub(crate) fn during<R>(&self, action: impl FnOnce() -> R) -> (R, Vec<MountItem>) {
        let before = self.lock().len();
        let result = action();
        let sent = self
            .lock()
            .iter()
            .skip(before)
            .map(|(_, item)| item.clone())
            .collect();
        (result, sent)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<(String, MountItem)>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// How many of `items` are layout work: a view's frame or a text set again.
pub(crate) fn layout_items(items: &[MountItem]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, MountItem::Frame { .. } | MountItem::Text { .. }))
        .count()
}

/// A spring an animation operation started, as the engine sent it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StartedSpring {
    /// The animated value.
    pub(crate) node: NodeId,
    /// Where it goes.
    pub(crate) to: f32,
    /// `k`.
    pub(crate) stiffness: f32,
    /// `c`, the damping coefficient.
    pub(crate) damping: f32,
    /// `m`.
    pub(crate) mass: f32,
    /// The velocity it starts with; `None` keeps the current one.
    pub(crate) velocity: Option<f32>,
    /// The highest frame rate it asks for; `None` is the screen's highest.
    pub(crate) frame_rate: Option<f32>,
}

/// The springs `items` start, in order.
pub(crate) fn started_springs(items: &[MountItem]) -> Vec<StartedSpring> {
    items
        .iter()
        .filter_map(|item| match item {
            MountItem::Animation(AnimationOp::Start {
                node,
                driver:
                    Driver::Spring {
                        to,
                        stiffness,
                        damping,
                        mass,
                        velocity,
                        frame_rate,
                    },
                ..
            }) => Some(StartedSpring {
                node: *node,
                to: *to,
                stiffness: *stiffness,
                damping: *damping,
                mass: *mass,
                velocity: *velocity,
                frame_rate: *frame_rate,
            }),
            _ => None,
        })
        .collect()
}

/// NSH's animation graph, fed recorded operations and ticked at [`REPLAY_HZ`] on a clock of its own.
#[derive(Default)]
pub(crate) struct Replay {
    graph: Graph,
    finished: Vec<Finished>,
    /// The graph's clock, in seconds.
    now: f64,
}

impl Replay {
    /// Feeds the graph the animation operations among `items`, in order, at the current time.
    pub(crate) fn apply(&mut self, items: &[MountItem]) {
        for item in items {
            if let MountItem::Animation(op) = item {
                self.graph.apply(op.clone(), &mut self.finished);
            }
        }
    }

    /// Ticks the graph `frames` times, one [`REPLAY_HZ`] frame apart, the first at the current time, and returns
    /// `node`'s value after each tick. An animation applied just before starts at the first of these ticks, so
    /// sample `i` is `i / REPLAY_HZ` seconds into it. The clock ends one frame after the last tick.
    pub(crate) fn run(&mut self, node: NodeId, frames: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frames);
        for _ in 0..frames {
            self.graph.tick(self.now, &mut self.finished);
            samples.push(self.graph.value(node).unwrap_or(f32::NAN));
            self.now += 1.0 / REPLAY_HZ;
        }
        samples
    }

    /// The value of `node` now, whether a value or a node computed from values (an interpolation, a sum): what a
    /// canvas slot bound to it draws with.
    pub(crate) fn value(&self, node: NodeId) -> f32 {
        self.graph.eval(node).number()
    }

    /// The color of `node` now, `0xRRGGBBAA`: what a canvas paint bound to an animated value of colors draws with.
    /// `None` when the node holds a number.
    pub(crate) fn color(&self, node: NodeId) -> Option<u32> {
        match self.graph.eval(node) {
            AnimationValue::Color(color) => Some(color),
            AnimationValue::Number(_) => None,
        }
    }

    /// True while an animation runs in the graph.
    pub(crate) fn is_animating(&self) -> bool {
        self.graph.is_animating()
    }
}

/// Where a spring is `t` seconds after it starts at `from` with velocity `velocity`, going to `to`: the closed-form
/// solution of `m·x″ + c·x′ + k·(x − to) = 0`, underdamped, critically damped or overdamped.
pub(crate) fn spring_position(
    from: f64,
    to: f64,
    velocity: f64,
    (stiffness, damping, mass): (f64, f64, f64),
    t: f64,
) -> f64 {
    let omega = (stiffness / mass).sqrt();
    let zeta = damping / (2.0 * (stiffness * mass).sqrt());
    let u0 = from - to;
    // An effects spring's ratio is 1 (androidx `ExpressiveMotionTokens`), but `c = 2·√k` in `f32` lands a hair
    // either side of it; the two non-critical forms lose all precision that close to 1.
    let offset = if (zeta - 1.0).abs() < CRITICAL_BAND {
        (u0 + (velocity + omega * u0) * t) * (-omega * t).exp()
    } else if zeta < 1.0 {
        let omega_d = omega * (1.0 - zeta * zeta).sqrt();
        (-zeta * omega * t).exp()
            * (u0 * (omega_d * t).cos()
                + (velocity + zeta * omega * u0) / omega_d * (omega_d * t).sin())
    } else {
        let root = (zeta * zeta - 1.0).sqrt();
        let r1 = -omega * (zeta - root);
        let r2 = -omega * (zeta + root);
        let a = (velocity - r2 * u0) / (r1 - r2);
        a * (r1 * t).exp() + (u0 - a) * (r2 * t).exp()
    };
    to + offset
}

/// How close to 1 a damping ratio is taken as critical: far above the `f32` error of `c = 2·√k` (about 1e-7), far
/// below any ratio M3 uses other than 1 (the lowest is 0.6).
const CRITICAL_BAND: f64 = 1e-6;

/// The largest gap between `samples` and the closed-form spring from `from` at rest, sample `i` taken at
/// `i / REPLAY_HZ` seconds.
pub(crate) fn worst_gap(samples: &[f32], from: f64, spring: &StartedSpring) -> f64 {
    let parameters = (
        f64::from(spring.stiffness),
        f64::from(spring.damping),
        f64::from(spring.mass),
    );
    samples
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a frame index stays far below 2^52"
            )]
            let t = i as f64 / REPLAY_HZ;
            let expected = spring_position(from, f64::from(spring.to), 0.0, parameters, t);
            (f64::from(value) - expected).abs()
        })
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use nsh_core::prelude::*;
    use nsh_engine::{Clock, EngineConfig, Root};

    use super::*;
    use crate::motion::{SPRING_MASS, SpringToken};

    /// `k`, `c`, `m` of a spring token, as a component would send them (ADR 0004).
    fn parameters(token: SpringToken) -> (f64, f64, f64) {
        let spec = token.spec();
        (
            f64::from(spec.stiffness),
            f64::from(spec.damping_coefficient()),
            f64::from(SPRING_MASS),
        )
    }

    /// `m·x″ + c·x′ + k·(x − to) = 0` integrated with fourth-order Runge–Kutta in steps of 10 µs: an answer that
    /// shares no formula with [`spring_position`].
    fn integrated(
        from: f64,
        to: f64,
        velocity: f64,
        (stiffness, damping, mass): (f64, f64, f64),
        t: f64,
    ) -> f64 {
        let acceleration =
            |position: f64, speed: f64| (-stiffness * (position - to) - damping * speed) / mass;
        let step = 1e-5;
        let (mut x, mut v) = (from, velocity);
        let mut time = 0.0;
        while time < t - step / 2.0 {
            let (k1x, k1v) = (v, acceleration(x, v));
            let (k2x, k2v) = (
                v + k1v * step / 2.0,
                acceleration(x + k1x * step / 2.0, v + k1v * step / 2.0),
            );
            let (k3x, k3v) = (
                v + k2v * step / 2.0,
                acceleration(x + k2x * step / 2.0, v + k2v * step / 2.0),
            );
            let (k4x, k4v) = (v + k3v * step, acceleration(x + k3x * step, v + k3v * step));
            x += (k1x + 2.0 * k2x + 2.0 * k3x + k4x) * step / 6.0;
            v += (k1v + 2.0 * k2v + 2.0 * k3v + k4v) * step / 6.0;
            time += step;
        }
        x
    }

    #[test]
    fn the_closed_form_matches_an_integration_of_every_m3_spring() {
        // Every Expressive spring (underdamped spatial ones, critical effects ones), plus an overdamped one, which
        // M3 does not use but the closed form must not get wrong; at rest and thrown against the motion.
        let springs = [
            SpringToken::DefaultSpatial,
            SpringToken::FastSpatial,
            SpringToken::SlowSpatial,
            SpringToken::DefaultEffects,
            SpringToken::FastEffects,
            SpringToken::SlowEffects,
        ]
        .map(parameters)
        .into_iter()
        .chain([(400.0, 60.0, 1.0)]);
        for spring in springs {
            for velocity in [0.0, -500.0] {
                for t in [0.0, 0.05, 0.1, 0.25, 0.5, 1.0] {
                    let closed = spring_position(0.0, 100.0, velocity, spring, t);
                    let numeric = integrated(0.0, 100.0, velocity, spring, t);
                    // 1e-4 of a 100-point travel: a thousandth of the replay's tolerance, and above the
                    // critical band's own error (about 1e-6 for the fast effects spring).
                    assert!(
                        (closed - numeric).abs() < 1e-4,
                        "{spring:?}, v0 {velocity}, t {t}: {closed} vs {numeric}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_critical_spring_is_continuous_across_the_band() {
        // Just inside and just outside the band that takes a ratio as 1, the positions agree: the band changes the
        // formula, not the answer.
        let k: f64 = 3800.0;
        let critical = 2.0 * k.sqrt();
        for t in [0.01, 0.05, 0.2] {
            let inside = spring_position(0.0, 1.0, 0.0, (k, critical * (1.0 + 0.5e-6), 1.0), t);
            let below = spring_position(0.0, 1.0, 0.0, (k, critical * (1.0 - 2e-6), 1.0), t);
            let above = spring_position(0.0, 1.0, 0.0, (k, critical * (1.0 + 2e-6), 1.0), t);
            assert!((inside - below).abs() < 1e-5, "t {t}: {inside} vs {below}");
            assert!((inside - above).abs() < 1e-5, "t {t}: {inside} vs {above}");
        }
    }

    #[test]
    fn a_spring_starts_where_it_starts_and_ends_on_its_target() {
        let spring = parameters(SpringToken::DefaultSpatial);
        assert!((spring_position(20.0, 100.0, 0.0, spring, 0.0) - 20.0).abs() < 1e-9);
        assert!((spring_position(20.0, 100.0, 0.0, spring, 10.0) - 100.0).abs() < 1e-9);
        // The default spatial spring overshoots (ζ = 0.8); an effects spring never does (ζ = 1).
        let peak = |spring| {
            (0..240)
                .map(|i| spring_position(0.0, 100.0, 0.0, spring, f64::from(i) / REPLAY_HZ))
                .fold(f64::MIN, f64::max)
        };
        assert!(peak(spring) > 101.0);
        assert!(peak(parameters(SpringToken::DefaultEffects)) <= 100.0);
    }

    /// A view whose translation a press springs to 100, with `token`'s spring.
    fn mover(token: SpringToken) -> impl IntoElement {
        let x = animated_value(0.0);
        let spec = token.spec();
        let spring = Spring {
            stiffness: spec.stiffness,
            damping: spec.damping_coefficient(),
            mass: SPRING_MASS,
            ..Spring::default()
        };
        view()
            .flex_1()
            .bind_translate_x(&x)
            .child(text("Go").on_press(move |(): &()| {
                let _ = x.spring(100.0, spring);
            }))
    }

    #[component]
    fn SpatialMover() -> impl IntoElement {
        mover(SpringToken::DefaultSpatial)
    }

    #[component]
    fn EffectsMover() -> impl IntoElement {
        mover(SpringToken::FastEffects)
    }

    fn start(token: SpringToken) -> (Headless, Recording) {
        let root = match token {
            SpringToken::FastEffects => Root::of::<EffectsMover>(),
            _ => Root::of::<SpatialMover>(),
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("Go").unwrap();
        (app, recording)
    }

    /// Tolerance on a replayed curve: 0.1 % of the travel (ADR 0010).
    const TOLERANCE: f64 = 0.1;

    #[test]
    fn a_replayed_spring_follows_the_closed_form_and_costs_the_engine_nothing() {
        for token in [SpringToken::DefaultSpatial, SpringToken::FastEffects] {
            let (mut app, recording) = start(token);
            app.press("Go").unwrap();
            let (ran, sent) = recording.during(|| app.run_for(Duration::from_millis(1500)));
            ran.unwrap();

            // One spring, with the token's converted parameters, at the screen's rate.
            let springs = started_springs(&recording.in_step("press(Go)"));
            let [spring] = springs.as_slice() else {
                panic!("{token:?}: {springs:?}");
            };
            let spec = token.spec();
            assert_eq!(spring.stiffness.to_bits(), spec.stiffness.to_bits());
            assert_eq!(
                spring.damping.to_bits(),
                spec.damping_coefficient().to_bits()
            );
            assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
            assert_eq!(spring.frame_rate, None);
            // While it plays, the engine sends nothing: no batch, so no layout either.
            assert!(sent.is_empty(), "{token:?}: {sent:?}");

            let mut replay = Replay::default();
            replay.apply(&recording.items());
            let samples = replay.run(spring.node, 180);
            let gap = worst_gap(&samples, 0.0, spring);
            assert!(gap < TOLERANCE, "{token:?}: {gap}");
            // It settles exactly on its target, and the graph stops.
            let rest = replay.run(spring.node, 600);
            assert!(!replay.is_animating(), "{token:?}");
            assert_eq!(rest.last().map(|v| v.to_bits()), Some(100.0_f32.to_bits()));
        }
    }

    #[test]
    fn the_comparison_catches_a_ratio_passed_as_the_coefficient() {
        // ADR 0004's bug: the curve the engine plays compared with a spring whose damping is the ratio itself.
        let (mut app, recording) = start(SpringToken::DefaultSpatial);
        app.press("Go").unwrap();
        let [spring] = started_springs(&recording.items())[..] else {
            panic!("one spring");
        };
        let mut replay = Replay::default();
        replay.apply(&recording.items());
        let samples = replay.run(spring.node, 180);
        let wrong = StartedSpring {
            damping: SpringToken::DefaultSpatial.spec().damping_ratio,
            ..spring
        };
        assert!(worst_gap(&samples, 0.0, &wrong) > 50.0);
        // And the standard M3 spring (k = 700, ζ = 0.9) instead of the Expressive one.
        let standard = StartedSpring {
            stiffness: 700.0,
            damping: 0.9 * 2.0 * 700.0_f32.sqrt(),
            ..spring
        };
        assert!(worst_gap(&samples, 0.0, &standard) > 10.0);
    }

    #[test]
    fn what_the_engine_sends_during_an_action_is_what_came_after_it_began_in_any_step() {
        let (mut app, recording) = start(SpringToken::DefaultSpatial);
        let mounted = recording.items().len();
        assert!(mounted > 0);

        // A press sends items: they are returned, and only they, not the mount's.
        let (pressed, sent) = recording.during(|| app.press("Go"));
        pressed.unwrap();
        assert!(!sent.is_empty());
        assert_eq!(recording.items().len(), mounted + sent.len());
        assert_eq!(
            format!("{sent:?}"),
            format!("{:?}", recording.items().split_off(mounted))
        );

        // A run, which renames its step (`clock.runFor(1500)`), is measured all the same: once the spring plays on
        // the graph nothing is sent, and the answer is empty because nothing came, not because of a step's name.
        let after_press = recording.items().len();
        let (ran, sent) = recording.during(|| app.run_for(Duration::from_millis(1500)));
        ran.unwrap();
        assert!(sent.is_empty(), "{sent:?}");
        assert_eq!(recording.items().len(), after_press);
    }

    #[test]
    fn layout_work_is_counted_and_nothing_else() {
        let (_app, recording) = start(SpringToken::DefaultSpatial);
        let mounted = recording.items();
        // Mounting lays out the view and its text, and creates the value without starting it.
        assert!(layout_items(&mounted) >= 2, "{mounted:?}");
        assert_eq!(started_springs(&mounted), Vec::new());
        let animation_only: Vec<MountItem> = mounted
            .into_iter()
            .filter(|item| matches!(item, MountItem::Animation(_)))
            .collect();
        assert!(!animation_only.is_empty());
        assert_eq!(layout_items(&animation_only), 0);
    }
}
