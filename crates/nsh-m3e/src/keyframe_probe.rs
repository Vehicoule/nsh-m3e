//! The measurements of ADR 0010 for animations that are keyframes, shared by the tests of every component that plays
//! them: what the engine sent, read as tables, and the closed form it is compared with.
//!
//! A component that plays keyframes sends one operation, `AnimationOp::StartComposite`, whose composite is a tree of
//! timing steps (`nsh_core::animation::keyframes`). [`started_tracks`] reads it back as one [`Track`] per animated
//! value: the steps one iteration takes and how many iterations run. [`Track::value_at`] is the value a track has `t`
//! milliseconds in, by the closed form of its own steps with the easing of [`cubic_bezier`]; and [`Replay`] (in
//! `animation_probe`) plays the same operations in NSH's animation graph, so the two can be compared at every frame.
//!
//! [`Replay`]: crate::animation_probe::Replay

use nsh_core::MountItem;
use nsh_core::animation::{AnimationOp, Composite, Driver, Easing, NodeId};

/// One step of the body of a loop of keyframes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Step {
    /// A value moves to `to` over `duration_ms` along `easing`, from where it is (a `Driver::Timing`).
    Run {
        to: f32,
        duration_ms: f32,
        easing: Easing,
        delay_ms: f32,
        frame_rate: Option<f32>,
    },
    /// Nothing moves for this many milliseconds.
    Delay(f32),
}

/// What the engine sent for one animated value: the steps of one iteration, and how many iterations run.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Track {
    /// The animated value.
    pub(crate) node: NodeId,
    /// How many times the steps run; `None` is until stopped.
    pub(crate) iterations: Option<u32>,
    /// The steps of one iteration, in order.
    pub(crate) steps: Vec<Step>,
}

impl Track {
    /// How long one iteration takes, in milliseconds.
    pub(crate) fn period_ms(&self) -> f64 {
        self.steps
            .iter()
            .map(|step| match *step {
                Step::Run {
                    duration_ms,
                    delay_ms,
                    ..
                } => f64::from(duration_ms) + f64::from(delay_ms),
                Step::Delay(ms) => f64::from(ms),
            })
            .sum()
    }

    /// Where the value is `ms` after the track starts, if it starts at `from`: each iteration plays the steps from
    /// the value the one before left (the first step moves to the first keyframe, at once when it takes no time).
    pub(crate) fn value_at(&self, from: f64, ms: f64) -> f64 {
        let period = self.period_ms();
        let (mut start, mut left, mut done) = (from, ms, 0_u32);
        while left >= period && self.iterations.is_none_or(|count| done < count) {
            start = self.run(start, period);
            left -= period;
            done += 1;
        }
        if self.iterations.is_some_and(|count| done >= count) {
            return start;
        }
        self.run(start, left)
    }

    /// The value after `ms` of one iteration that starts at `from`.
    fn run(&self, from: f64, ms: f64) -> f64 {
        let (mut value, mut left) = (from, ms);
        for step in &self.steps {
            match *step {
                Step::Delay(wait) => {
                    left -= f64::from(wait);
                    if left < 0.0 {
                        return value;
                    }
                }
                Step::Run {
                    to,
                    duration_ms,
                    easing,
                    delay_ms,
                    ..
                } => {
                    left -= f64::from(delay_ms);
                    if left < 0.0 {
                        return value;
                    }
                    let duration = f64::from(duration_ms);
                    if left < duration {
                        let progress = ease(easing, left / duration);
                        return value + (f64::from(to) - value) * progress;
                    }
                    left -= duration;
                    value = f64::from(to);
                }
            }
        }
        value
    }
}

/// `easing` at `x`, from 0 to 1.
fn ease(easing: Easing, x: f64) -> f64 {
    match easing {
        Easing::Linear => x,
        Easing::Bezier(x1, y1, x2, y2) => cubic_bezier(
            (f64::from(x1), f64::from(y1)),
            (f64::from(x2), f64::from(y2)),
            x,
        ),
        other => panic!("an easing the probe does not know: {other:?}"),
    }
}

/// The cubic Bézier easing through the control points `first` and `second` (as CSS `cubic-bezier()`) at time `x`,
/// found a way that shares nothing with NSH's solver (which iterates Newton's method, then bisects): the curve is
/// walked at a hundred thousand values of its parameter, and the progress is read off the segment that holds `x`.
pub(crate) fn cubic_bezier(first: (f64, f64), second: (f64, f64), x: f64) -> f64 {
    const SAMPLES: u32 = 100_000;
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let point = |t: f64| {
        let u = 1.0 - t;
        let blend = |a: f64, b: f64| 3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t;
        (blend(first.0, second.0), blend(first.1, second.1))
    };
    let mut before = (0.0, 0.0);
    for i in 1..=SAMPLES {
        let at = point(f64::from(i) / f64::from(SAMPLES));
        if at.0 >= x {
            let share = (x - before.0) / (at.0 - before.0);
            return before.1 + (at.1 - before.1) * share;
        }
        before = at;
    }
    1.0
}

/// The tracks of every composite `items` start, one list for each: the composites a parallel group holds side by
/// side, the loops, and the sequences of timing steps inside them.
pub(crate) fn started_tracks(items: &[MountItem]) -> Vec<Vec<Track>> {
    items
        .iter()
        .filter_map(|item| match item {
            MountItem::Animation(AnimationOp::StartComposite { spec, .. }) => {
                let mut tracks = Vec::new();
                collect(spec, Some(1), &mut tracks);
                Some(tracks)
            }
            _ => None,
        })
        .collect()
}

/// Adds to `tracks` what `composite` runs, `iterations` times (`None`: until stopped) when it is the body of a loop and
/// once otherwise.
fn collect(composite: &Composite, iterations: Option<u32>, tracks: &mut Vec<Track>) {
    match composite {
        Composite::Parallel { children, .. } => {
            for child in children {
                collect(child, Some(1), tracks);
            }
        }
        Composite::Loop {
            child, iterations, ..
        } => collect(child, *iterations, tracks),
        Composite::Sequence(steps) => {
            let steps: Vec<Step> = steps.iter().map(step).collect();
            let node = steps_node(composite);
            tracks.push(Track {
                node,
                iterations,
                steps,
            });
        }
        other => panic!("a composite the probe does not read as keyframes: {other:?}"),
    }
}

/// The step of a sequence that `composite` is.
fn step(composite: &Composite) -> Step {
    match composite {
        Composite::Run {
            driver:
                Driver::Timing {
                    to,
                    duration_ms,
                    easing,
                    delay_ms,
                    frame_rate,
                },
            ..
        } => Step::Run {
            to: *to,
            duration_ms: *duration_ms,
            easing: *easing,
            delay_ms: *delay_ms,
            frame_rate: *frame_rate,
        },
        Composite::Delay(ms) => Step::Delay(*ms),
        other => panic!("a step that is not a timing run or a delay: {other:?}"),
    }
}

/// The one value a sequence of timing steps animates.
fn steps_node(composite: &Composite) -> NodeId {
    let mut nodes = Vec::new();
    composite.nodes(&mut nodes);
    let [node] = nodes[..] else {
        panic!("a sequence of one value, not {nodes:?}");
    };
    node
}

/// How many composites `items` start.
pub(crate) fn started_composites(items: &[MountItem]) -> usize {
    items
        .iter()
        .filter(|item| {
            matches!(
                item,
                MountItem::Animation(AnimationOp::StartComposite { .. })
            )
        })
        .count()
}

#[cfg(test)]
mod tests {
    use nsh_core::animation::keyframes;
    use nsh_core::prelude::{Keyframe, KeyframeTiming};

    use super::*;
    use crate::animation_probe::{REPLAY_HZ, Replay};

    /// CSS `ease-in-out`, `cubic-bezier(0.42, 0, 0.58, 1)`: symmetric about the middle, where it is 0.5.
    #[test]
    fn the_closed_form_bezier_is_the_curve_css_gives() {
        let ease_in_out = |x| cubic_bezier((0.42, 0.0), (0.58, 1.0), x);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-6);
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        // Out of the range it holds the end it is past.
        assert_eq!(ease_in_out(-3.0), 0.0);
        assert_eq!(ease_in_out(3.0), 1.0);
        // A linear curve (both control points on the diagonal) is the identity.
        for x in [0.1, 0.37, 0.9] {
            let linear = cubic_bezier((1.0 / 3.0, 1.0 / 3.0), (2.0 / 3.0, 2.0 / 3.0), x);
            assert!((linear - x).abs() < 1e-6, "{x}: {linear}");
        }
        // Material's emphasized accelerate curve, control points `(0.3, 0)` and `(0.8, 0.15)`, starts slowly and ends
        // fast: a tenth of the way it has not made a hundredth, nine tenths of the way it is past a half.
        let accelerate = |x| cubic_bezier((0.3, 0.0), (0.8, 0.15), x);
        assert!(accelerate(0.1) < 0.01);
        assert!(accelerate(0.9) > 0.6 && accelerate(0.9) < 0.7);
        // A point worked out by hand: at the parameter 0.5 each coordinate of a cubic is 0.375 · (first + second) +
        // 0.125, so the curve passes through (0.375 · 1.1 + 0.125, 0.375 · 0.15 + 0.125) = (0.5375, 0.18125).
        assert!((accelerate(0.5375) - 0.18125).abs() < 1e-5);
    }

    fn timing() -> KeyframeTiming {
        KeyframeTiming::new(1000.0)
            .easing(Easing::Linear)
            .infinite()
    }

    #[test]
    fn keyframes_are_read_back_as_the_steps_they_send() {
        let frames = [
            Keyframe::at(0.0, 0.0),
            Keyframe::at(0.25, 0.0).ease(Easing::Bezier(0.3, 0.0, 0.8, 0.15)),
            Keyframe::at(0.75, 1.0),
            Keyframe::at(1.0, 1.0),
        ];
        let composite = keyframes(7, &frames, &timing());
        let items = [MountItem::Animation(AnimationOp::StartComposite {
            animation: 1,
            key: 1,
            spec: composite,
        })];
        let [tracks] = &started_tracks(&items)[..] else {
            panic!("one composite");
        };
        let [track] = &tracks[..] else {
            panic!("one value: {tracks:?}");
        };
        assert_eq!((track.node, track.iterations), (7, None));
        // The first step moves to the first keyframe at once; each next one is a segment, with the easing the keyframe
        // that starts it gives: to 0 over 250 ms, to 1 over 500 ms along the curve, to 1 over 250 ms.
        let expected = [
            (0.0, 0.0, Easing::Linear),
            (0.0, 250.0, Easing::Linear),
            (1.0, 500.0, Easing::Bezier(0.3, 0.0, 0.8, 0.15)),
            (1.0, 250.0, Easing::Linear),
        ];
        let runs: Vec<(f32, f32, Easing)> = track
            .steps
            .iter()
            .map(|step| match *step {
                Step::Run {
                    to,
                    duration_ms,
                    easing,
                    delay_ms,
                    frame_rate,
                } => {
                    assert_eq!((delay_ms, frame_rate), (0.0, None));
                    (to, duration_ms, easing)
                }
                Step::Delay(ms) => panic!("a delay of {ms}"),
            })
            .collect();
        assert_eq!(runs, expected);
        assert_eq!(track.period_ms(), 1000.0);
        assert_eq!(started_composites(&items), 1);
        assert_eq!(started_composites(&[]), 0);
    }

    #[test]
    fn a_single_pass_of_keyframes_is_one_iteration() {
        let timing = KeyframeTiming::new(400.0).easing(Easing::Linear);
        let composite = keyframes(
            3,
            &[Keyframe::at(0.0, 0.0), Keyframe::at(1.0, 10.0)],
            &timing,
        );
        let items = [MountItem::Animation(AnimationOp::StartComposite {
            animation: 1,
            key: 1,
            spec: composite,
        })];
        let tracks = started_tracks(&items).remove(0);
        assert_eq!(tracks[0].iterations, Some(1));
        // Linear from 0 to 10 over 400 ms, and 10 after it.
        let track = &tracks[0];
        assert!((track.value_at(0.0, 100.0) - 2.5).abs() < 1e-9);
        assert!((track.value_at(0.0, 400.0) - 10.0).abs() < 1e-9);
        assert!((track.value_at(0.0, 9000.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn a_looped_track_starts_every_iteration_at_its_first_keyframe() {
        let timing = KeyframeTiming::new(1000.0)
            .easing(Easing::Linear)
            .infinite();
        let composite = keyframes(
            1,
            &[Keyframe::at(0.0, 0.0), Keyframe::at(0.5, 1.0)],
            &timing,
        );
        let items = [MountItem::Animation(AnimationOp::StartComposite {
            animation: 1,
            key: 1,
            spec: composite,
        })];
        let track = started_tracks(&items).remove(0).remove(0);
        // Up over the first half, held at 1 over the second, then back to 0 at once, every time.
        for (ms, expected) in [
            (0.0, 0.0),
            (250.0, 0.5),
            (500.0, 1.0),
            (900.0, 1.0),
            (1000.0, 0.0),
            (1250.0, 0.5),
            (1900.0, 1.0),
            (2250.0, 0.5),
            (7250.0, 0.5),
        ] {
            let value = track.value_at(0.0, ms);
            assert!((value - expected).abs() < 1e-9, "{ms}: {value}");
        }
    }

    /// A graph that plays `composite` on a value, sampled at [`REPLAY_HZ`].
    fn played(composite: Composite, node: NodeId, frames: usize) -> Vec<f32> {
        let mut replay = Replay::default();
        replay.apply(&[
            MountItem::Animation(AnimationOp::CreateValue { node, value: 0.0 }),
            MountItem::Animation(AnimationOp::StartComposite {
                animation: 2,
                key: 2,
                spec: composite,
            }),
        ]);
        replay.run(node, frames)
    }

    #[test]
    fn the_graph_plays_what_the_table_says() {
        // A delayed rise with a curve, in a loop: the shape of every sweep of the indeterminate indicator.
        let frames = [
            Keyframe::at(0.0, 0.0),
            Keyframe::at(0.2, 0.0).ease(Easing::Bezier(0.3, 0.0, 0.8, 0.15)),
            Keyframe::at(0.8, 1.0),
            Keyframe::at(1.0, 1.0),
        ];
        let composite = keyframes(5, &frames, &timing());
        let items = [MountItem::Animation(AnimationOp::StartComposite {
            animation: 2,
            key: 2,
            spec: composite.clone(),
        })];
        let track = started_tracks(&items).remove(0).remove(0);
        let samples = played(composite, 5, 300);
        for (i, sample) in (0_u32..).zip(&samples) {
            let ms = f64::from(i) * 1000.0 / REPLAY_HZ;
            // The frame that lands on the end of an iteration may be on either side of the jump back to 0.
            if (ms % 1000.0).min(1000.0 - ms % 1000.0) < 1e-6 {
                continue;
            }
            let expected = track.value_at(0.0, ms);
            assert!(
                (f64::from(*sample) - expected).abs() < 1e-3,
                "{ms} ms: {sample} vs {expected}"
            );
        }
    }
}
