//! The drawn container's measurements, shared by the tests of every component that morphs ([ADR 0011]): the paths
//! its canvases trace, read with the animated values as they are, and the spring a press starts.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md

use std::f32::consts::{FRAC_PI_2, PI};

use nsh_core::MountItem;
use nsh_core::animation::AnimationOp;
use nsh_core::canvas::{CanvasOp, Drawing, PathOp};
use nsh_core::geometry::{Points, Size};

use crate::animation_probe::{Recording, Replay, StartedSpring, started_springs};
use crate::motion::{SPRING_MASS, SpringToken};

/// One step of a path, its numbers read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Step {
    Move(f32, f32),
    Line(f32, f32),
    Arc {
        x: f32,
        y: f32,
        radius: f32,
        start: f32,
        end: f32,
        ccw: bool,
    },
    Close,
}

/// The paths `drawing` traces, each begun by `beginPath`, the animated values among their numbers read in
/// `replay` as they are now.
pub(crate) fn paths(drawing: &Drawing, replay: &Replay) -> Vec<Vec<Step>> {
    let slot = |slot: u16| replay.value(drawing.slots[usize::from(slot)]);
    let mut paths: Vec<Vec<Step>> = Vec::new();
    for op in &drawing.ops {
        let step = match op {
            CanvasOp::BeginPath => {
                paths.push(Vec::new());
                continue;
            }
            CanvasOp::Path(PathOp::MoveTo(x, y)) => Step::Move(x.get(slot), y.get(slot)),
            CanvasOp::Path(PathOp::LineTo(x, y)) => Step::Line(x.get(slot), y.get(slot)),
            CanvasOp::Path(PathOp::Arc {
                x,
                y,
                radius,
                start,
                end,
                ccw,
            }) => Step::Arc {
                x: x.get(slot),
                y: y.get(slot),
                radius: radius.get(slot),
                start: start.get(slot),
                end: end.get(slot),
                ccw: *ccw,
            },
            CanvasOp::Path(PathOp::Close) => Step::Close,
            CanvasOp::Path(other) => panic!("not a step of a rounded rectangle: {other:?}"),
            _ => continue,
        };
        paths.last_mut().expect("a path begun").push(step);
    }
    paths
}

/// A rectangle `inset` points inside a container of `size`, its corners rounded by `radius`, as ADR 0011 traces
/// it: clockwise from the end of the top left corner, a line and a quarter circle for each side.
pub(crate) fn rounded_rectangle(size: Size<Points>, inset: f32, radius: f32) -> Vec<Step> {
    rounded_sides(size, inset, [radius, radius])
}

/// [`rounded_rectangle`] with its left corners rounded by the first of `[left, right]` and its right corners by the
/// second.
pub(crate) fn rounded_sides(size: Size<Points>, inset: f32, [l, r]: [f32; 2]) -> Vec<Step> {
    let (left, top) = (inset, inset);
    let (right, bottom) = (size.width.0 - inset, size.height.0 - inset);
    let quarter = |x, y, radius, start: f32| Step::Arc {
        x,
        y,
        radius,
        start,
        end: start + FRAC_PI_2,
        ccw: false,
    };
    vec![
        Step::Move(left + l, top),
        Step::Line(right - r, top),
        quarter(right - r, top + r, r, -FRAC_PI_2),
        Step::Line(right, bottom - r),
        quarter(right - r, bottom - r, r, 0.0),
        Step::Line(left + l, bottom),
        quarter(left + l, bottom - l, l, FRAC_PI_2),
        Step::Line(left, top + l),
        quarter(left + l, top + l, l, PI),
        Step::Close,
    ]
}

/// The kind of `step`, and its numbers.
fn numbers(step: &Step) -> (&'static str, Vec<f32>) {
    match *step {
        Step::Move(x, y) => ("move", vec![x, y]),
        Step::Line(x, y) => ("line", vec![x, y]),
        Step::Arc {
            x,
            y,
            radius,
            start,
            end,
            ccw,
        } => (
            if ccw { "counterclockwise arc" } else { "arc" },
            vec![x, y, radius, start, end],
        ),
        Step::Close => ("close", Vec::new()),
    }
}

/// `path` is `expected`, each number within a thousandth of a point (the `f32` sums a slot goes through).
pub(crate) fn assert_traces(path: &[Step], expected: &[Step], what: &str) {
    let same = path.len() == expected.len()
        && path.iter().zip(expected).all(|(step, wanted)| {
            let (kind, found) = numbers(step);
            let (wanted_kind, wanted_numbers) = numbers(wanted);
            kind == wanted_kind
                && found
                    .iter()
                    .zip(&wanted_numbers)
                    .all(|(a, b)| (a - b).abs() < 1e-3)
        });
    assert!(same, "{what}:\n{path:?}\nis not\n{expected:?}");
}

/// The animation graph with every operation `recording` holds so far.
pub(crate) fn replay(recording: &Recording) -> Replay {
    let mut replay = Replay::default();
    replay.apply(&recording.items());
    replay
}

/// Step `step` springs the corners once, with the morph's spring (`DefaultEffects`, androidx `Button.kt`'s
/// `shapeByInteraction`, converted as ADR 0004 says), to `to`, from where they are and as fast as they move, at
/// the screen's rate; and nothing else in the step starts or sets a value.
pub(crate) fn spring_in(items: &[MountItem], step: &str, to: f32) -> StartedSpring {
    spring_with(items, step, to, SpringToken::DefaultEffects)
}

/// [`spring_in`] for a component whose corners move with another spring: the list item's, `FastSpatial`.
pub(crate) fn spring_with(
    items: &[MountItem],
    step: &str,
    to: f32,
    token: SpringToken,
) -> StartedSpring {
    let springs = started_springs(items);
    let [spring] = springs[..] else {
        panic!("{step}: one spring, not {springs:?}");
    };
    let spec = token.spec();
    assert_eq!(spring.to.to_bits(), to.to_bits(), "{step}: {spring:?}");
    assert_eq!(spring.stiffness.to_bits(), spec.stiffness.to_bits());
    assert_eq!(
        spring.damping.to_bits(),
        spec.damping_coefficient().to_bits()
    );
    assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
    assert_eq!(spring.velocity, None, "{step}");
    assert_eq!(spring.frame_rate, None, "{step}");
    assert_eq!(set_values(items), 0, "{step}: {items:?}");
    spring
}

/// How many values `items` set at once.
pub(crate) fn set_values(items: &[MountItem]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, MountItem::Animation(AnimationOp::Set { .. })))
        .count()
}

/// The spring's `k`, `c` and `m`, as the closed form takes them.
pub(crate) fn parameters(spring: &StartedSpring) -> (f64, f64, f64) {
    (
        f64::from(spring.stiffness),
        f64::from(spring.damping),
        f64::from(spring.mass),
    )
}

/// Tolerance on a replayed curve: 0.1 % of its travel (ADR 0010).
pub(crate) fn tolerance(from: f64, to: f64) -> f64 {
    (to - from).abs() / 1000.0
}
