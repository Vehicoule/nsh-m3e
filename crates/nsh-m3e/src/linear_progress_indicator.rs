//! The Material 3 Expressive linear progress indicator: a track and an active indicator along it, with a dot at the
//! end of the track (the stop), flat or wavy, that shows how far the work is or that it is under way.
//!
//! It follows androidx's `LinearProgressIndicator` in `ProgressIndicator.kt` at the pinned commit, as [ADR 0012] says.
//! Where a value is not a token it is a literal with its line in androidx: [`DEFAULT_WIDTH`], the stop's trailing
//! space, and the times and curve of the indeterminate motion.
//!
//! The indicator is as wide as [`LinearProgressIndicatorProps::width`] and as tall as its stroke
//! (`LinearProgressIndicatorTokens.Height`). It is drawn from the end it starts at: the left in a left-to-right layout,
//! the right in a right-to-left one, as androidx mirrors it. The direction is the app's
//! (`nsh_core::element::layout_rtl`), as it is for `nsh-base`'s sliders: a `direction` style on a part of the tree
//! does not turn it.
//!
//! # Three layers
//!
//! The indicator draws on three canvases, from the back: the first line, the track, and the last line (with the stop).
//! A determinate indicator draws its track on the middle one and its indicator and stop on the last, so the first is
//! empty. An indeterminate one draws androidx's five strokes in androidx's order, which alternates the colors: the
//! track before line 1, line 1, the track between the lines, line 2, the track after line 2. Three canvases give that
//! order for the two colors, because the track before line 1 and the track after line 2 never touch a line (their
//! caps keep the gap away from it), so they can go with the track between the lines. The one that touches a line is
//! the track between: line 1 and line 2 can overlap it, and androidx draws it over line 1 and under line 2, as the
//! layers do.
//!
//! # The indeterminate motion
//!
//! androidx moves four values from 0 to 1 and back, the head and the tail of two lines, each with its own
//! `keyframes` in a cycle of 1750 ms (`ProgressIndicator.kt`, lines 980 to 1025), and draws whatever they say. Here
//! each is an animated value that NSH's animation graph moves natively, with the same keyframes, so the engine sends no
//! batch while it plays. What androidx decides with an `if` each frame cannot be decided by a canvas that is not
//! recorded again, so every `if` is a number of the drawing that is 1 or 0: the stroke's `globalAlpha`. Each is a
//! step, made as a ramp (`RAMP` wide, `ZERO_RAMP` at 0), narrower than anything a frame can show.
//!
//! With reduce motion on, the indicator does not move: it draws one still pose, the strokes androidx draws at one
//! instant of the cycle (`still_pose`).
//!
//! # The wavy form
//!
//! [`ProgressShape::Wavy`] draws the same indicator with an active line that waves, as androidx's
//! `LinearWavyProgressIndicator` does. It is its own drawing (`wavy`): taller, with a track path of its own, and a wave
//! that is drawn once and moved by the animation graph. It takes the same four forms: determinate and indeterminate,
//! each of which moves (the wave travels, the indeterminate lines go round) or, with reduce motion, holds still.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use std::rc::Rc;

use nsh_core::SvgPaint;
use nsh_core::canvas::LineCap;
use nsh_core::element::{CanvasArg, layout_rtl};
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::progress::{
    self, Progress, ProgressShape, easing, indicator_color, plays_loops, track_color,
};
use crate::tokens::CubicBezier;
use crate::tokens::androidx::linear_progress_indicator_tokens as tokens;
use crate::tokens::androidx::motion_tokens;

mod wavy;

/// The width of a linear progress indicator when none is given: androidx `LinearIndicatorWidth`, "given in the spec
/// but not defined as a token" (`ProgressIndicator.kt`, lines 1028 to 1031).
pub const DEFAULT_WIDTH: Points = Points(240.0);

/// The most the stop is moved in from the end of the track: androidx `StopIndicatorTrailingSpace`
/// (`ProgressIndicator.kt`, lines 1037 and 1038), which `drawStopIndicator` reads where the token
/// `LinearProgressIndicatorTokens.StopTrailingSpace` (0) is not used.
const STOP_TRAILING_SPACE: Points = Points(6.0);

/// How a line ends: androidx `StrokeCap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cap {
    /// Cut straight at the line's end.
    Butt,
    /// A half circle past the line's end.
    Round,
}

impl Cap {
    /// The same ending for a canvas.
    fn to_canvas(self) -> LineCap {
        match self {
            Self::Butt => LineCap::Butt,
            Self::Round => LineCap::Round,
        }
    }
}

/// The cap of the indicator's lines: round (`ProgressIndicatorDefaults.LinearStrokeCap`), unless the indicator is
/// narrower than it is tall, where there is no room for the caps and androidx draws butt lines
/// (`drawLinearIndicator`: "if there isn't enough space to draw the stroke caps, fall back to StrokeCap.Butt").
fn cap(width: f32, height: f32) -> Cap {
    if height > width {
        Cap::Butt
    } else {
        Cap::Round
    }
}

/// `x` held between `low` and `high`, as androidx's `fastCoerceIn` does it: a NaN stays a NaN. Where `f32::clamp`
/// panics (a bound that is not a number, or a low above the high) this does not: it gives `high`, or `x` when a bound
/// is not a number.
fn held(x: f32, low: f32, high: f32) -> f32 {
    if x.is_nan() { x } else { x.max(low).min(high) }
}

/// Where `fraction` of the width is, in points from the left: for a round line, held inside the indicator by half the
/// stroke, which its caps reach past its ends (`drawLinearIndicator`'s `adjustedBarStart` and `adjustedBarEnd`).
fn pixel(fraction: f32, width: f32, height: f32) -> f32 {
    let at = fraction * width;
    match cap(width, height) {
        Cap::Butt => at,
        Cap::Round => held(at, height / 2.0, width - height / 2.0),
    }
}

/// A horizontal line of the indicator, along its middle: where it starts and ends, in points from the left of the
/// indicator, and how it ends. With a round cap the ends are the centers of the caps, which reach half the stroke
/// past them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Line {
    from: f32,
    to: f32,
    cap: Cap,
}

/// The line androidx's `drawLinearIndicator` draws from `start` to `end` (fractions of the width, as the `from` end
/// goes), or `None` where it draws nothing: a round line of no length, and a fraction or a size that is not a number.
/// A line that goes backwards (`start` past `end`) is drawn between its ends all the same.
fn line(start: f32, end: f32, width: f32, height: f32, ltr: bool) -> Option<Line> {
    // The mirror of a right-to-left layout: `barStart = (1 - end) * width`, `barEnd = (1 - start) * width`.
    let (from, to) = if ltr {
        (pixel(start, width, height), pixel(end, width, height))
    } else {
        (
            pixel(1.0 - end, width, height),
            pixel(1.0 - start, width, height),
        )
    };
    let cap = cap(width, height);
    let has_length = match cap {
        Cap::Butt => true,
        Cap::Round => (end - start).abs() > 0.0,
    };
    let finite = width.is_finite() && height.is_finite() && from.is_finite() && to.is_finite();
    (has_length && finite).then_some(Line { from, to, cap })
}

/// The gap between the indicator and the track as a fraction of the width: androidx's `gapSizeFraction`. The caps eat
/// into the gap, so a round line's gap is wider by the stroke (`adjustedGapSize`).
fn gap_fraction(width: f32, height: f32, gap: f32) -> f32 {
    let adjusted_gap = match cap(width, height) {
        Cap::Butt => gap,
        Cap::Round => gap + height,
    };
    adjusted_gap / width
}

/// What a determinate flat indicator draws, under the track and over it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Determinate {
    track: Option<Line>,
    indicator: Option<Line>,
}

/// The track and the indicator for `progress`, as `LinearProgressIndicator(progress, …)` draws them: the indicator
/// from the start to the progress, the track from the progress and a gap to the end, and no track once the gap
/// reaches the end. `progress` is the raw fraction: it is held to 0 to 1 here, and a NaN draws neither, as androidx's
/// comparisons with one are false.
fn determinate(progress: f32, width: f32, height: f32, gap: f32, ltr: bool) -> Determinate {
    let coerced = progress.clamp(0.0, 1.0);
    let gap_fraction = gap_fraction(width, height, gap);
    let track_start = coerced + coerced.min(gap_fraction);
    Determinate {
        track: (track_start <= 1.0)
            .then(|| line(track_start, 1.0, width, height, ltr))
            .flatten(),
        indicator: line(0.0, coerced, width, height, ltr),
    }
}

/// The dot at the end of the track: its center and radius, in points from the indicator's top left.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stop {
    x: f32,
    y: f32,
    radius: f32,
}

/// The stop of `ProgressIndicatorDefaults.drawStopIndicator` with a round cap: a circle of `size` points, no larger
/// than the indicator is tall, against the end of the track, moved in by half what the stroke has to spare, but by no
/// more than [`STOP_TRAILING_SPACE`]. In a right-to-left layout the end is the left one.
fn stop(size: f32, width: f32, height: f32, ltr: bool) -> Stop {
    let diameter = size.min(height);
    let offset = ((height - diameter) / 2.0).min(STOP_TRAILING_SPACE.0);
    let from_end = diameter / 2.0 + offset;
    Stop {
        x: if ltr { width - from_end } else { from_end },
        y: height / 2.0,
        radius: diameter / 2.0,
    }
}

// ---- the indeterminate motion: androidx's keyframes ----------------------------------------------------------------

/// The cycle of the indeterminate indicator: androidx `LinearAnimationDuration` (`ProgressIndicator.kt`, line 1049),
/// in milliseconds, as `KeyframeTiming` takes it.
const CYCLE_MS: f32 = 1750.0;

/// How one of the four values of the indeterminate indicator moves in a cycle: androidx's
/// `keyframes { durationMillis = LinearAnimationDuration; 0f at delay using easing; 1f at delay + duration }`. The
/// value is 0 until `delay_ms`, rises to 1 over `duration_ms` along [`SWEEP_EASING`], and is 1 until the cycle ends.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sweep {
    /// `…Delay`, in milliseconds.
    delay_ms: f32,
    /// `…Duration`, in milliseconds.
    duration_ms: f32,
}

/// `firstLineHead`: `FirstLineHeadDelay` and `FirstLineHeadDuration` (`ProgressIndicator.kt`, lines 1058 and 1052),
/// in the spec at lines 980 to 989.
const FIRST_HEAD: Sweep = Sweep {
    delay_ms: 0.0,
    duration_ms: 1000.0,
};

/// `firstLineTail`: `FirstLineTailDelay` and `FirstLineTailDuration` (lines 1059 and 1053), in the spec at lines 992
/// to 1001.
const FIRST_TAIL: Sweep = Sweep {
    delay_ms: 250.0,
    duration_ms: 1000.0,
};

/// `secondLineHead`: `SecondLineHeadDelay` and `SecondLineHeadDuration` (lines 1060 and 1054), in the spec at lines
/// 1004 to 1013.
const SECOND_HEAD: Sweep = Sweep {
    delay_ms: 650.0,
    duration_ms: 850.0,
};

/// `secondLineTail`: `SecondLineTailDelay` and `SecondLineTailDuration` (lines 1061 and 1055), in the spec at lines
/// 1016 to 1025.
const SECOND_TAIL: Sweep = Sweep {
    delay_ms: 900.0,
    duration_ms: 850.0,
};

/// The curve of every rise: `LinearIndeterminateProgressEasing` (`ProgressIndicator.kt`, line 1063),
/// `MotionTokens.EasingEmphasizedAccelerateCubicBezier`.
const SWEEP_EASING: CubicBezier = motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;

impl Sweep {
    /// The keyframes androidx's spec has: the value at the start of the cycle (0, `initialValue`), 0 at the delay with
    /// the easing for the rise that starts there, 1 when the rise ends, and 1 at the end of the cycle (`targetValue`).
    /// A keyframe androidx writes at the very start or end replaces the implicit one there, so none is doubled.
    fn keyframes(self) -> Vec<Keyframe> {
        let offset = |ms: f32| ms / CYCLE_MS;
        let end = self.delay_ms + self.duration_ms;
        let mut frames = Vec::with_capacity(4);
        if self.delay_ms > 0.0 {
            frames.push(Keyframe::at(0.0, 0.0));
        }
        frames.push(Keyframe::at(offset(self.delay_ms), 0.0).ease(easing(SWEEP_EASING)));
        frames.push(Keyframe::at(offset(end), 1.0));
        if end < CYCLE_MS {
            frames.push(Keyframe::at(1.0, 1.0));
        }
        frames
    }

    /// Where the value is `ms` into the cycle, held to the cycle: what its keyframes give.
    fn at(self, ms: f32) -> f32 {
        ease(SWEEP_EASING, (ms - self.delay_ms) / self.duration_ms)
    }
}

/// `curve` at `x` (0 to 1): the progress at that time, as CSS `cubic-bezier()` and androidx's `CubicBezierEasing`
/// give it. Found by bisection on the curve's parameter, which is as precise as `f32` is.
fn ease(curve: CubicBezier, x: f32) -> f32 {
    if x.is_nan() || x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    // One coordinate of the curve at parameter `t`, from its first and second control points.
    let along = |first: f32, second: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * first + 3.0 * u * t * t * second + t * t * t
    };
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..BISECTIONS {
        let t = f32::midpoint(low, high);
        if along(curve.x1, curve.x2, t) < x {
            low = t;
        } else {
            high = t;
        }
    }
    along(curve.y1, curve.y2, f32::midpoint(low, high))
}

/// How many times [`ease`] halves the parameter's range: 2^-30 is far below what an `f32` fraction can tell apart.
const BISECTIONS: u32 = 30;

/// Four values of the indeterminate indicator, as fractions of its width: androidx's `firstLineHead`,
/// `firstLineTail`, `secondLineHead` and `secondLineTail`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Course<T> {
    first_head: T,
    first_tail: T,
    second_head: T,
    second_tail: T,
}

impl<T> Course<T> {
    /// Each of the four through `f`.
    fn map<U>(self, mut f: impl FnMut(T) -> U) -> Course<U> {
        Course {
            first_head: f(self.first_head),
            first_tail: f(self.first_tail),
            second_head: f(self.second_head),
            second_tail: f(self.second_tail),
        }
    }
}

/// Where the four values are `ms` into the cycle.
fn course_at(ms: f32) -> Course<f32> {
    Course {
        first_head: FIRST_HEAD.at(ms),
        first_tail: FIRST_TAIL.at(ms),
        second_head: SECOND_HEAD.at(ms),
        second_tail: SECOND_TAIL.at(ms),
    }
}

/// The instant of the cycle that the still pose is taken at: 1000 ms, when the first line's head reaches the end of
/// the track (`FirstLineHeadDelay + FirstLineHeadDuration`, `ProgressIndicator.kt`, lines 1058 and 1052). androidx has
/// no still pose: this instant is this crate's choice. At it both lines show, the first long and ending at the end of
/// the track, the second short and at its start, with the track between them and a gap on each side.
const STILL_POSE_AT_MS: f32 = 1000.0;

/// The strokes of an indeterminate indicator that moves no more (reduce motion): the values of androidx's animations
/// at [`STILL_POSE_AT_MS`]. This is the one place that says which pose the indicator holds; another instant is a
/// change to the constant above, nothing else.
fn still_pose() -> Course<f32> {
    course_at(STILL_POSE_AT_MS)
}

/// What an indeterminate flat indicator draws, in androidx's order (`ProgressIndicator.kt`, lines 294 to 333).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Indeterminate {
    /// "Track before line 1".
    track_before: Option<Line>,
    /// "Line 1".
    first: Option<Line>,
    /// "Track between line 1 and line 2".
    track_between: Option<Line>,
    /// "Line 2".
    second: Option<Line>,
    /// "Track after line 2".
    track_after: Option<Line>,
}

/// The five strokes androidx draws for these values, each only where its `if` holds (lines 295, 301, 312, 319 and
/// 330), the ends of each as the lines 296, 314 and 331 give them.
fn indeterminate(at: Course<f32>, width: f32, height: f32, gap: f32, ltr: bool) -> Indeterminate {
    let gap = gap_fraction(width, height, gap);
    let Course {
        first_head: head1,
        first_tail: tail1,
        second_head: head2,
        second_tail: tail2,
    } = at;
    let draw = |start: f32, end: f32| line(start, end, width, height, ltr);
    // "if (head > 0) head + gapSizeFraction else 0f", and "if (tail < 1f) tail - gapSizeFraction else 1f".
    let lead_at = |head: f32| if head > 0.0 { head + gap } else { 0.0 };
    let trail_at = |tail: f32| if tail < 1.0 { tail - gap } else { 1.0 };
    Indeterminate {
        track_before: (head1 < 1.0 - gap)
            .then(|| draw(lead_at(head1), 1.0))
            .flatten(),
        first: (head1 - tail1 > 0.0).then(|| draw(head1, tail1)).flatten(),
        track_between: (tail1 > gap)
            .then(|| draw(lead_at(head2), trail_at(tail1)))
            .flatten(),
        second: (head2 - tail2 > 0.0).then(|| draw(head2, tail2)).flatten(),
        track_after: (tail2 > gap).then(|| draw(0.0, trail_at(tail2))).flatten(),
    }
}

// ---- the moving strokes: the same arithmetic as numbers of the drawing ---------------------------------------------

/// How wide the ramp is that stands in for a step where a value crosses a threshold away from 0: 1e-6 of the width
/// (0.00024 points on the default 240 points). A canvas that is not recorded again cannot branch on a value that
/// moves, so each of androidx's `if`s is a number that is 1 where it holds and 0 where it does not, and a step between
/// them is a ramp. It is not androidx's: it is as narrow as `f32` allows without the numbers being noise (about 8
/// times the spacing of `f32` just above 1, where a value ends its rise), so a frame lands in one only by chance
/// (about a thousandth of a millisecond of a rise), and what it draws then is a stroke a little faint or a little
/// short for that frame, not a wrong one.
const RAMP: f32 = 1e-6;

/// How wide the ramp is where a value crosses 0: 1e-12. Every rise starts at 0 and leaves it slowly (the curve is
/// about 0.56 x² of the way at the fraction x of the rise), so a rise is below this for about a microsecond, where
/// [`RAMP`] would take a whole millisecond. It is far above the noise the graph leaves at the instant a rise starts
/// (a value of 8e-33, where androidx's is exactly 0) and far below the least a difference of two fractions can be
/// when it is not 0 (6e-8 near 1), so it reads that noise as 0 and every other difference as a line.
const ZERO_RAMP: f32 = 1e-12;

/// 1 where `x` is above 0, 0 at and below it.
fn step(x: &AnimatedNode) -> AnimatedNode {
    x.interpolate(&[0.0, ZERO_RAMP], &[0.0, 1.0])
}

/// 1 where `x` is above `at`, 0 at and below it.
fn above(x: &AnimatedNode, at: f32) -> AnimatedNode {
    x.interpolate(&[at, at + RAMP], &[0.0, 1.0])
}

/// 1 where `x` is below `at`, 0 at and above it.
fn below(x: &AnimatedNode, at: f32) -> AnimatedNode {
    x.interpolate(&[at - RAMP, at], &[1.0, 0.0])
}

/// 1 where `x` is not 0, 0 where it is: its square, which is 0 or above, as a step.
fn nonzero(x: &AnimatedNode) -> AnimatedNode {
    x.mul(x)
        .interpolate(&[0.0, ZERO_RAMP * ZERO_RAMP], &[0.0, 1.0])
}

/// "if (head > 0) head + gapSizeFraction else 0f" of a head that moves: 0 at 0, the gap once it is past the ramp (the
/// head `ZERO_RAMP` past 0 has the gap added to it, which is the gap to `f32`), and the gap past the head from there.
fn lead(head: &AnimatedNode, gap: f32) -> AnimatedNode {
    head.interpolate(&[0.0, ZERO_RAMP, 1.0], &[0.0, gap, gap + 1.0])
}

/// "if (tail < 1f) tail - gapSizeFraction else 1f" of a tail that moves.
fn trail(tail: &AnimatedNode, gap: f32) -> AnimatedNode {
    tail.interpolate(&[0.0, 1.0 - RAMP, 1.0], &[-gap, 1.0 - RAMP - gap, 1.0])
}

/// A number of a drawing that is fixed or moves.
#[derive(Debug, Clone, Copy)]
enum Value {
    /// A constant.
    Fixed(f32),
    /// A node of the animation graph.
    Moving(AnimatedNode),
}

impl From<Value> for CanvasArg {
    fn from(value: Value) -> Self {
        match value {
            Value::Fixed(number) => number.into(),
            Value::Moving(node) => (&node).into(),
        }
    }
}

/// What a line that moves is laid out in: the indicator's size and gap, and its direction.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    width: f32,
    height: f32,
    /// The gap between the indicator and the track, in points (the token).
    gap: f32,
    ltr: bool,
}

impl Geometry {
    /// The gap as a fraction of the width.
    fn gap_fraction(self) -> f32 {
        gap_fraction(self.width, self.height, self.gap)
    }

    /// Where `fraction` of the width is, as a number of the drawing, from the right when `mirrored` (what [`pixel`]
    /// gives for a fixed fraction, as a node of the graph for one that moves).
    fn pixel(self, fraction: Value, mirrored: bool) -> Value {
        let (width, height) = (self.width, self.height);
        let flip = |fraction: f32| if mirrored { 1.0 - fraction } else { fraction };
        let moving = match fraction {
            Value::Fixed(fraction) => return Value::Fixed(pixel(flip(fraction), width, height)),
            Value::Moving(node) => node,
        };
        Value::Moving(match cap(width, height) {
            Cap::Butt if mirrored => moving.mul(-width).add(width),
            Cap::Butt => moving.mul(width),
            Cap::Round => {
                let inset = height / 2.0;
                let (near, far) = (inset, width - inset);
                // Held between the insets, in fractions, and mapped to them in points: `held(fraction * width)`. As wide
                // as it is tall both insets are the middle, and the graph holds the number there.
                let (low, high) = (inset / width, 1.0 - inset / width);
                let out = if mirrored { [far, near] } else { [near, far] };
                moving.interpolate(&[low, high], &out)
            }
        })
    }

    /// A line from `start` to `end`, fractions of the width, shown as much as `alpha` says.
    fn line(self, start: Value, end: Value, alpha: AnimatedNode) -> Moving {
        // `drawLinearIndicator`'s mirror: `barStart = (1 - end) * width`, `barEnd = (1 - start) * width`.
        let (from, to) = if self.ltr {
            (self.pixel(start, false), self.pixel(end, false))
        } else {
            (self.pixel(end, true), self.pixel(start, true))
        };
        Moving { from, to, alpha }
    }
}

/// A line of the indeterminate indicator whose ends move and that is there or not: `alpha` is 1 where androidx draws
/// it and 0 where it does not.
#[derive(Debug, Clone, Copy)]
struct Moving {
    from: Value,
    to: Value,
    alpha: AnimatedNode,
}

/// "Line 1": from the head to the tail, where the head is past the tail (line 301).
fn first_line(at: Course<AnimatedNode>, geometry: Geometry) -> Moving {
    let shown = step(&at.first_head.sub(&at.first_tail));
    geometry.line(
        Value::Moving(at.first_head),
        Value::Moving(at.first_tail),
        shown,
    )
}

/// "Line 2": the same for the second line (line 319).
fn second_line(at: Course<AnimatedNode>, geometry: Geometry) -> Moving {
    let shown = step(&at.second_head.sub(&at.second_tail));
    geometry.line(
        Value::Moving(at.second_head),
        Value::Moving(at.second_tail),
        shown,
    )
}

/// "Track before line 1": from a gap after the head to the end, while the head is more than a gap from the end
/// (lines 295 to 298). It never has no length while it is shown, so only the one `if` is a number.
fn track_before(at: Course<AnimatedNode>, geometry: Geometry) -> Moving {
    let gap = geometry.gap_fraction();
    geometry.line(
        Value::Moving(lead(&at.first_head, gap)),
        Value::Fixed(1.0),
        below(&at.first_head, 1.0 - gap),
    )
}

/// "Track between line 1 and line 2": from a gap after the second head to a gap before the first tail, while the
/// first tail is more than a gap from the start (lines 312 to 316). Its ends can cross, which androidx draws as a
/// line between them, and be equal, which a round line does not draw (`abs(endFraction - startFraction) > 0`).
fn track_between(at: Course<AnimatedNode>, geometry: Geometry) -> Moving {
    let gap = geometry.gap_fraction();
    let start = lead(&at.second_head, gap);
    let end = trail(&at.first_tail, gap);
    let shown = above(&at.first_tail, gap).mul(&nonzero(&end.sub(&start)));
    geometry.line(Value::Moving(start), Value::Moving(end), shown)
}

/// "Track after line 2": from the start to a gap before the second tail, while the second tail is more than a gap
/// from the start (lines 330 to 333). Like the track before line 1, it never has no length while it is shown.
fn track_after(at: Course<AnimatedNode>, geometry: Geometry) -> Moving {
    let gap = geometry.gap_fraction();
    geometry.line(
        Value::Fixed(0.0),
        Value::Moving(trail(&at.second_tail, gap)),
        above(&at.second_tail, gap),
    )
}

/// The four keyframe animations of the indeterminate indicator, played together and over and over, on `values`.
fn sweeps(values: Course<AnimatedValue>) -> Animation {
    let timing = KeyframeTiming::new(CYCLE_MS)
        .easing(Easing::Linear)
        .infinite();
    let rise = |value: &AnimatedValue, sweep: Sweep| {
        Animation::keyframes(value, sweep.keyframes(), timing)
    };
    Animation::parallel([
        rise(&values.first_head, FIRST_HEAD),
        rise(&values.first_tail, FIRST_TAIL),
        rise(&values.second_head, SECOND_HEAD),
        rise(&values.second_tail, SECOND_TAIL),
    ])
}

// ---- drawing -------------------------------------------------------------------------------------------------------

/// What the indicator is doing, which says what its layers draw.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    /// Showing this fraction, as the progress gave it.
    Determinate(f32),
    /// Indeterminate, its values moving.
    Moving,
    /// Indeterminate, held in its still pose (reduce motion).
    Still,
}

/// Adds to `painter` a stroke along `line`, `thickness` points thick, at height `y`.
fn stroke(painter: Painter, line: Line, y: f32, thickness: f32) -> Painter {
    painter
        .line_cap(line.cap.to_canvas())
        .line_width(thickness)
        .begin_path()
        .move_to(line.from, y)
        .line_to(line.to, y)
        .stroke()
}

/// Adds to `painter` a stroke along a line that moves, `thickness` points thick, at height `y`, as faint as the
/// line's `alpha` says.
fn stroke_moving(painter: Painter, line: Moving, y: f32, thickness: f32, cap: Cap) -> Painter {
    painter
        .global_alpha(&line.alpha)
        .line_cap(cap.to_canvas())
        .line_width(thickness)
        .begin_path()
        .move_to(line.from, y)
        .line_to(line.to, y)
        .stroke()
}

/// Adds `line` to `painter`, when there is one.
fn stroke_still(painter: Painter, line: Option<Line>, y: f32, thickness: f32) -> Painter {
    match line {
        Some(line) => stroke(painter, line, y, thickness),
        None => painter,
    }
}

/// The two values that move a wave.
#[derive(Debug, Clone, Copy)]
struct WaveNodes {
    /// How high the wave is, from 0 (a flat line) to 1.
    amplitude: AnimatedNode,
    /// How far the wave has travelled, in wavelengths, counted up for ever.
    phase: AnimatedNode,
}

/// What the layers draw, with the sizes they draw in.
#[derive(Debug, Clone, Copy)]
struct Layers {
    mode: Mode,
    /// The four values of the indeterminate motion.
    course: Course<AnimatedNode>,
    geometry: Geometry,
    /// Flat or wavy.
    shape: ProgressShape,
    /// The values that move the wave of a wavy indicator.
    wave: WaveNodes,
    /// Whether the wave travels: not with reduce motion, where it holds the position it starts at.
    travels: bool,
}

impl Layers {
    /// Where the strokes are along the indicator's height, and how thick: the middle, and all of it
    /// (`drawLinearIndicator`'s `yOffset` and `strokeWidth`).
    fn pen(self) -> (f32, f32) {
        (self.geometry.height / 2.0, self.geometry.height)
    }

    /// The still pose's strokes.
    fn still(self) -> Indeterminate {
        let geometry = self.geometry;
        indeterminate(
            still_pose(),
            geometry.width,
            geometry.height,
            geometry.gap,
            geometry.ltr,
        )
    }

    /// The painter every layer starts from: strokes and fills take the canvas view's colors.
    fn painter() -> Painter {
        Painter::new()
            .stroke_style_from_view(SvgPaint::None)
            .fill_style_from_view(SvgPaint::None)
    }

    /// The cap of the strokes.
    fn cap(self) -> Cap {
        cap(self.geometry.width, self.geometry.height)
    }

    /// What a wavy indicator draws: the pieces of its track and the windows of its lines for the mode, the wave of that
    /// mode, the wave's motion, and the stop (a determinate indicator's).
    fn wavy(self) -> Wavy {
        let geometry = self.geometry;
        let width = geometry.width;
        let fixed = |start: f32, end: f32| (Value::Fixed(start), Value::Fixed(end));
        let (pairs, length, stop) = match self.mode {
            Mode::Determinate(fraction) => {
                let fraction = if fraction.is_finite() {
                    fraction.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (
                    vec![fixed(0.0, fraction)],
                    wavy::DETERMINATE_WAVELENGTH,
                    wavy::stop(fraction, width),
                )
            }
            Mode::Moving => {
                let at = self.course.map(Value::Moving);
                (
                    vec![
                        (at.first_tail, at.first_head),
                        (at.second_tail, at.second_head),
                    ],
                    wavy::INDETERMINATE_WAVELENGTH,
                    None,
                )
            }
            Mode::Still => {
                let at = still_pose();
                (
                    vec![
                        fixed(at.first_tail, at.first_head),
                        fixed(at.second_tail, at.second_head),
                    ],
                    wavy::INDETERMINATE_WAVELENGTH,
                    None,
                )
            }
        };
        let wave = wavy::Wave::new(length);
        Wavy {
            layout: wavy::layout(&pairs, width),
            wave,
            motion: wavy::Motion::of(self.wave.amplitude, self.wave.phase, wave, self.travels),
            stop,
            frame: wavy::Frame {
                width,
                ltr: geometry.ltr,
            },
        }
    }

    /// The first line, at the back: line 1 of an indeterminate indicator. A determinate one has none, and neither has a
    /// wavy one, which draws its lines together.
    fn first_line(self) -> Painter {
        if self.shape == ProgressShape::Wavy {
            return Self::painter();
        }
        let (y, thickness) = self.pen();
        match self.mode {
            Mode::Determinate(_) => Self::painter(),
            Mode::Moving => stroke_moving(
                Self::painter(),
                first_line(self.course, self.geometry),
                y,
                thickness,
                self.cap(),
            ),
            Mode::Still => stroke_still(Self::painter(), self.still().first, y, thickness),
        }
    }

    /// The track: a determinate indicator's, or the three tracks of an indeterminate one.
    fn track(self) -> Painter {
        if self.shape == ProgressShape::Wavy {
            let wavy = self.wavy();
            return wavy::track(Self::painter(), &wavy.layout, wavy.frame);
        }
        let (y, thickness) = self.pen();
        let geometry = self.geometry;
        match self.mode {
            Mode::Determinate(fraction) => {
                let drawn = determinate(
                    fraction,
                    geometry.width,
                    geometry.height,
                    geometry.gap,
                    geometry.ltr,
                );
                stroke_still(Self::painter(), drawn.track, y, thickness)
            }
            Mode::Moving => {
                let cap = self.cap();
                [track_before, track_between, track_after].into_iter().fold(
                    Self::painter(),
                    |painter, track| {
                        stroke_moving(painter, track(self.course, geometry), y, thickness, cap)
                    },
                )
            }
            Mode::Still => {
                let still = self.still();
                [still.track_before, still.track_between, still.track_after]
                    .into_iter()
                    .fold(Self::painter(), |painter, track| {
                        stroke_still(painter, track, y, thickness)
                    })
            }
        }
    }

    /// The last line, at the front: a determinate indicator's, with the stop; line 2 of an indeterminate one. A wavy
    /// indicator draws all its lines here, and the stop.
    fn last_line(self) -> Painter {
        if self.shape == ProgressShape::Wavy {
            let wavy = self.wavy();
            return wavy::lines(
                Self::painter(),
                &wavy.layout,
                wavy.wave,
                wavy.motion,
                wavy.stop,
                wavy.frame,
            );
        }
        let (y, thickness) = self.pen();
        let geometry = self.geometry;
        match self.mode {
            Mode::Determinate(fraction) => {
                let drawn = determinate(
                    fraction,
                    geometry.width,
                    geometry.height,
                    geometry.gap,
                    geometry.ltr,
                );
                let painter = stroke_still(Self::painter(), drawn.indicator, y, thickness);
                let dot = stop(
                    tokens::STOP_SIZE.0,
                    geometry.width,
                    geometry.height,
                    geometry.ltr,
                );
                painter
                    .begin_path()
                    .arc(dot.x, dot.y, dot.radius, 0.0, std::f32::consts::TAU)
                    .fill()
            }
            Mode::Moving => stroke_moving(
                Self::painter(),
                second_line(self.course, geometry),
                y,
                thickness,
                self.cap(),
            ),
            Mode::Still => stroke_still(Self::painter(), self.still().second, y, thickness),
        }
    }
}

/// What a wavy indicator draws, for a mode.
struct Wavy {
    layout: wavy::Layout,
    wave: wavy::Wave,
    motion: wavy::Motion,
    stop: Option<Stop>,
    frame: wavy::Frame,
}

/// A canvas filling the indicator.
fn layer(canvas: CanvasView) -> CanvasView {
    canvas.absolute().inset(px(0.0))
}

/// The props of a [`LinearProgressIndicator`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct LinearProgressIndicatorProps {
    /// How far the work is; indeterminate by default.
    #[prop(default)]
    pub progress: Prop<Progress>,
    /// What screen readers say: what is in progress. An indicator shows no text, so this is its only name.
    pub label: Prop<Option<SharedString>>,
    /// Whether the active line is straight or waves: flat by default.
    #[prop(default)]
    pub shape: Prop<ProgressShape>,
    /// How wide it is: [`DEFAULT_WIDTH`] by default.
    #[prop(default = DEFAULT_WIDTH)]
    pub width: Prop<Points>,
}

/// A Material 3 Expressive linear progress indicator.
///
/// It is a progress bar for screen readers, in the range 0 to 1 with the progress as its value, and none while
/// indeterminate. An indeterminate indicator moves on its own as long as it is indeterminate and on screen, played by
/// the platform's animation graph; with reduce motion on it holds one pose and moves no more.
///
/// ```no_run
/// use nsh_core::geometry::percentage;
/// use nsh_core::prelude::*;
/// use nsh_m3e::linear_progress_indicator::linear_progress_indicator;
/// use nsh_m3e::progress::Progress;
///
/// fn download(done: Option<f32>) -> impl IntoElement {
///     linear_progress_indicator()
///         .progress(done.map_or(Progress::Indeterminate, |done| {
///             Progress::Determinate(percentage(done))
///         }))
///         .label("Downloading")
/// }
/// ```
#[component]
pub fn LinearProgressIndicator(
    props: LinearProgressIndicatorProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let height = move || match props.shape.get() {
        ProgressShape::Flat => tokens::HEIGHT,
        ProgressShape::Wavy => wavy::HEIGHT,
    };
    let size = move || props.width.get();
    let values = Course {
        first_head: animated_value(0.0),
        first_tail: animated_value(0.0),
        second_head: animated_value(0.0),
        second_tail: animated_value(0.0),
    };
    let course = values.map(|value| *value.node());
    let mode = computed(move || {
        let progress = props.progress.get();
        match progress {
            Progress::Determinate(fraction) if !progress.is_indeterminate_in(props.shape.get()) => {
                Mode::Determinate(fraction.0)
            }
            _ if plays_loops() => Mode::Moving,
            _ => Mode::Still,
        }
    });
    // Nothing moves unless the indicator is indeterminate and may move; the values rest where they are when it stops,
    // and start again from the start of the cycle when it plays again.
    let moving = computed(move || mode.get() == Mode::Moving);
    let animation = sweeps(values);
    effect(move || {
        if moving.get() {
            let _ = animation.start();
        } else {
            animation.stop();
        }
    });
    // The wave of a wavy indicator: how high it is, and how far it has travelled. It starts as high as it is to be, and
    // moves only where it is wavy, may move, and has a height to show.
    let wavy = computed(move || props.shape.get() == ProgressShape::Wavy);
    let waving = Rc::new(wavy::Waving::new(
        wavy::target(untracked(|| mode.get())),
        untracked(|| wavy.get()),
    ));
    let wave = WaveNodes {
        amplitude: waving.amplitude(),
        phase: waving.phase(),
    };
    let reach = Rc::clone(&waving);
    effect(move || reach.reach(wavy.get(), plays_loops(), wavy::target(mode.get())));
    let travel = computed(move || wavy::travel(wavy.get(), plays_loops(), mode.get()));
    effect(move || waving.go(travel.get()));
    let layers = move || Layers {
        mode: mode.get(),
        course,
        geometry: Geometry {
            width: size().0,
            height: height().0,
            gap: tokens::TRACK_ACTIVE_SPACE.0,
            ltr: !layout_rtl(),
        },
        shape: props.shape.get(),
        wave,
        travels: plays_loops(),
    };
    let first = canvas(move || layers().first_line()).stroke_color(indicator_color());
    let track = canvas(move || layers().track()).stroke_color(track_color());
    let last = canvas(move || layers().last_line())
        .stroke_color(indicator_color())
        .fill_color(indicator_color());
    progress::accessible(
        view(),
        move || props.progress.get().reading_in(props.shape.get()),
        move || props.label.get(),
    )
    .w(size)
    .h(height)
    .child(layer(first))
    .child(layer(track))
    .child(layer(last))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::f32::consts::TAU;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::animation::AnimationOp;
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num, PathOp};
    use nsh_core::geometry::{Bounds, Percentage, Size, percentage};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, AccessibilityValue, Color, MountItem, NodeId, ViewKind};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::androidx_probe::{MOTION_TOKENS_KT, PROGRESS_INDICATOR_KT, line as kotlin_line};
    use crate::animation_probe::{REPLAY_HZ, Recording, Replay, layout_items};
    use crate::color::{ColorRole, ColorScheme};
    use crate::keyframe_probe::{Step, Track, cubic_bezier, started_composites, started_tracks};
    use crate::morph_probe::replay;

    /// `a` and `b` are the same to a thousandth of a point: the `f32` sums a fraction goes through.
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    // ---- the geometry, from androidx's arithmetic -------------------------------------------------------------

    /// A round line with these ends.
    fn round(from: f32, to: f32) -> Line {
        Line {
            from,
            to,
            cap: Cap::Round,
        }
    }

    /// `line` with a tolerance on its ends: `Some` with its ends and cap close to `expected`'s, or `None`.
    fn same_line(line: Option<Line>, expected: Option<Line>, what: &str) {
        match (line, expected) {
            (Some(line), Some(expected)) => assert!(
                near(line.from, expected.from)
                    && near(line.to, expected.to)
                    && line.cap == expected.cap,
                "{what}: {line:?} is not {expected:?}"
            ),
            (None, None) => {}
            (line, expected) => panic!("{what}: {line:?} is not {expected:?}"),
        }
    }

    fn assert_determinate(
        progress: f32,
        ltr: bool,
        track: Option<Line>,
        indicator: Option<Line>,
        what: &str,
    ) {
        let drawn = determinate(progress, 240.0, 4.0, 4.0, ltr);
        same_line(drawn.track, track, &format!("{what}: track"));
        same_line(drawn.indicator, indicator, &format!("{what}: indicator"));
    }

    #[test]
    fn a_240_point_indicator_draws_the_track_and_the_indicator_as_androidx_does() {
        // Width 240, height 4, gap 4: round caps, so the gap is 4 + 4 = 8 points, a fraction 1/30 of the width, and
        // the lines' ends stay 2 points (half the stroke) inside the indicator (`LinearProgressIndicator`,
        // `drawLinearIndicator`).
        // Nothing done: the track from 0 + min(0, 1/30) = 0, the whole width, and no indicator (a round line of no
        // length is not drawn).
        assert_determinate(0.0, true, Some(round(2.0, 238.0)), None, "0");
        // Half: the indicator from 0 to 120, which its cap takes to 122; the track from 0.5 + 1/30 of the width, 128,
        // so that 4 points are left between the caps (122 to 126).
        assert_determinate(
            0.5,
            true,
            Some(round(128.0, 238.0)),
            Some(round(2.0, 120.0)),
            "0.5",
        );
        // Less than the gap: the track starts at twice the progress, 0.01 + min(0.01, 1/30) = 0.02, not at its gap.
        assert_determinate(
            0.01,
            true,
            Some(round(4.8, 238.0)),
            Some(round(2.0, 2.4)),
            "0.01",
        );
        // The track runs out when its start passes the end: 0.96 + 1/30 = 0.99333 is not past it, and ends where it
        // starts, a dot; 0.97 + 1/30 = 1.00333 is, and nothing is drawn.
        assert_determinate(
            0.96,
            true,
            Some(round(238.0, 238.0)),
            Some(round(2.0, 230.4)),
            "0.96",
        );
        assert_determinate(0.97, true, None, Some(round(2.0, 232.8)), "0.97");
        // All done.
        assert_determinate(1.0, true, None, Some(round(2.0, 238.0)), "1");
    }

    #[test]
    fn a_progress_outside_the_range_draws_as_the_nearest_end() {
        for below in [-0.5, -1.0, f32::NEG_INFINITY] {
            assert_determinate(below, true, Some(round(2.0, 238.0)), None, "below");
        }
        for above in [1.5, 2.0, f32::INFINITY] {
            assert_determinate(above, true, None, Some(round(2.0, 238.0)), "above");
        }
    }

    #[test]
    fn a_progress_that_is_not_a_number_draws_neither_the_track_nor_the_indicator() {
        // androidx's comparisons with a NaN are false: `trackStartFraction <= 1f` and `abs(end - start) > 0`.
        assert_determinate(f32::NAN, true, None, None, "NaN");
        assert_determinate(f32::NAN, false, None, None, "NaN, right to left");
    }

    #[test]
    fn right_to_left_draws_from_the_right() {
        // The same lines mirrored: `barStart = (1 - end) * width` and `barEnd = (1 - start) * width`.
        assert_determinate(0.0, false, Some(round(2.0, 238.0)), None, "0");
        assert_determinate(
            0.5,
            false,
            Some(round(2.0, 112.0)),
            Some(round(120.0, 238.0)),
            "0.5",
        );
        assert_determinate(
            0.96,
            false,
            Some(round(2.0, 2.0)),
            Some(round(9.6, 238.0)),
            "0.96",
        );
        assert_determinate(1.0, false, None, Some(round(2.0, 238.0)), "1");
    }

    #[test]
    fn an_indicator_narrower_than_it_is_tall_has_butt_lines_and_the_gap_as_it_is() {
        // Height 4 is more than width 3: no room for round caps, so `drawLine` draws butt lines with no inset, and
        // `adjustedGapSize` is the gap itself, 4 points, a fraction 4/3 of the width.
        let butt = |from: f32, to: f32| {
            Some(Line {
                from,
                to,
                cap: Cap::Butt,
            })
        };
        let drawn = determinate(0.5, 3.0, 4.0, 4.0, true);
        // 0.5 + min(0.5, 4/3) = 1: the track starts at the end.
        same_line(drawn.track, butt(3.0, 3.0), "track");
        same_line(drawn.indicator, butt(0.0, 1.5), "indicator");
        // A butt line of no length is drawn, as androidx's `drawLine` is called with it.
        let none = determinate(0.0, 3.0, 4.0, 4.0, true);
        same_line(none.indicator, butt(0.0, 0.0), "no progress");
        // As wide as it is tall, the caps fit, and the lines stay inside by half the stroke on both sides.
        let square = determinate(0.5, 4.0, 4.0, 4.0, true);
        same_line(square.indicator, Some(round(2.0, 2.0)), "square");
        assert_eq!(cap(4.0, 4.0), Cap::Round);
        assert_eq!(cap(3.9, 4.0), Cap::Butt);
        assert_eq!(cap(240.0, 4.0), Cap::Round);
    }

    #[test]
    fn a_line_is_drawn_from_the_end_it_starts_at() {
        assert_eq!(
            line(0.25, 0.75, 100.0, 4.0, true).map(|l| (l.from, l.to)),
            Some((25.0, 75.0))
        );
        assert_eq!(
            line(0.25, 0.75, 100.0, 4.0, false).map(|l| (l.from, l.to)),
            Some((25.0, 75.0))
        );
        assert_eq!(
            line(0.0, 0.5, 100.0, 4.0, true).map(|l| (l.from, l.to)),
            Some((2.0, 50.0))
        );
        assert_eq!(
            line(0.0, 0.5, 100.0, 4.0, false).map(|l| (l.from, l.to)),
            Some((50.0, 98.0))
        );
        // A line of no length, or that goes backwards, with a round cap: none for no length, and a backwards one is
        // drawn from where it starts to where it ends (`abs`).
        assert_eq!(line(0.5, 0.5, 100.0, 4.0, true), None);
        assert_eq!(
            line(0.75, 0.25, 100.0, 4.0, true).map(|l| (l.from, l.to)),
            Some((75.0, 25.0))
        );
        assert_eq!(line(f32::NAN, 0.5, 100.0, 4.0, true), None);
        assert_eq!(
            line(0.0, f32::INFINITY, 100.0, 4.0, true).map(|l| (l.from, l.to)),
            Some((2.0, 98.0))
        );
    }

    #[test]
    fn a_line_with_an_end_or_a_size_that_is_not_finite_draws_nothing() {
        // A butt line (narrower than it is tall) takes its ends as the fractions give them, so a fraction that is
        // infinite or not a number is an end that is, at either end of the line.
        assert_eq!(line(0.0, f32::INFINITY, 3.0, 4.0, true), None);
        assert_eq!(line(f32::NEG_INFINITY, 0.5, 3.0, 4.0, true), None);
        assert_eq!(line(0.0, f32::NAN, 3.0, 4.0, true), None);
        assert_eq!(line(f32::NAN, 0.5, 3.0, 4.0, true), None);
        // The same line with ends that are numbers is drawn.
        assert_eq!(
            line(0.0, 0.5, 3.0, 4.0, true).map(|l| (l.from, l.to, l.cap)),
            Some((0.0, 1.5, Cap::Butt))
        );
        // An infinite height with a finite width: the ends are numbers (butt, as it is taller than wide), the size is
        // not.
        assert_eq!(line(0.0, 0.5, 100.0, f32::INFINITY, true), None);
        assert_eq!(line(0.0, 0.5, 100.0, f32::NAN, true), None);
        // And an infinite width.
        assert_eq!(line(0.0, 0.5, f32::INFINITY, 4.0, true), None);
        assert_eq!(line(0.0, 0.5, f32::NAN, 4.0, true), None);
    }

    #[test]
    fn the_stop_sits_against_the_end_of_the_track() {
        // 4 points in 4: no room to spare, so the dot's edge is the indicator's end.
        let at = |size: f32, width: f32, height: f32, ltr: bool| {
            let stop = stop(size, width, height, ltr);
            (stop.x, stop.y, stop.radius)
        };
        assert_eq!(at(4.0, 240.0, 4.0, true), (238.0, 2.0, 2.0));
        assert_eq!(at(4.0, 240.0, 4.0, false), (2.0, 2.0, 2.0));
        // A taller indicator moves the stop in by half what is spare: (10 - 4) / 2 = 3, and at most 6 points.
        assert_eq!(at(4.0, 240.0, 10.0, true), (235.0, 5.0, 2.0));
        assert_eq!(at(4.0, 240.0, 10.0, false), (5.0, 5.0, 2.0));
        assert_eq!(at(4.0, 240.0, 24.0, true), (232.0, 12.0, 2.0));
        assert_eq!(at(4.0, 240.0, 30.0, true), (232.0, 15.0, 2.0));
        // The stop cannot be bigger than the indicator is tall.
        assert_eq!(at(20.0, 240.0, 10.0, true), (235.0, 5.0, 5.0));
    }

    #[test]
    fn every_literal_is_its_line_in_androidx() {
        assert_eq!(DEFAULT_WIDTH, Points(240.0));
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 1030),
            "internal val LinearIndicatorWidth"
        );
        assert_eq!(kotlin_line(PROGRESS_INDICATOR_KT, 1031), "get() = 240.dp");
        assert_eq!(STOP_TRAILING_SPACE, Points(6.0));
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 1037),
            "internal val StopIndicatorTrailingSpace"
        );
        assert_eq!(kotlin_line(PROGRESS_INDICATOR_KT, 1038), "get() = 6.dp");
        // The tokens the sizes come from, and where androidx reads them.
        assert_eq!(tokens::HEIGHT, Points(4.0));
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 1035),
            "get() = LinearProgressIndicatorTokens.Height"
        );
        assert_eq!(tokens::STOP_SIZE, Points(4.0));
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 850),
            "public val LinearTrackStopIndicatorSize: Dp = LinearProgressIndicatorTokens.StopSize"
        );
        assert_eq!(tokens::TRACK_ACTIVE_SPACE, Points(4.0));
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 853),
            "public val LinearIndicatorTrackGapSize: Dp = LinearProgressIndicatorTokens.TrackActiveSpace"
        );
        // Round caps by default, and the arithmetic the geometry repeats.
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 841),
            "public val LinearStrokeCap: StrokeCap = StrokeCap.Round"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 155),
            "val coercedProgress = { progress().fastCoerceIn(0f, 1f) }"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 168),
            "if (strokeCap == StrokeCap.Butt || size.height > size.width) {"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 171),
            "gapSize + strokeWidth.toDp()"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 178),
            "currentCoercedProgress + min(currentCoercedProgress, gapSizeFraction)"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 179),
            "if (trackStartFraction <= 1f) {"
        );
        assert_eq!(
            kotlin_line(PROGRESS_INDICATOR_KT, 918),
            "val stopOffset = ((size.height - adjustedStopSize) / 2).fastCoerceAtMost(maxStopOffset)"
        );
    }

    // ---- the component, mounted ---------------------------------------------------------------------------------

    /// What the probe places: a linear progress indicator with these props, and lines that change its progress.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        progress: Prop<Progress>,
        #[prop(default)]
        shape: Prop<ProgressShape>,
        #[prop(default = DEFAULT_WIDTH)]
        width: Prop<Points>,
        label: Prop<Option<SharedString>>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let progress = signal(props.progress.get_untracked());
        let shape = signal(props.shape.get_untracked());
        let shown = signal(true);
        // A row, so that the indicator keeps its own size, not stretched across a column.
        view()
            .child(view().flex_row().child(when(shown, move || {
                linear_progress_indicator()
                    .progress(move || progress.get())
                    .shape(move || shape.get())
                    .width(props.width)
                    .label(props.label)
            })))
            .child(text("Half").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(0.5)));
            }))
            .child(text("Full").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(1.0)));
            }))
            .child(text("Low").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(0.05)));
            }))
            .child(text("Nan").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(f32::NAN)));
            }))
            .child(text("Waves").on_press(move |(): &()| shape.set(ProgressShape::Wavy)))
            .child(text("Straight").on_press(move |(): &()| shape.set(ProgressShape::Flat)))
            .child(text("Loop").on_press(move |(): &()| progress.set(Progress::Indeterminate)))
            .child(text("Remove").on_press(move |(): &()| shown.set(false)))
            .child(text("ready"))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        progress: Progress,
        shape: ProgressShape,
        width: Points,
        label: Option<&'static str>,
        dark: bool,
        rtl: bool,
        reduce_motion: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                progress: Progress::Determinate(percentage(0.0)),
                shape: ProgressShape::Flat,
                width: DEFAULT_WIDTH,
                label: None,
                dark: false,
                rtl: false,
                reduce_motion: false,
            }
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            progress: Prop::new(config.progress),
            shape: Prop::new(config.shape),
            width: Prop::new(config.width),
            label: Prop::new(config.label.map(SharedString::from)),
        });
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            rtl: config.rtl,
            reduce_motion: config.reduce_motion,
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:ready").unwrap();
        (app, recording)
    }

    /// The indicator's views, and the last frame and drawing each was sent.
    struct Mounted {
        /// The progress bar: the whole indicator.
        root: NodeId,
        /// The canvas at the back, which draws line 1 of an indeterminate indicator.
        first: NodeId,
        /// The canvas that draws the track.
        track: NodeId,
        /// The canvas at the front, which draws the indicator and the stop, or line 2 of an indeterminate indicator.
        indicator: NodeId,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        label: Option<Arc<str>>,
        value: Option<AccessibilityValue>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut root = None;
            let mut label = None;
            let mut value = None;
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        kinds.insert(*id, *kind);
                    }
                    MountItem::Insert {
                        parent,
                        child,
                        before,
                    } => {
                        let siblings: &mut Vec<NodeId> = children.entry(*parent).or_default();
                        let at = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(at, *child);
                    }
                    MountItem::ViewProps { id, props }
                        if props.accessibility.role == AccessibilityRole::Progressbar =>
                    {
                        root = Some(*id);
                        label = props.accessibility.label.clone();
                        value = props.accessibility.value.clone();
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    MountItem::Canvas { id, drawing } => {
                        drawings.insert(*id, Arc::clone(drawing));
                    }
                    _ => {}
                }
            }
            let root = root.expect("a view with the progress bar role");
            let canvases: Vec<NodeId> = children
                .get(&root)
                .into_iter()
                .flatten()
                .filter(|child| kinds.get(child) == Some(&ViewKind::Canvas))
                .copied()
                .collect();
            // The first line at the back, the track, the last line at the front; and nothing else in the progress bar.
            let [first, track, indicator] = canvases[..] else {
                panic!("three canvases in the progress bar: {items:?}");
            };
            assert_eq!(children.get(&root).map(Vec::len), Some(3), "{items:?}");
            Self {
                root,
                first,
                track,
                indicator,
                frames,
                drawings,
                label,
                value,
            }
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// The indicator's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.root].size
        }
    }

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// A stroked line of a drawing: its ends and height, its width and cap, and its color.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Stroked {
        from: (f32, f32),
        to: (f32, f32),
        width: f32,
        cap: LineCap,
        color: Color,
    }

    /// A filled dot of a drawing: its center, radius and color.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Dot {
        x: f32,
        y: f32,
        radius: f32,
        color: Color,
    }

    /// The constant `num` is.
    fn constant(num: Num) -> f32 {
        match num {
            Num::Const(value) => value,
            Num::Slot(slot) => panic!("slot {slot} where a constant was drawn"),
        }
    }

    /// The lines `ops` stroke, in order, and the circles they fill, each with its color.
    fn drawn(ops: &[CanvasOp]) -> (Vec<Stroked>, Vec<Dot>) {
        let (mut strokes, mut dots) = (Vec::new(), Vec::new());
        let (mut width, mut cap) = (1.0, LineCap::Butt);
        let (mut stroke, mut fill) = (Color(0), Color(0));
        let mut path: Vec<PathOp> = Vec::new();
        for op in ops {
            match op {
                CanvasOp::LineWidth(w) => width = constant(*w),
                CanvasOp::LineCap(c) => cap = *c,
                CanvasOp::StrokeStyle(canvas::Paint::Color(color)) => stroke = *color,
                CanvasOp::FillStyle(canvas::Paint::Color(color)) => fill = *color,
                CanvasOp::BeginPath => path.clear(),
                CanvasOp::Path(step) => path.push(step.clone()),
                CanvasOp::Stroke(None) => {
                    let [PathOp::MoveTo(x0, y0), PathOp::LineTo(x1, y1)] = &path[..] else {
                        panic!("a stroke of one line: {path:?}");
                    };
                    strokes.push(Stroked {
                        from: (constant(*x0), constant(*y0)),
                        to: (constant(*x1), constant(*y1)),
                        width,
                        cap,
                        color: stroke,
                    });
                }
                CanvasOp::Fill(None, _) => {
                    let [
                        PathOp::Arc {
                            x,
                            y,
                            radius,
                            start,
                            end,
                            ccw: false,
                        },
                    ] = &path[..]
                    else {
                        panic!("a fill of one circle: {path:?}");
                    };
                    assert_eq!((constant(*start), constant(*end)), (0.0, TAU));
                    dots.push(Dot {
                        x: constant(*x),
                        y: constant(*y),
                        radius: constant(*radius),
                        color: fill,
                    });
                }
                other => panic!("an operation the indicator does not draw with: {other:?}"),
            }
        }
        (strokes, dots)
    }

    fn color(hex: u32) -> Color {
        Color((hex << 8) | 255)
    }

    fn scheme_color(role: ColorRole, dark: bool) -> Color {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        color(scheme.get(role).to_hex())
    }

    /// What the canvases of a mounted indicator draw.
    struct Drawn {
        /// The lines of the canvas at the back.
        first: Vec<Stroked>,
        /// The track's lines.
        track: Vec<Stroked>,
        /// The indicator's lines.
        indicator: Vec<Stroked>,
        /// The dots over them.
        dots: Vec<Dot>,
    }

    fn drawn_by(mounted: &Mounted) -> Drawn {
        let (first, no_dots) = drawn(&mounted.drawing(mounted.first).ops);
        assert_eq!(no_dots, Vec::new(), "the first line has no dot");
        let (track, no_dots) = drawn(&mounted.drawing(mounted.track).ops);
        assert_eq!(no_dots, Vec::new(), "the track has no dot");
        let (indicator, dots) = drawn(&mounted.drawing(mounted.indicator).ops);
        Drawn {
            first,
            track,
            indicator,
            dots,
        }
    }

    /// `strokes` is one round line of the indicator's 4-point stroke, along its middle, from `from` to `to`, in
    /// `color`.
    fn assert_line(strokes: &[Stroked], from: f32, to: f32, color: Color, what: &str) {
        let [only] = strokes else {
            panic!("{what}: one line, not {strokes:?}");
        };
        assert!(
            near(only.from.0, from) && near(only.to.0, to),
            "{what}: {only:?} is not {from} to {to}"
        );
        assert_eq!((only.from.1, only.to.1), (2.0, 2.0), "{what}");
        assert_eq!(only.width, 4.0, "{what}");
        assert_eq!(only.cap, LineCap::Round, "{what}");
        assert_eq!(only.color, color, "{what}");
    }

    fn determinate_at(fraction: f32) -> Config {
        Config {
            progress: Progress::Determinate(percentage(fraction)),
            ..Config::default()
        }
    }

    #[test]
    fn the_track_and_the_indicator_are_drawn_as_the_geometry_says_in_the_token_colors() {
        for dark in [false, true] {
            let track = scheme_color(ColorRole::SecondaryContainer, dark);
            let primary = scheme_color(ColorRole::Primary, dark);
            for (fraction, track_line, indicator_line) in [
                (0.0, Some((2.0, 238.0)), None),
                (0.5, Some((128.0, 238.0)), Some((2.0, 120.0))),
                (0.97, None, Some((2.0, 232.8))),
                (1.0, None, Some((2.0, 238.0))),
            ] {
                let what = format!("{fraction} dark: {dark}");
                let mounted = mount(Config {
                    dark,
                    ..determinate_at(fraction)
                });
                let drawn = drawn_by(&mounted);
                // A determinate indicator has no first line: the canvas at the back is empty.
                assert_eq!(drawn.first, Vec::new(), "{what}");
                match track_line {
                    Some((from, to)) => assert_line(&drawn.track, from, to, track, &what),
                    None => assert_eq!(drawn.track, Vec::new(), "{what}"),
                }
                match indicator_line {
                    Some((from, to)) => assert_line(&drawn.indicator, from, to, primary, &what),
                    None => assert_eq!(drawn.indicator, Vec::new(), "{what}"),
                }
                // The stop, always, in the indicator's color: a 4-point dot against the end of the track.
                assert_eq!(
                    drawn.dots,
                    [Dot {
                        x: 238.0,
                        y: 2.0,
                        radius: 2.0,
                        color: primary
                    }],
                    "{what}"
                );
            }
        }
    }

    #[test]
    fn the_indicator_is_240_by_4_points_unless_it_is_given_a_width() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.size(), Size::new(Points(240.0), Points(4.0)));
        // The canvases cover it.
        for id in [mounted.track, mounted.indicator] {
            assert_eq!(
                mounted.frames[&id],
                Bounds::new(point(px(0.0), px(0.0)), mounted.size())
            );
        }
        let narrow = mount(Config {
            width: Points(100.0),
            progress: Progress::Determinate(percentage(0.5)),
            ..Config::default()
        });
        assert_eq!(narrow.size(), Size::new(Points(100.0), Points(4.0)));
        let drawn = drawn_by(&narrow);
        // 100 points: the gap is 8 points, 0.08 of the width; 0.5 + 0.08 = 0.58 of 100.
        assert_line(
            &drawn.indicator,
            2.0,
            50.0,
            scheme_color(ColorRole::Primary, false),
            "narrow",
        );
        assert_line(
            &drawn.track,
            58.0,
            98.0,
            scheme_color(ColorRole::SecondaryContainer, false),
            "narrow",
        );
        assert_eq!(drawn.dots.len(), 1);
        assert!(near(drawn.dots[0].x, 98.0), "{:?}", drawn.dots);
    }

    #[test]
    fn right_to_left_mirrors_the_lines_and_the_stop() {
        let mounted = mount(Config {
            rtl: true,
            ..determinate_at(0.5)
        });
        let drawn = drawn_by(&mounted);
        assert_line(
            &drawn.track,
            2.0,
            112.0,
            scheme_color(ColorRole::SecondaryContainer, false),
            "track",
        );
        assert_line(
            &drawn.indicator,
            120.0,
            238.0,
            scheme_color(ColorRole::Primary, false),
            "indicator",
        );
        assert_eq!(
            drawn
                .dots
                .iter()
                .map(|dot| (dot.x, dot.y))
                .collect::<Vec<_>>(),
            [(2.0, 2.0)]
        );
    }

    fn range(mounted: &Mounted) -> Option<(Option<f32>, Option<f32>, Option<f32>)> {
        mounted
            .value
            .as_ref()
            .map(|value| (value.min, value.max, value.now))
    }

    #[test]
    fn screen_readers_get_a_progress_bar_from_0_to_1_with_the_progress_as_its_value() {
        let reading = |progress: Progress| {
            let mounted = mount(Config {
                progress,
                ..Config::default()
            });
            assert_eq!(mounted.value.as_ref().and_then(|v| v.text.clone()), None);
            range(&mounted)
        };
        let determinate = |fraction: f32| reading(Progress::Determinate(percentage(fraction)));
        assert_eq!(determinate(0.4), Some((Some(0.0), Some(1.0), Some(0.4))));
        assert_eq!(determinate(0.0), Some((Some(0.0), Some(1.0), Some(0.0))));
        // Out of range, held to it; a NaN read as 0.
        assert_eq!(determinate(3.0), Some((Some(0.0), Some(1.0), Some(1.0))));
        assert_eq!(determinate(-3.0), Some((Some(0.0), Some(1.0), Some(0.0))));
        assert_eq!(
            determinate(f32::NAN),
            Some((Some(0.0), Some(1.0), Some(0.0)))
        );
        // Indeterminate: the range and no value.
        assert_eq!(
            reading(Progress::Indeterminate),
            Some((Some(0.0), Some(1.0), None))
        );
    }

    #[test]
    fn screen_readers_name_it_by_its_label_when_it_has_one() {
        assert_eq!(mount(Config::default()).label, None);
        let labelled = mount(Config {
            label: Some("Loading"),
            ..Config::default()
        });
        assert_eq!(labelled.label.as_deref(), Some("Loading"));
    }

    #[test]
    fn a_new_progress_draws_the_new_lines_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        for (name, track, indicator) in [
            ("Half", (128.0, 238.0), Some((2.0, 120.0))),
            ("Full", (0.0, 0.0), Some((2.0, 238.0))),
        ] {
            app.press(name)?;
            let step = format!("press({name})");
            let items = recording.in_step(&step);
            assert_eq!(layout_items(&items), 0, "{name}: {items:?}");
            let mounted = Mounted::of(&recording.items());
            let drawn = drawn_by(&mounted);
            // The indicator was drawn again, and the track too when it changed.
            assert!(
                items.iter().any(
                    |item| matches!(item, MountItem::Canvas { id, .. } if *id == mounted.indicator)
                ),
                "{name}: {items:?}"
            );
            match indicator {
                Some((from, to)) => assert_line(
                    &drawn.indicator,
                    from,
                    to,
                    scheme_color(ColorRole::Primary, false),
                    name,
                ),
                None => assert_eq!(drawn.indicator, Vec::new(), "{name}"),
            }
            if name == "Half" {
                assert_line(
                    &drawn.track,
                    track.0,
                    track.1,
                    scheme_color(ColorRole::SecondaryContainer, false),
                    name,
                );
            } else {
                assert_eq!(drawn.track, Vec::new(), "{name}");
            }
        }
        Ok(())
    }

    #[test]
    fn a_determinate_indicator_starts_no_animation_and_leaves_the_graph_still() {
        for fraction in [0.0, 0.5, 1.0] {
            let (mut app, recording) = start(determinate_at(fraction));
            app.step("settle");
            app.run_for(std::time::Duration::from_secs(1)).unwrap();
            let items = recording.items();
            assert!(
                !items
                    .iter()
                    .any(|item| matches!(item, MountItem::Animation(_))
                        && !matches!(item, MountItem::Animation(AnimationOp::CreateValue { .. }))),
                "{fraction}: {items:?}"
            );
            assert!(!replay(&recording).is_animating(), "{fraction}");
            assert!(recording.in_step("settle").is_empty(), "{fraction}");
        }
    }

    #[test]
    fn the_indicator_is_one_element_with_nothing_else_in_it() {
        // `Mounted::of` insists on the progress bar's three canvases and nothing else; its role is the one element.
        let mounted = mount(Config::default());
        assert_ne!(mounted.first, mounted.track);
        assert_ne!(mounted.track, mounted.indicator);
        assert_ne!(mounted.first, mounted.indicator);
        let _: Percentage = percentage(0.0);
    }

    // ---- the indeterminate motion, from androidx's keyframes ---------------------------------------------------

    /// The control points of every rise, as `MotionTokens.EasingEmphasizedAccelerateCubicBezier` spells them,
    /// `CubicBezierEasing(0.3f, 0.0f, 0.8f, 0.15f)` (`MotionTokens.kt`, lines 43 and 44).
    const ACCELERATE: ((f64, f64), (f64, f64)) = ((0.3, 0.0), (0.8, 0.15));

    /// androidx's four sweeps as `ProgressIndicator.kt` spells them (lines 1052 to 1061): the name, the delay and the
    /// duration, in milliseconds.
    const SWEEPS: [(&str, f32, f32); 4] = [
        ("first head", 0.0, 1000.0),
        ("first tail", 250.0, 1000.0),
        ("second head", 650.0, 850.0),
        ("second tail", 900.0, 850.0),
    ];

    /// The steps one iteration of each sweep takes, worked out by hand from the lines above: the value goes to 0 at
    /// once (the keyframe at the start of the cycle), waits there until its delay, rises to 1 over its duration along
    /// the curve, and holds 1 until the cycle ends, 1750 ms in. Each is where it goes, how long it takes in
    /// milliseconds, and whether the curve shapes it.
    const STEPS: [&[(f32, f32, bool)]; 4] = [
        // 0 + 1000 = 1000: held 750 ms.
        &[(0.0, 0.0, false), (1.0, 1000.0, true), (1.0, 750.0, false)],
        // 250 + 1000 = 1250: held 500 ms.
        &[
            (0.0, 0.0, false),
            (0.0, 250.0, false),
            (1.0, 1000.0, true),
            (1.0, 500.0, false),
        ],
        // 650 + 850 = 1500: held 250 ms.
        &[
            (0.0, 0.0, false),
            (0.0, 650.0, false),
            (1.0, 850.0, true),
            (1.0, 250.0, false),
        ],
        // 900 + 850 = 1750: the cycle ends as it rises.
        &[(0.0, 0.0, false), (0.0, 900.0, false), (1.0, 850.0, true)],
    ];

    /// The four sweeps of the component, in the order of [`SWEEPS`].
    const SWEEP_CONSTS: [Sweep; 4] = [FIRST_HEAD, FIRST_TAIL, SECOND_HEAD, SECOND_TAIL];

    /// What androidx's `keyframes { durationMillis = 1750; 0f at delay using easing; 1f at delay + duration }` gives
    /// `ms` into the animation, written from the Kotlin and not from the component: 0 until the delay, the curve to 1
    /// over the duration, 1 until the cycle ends, and the cycle again.
    fn keyframed(delay: f32, duration: f32, ms: f64) -> f64 {
        let at = ms % 1750.0;
        let (delay, duration) = (f64::from(delay), f64::from(duration));
        if at <= delay {
            0.0
        } else if at >= delay + duration {
            1.0
        } else {
            cubic_bezier(ACCELERATE.0, ACCELERATE.1, (at - delay) / duration)
        }
    }

    #[test]
    fn every_literal_of_the_indeterminate_motion_is_its_line_in_androidx() {
        for (number, text) in [
            (
                980,
                "internal val linearIndeterminateFirstLineHeadAnimationSpec",
            ),
            (985, "durationMillis = LinearAnimationDuration"),
            (
                986,
                "0f at FirstLineHeadDelay using LinearIndeterminateProgressEasing",
            ),
            (987, "1f at FirstLineHeadDuration + FirstLineHeadDelay"),
            (
                992,
                "internal val linearIndeterminateFirstLineTailAnimationSpec",
            ),
            (
                998,
                "0f at FirstLineTailDelay using LinearIndeterminateProgressEasing",
            ),
            (999, "1f at FirstLineTailDuration + FirstLineTailDelay"),
            (
                1004,
                "internal val linearIndeterminateSecondLineHeadAnimationSpec",
            ),
            (
                1010,
                "0f at SecondLineHeadDelay using LinearIndeterminateProgressEasing",
            ),
            (1011, "1f at SecondLineHeadDuration + SecondLineHeadDelay"),
            (
                1016,
                "internal val linearIndeterminateSecondLineTailAnimationSpec",
            ),
            (
                1022,
                "0f at SecondLineTailDelay using LinearIndeterminateProgressEasing",
            ),
            (1023, "1f at SecondLineTailDuration + SecondLineTailDelay"),
            (1049, "internal const val LinearAnimationDuration = 1750"),
            (1052, "internal const val FirstLineHeadDuration = 1000"),
            (1053, "internal const val FirstLineTailDuration = 1000"),
            (1054, "internal const val SecondLineHeadDuration = 850"),
            (1055, "internal const val SecondLineTailDuration = 850"),
            (1058, "internal const val FirstLineHeadDelay = 0"),
            (1059, "internal const val FirstLineTailDelay = 250"),
            (1060, "internal const val SecondLineHeadDelay = 650"),
            (1061, "internal const val SecondLineTailDelay = 900"),
            (
                1063,
                "internal val LinearIndeterminateProgressEasing = MotionTokens.EasingEmphasizedAccelerateCubicBezier",
            ),
        ] {
            assert_eq!(kotlin_line(PROGRESS_INDICATOR_KT, number), text);
        }
        // The curve the line 1063 names.
        assert_eq!(
            kotlin_line(MOTION_TOKENS_KT, 43),
            "inline val EasingEmphasizedAccelerateCubicBezier: CubicBezierEasing"
        );
        assert_eq!(
            kotlin_line(MOTION_TOKENS_KT, 44),
            "get() = CubicBezierEasing(0.3f, 0.0f, 0.8f, 0.15f)"
        );
        // What the component has, number for number.
        assert_eq!(CYCLE_MS.to_bits(), 1750.0_f32.to_bits());
        for (sweep, (name, delay, duration)) in SWEEP_CONSTS.into_iter().zip(SWEEPS) {
            assert_eq!(
                (sweep.delay_ms, sweep.duration_ms),
                (delay, duration),
                "{name}"
            );
        }
        let curve = (
            (SWEEP_EASING.x1, SWEEP_EASING.y1),
            (SWEEP_EASING.x2, SWEEP_EASING.y2),
        );
        assert_eq!(curve, ((0.3, 0.0), (0.8, 0.15)));
        assert_eq!(easing(SWEEP_EASING), Easing::Bezier(0.3, 0.0, 0.8, 0.15));
    }

    /// The offset, value and curve of each of `sweep`'s keyframes, the offset in milliseconds of the cycle.
    fn schedule(sweep: Sweep) -> Vec<(f32, f32, Option<Easing>)> {
        sweep
            .keyframes()
            .iter()
            .map(|frame| {
                (
                    frame.offset.map_or(f32::NAN, |offset| offset * CYCLE_MS),
                    frame.value,
                    frame.easing,
                )
            })
            .collect()
    }

    #[test]
    fn a_sweep_is_androidxs_keyframes_with_none_of_them_doubled() {
        let curve = Some(Easing::Bezier(0.3, 0.0, 0.8, 0.15));
        let expected: [Vec<(f32, f32, Option<Easing>)>; 4] = [
            // A delay of 0: the keyframe at 0 is the one at the delay, with the curve; 1 at 1000, and held to the end.
            vec![(0.0, 0.0, curve), (1000.0, 1.0, None), (1750.0, 1.0, None)],
            vec![
                (0.0, 0.0, None),
                (250.0, 0.0, curve),
                (1250.0, 1.0, None),
                (1750.0, 1.0, None),
            ],
            vec![
                (0.0, 0.0, None),
                (650.0, 0.0, curve),
                (1500.0, 1.0, None),
                (1750.0, 1.0, None),
            ],
            // 900 + 850 is the end of the cycle: the keyframe at 1750 is the one the rise ends on.
            vec![(0.0, 0.0, None), (900.0, 0.0, curve), (1750.0, 1.0, None)],
        ];
        for ((sweep, (name, ..)), expected) in SWEEP_CONSTS.into_iter().zip(SWEEPS).zip(expected) {
            let found = schedule(sweep);
            assert_eq!(found.len(), expected.len(), "{name}: {found:?}");
            for (found, expected) in found.iter().zip(&expected) {
                assert!(
                    near(found.0, expected.0),
                    "{name}: {found:?} is not {expected:?}"
                );
                assert_eq!(
                    (found.1, found.2),
                    (expected.1, expected.2),
                    "{name}: {found:?}"
                );
            }
        }
    }

    #[test]
    fn the_eased_progress_is_the_curve_the_token_names() {
        // By hand: at the parameter 0.5 a cubic Bézier is 0.375 · (first + second) + 0.125 in each coordinate, so the
        // curve through (0.3, 0), (0.8, 0.15) passes through (0.375 · 1.1 + 0.125, 0.375 · 0.15 + 0.125) =
        // (0.5375, 0.18125).
        assert!((ease(SWEEP_EASING, 0.5375) - 0.18125).abs() < 1e-5);
        // The closed form of the probe walks the curve a way that shares nothing with the bisection.
        for step in 0_u8..=40 {
            let x = f32::from(step) / 40.0;
            let expected = cubic_bezier(ACCELERATE.0, ACCELERATE.1, f64::from(step) / 40.0);
            let found = f64::from(ease(SWEEP_EASING, x));
            assert!(
                (found - expected).abs() < 1e-5,
                "{x}: {found} vs {expected}"
            );
        }
        // Held to the ends outside the range, and a NaN is the start.
        for (x, expected) in [
            (-1.0, 0.0),
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 1.0),
            (f32::NEG_INFINITY, 0.0),
            (f32::INFINITY, 1.0),
            (f32::NAN, 0.0),
        ] {
            assert_eq!(ease(SWEEP_EASING, x), expected, "{x}");
        }
        // Slow at first and fast at the end: a tenth of the way is not a hundredth of the rise, nine tenths is past
        // half of it.
        assert!(ease(SWEEP_EASING, 0.1) < 0.01);
        assert!(ease(SWEEP_EASING, 0.9) > 0.6);
    }

    /// Where the curve's time at the parameter the bisection tries is exactly the time asked for, it keeps the lower
    /// half (`<`, not `<=`). The closed form above cannot tell the two rules apart, since they differ by a few units in
    /// the last place of the result, so this pins the result to its bits: it is what `ease` gave when this was written
    /// (the same time asked for, 0.0625, gives 0.0021616826 with the other rule).
    #[test]
    fn the_bisection_keeps_the_lower_half_where_the_curve_passes_exactly_through_the_time() {
        assert_eq!(ease(SWEEP_EASING, 0.0625).to_bits(), 0x3b0d_aaff);
    }

    #[test]
    fn a_sweep_is_0_until_its_delay_and_1_from_its_end() {
        for (sweep, (name, delay, duration)) in SWEEP_CONSTS.into_iter().zip(SWEEPS) {
            assert_eq!(sweep.at(0.0), 0.0, "{name} at the start of the cycle");
            assert_eq!(sweep.at(delay - 1.0), 0.0, "{name} before its delay");
            assert_eq!(sweep.at(delay), 0.0, "{name} at its delay");
            assert_eq!(sweep.at(delay + duration), 1.0, "{name} at its end");
            assert_eq!(sweep.at(delay + duration + 100.0), 1.0, "{name} after it");
            // 0.5375 of the way, where the curve is 0.18125 (see above).
            let value = sweep.at(delay + 0.5375 * duration);
            assert!((value - 0.18125).abs() < 1e-4, "{name}: {value}");
            // And as the Kotlin keyframes give it, a little before the end of the rise too.
            let late = delay + 0.97 * duration;
            let expected = keyframed(delay, duration, f64::from(late));
            assert!(
                (f64::from(sweep.at(late)) - expected).abs() < 1e-4,
                "{name}"
            );
        }
        // The four of the cycle, `ms` in.
        let course = course_at(1000.0);
        assert_eq!(course.first_head, FIRST_HEAD.at(1000.0));
        assert_eq!(course.first_tail, FIRST_TAIL.at(1000.0));
        assert_eq!(course.second_head, SECOND_HEAD.at(1000.0));
        assert_eq!(course.second_tail, SECOND_TAIL.at(1000.0));
    }

    #[test]
    fn the_still_pose_is_the_motion_a_second_in_when_the_first_head_reaches_the_end() {
        // The instant is the one the first head's rise ends at: its delay and its duration.
        assert_eq!(
            STILL_POSE_AT_MS.to_bits(),
            (FIRST_HEAD.delay_ms + FIRST_HEAD.duration_ms).to_bits()
        );
        let pose = still_pose();
        // Worked out with the curve by bisection to 100 steps in `f64`, in the order head 1, tail 1, head 2, tail 2.
        let expected = [1.0, 0.405_585_5, 0.100_629_6, 0.007_673_52];
        let found = [
            pose.first_head,
            pose.first_tail,
            pose.second_head,
            pose.second_tail,
        ];
        for ((found, expected), (name, delay, duration)) in
            found.into_iter().zip(expected).zip(SWEEPS)
        {
            assert!(
                (found - expected).abs() < 1e-6,
                "{name}: {found} vs {expected}"
            );
            // And it is what androidx's keyframes give at that instant.
            let kotlin = keyframed(delay, duration, 1000.0);
            assert!(
                (f64::from(found) - kotlin).abs() < 1e-5,
                "{name}: {found} vs {kotlin}"
            );
        }
        // Head 1 is at the end exactly, which is what makes it the pose with the first line from the end.
        assert_eq!(pose.first_head, 1.0);
    }

    // ---- the five strokes, from androidx's arithmetic ----------------------------------------------------------

    /// A line of the 240 by 4 point indicator with a gap of 4 (a gap of 1/30 of the width), or none.
    fn drawn_lines(at: Course<f32>, width: f32, ltr: bool) -> [Option<Line>; 5] {
        let drawn = indeterminate(at, width, 4.0, 4.0, ltr);
        [
            drawn.track_before,
            drawn.first,
            drawn.track_between,
            drawn.second,
            drawn.track_after,
        ]
    }

    fn assert_strokes(found: [Option<Line>; 5], expected: [Option<(f32, f32)>; 5], what: &str) {
        for ((found, expected), name) in found.into_iter().zip(expected).zip([
            "track before line 1",
            "line 1",
            "track between",
            "line 2",
            "track after line 2",
        ]) {
            same_line(
                found,
                expected.map(|(from, to)| round(from, to)),
                &format!("{what}: {name}"),
            );
        }
    }

    fn course(first_head: f32, first_tail: f32, second_head: f32, second_tail: f32) -> Course<f32> {
        Course {
            first_head,
            first_tail,
            second_head,
            second_tail,
        }
    }

    #[test]
    fn five_strokes_are_drawn_where_androidx_draws_them() {
        // Width 240, height 4, gap 4: the gap is 4 + 4 = 8 points, 1/30 of the width, and the ends stay 2 points in.
        // The start of the cycle: nothing but the track before line 1, from the start, which is the whole track.
        assert_strokes(
            drawn_lines(course(0.0, 0.0, 0.0, 0.0), 240.0, true),
            [Some((2.0, 238.0)), None, None, None, None],
            "start",
        );
        // Both lines out: head 1 at 0.5, tail 1 at 0.25, head 2 at 0.1, tail 2 at 0.05.
        // Track before: from 0.5 + 1/30 = 0.5333 of 240, 128, to the end, 238. Line 1: 120 to 60. Track between: from
        // 0.1 + 1/30 = 0.1333, 32, to 0.25 - 1/30 = 0.2167, 52. Line 2: 24 to 12. Track after: from 0 to
        // 0.05 - 1/30 = 0.01667, 4, held 2 in from the start.
        assert_strokes(
            drawn_lines(course(0.5, 0.25, 0.1, 0.05), 240.0, true),
            [
                Some((128.0, 238.0)),
                Some((120.0, 60.0)),
                Some((32.0, 52.0)),
                Some((24.0, 12.0)),
                Some((2.0, 4.0)),
            ],
            "both lines",
        );
        // The same from the right.
        assert_strokes(
            drawn_lines(course(0.5, 0.25, 0.1, 0.05), 240.0, false),
            [
                Some((2.0, 112.0)),
                Some((180.0, 120.0)),
                Some((188.0, 208.0)),
                Some((228.0, 216.0)),
                Some((236.0, 238.0)),
            ],
            "both lines, right to left",
        );
        // The first head within a gap of the end: no track before it. 0.97 is past 1 - 1/30 = 0.9667, 0.96 is not,
        // and its track starts at 0.96 + 1/30 = 0.99333, past the end of the strokes, held 2 in from it.
        assert_strokes(
            drawn_lines(course(0.97, 0.5, 0.0, 0.0), 240.0, true),
            [None, Some((232.8, 120.0)), Some((2.0, 112.0)), None, None],
            "head 1 at 0.97",
        );
        assert_strokes(
            drawn_lines(course(0.96, 0.5, 0.0, 0.0), 240.0, true),
            [
                Some((238.0, 238.0)),
                Some((230.4, 120.0)),
                Some((2.0, 112.0)),
                None,
                None,
            ],
            "head 1 at 0.96",
        );
        // A tail within a gap of the start: no track after it, or between the lines; a second line whose head is not
        // past its tail has no length.
        assert_strokes(
            drawn_lines(course(0.5, 0.03, 0.0, 0.03), 240.0, true),
            [Some((128.0, 238.0)), Some((120.0, 7.2)), None, None, None],
            "tails at 0.03",
        );
        // The end of the cycle: every value at 1. Line 1 and line 2 have no length; the track between ends where it
        // starts (the second head's gap past the end, held in), and the track after is the whole width.
        assert_strokes(
            drawn_lines(course(1.0, 1.0, 1.0, 1.0), 240.0, true),
            [None, None, Some((238.0, 238.0)), None, Some((2.0, 238.0))],
            "end",
        );
    }

    #[test]
    fn androidxs_conditions_are_strict_at_their_thresholds() {
        // `gapSizeFraction` of the 240 point indicator, as the `f32` the geometry computes: (4 + 4) / 240.
        let gap = 8.0_f32 / 240.0;
        // The track before line 1 is drawn while the head is below 1 - gap: at it, not.
        let at = drawn_lines(course(1.0 - gap, 0.5, 0.0, 0.0), 240.0, true);
        assert_eq!(at[0], None, "head 1 at 1 - gap");
        // The track between the lines and the track after line 2 are drawn while a tail is above the gap: at it, not.
        let at = drawn_lines(course(0.5, gap, 0.0, gap), 240.0, true);
        assert_eq!((at[2], at[4]), (None, None), "tails at the gap");
        // A head at 0 has no gap before it: the track between starts at 0, held 2 in from the start; a head past
        // 0 does, by the gap.
        let at = drawn_lines(course(0.5, 0.5, 0.0, 0.0), 240.0, true);
        same_line(at[2], Some(round(2.0, 112.0)), "head 2 at 0");
        // A tail at 1 has no gap after it: the track between ends at 1, held 2 in from the end.
        let at = drawn_lines(course(1.0, 1.0, 0.5, 0.5), 240.0, true);
        same_line(at[2], Some(round(120.0 + 8.0, 238.0)), "tail 1 at 1");
        // A line whose head is not past its tail has none, round or butt: at the end of the cycle neither is drawn.
        for width in [240.0, 3.0] {
            let at = drawn_lines(course(1.0, 1.0, 1.0, 1.0), width, true);
            assert_eq!((at[1], at[3]), (None, None), "{width} wide, all at 1");
            let at = drawn_lines(course(0.5, 0.5, 0.25, 0.25), width, true);
            assert_eq!((at[1], at[3]), (None, None), "{width} wide, heads at tails");
        }
    }

    #[test]
    fn androidxs_conditions_are_strict_at_their_thresholds_for_butt_lines_too() {
        // A butt line of no length is drawn (androidx calls `drawLine` with it), so a condition that held one step
        // too far would show here as a stroke of no length; with round caps it shows as nothing. A butt indicator
        // 10 points wide and 12 tall with a gap of 2 has a gap of 2/10, as the `f32` the geometry computes.
        let gap = 2.0_f32 / 10.0;
        let strokes = |at: Course<f32>| {
            let drawn = indeterminate(at, 10.0, 12.0, 2.0, true);
            [drawn.track_before, drawn.track_between, drawn.track_after]
        };
        // The track before line 1 is drawn while the head is below 1 - gap: at it, not, and below it, it is.
        let at = strokes(course(1.0 - gap, 0.5, 0.0, 0.0));
        assert_eq!(at[0], None, "head 1 at 1 - gap");
        let below = strokes(course(0.75, 0.5, 0.0, 0.0));
        let line = below[0].expect("track before line 1, head 1 below 1 - gap");
        assert_eq!(line.cap, Cap::Butt);
        assert!(near(line.from, (0.75 + gap) * 10.0) && near(line.to, 10.0));
        // The track between the lines is drawn while the first tail is above the gap: at it, not, above it, it is.
        let at = strokes(course(0.5, gap, 0.0, 0.5));
        assert_eq!(at[1], None, "tail 1 at the gap");
        let above = strokes(course(0.5, 0.5, 0.0, 0.5));
        let line = above[1].expect("track between, tail 1 above the gap");
        assert!(near(line.from, 0.0) && near(line.to, (0.5 - gap) * 10.0));
        // The track after line 2 is drawn while the second tail is above the gap: at it, not, above it, it is.
        let at = strokes(course(0.5, 0.5, 0.0, gap));
        assert_eq!(at[2], None, "tail 2 at the gap");
        let above = strokes(course(0.5, 0.5, 0.0, 0.5));
        let line = above[2].expect("track after line 2, tail 2 above the gap");
        assert!(near(line.from, 0.0) && near(line.to, (0.5 - gap) * 10.0));
    }

    #[test]
    fn a_line_narrower_than_it_is_tall_is_drawn_butt_with_the_gap_as_it_is() {
        // Width 3, height 4: no inset, a gap of 4 points, 4/3 of the width, so no track fits before line 1, between
        // the lines or after line 2; the lines are the fractions of 3 points as they are.
        let found = drawn_lines(course(0.5, 0.25, 0.1, 0.05), 3.0, true);
        assert_eq!(found[0], None);
        assert_eq!(found[2], None);
        assert_eq!(found[4], None);
        let butt = |line: Option<Line>| line.map(|line| (line.from, line.to, line.cap));
        assert_eq!(butt(found[1]), Some((1.5, 0.75, Cap::Butt)));
        let second = butt(found[3]).expect("line 2");
        assert!(near(second.0, 0.3) && near(second.1, 0.15) && second.2 == Cap::Butt);
    }

    #[test]
    fn the_strokes_never_touch_across_a_gap_that_the_layers_would_draw_in_another_order() {
        // androidx draws, in order: track before, line 1, track between, line 2, track after. The three canvases draw
        // line 1, then the three tracks, then line 2, which only changes what is on top where a track before line 1
        // overlaps line 1 (it is drawn over it, as androidx draws it under) or a track after line 2 overlaps line 2
        // (it is under it, as androidx draws it over). Neither does, at any instant, for any width.
        let reach = |line: Line| match line.cap {
            Cap::Butt => 0.0,
            Cap::Round => 2.0,
        };
        let span = |line: Line| {
            (
                line.from.min(line.to) - reach(line),
                line.from.max(line.to) + reach(line),
            )
        };
        let overlap = |a: Line, b: Line| {
            let (a, b) = (span(a), span(b));
            a.1.min(b.1) - a.0.max(b.0)
        };
        let mut between_over_first = 0_u32;
        for width in [
            3.0, 4.0, 5.0, 8.0, 12.0, 16.0, 24.0, 48.0, 100.0, 240.0, 480.0, 1000.0,
        ] {
            for ltr in [true, false] {
                for tenth in 0_u16..17_500 {
                    let at = course_at(f32::from(tenth) / 10.0);
                    let [before, first, between, second, after] = drawn_lines(at, width, ltr);
                    if let (Some(a), Some(b)) = (before, first) {
                        assert!(
                            overlap(a, b) < 1e-3,
                            "{width} wide, {tenth}: {a:?} over {b:?}"
                        );
                    }
                    if let (Some(a), Some(b)) = (after, second) {
                        assert!(
                            overlap(a, b) < 1e-3,
                            "{width} wide, {tenth}: {a:?} under {b:?}"
                        );
                    }
                    // Where they do overlap, as the layers draw them: the track between over line 1, under line 2.
                    if let (Some(a), Some(b)) = (between, first)
                        && overlap(a, b) > 1e-3
                    {
                        between_over_first += 1;
                    }
                }
            }
        }
        // So the order that the check above leaves out is one that matters: it does happen.
        assert!(between_over_first > 0);
    }

    // ---- the gates: the `if`s of androidx as numbers of the drawing --------------------------------------------

    #[derive(Debug, Clone, PartialEq, Props)]
    struct GatesProps {
        #[prop(default = Points(0.25))]
        gap: Prop<Points>,
    }

    /// Draws the six numbers that [`Gate`] reads: each gate of the indicator on one animated value.
    #[component]
    fn Gates(props: GatesProps) -> impl IntoElement {
        let gap = props.gap.get_untracked().0;
        let value = animated_value(0.0);
        view()
            .child(
                canvas(move || {
                    let x = value.node();
                    Painter::new()
                        .begin_path()
                        .move_to(&step(x), &above(x, 0.5))
                        .line_to(&below(x, 0.5), &nonzero(x))
                        .move_to(&lead(x, gap), &trail(x, gap))
                })
                .w(px(10.0))
                .h(px(10.0)),
            )
            .child(text("ready"))
    }

    /// The gates of the indicator on a value that is set by hand and read in NSH's animation graph, with a gap of a
    /// quarter and thresholds at one half: `step`, `above`, `below`, `nonzero`, `lead` and `trail`, in that order.
    struct Gate {
        replay: Replay,
        value: NodeId,
        outputs: [NodeId; 6],
    }

    impl Gate {
        fn new() -> Self {
            let root = Root::new::<Gates>(|| GatesProps {
                gap: Prop::new(Points(0.25)),
            });
            let mut app = Headless::start(EngineConfig {
                clock: Clock::manual(SystemTime::UNIX_EPOCH),
                ..EngineConfig::new(root)
            })
            .unwrap();
            let recording = Recording::attach(&mut app);
            app.wait_for("text:ready").unwrap();
            let items = recording.items();
            let value = items
                .iter()
                .find_map(|item| match item {
                    MountItem::Animation(AnimationOp::CreateValue { node, .. }) => Some(*node),
                    _ => None,
                })
                .expect("the animated value");
            let drawing = items
                .iter()
                .find_map(|item| match item {
                    MountItem::Canvas { drawing, .. } => Some(Arc::clone(drawing)),
                    _ => None,
                })
                .expect("a drawing");
            let numbers: Vec<Num> = drawing
                .ops
                .iter()
                .filter_map(|op| match op {
                    CanvasOp::Path(PathOp::MoveTo(x, y) | PathOp::LineTo(x, y)) => Some([*x, *y]),
                    _ => None,
                })
                .flatten()
                .collect();
            let outputs: Vec<NodeId> = numbers
                .iter()
                .map(|num| match num {
                    Num::Slot(slot) => drawing.slots[usize::from(*slot)],
                    Num::Const(value) => panic!("a constant {value} where a gate was drawn"),
                })
                .collect();
            let mut replay = Replay::default();
            replay.apply(&items);
            Self {
                replay,
                value,
                outputs: outputs.try_into().expect("six gates"),
            }
        }

        /// The six gates with the value at `x`.
        fn at(&mut self, x: f32) -> [f32; 6] {
            self.replay.apply(&[MountItem::Animation(AnimationOp::Set {
                node: self.value,
                value: x,
            })]);
            self.outputs.map(|node| self.replay.value(node))
        }
    }

    /// `found` is `expected` to a millionth.
    fn assert_gates(found: [f32; 6], expected: [f32; 6], what: &str) {
        let same = found
            .iter()
            .zip(expected)
            .all(|(found, expected)| (found - expected).abs() < 1e-6);
        assert!(same, "{what}: {found:?} is not {expected:?}");
    }

    #[test]
    fn the_ramps_are_narrower_than_a_frame_can_show_and_wider_than_the_noise_of_a_fraction() {
        let (ramp, zero) = (std::hint::black_box(RAMP), std::hint::black_box(ZERO_RAMP));
        assert_eq!((ramp, zero), (1e-6, 1e-12));
        // The widest indicator a screen holds, 16384 points: a ramp is a hundredth of a point there. And it is
        // eight times what a fraction near 1 can tell apart, so it is not rounded away.
        assert!(ramp * 16_384.0 < 0.02);
        assert!(ramp > 8.0 * f32::EPSILON);
        // The zero ramp: its square, which the gate on a difference ramps over, is a normal `f32`; and it is fifty
        // thousand times below the least that two fractions near 1 differ by (`f32::EPSILON / 2`, 6e-8).
        assert!(zero * zero > f32::MIN_POSITIVE);
        assert!(zero * 5e4 < f32::EPSILON / 2.0);
    }

    #[test]
    fn each_gate_is_1_where_androidxs_condition_holds_and_0_where_it_does_not() {
        let mut gate = Gate::new();
        // Order: step(x), above(x, 0.5), below(x, 0.5), nonzero(x), lead(x, 0.25), trail(x, 0.25).
        for (x, expected, what) in [
            (-1.0, [0.0, 0.0, 1.0, 1.0, 0.0, -0.25], "far below"),
            (0.0, [0.0, 0.0, 1.0, 0.0, 0.0, -0.25], "at 0"),
            (0.25, [1.0, 0.0, 1.0, 1.0, 0.5, 0.0], "at a quarter"),
            // At the thresholds: neither above nor below, which is androidx's strict `>` and `<`.
            (0.5, [1.0, 0.0, 0.0, 1.0, 0.75, 0.25], "at one half"),
            (
                0.5 + 2.0 * RAMP,
                [1.0, 1.0, 0.0, 1.0, 0.750_002, 0.250_002],
                "just above one half",
            ),
            (
                0.5 - 2.0 * RAMP,
                [1.0, 0.0, 1.0, 1.0, 0.749_998, 0.249_998],
                "just below one half",
            ),
            (1.0, [1.0, 1.0, 0.0, 1.0, 1.25, 1.0], "at 1"),
            (2.0, [1.0, 1.0, 0.0, 1.0, 1.25, 1.0], "past 1"),
        ] {
            assert_gates(gate.at(x), expected, what);
        }
        // A ramp above 0: `step` and `nonzero` are 1, `lead` is a gap and a ramp, and `trail` is a ramp above its
        // start.
        assert_gates(
            gate.at(RAMP),
            [1.0, 0.0, 1.0, 1.0, 0.250_001, -0.249_999],
            "a ramp above 0",
        );
        // Across the zero ramp `step` and `lead` rise, and `nonzero` (a square, so over the ramp squared) with them.
        assert_gates(
            gate.at(ZERO_RAMP),
            [1.0, 0.0, 1.0, 1.0, 0.25, -0.25],
            "a zero ramp above 0",
        );
        let half = gate.at(ZERO_RAMP / 2.0);
        assert!(
            (half[0] - 0.5).abs() < 0.1 && (half[4] - 0.125).abs() < 0.02,
            "{half:?}"
        );
        let tiny = gate.at(1e-13);
        assert!(tiny[0] < 0.2 && tiny[3] < 0.1 && tiny[4] < 0.05, "{tiny:?}");
        // A noise of 8e-33, as the graph leaves at the instant a rise starts, is 0.
        assert_gates(
            gate.at(8e-33),
            [0.0, 0.0, 1.0, 0.0, 0.0, -0.25],
            "the noise of a rise's start",
        );
        // `nonzero` is 1 for a value either side of 0, from a millionth.
        let (small, negative) = (gate.at(1e-9), gate.at(-1e-9));
        assert!((small[3] - 1.0).abs() < 1e-6, "{small:?}");
        assert!((negative[3] - 1.0).abs() < 1e-6, "{negative:?}");
    }

    #[test]
    fn the_lead_and_the_trail_jump_by_a_gap_where_androidx_gives_the_gap_to_one_end() {
        let mut gate = Gate::new();
        // `if (head > 0) head + gap else 0`: 0 at 0, the gap once past the zero ramp, and a gap more than the head
        // from there.
        let (at_0, at_ramp, at_half) = (gate.at(0.0)[4], gate.at(ZERO_RAMP)[4], gate.at(0.5)[4]);
        assert!(
            at_0.abs() < 1e-9 && (at_ramp - 0.25).abs() < 1e-8 && (at_half - 0.75).abs() < 1e-6
        );
        // `if (tail < 1) tail - gap else 1`: a gap less than the tail, until the ramp below 1, and 1 from there.
        let (before, at_1) = (gate.at(1.0 - RAMP)[5], gate.at(1.0)[5]);
        assert!((before - (0.75 - RAMP)).abs() < 1e-6 && (at_1 - 1.0).abs() < 1e-9);
    }

    // ---- the component, indeterminate ---------------------------------------------------------------------------

    fn indeterminate_config() -> Config {
        Config {
            progress: Progress::Indeterminate,
            ..Config::default()
        }
    }

    /// The four tracks the engine started for an indeterminate indicator: one composite, one track for each value.
    fn four_tracks(items: &[MountItem]) -> Vec<Track> {
        let started = started_tracks(items);
        let [tracks] = &started[..] else {
            panic!("one composite started: {started:?}");
        };
        assert_eq!(tracks.len(), 4, "{tracks:?}");
        tracks.clone()
    }

    /// How many items of `items` are animation operations `matches` accepts.
    fn operations(items: &[MountItem], matches: fn(&AnimationOp) -> bool) -> usize {
        items
            .iter()
            .filter(|item| matches!(item, MountItem::Animation(op) if matches(op)))
            .count()
    }

    fn is_stop(op: &AnimationOp) -> bool {
        matches!(op, AnimationOp::StopAnimation { .. })
    }

    fn is_release(op: &AnimationOp) -> bool {
        matches!(op, AnimationOp::ReleaseComposite { .. })
    }

    fn is_not_a_new_value(op: &AnimationOp) -> bool {
        !matches!(op, AnimationOp::CreateValue { .. })
    }

    #[test]
    fn an_indeterminate_indicator_starts_one_animation_of_four_forever_looping_tracks() {
        let (_app, recording) = start(indeterminate_config());
        let items = recording.items();
        assert_eq!(started_composites(&items), 1);
        let tracks = four_tracks(&items);
        let mut nodes: Vec<NodeId> = tracks.iter().map(|track| track.node).collect();
        nodes.sort_unstable();
        nodes.dedup();
        assert_eq!(nodes.len(), 4, "four different values");
        let curve = Easing::Bezier(0.3, 0.0, 0.8, 0.15);
        for ((track, (name, ..)), steps) in tracks.iter().zip(SWEEPS).zip(STEPS) {
            assert_eq!(track.iterations, None, "{name}: until stopped");
            // One cycle: 1750 ms, to the millisecond's thousandth.
            assert!(
                (track.period_ms() - 1750.0).abs() < 1e-2,
                "{name}: {track:?}"
            );
            assert_eq!(track.steps.len(), steps.len(), "{name}: {track:?}");
            for (step, (to, duration, shaped)) in track.steps.iter().zip(steps) {
                let Step::Run {
                    to: found_to,
                    duration_ms,
                    easing: found_easing,
                    delay_ms,
                    frame_rate,
                } = *step
                else {
                    panic!("{name}: a run, not {step:?}");
                };
                assert_eq!(found_to, *to, "{name}: {step:?}");
                assert!(
                    (duration_ms - duration).abs() < 1e-2,
                    "{name}: {step:?} is not {duration} ms"
                );
                assert_eq!(
                    found_easing,
                    if *shaped { curve } else { Easing::Linear },
                    "{name}: {step:?}"
                );
                // No step of its own delay, and every one at the screen's highest rate (ADR 0010, check 4).
                assert_eq!((delay_ms, frame_rate), (0.0, None), "{name}: {step:?}");
            }
        }
    }

    #[test]
    fn the_graph_plays_the_four_sweeps_at_120_hz_as_androidxs_keyframes_give_them() {
        let (_app, recording) = start(indeterminate_config());
        let items = recording.items();
        let tracks = four_tracks(&items);
        let mut replay = Replay::default();
        replay.apply(&items);
        // Two and a half cycles, a frame at a time.
        for frame in 0_u32..525 {
            replay.run(tracks[0].node, 1);
            let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
            // The frame on the end of a cycle may be on either side of the jump back to the start.
            if (ms % 1750.0).min(1750.0 - ms % 1750.0) < 1e-6 {
                continue;
            }
            for (track, (name, delay, duration)) in tracks.iter().zip(SWEEPS) {
                let expected = keyframed(delay, duration, ms);
                let found = f64::from(replay.value(track.node));
                assert!(
                    (found - expected).abs() < 1e-3,
                    "{name} at {ms} ms: {found} is not {expected}"
                );
                // What the engine sent says the same, read as a table and not played.
                let table = track.value_at(0.0, ms);
                assert!(
                    (table - expected).abs() < 1e-3,
                    "{name} at {ms} ms: the table's {table} is not {expected}"
                );
                // At the instant a rise starts or ends the graph's value is a hair off the 0 or the 1 that androidx's
                // keyframe has: closer than the ramps that read it as that (`ZERO_RAMP`, `RAMP`).
                let at = ms % 1750.0;
                if (at - f64::from(delay)).abs() < 1e-6 {
                    assert!(found < f64::from(ZERO_RAMP), "{name} at {ms} ms: {found}");
                }
                if (at - f64::from(delay + duration)).abs() < 1e-6 {
                    assert!(found > 1.0 - f64::from(RAMP), "{name} at {ms} ms: {found}");
                }
            }
        }
    }

    /// One stroke of an indeterminate indicator at one instant, with how much of it shows (the `globalAlpha`).
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Faint {
        stroke: Stroked,
        alpha: f32,
    }

    /// The strokes `drawing` makes with the animated values as `replay` holds them now.
    fn faint_strokes(drawing: &Drawing, replay: &Replay) -> Vec<Faint> {
        let number = |num: Num| num.get(|slot| replay.value(drawing.slots[usize::from(slot)]));
        let mut strokes = Vec::new();
        let (mut alpha, mut width, mut cap) = (1.0, 1.0, LineCap::Butt);
        let mut stroke = Color(0);
        let mut path: Vec<PathOp> = Vec::new();
        for op in &drawing.ops {
            match op {
                CanvasOp::GlobalAlpha(value) => alpha = number(*value),
                CanvasOp::LineWidth(value) => width = number(*value),
                CanvasOp::LineCap(value) => cap = *value,
                CanvasOp::StrokeStyle(canvas::Paint::Color(color)) => stroke = *color,
                // The fill is the canvas view's, for the stop a determinate indicator draws; nothing is filled here (a
                // test of its own says so).
                CanvasOp::FillStyle(_) => {}
                CanvasOp::BeginPath => path.clear(),
                CanvasOp::Path(step) => path.push(step.clone()),
                CanvasOp::Stroke(None) => {
                    let [PathOp::MoveTo(x0, y0), PathOp::LineTo(x1, y1)] = &path[..] else {
                        panic!("a stroke of one line: {path:?}");
                    };
                    strokes.push(Faint {
                        stroke: Stroked {
                            from: (number(*x0), number(*y0)),
                            to: (number(*x1), number(*y1)),
                            width,
                            cap,
                            color: stroke,
                        },
                        alpha,
                    });
                }
                other => panic!("an operation the moving indicator does not draw with: {other:?}"),
            }
        }
        strokes
    }

    /// `faint` shows as `expected` says: as a whole, along that line, or not at all.
    fn assert_faint(faint: Faint, expected: Option<Line>, color: Color, what: &str) {
        let Faint { stroke, alpha } = faint;
        assert_eq!(stroke.color, color, "{what}");
        assert_eq!(
            (stroke.from.1, stroke.to.1, stroke.width),
            (2.0, 2.0, 4.0),
            "{what}"
        );
        match expected {
            Some(line) => {
                assert!(
                    (0.999..1.001).contains(&alpha),
                    "{what}: drawn at {alpha}, not whole: {stroke:?}"
                );
                assert!(
                    near(stroke.from.0, line.from) && near(stroke.to.0, line.to),
                    "{what}: {stroke:?} is not {line:?}"
                );
                assert_eq!(stroke.cap, line.cap.to_canvas(), "{what}");
            }
            None => assert!(
                alpha.abs() < 1e-3,
                "{what}: drawn at {alpha}, not gone: {stroke:?}"
            ),
        }
    }

    /// `a` and `b` are the same strokes to a thousandth of a point: the same lines, each along the same ends.
    fn same_strokes(a: [Option<Line>; 5], b: [Option<Line>; 5]) -> bool {
        a.into_iter().zip(b).all(|pair| match pair {
            (Some(a), Some(b)) => near(a.from, b.from) && near(a.to, b.to) && a.cap == b.cap,
            (None, None) => true,
            _ => false,
        })
    }

    /// Whether a value that is moving is within a few ramps of one of androidx's conditions: moving any one of the
    /// four by three ramps, either way, changes the strokes. A value at rest (exactly 0 before its rise, exactly 1
    /// after it) is not moving, and its condition is decided. Where a moving value is that close, androidx's `if` is a
    /// step and the drawing's gate is a ramp, and a value a hair from the threshold (or the noise of the graph at the
    /// instant a rise starts) may fall either side: both draw the pose of a moment a millisecond earlier or later,
    /// and neither is wrong.
    fn near_a_condition(at: Course<f32>, width: f32, ltr: bool) -> bool {
        let base = drawn_lines(at, width, ltr);
        let values = [at.first_head, at.first_tail, at.second_head, at.second_tail];
        let at_rest = |value: f32| value.to_bits() == 0 || value.to_bits() == 1.0_f32.to_bits();
        (0..4).filter(|i| !at_rest(values[*i])).any(|i| {
            [-3.0, 3.0].into_iter().any(|ramps| {
                let mut moved = values;
                moved[i] += ramps * RAMP;
                let moved = course(moved[0], moved[1], moved[2], moved[3]);
                !same_strokes(drawn_lines(moved, width, ltr), base)
            })
        })
    }

    /// Plays an indeterminate indicator of this size for two and a half cycles, a 120 Hz frame at a time, and checks at
    /// every frame that what the canvases draw is what the geometry says for the values the graph holds then.
    fn assert_the_canvases_follow_the_geometry(width: f32, rtl: bool) {
        let (_app, recording) = start(Config {
            width: Points(width),
            rtl,
            ..indeterminate_config()
        });
        let items = recording.items();
        let mounted = Mounted::of(&items);
        let nodes: Vec<NodeId> = four_tracks(&items).iter().map(|track| track.node).collect();
        let mut replay = Replay::default();
        replay.apply(&items);
        let (primary, container) = (
            scheme_color(ColorRole::Primary, false),
            scheme_color(ColorRole::SecondaryContainer, false),
        );
        let mut undecided = 0_u32;
        for frame in 0_u16..525 {
            replay.run(nodes[0], 1);
            let value = |i: usize| replay.value(nodes[i]);
            let at = course(value(0), value(1), value(2), value(3));
            if near_a_condition(at, width, !rtl) {
                undecided += 1;
                continue;
            }
            let [before, first, between, second, after] = drawn_lines(at, width, !rtl);
            let what =
                |name: &str| format!("{width} wide, rtl {rtl}, frame {frame}, {name}: {at:?}");
            let first_canvas = faint_strokes(mounted.drawing(mounted.first), &replay);
            let [line_1] = first_canvas[..] else {
                panic!("one stroke on the first canvas: {first_canvas:?}");
            };
            assert_faint(line_1, first, primary, &what("line 1"));
            let tracks = faint_strokes(mounted.drawing(mounted.track), &replay);
            let [track_1, track_2, track_3] = tracks[..] else {
                panic!("three strokes on the track canvas: {tracks:?}");
            };
            assert_faint(track_1, before, container, &what("track before line 1"));
            assert_faint(track_2, between, container, &what("track between"));
            assert_faint(track_3, after, container, &what("track after line 2"));
            let last_canvas = faint_strokes(mounted.drawing(mounted.indicator), &replay);
            let [line_2] = last_canvas[..] else {
                panic!("one stroke on the last canvas: {last_canvas:?}");
            };
            assert_faint(line_2, second, primary, &what("line 2"));
        }
        // The frames that are left out are those that land on the instant a rise starts or ends, where the graph's
        // value is a hair off the 0 or the 1 that androidx's keyframe has exactly (its loop's period is 1750 ms to
        // a hundredth of a millisecond, `f32` offsets times the duration): 78, 288 and 498, where the second head
        // starts, and 150 and 360, where the first tail ends. A rise whose frame lands on its instant is left to
        // chance on a device; a 120 Hz clock that starts with the animation lands on these.
        assert!(
            undecided <= 5,
            "{width} wide, rtl {rtl}: {undecided} of 525 frames left out"
        );
    }

    #[test]
    fn what_the_canvases_draw_at_every_frame_is_what_the_geometry_says() {
        // 240 is the default; 100 overlaps the tracks with the lines; 12 has round caps with little room; 4 is as wide
        // as it is tall; 3 is narrower, so butt.
        for width in [240.0, 100.0, 12.0, 4.0, 3.0] {
            for rtl in [false, true] {
                assert_the_canvases_follow_the_geometry(width, rtl);
            }
        }
    }

    #[test]
    fn an_indeterminate_indicator_draws_no_stop() {
        let (_app, recording) = start(indeterminate_config());
        let items = recording.items();
        let mounted = Mounted::of(&items);
        for id in [mounted.first, mounted.track, mounted.indicator] {
            let fills = mounted
                .drawing(id)
                .ops
                .iter()
                .filter(|op| matches!(op, CanvasOp::Fill(..)))
                .count();
            assert_eq!(fills, 0);
        }
    }

    #[test]
    fn an_indeterminate_indicator_sends_nothing_while_it_plays() -> Result<(), String> {
        let (mut app, recording) = start(indeterminate_config());
        let before = recording.items().len();
        // Two and a half cycles on the manual clock: no batch at all, so no layout, no text and no drawing again.
        app.run_for(std::time::Duration::from_millis(4375))?;
        assert_eq!(recording.items().len(), before);
        Ok(())
    }

    #[test]
    fn an_indeterminate_indicator_has_the_range_and_no_value_for_screen_readers() {
        let mounted = mount(indeterminate_config());
        assert_eq!(range(&mounted), Some((Some(0.0), Some(1.0), None)));
        assert_eq!(mounted.size(), Size::new(Points(240.0), Points(4.0)));
    }

    // ---- the animation's life ---------------------------------------------------------------------------------

    #[test]
    fn the_animation_plays_only_while_the_indicator_is_indeterminate_and_there()
    -> Result<(), String> {
        let (mut app, recording) = start(indeterminate_config());
        assert!(replay(&recording).is_animating(), "playing from the start");
        // Becoming determinate stops it, starts none, and lays nothing out.
        app.press("Half")?;
        let items = recording.in_step("press(Half)");
        assert_eq!(operations(&items, is_stop), 1, "{items:?}");
        assert_eq!(started_composites(&items), 0, "{items:?}");
        assert_eq!(layout_items(&items), 0, "{items:?}");
        assert!(!replay(&recording).is_animating());
        // The determinate lines are drawn, and the canvas at the back is cleared.
        let drawn = drawn_by(&Mounted::of(&recording.items()));
        assert_eq!(drawn.first, Vec::new());
        assert_line(
            &drawn.indicator,
            2.0,
            120.0,
            scheme_color(ColorRole::Primary, false),
            "Half",
        );
        // Indeterminate again starts it once more.
        app.press("Loop")?;
        let items = recording.in_step("press(Loop)");
        assert_eq!(started_composites(&items), 1, "{items:?}");
        assert_eq!(operations(&items, is_stop), 0, "{items:?}");
        assert!(replay(&recording).is_animating());
        // Taking it off the screen releases it: nothing is left animating.
        app.press("Remove")?;
        let items = recording.in_step("press(Remove)");
        assert!(operations(&items, is_release) >= 1, "{items:?}");
        // The values are released with the indicator. A loop that has lost its values cannot go on, and NSH's graph
        // ends it at its next frame (`Step::Run` in `animation/graph.rs`: "the value is gone (released): the
        // composite cannot go on"), so one frame after it nothing is animating.
        let mut gone = replay(&recording);
        gone.run(1, 1);
        assert!(!gone.is_animating(), "{items:?}");
        Ok(())
    }

    #[test]
    fn an_equal_progress_neither_restarts_nor_stops_the_animation() -> Result<(), String> {
        let (mut app, recording) = start(indeterminate_config());
        app.press("Loop")?;
        let items = recording.in_step("press(Loop)");
        assert_eq!(operations(&items, |_| true), 0, "{items:?}");
        assert_eq!(layout_items(&items), 0, "{items:?}");
        assert!(replay(&recording).is_animating());
        Ok(())
    }

    #[test]
    fn a_determinate_indicator_changed_in_mid_flight_stays_where_it_is_and_starts_over_when_it_loops_again()
    -> Result<(), String> {
        let (mut app, recording) = start(indeterminate_config());
        let items = recording.items();
        let tracks = four_tracks(&items);
        let mut replay = Replay::default();
        replay.apply(&items);
        // 100 frames in, 0.83 s, part way through the first rises.
        replay.run(tracks[0].node, 100);
        app.press("Half")?;
        replay.apply(&recording.in_step("press(Half)"));
        let held: Vec<u32> = tracks
            .iter()
            .map(|track| replay.value(track.node).to_bits())
            .collect();
        // Stopped: the values stay where they were, and nothing is animating.
        replay.run(tracks[0].node, 30);
        let now: Vec<u32> = tracks
            .iter()
            .map(|track| replay.value(track.node).to_bits())
            .collect();
        assert_eq!(held, now);
        assert!(!replay.is_animating());
        // Indeterminate again: every value starts its cycle over, from the start, as androidx's transition does.
        app.press("Loop")?;
        replay.apply(&recording.in_step("press(Loop)"));
        assert!(replay.is_animating());
        for frame in 0_u32..300 {
            replay.run(tracks[0].node, 1);
            let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
            // The frame on the end of a cycle may be on either side of the jump back to the start.
            if (ms % 1750.0).min(1750.0 - ms % 1750.0) < 1e-6 {
                continue;
            }
            for (track, (name, delay, duration)) in tracks.iter().zip(SWEEPS) {
                let expected = keyframed(delay, duration, ms);
                let found = f64::from(replay.value(track.node));
                assert!(
                    (found - expected).abs() < 1e-3,
                    "{name} at {ms} ms after the restart: {found} is not {expected}"
                );
            }
        }
        Ok(())
    }

    // ---- reduce motion ---------------------------------------------------------------------------------------

    fn still(rtl: bool) -> Config {
        Config {
            reduce_motion: true,
            rtl,
            ..indeterminate_config()
        }
    }

    #[test]
    fn with_reduce_motion_an_indeterminate_indicator_holds_the_still_pose_and_moves_no_more()
    -> Result<(), String> {
        let (mut app, recording) = start(still(false));
        app.run_for(std::time::Duration::from_secs(3))?;
        let items = recording.items();
        // Nothing starts, and nothing is left running.
        assert_eq!(started_composites(&items), 0);
        assert_eq!(operations(&items, is_not_a_new_value), 0, "{items:?}");
        assert!(!replay(&recording).is_animating());
        // What is drawn is the pose at 1000 ms, by the numbers of the Kotlin worked out in the geometry test: line 1
        // from the end to 0.4056, the track between the lines, and line 2 from 0.1006 to the start. No track before
        // line 1, none after line 2 (their conditions do not hold), and no stop.
        let mounted = Mounted::of(&items);
        let drawn = drawn_by(&mounted);
        let (primary, container) = (
            scheme_color(ColorRole::Primary, false),
            scheme_color(ColorRole::SecondaryContainer, false),
        );
        assert_line(&drawn.first, 238.0, 97.340_52, primary, "line 1");
        assert_line(
            &drawn.track,
            32.151_11,
            89.340_52,
            container,
            "the track between",
        );
        assert_line(&drawn.indicator, 24.151_11, 2.0, primary, "line 2");
        assert_eq!(drawn.dots, Vec::new());
        // The same element for screen readers as the moving one.
        assert_eq!(range(&mounted), Some((Some(0.0), Some(1.0), None)));
        Ok(())
    }

    #[test]
    fn the_still_pose_is_mirrored_in_right_to_left() {
        let drawn = drawn_by(&mount(still(true)));
        let (primary, container) = (
            scheme_color(ColorRole::Primary, false),
            scheme_color(ColorRole::SecondaryContainer, false),
        );
        // 240 - 238 = 2 and 240 - 97.34 = 142.66 and so on: `barStart = (1 - end) * width`.
        assert_line(&drawn.first, 142.659_48, 2.0, primary, "line 1");
        assert_line(
            &drawn.track,
            150.659_48,
            207.848_89,
            container,
            "the track between",
        );
        assert_line(&drawn.indicator, 238.0, 215.848_89, primary, "line 2");
    }

    #[test]
    fn the_still_pose_is_the_one_the_geometry_gives_for_the_pose_in_both_directions() {
        for rtl in [false, true] {
            let drawn = drawn_by(&mount(still(rtl)));
            let expected = drawn_lines(still_pose(), 240.0, !rtl);
            let [before, first, between, second, after] = expected;
            assert_eq!((before, after), (None, None));
            let found =
                |strokes: &[Stroked]| strokes.first().map(|stroke| (stroke.from.0, stroke.to.0));
            for (strokes, line) in [
                (&drawn.first, first),
                (&drawn.track, between),
                (&drawn.indicator, second),
            ] {
                let line = line.expect("a line in the pose");
                let (from, to) = found(strokes).expect("a stroke");
                assert!(
                    near(from, line.from) && near(to, line.to),
                    "{from} to {to} is not {line:?}"
                );
            }
        }
    }

    #[test]
    fn reduce_motion_leaves_a_determinate_indicator_as_it_is() {
        for fraction in [0.0, 0.5, 1.0] {
            let moving = drawn_by(&mount(determinate_at(fraction)));
            let still = drawn_by(&mount(Config {
                reduce_motion: true,
                ..determinate_at(fraction)
            }));
            assert_eq!(
                (moving.first, moving.track, moving.indicator, moving.dots),
                (still.first, still.track, still.indicator, still.dots),
                "{fraction}"
            );
        }
    }

    #[test]
    fn the_indicator_follows_the_reduce_motion_setting_when_it_changes() -> Result<(), String> {
        let (mut app, recording) = start(indeterminate_config());
        assert!(replay(&recording).is_animating());
        app.set_environment("reduceMotion", |environment| {
            environment.reduce_motion = true;
        })?;
        let items = recording.in_step("reduceMotion");
        assert_eq!(operations(&items, is_stop), 1, "{items:?}");
        assert_eq!(layout_items(&items), 0, "{items:?}");
        assert!(!replay(&recording).is_animating());
        // The canvases were drawn again with the pose: constants.
        let drawn = drawn_by(&Mounted::of(&recording.items()));
        assert_eq!(drawn.first.len(), 1);
        assert_eq!(drawn.track.len(), 1);
        assert_eq!(drawn.indicator.len(), 1);
        // And when motion is allowed again it plays, from the start.
        app.set_environment("motion", |environment| {
            environment.reduce_motion = false;
        })?;
        let items = recording.in_step("motion");
        assert_eq!(started_composites(&items), 1, "{items:?}");
        assert!(replay(&recording).is_animating());
        Ok(())
    }

    mod wavy_form;
}
