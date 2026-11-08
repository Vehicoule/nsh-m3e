//! The wavy form of the linear progress indicator: a track that is straight, and an active indicator that waves.
//!
//! It follows androidx's `LinearWavyProgressIndicator` (`WavyProgressIndicator.kt`) and its drawing in
//! `LinearWavyProgressModifiers.kt`, at the pinned commit, as [ADR 0012] says. What the form does that the flat one
//! does not:
//!
//! - It is [`HEIGHT`] points tall, the stroke's thickness and room for the wave, and the track and the lines are drawn
//!   along its middle. The strokes are 4 points thick with round caps (`WavyProgressIndicatorDefaults.linearIndicatorStroke`
//!   and `linearTrackStroke`).
//! - The active indicator is a wave: a chain of quadratic curves, each half a wave long, that bend by an amplitude and
//!   travel along the indicator by one wavelength a second. The amplitude is 1 between a tenth and 95 % of the progress and
//!   0 outside it, and it is a 500 ms transition when the progress crosses one of them. A wave that loses its height
//!   goes on travelling for those 500 ms and then holds (`Travel::Settle`); a wave that turns flat stops both of its
//!   values where they are, as nothing draws them any more.
//! - The track is drawn by the same arithmetic as the flat indicator's, in a path of androidx's own, whose pieces keep a
//!   gap from the lines.
//!
//! # How a wave is drawn without drawing it again
//!
//! A wave moves every frame, so a canvas that is recorded again for each frame would send a batch for each (ADR 0010).
//! The drawing is recorded once instead and its numbers are nodes of the animation graph, as the flat indicator's are:
//!
//! - The wave is a path of quadratic curves, long enough for the indicator and one wavelength more, whose control
//!   points' heights are the amplitude times the control height (androidx's `controlY`, the height less the stroke). The
//!   amplitude is a node, so the path breathes natively. The path's x coordinates never change: the wave's travel is a
//!   translation by a node, `-shift`, as androidx's `translate(x = -waveShift)`.
//! - androidx cuts a piece out of that path for each line (`PathMeasure.getSegment`). Here the whole path is drawn and
//!   clipped to the line's window: two half planes, each bounded by a line through the window's end that is
//!   perpendicular to the wave there, so the cut is where a cut of the path is. A clip's frame is a matrix of nodes
//!   (`[1 -r; r 1]`, `r` the slope of the wave at the end), which is a rotation to within a scale and takes no trigonometry.
//! - The rounded ends are filled circles at the window's ends, where the clip cut the path square (`StrokeCap.Round` on
//!   a cut path is a half circle past its end).
//! - Where the wave is under a window's end (its height and its slope) is read from tables of the nodes: the wave is a
//!   parabola per half wave, so a table of 65 points is off by less than 0.003 points, and its slope is exact in 3.
//! - What androidx decides with an `if` of the progress is a gate: a number that is 1 or 0, as in the flat indicator.
//!
//! The same arithmetic runs on fixed numbers where nothing moves (a determinate indicator's track, the still pose of
//! reduce motion), through [`Value`], which computes a fixed number as one and builds a node of the graph for a number
//! that moves.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use std::cell::Cell;
use std::f32::consts::TAU;

use nsh_core::canvas::{CanvasOp, LineCap, Num, PathOp};
use nsh_core::element::CanvasArg;
use nsh_core::geometry::{Bounds, Points, point, px, size};
use nsh_core::prelude::*;

use super::{Mode, Stop, Value, held, nonzero, step};
use crate::progress::easing;
use crate::tokens::CubicBezier;
use crate::tokens::androidx::linear_progress_indicator_tokens as tokens;
use crate::tokens::androidx::motion_tokens;

/// How tall a wavy indicator is: `WavyProgressIndicatorDefaults.LinearContainerHeight`, which is
/// `LinearProgressIndicatorTokens.WaveHeight` (`WavyProgressIndicator.kt`, lines 465 and 466).
pub(super) const HEIGHT: Points = tokens::WAVE_HEIGHT;

/// How thick the active indicator is: `linearIndicatorStroke`, `LinearProgressIndicatorTokens.ActiveThickness`
/// (`WavyProgressIndicator.kt`, lines 411 to 420).
pub(super) const INDICATOR_THICKNESS: f32 = tokens::ACTIVE_THICKNESS.0;

/// How thick the track is: `linearTrackStroke`, `LinearProgressIndicatorTokens.TrackThickness`
/// (`WavyProgressIndicator.kt`, lines 435 to 444).
pub(super) const TRACK_THICKNESS: f32 = tokens::TRACK_THICKNESS.0;

/// The gap between the indicator and the track: `LinearIndicatorTrackGapSize`, `LinearProgressIndicatorTokens.TrackActiveSpace`
/// (`WavyProgressIndicator.kt`, lines 483 and 484).
pub(super) const GAP: f32 = tokens::TRACK_ACTIVE_SPACE.0;

/// The most the stop is wide: `LinearTrackStopIndicatorSize`, `LinearProgressIndicatorTokens.StopSize`
/// (`WavyProgressIndicator.kt`, lines 471 and 472).
pub(super) const STOP_SIZE: f32 = tokens::STOP_SIZE.0;

/// The wavelength of a determinate wavy indicator: `LinearDeterminateWavelength`, `ActiveWaveWavelength`
/// (`WavyProgressIndicator.kt`, line 459).
pub(super) const DETERMINATE_WAVELENGTH: f32 = tokens::ACTIVE_WAVE_WAVELENGTH.0;

/// The wavelength of an indeterminate wavy indicator: `LinearIndeterminateWavelength`,
/// `IndeterminateActiveWaveWavelength` (`WavyProgressIndicator.kt`, lines 461 to 463).
pub(super) const INDETERMINATE_WAVELENGTH: f32 = tokens::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH.0;

/// The progress at or below which `indicatorAmplitude` is 0 ("Sets the amplitude to the max on 10%, and back to zero on
/// 95% of the progress", `WavyProgressIndicator.kt`, lines 493 to 500).
pub(super) const FLAT_AT_OR_BELOW: f32 = 0.1;

/// The progress at or above which `indicatorAmplitude` is 0 (`WavyProgressIndicator.kt`, line 495).
pub(super) const FLAT_AT_OR_ABOVE: f32 = 0.95;

/// How long the amplitude takes to change: `MotionTokens.DurationLong2` for both `IncreasingAmplitudeAnimationSpec` and
/// `DecreasingAmplitudeAnimationSpec` (`WavyProgressIndicator.kt`, lines 503 to 515), in milliseconds.
pub(super) const AMPLITUDE_MS: f32 = motion_tokens::DURATION_LONG2;

/// The curve of an amplitude that grows: `MotionTokens.EasingStandardCubicBezier` (`WavyProgressIndicator.kt`, line 507).
pub(super) const GROW: CubicBezier = motion_tokens::EASING_STANDARD_CUBIC_BEZIER;

/// The curve of an amplitude that shrinks: `MotionTokens.EasingEmphasizedAccelerateCubicBezier`
/// (`WavyProgressIndicator.kt`, line 514).
pub(super) const SHRINK: CubicBezier = motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;

/// How long the wave takes to travel one wavelength, in milliseconds: `(wavelength / waveSpeed) * 1000`, and the speed
/// is the wavelength by default, "to render an animation that moves the wave by one wave length per second"
/// (`WavyProgressIndicator.kt`, lines 100 to 103 and 130; `LinearWavyProgressModifiers.kt`, lines 102, 103 and 387 to
/// 390).
pub(super) const TRAVEL_MS: f32 = 1000.0;

/// How many points of the wave a table of its height holds, per half wave: the table is off by less than 0.003 points of
/// a wave 12 points high.
pub(super) const HEIGHT_STEPS: u16 = 32;

// ---- numbers that are fixed or move --------------------------------------------------------------------------------

impl Value {
    /// The sum.
    fn plus(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fixed(a), Self::Fixed(b)) => Self::Fixed(a + b),
            (Self::Moving(a), Self::Fixed(b)) | (Self::Fixed(b), Self::Moving(a)) => {
                Self::Moving(a.add(b))
            }
            (Self::Moving(a), Self::Moving(b)) => Self::Moving(a.add(&b)),
        }
    }

    /// The difference.
    fn minus(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fixed(a), Self::Fixed(b)) => Self::Fixed(a - b),
            (Self::Moving(a), Self::Fixed(b)) => Self::Moving(a.sub(b)),
            (Self::Fixed(a), Self::Moving(b)) => Self::Moving(b.mul(-1.0).add(a)),
            (Self::Moving(a), Self::Moving(b)) => Self::Moving(a.sub(&b)),
        }
    }

    /// The product.
    fn times(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fixed(a), Self::Fixed(b)) => Self::Fixed(a * b),
            (Self::Moving(a), Self::Fixed(b)) | (Self::Fixed(b), Self::Moving(a)) => {
                Self::Moving(a.mul(b))
            }
            (Self::Moving(a), Self::Moving(b)) => Self::Moving(a.mul(&b)),
        }
    }

    /// Times `k`.
    fn scaled(self, k: f32) -> Self {
        self.times(Self::Fixed(k))
    }

    /// Plus `k`.
    fn shifted(self, k: f32) -> Self {
        self.plus(Self::Fixed(k))
    }

    /// Modulo `m`, never below 0, as the animation graph takes a modulo.
    fn modulo(self, m: f32) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(if m == 0.0 { 0.0 } else { ((x % m) + m) % m }),
            Self::Moving(x) => Self::Moving(x.modulo(m)),
        }
    }

    /// Held between `low` and `high` (androidx `fastCoerceIn`); where `high` is not above `low`, `high`.
    fn clamped(self, low: f32, high: f32) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(held(x, low, high)),
            Self::Moving(x) if high > low => {
                Self::Moving(x.interpolate(&[low, high], &[low, high]))
            }
            Self::Moving(_) => Self::Fixed(high),
        }
    }

    /// At least `low` (androidx `max(low, x)`); a number that moves is never above `ceiling`.
    fn at_least(self, low: f32, ceiling: f32) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(held(x, low, f32::INFINITY)),
            Self::Moving(_) => self.clamped(low, ceiling),
        }
    }

    /// At most `high` (androidx `min(x, high)`); a number that moves is never below `floor`.
    fn at_most(self, high: f32, floor: f32) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(x.min(high)),
            Self::Moving(_) => self.clamped(floor, high),
        }
    }

    /// 1 where this is above 0, 0 where it is not.
    fn positive(self) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(f32::from(x > 0.0)),
            Self::Moving(x) => Self::Moving(step(&x)),
        }
    }

    /// 1 where this is not 0, 0 where it is (`abs(x) > 0`).
    fn differs(self) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(f32::from(x.abs() > 0.0)),
            Self::Moving(x) => Self::Moving(nonzero(&x)),
        }
    }

    /// 1 where this is above `other`, 0 where it is not.
    fn gt(self, other: Self) -> Self {
        match (self, other) {
            (Self::Fixed(a), Self::Fixed(b)) => Self::Fixed(f32::from(a > b)),
            _ => self.minus(other).positive(),
        }
    }

    /// 1 where this is below `other`, 0 where it is not.
    fn lt(self, other: Self) -> Self {
        other.gt(self)
    }

    /// 1 where this is not below `other`, 0 where it is.
    fn ge(self, other: Self) -> Self {
        Self::Fixed(1.0).minus(self.lt(other))
    }

    /// `then` where the gate is 1, `otherwise` where it is 0: androidx's `if`.
    fn select(gate: Self, then: Self, otherwise: Self) -> Self {
        match gate {
            Self::Fixed(g) => {
                if g > 0.5 {
                    then
                } else {
                    otherwise
                }
            }
            Self::Moving(_) => otherwise.plus(gate.times(then.minus(otherwise))),
        }
    }

    /// `curve` of this: the curve itself where this is fixed, and where it moves the table of `curve` at `at`.
    fn through(self, curve: impl Fn(f32) -> f32, at: &[f32]) -> Self {
        match self {
            Self::Fixed(x) => Self::Fixed(curve(x)),
            Self::Moving(x) => {
                let output: Vec<f32> = at.iter().map(|x| curve(*x)).collect();
                Self::Moving(x.interpolate(at, &output))
            }
        }
    }
}

// ---- the wave ------------------------------------------------------------------------------------------------------

/// A wave: its wavelength, and the height of the control point of each of its half waves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Wave {
    /// How long a wave is, in points.
    pub(super) length: f32,
    /// The height of a half wave's control point: the indicator's height less the stroke's width
    /// (`LinearWavyProgressModifiers.kt`, line 857). The curve peaks at half of it.
    pub(super) crest: f32,
}

impl Wave {
    /// The wave of `length` points in a wavy indicator.
    pub(super) fn new(length: f32) -> Self {
        Self {
            length,
            crest: HEIGHT.0 - INDICATOR_THICKNESS,
        }
    }

    /// How long a half wave is.
    fn half(self) -> f32 {
        self.length / 2.0
    }

    /// The height of the wave of amplitude 1 above the indicator's middle, at `v` on the wave's own scale, where each
    /// half wave is 1 long and the wave is 2 long.
    ///
    /// A half wave is the quadratic curve from (0, 0), by the control point (half / 2, crest), to (half, 0): x is
    /// linear in the curve's parameter, `t = v`, and y is `2 · crest · t · (1 - t)`. The next half wave bends the
    /// other way.
    pub(super) fn height(self, v: f32) -> f32 {
        let (t, sign) = if v < 1.0 { (v, 1.0) } else { (v - 1.0, -1.0) };
        sign * 2.0 * self.crest * t * (1.0 - t)
    }

    /// The slope of the wave of amplitude 1, at `v` on the wave's own scale (the height's derivative along x).
    pub(super) fn slope(self, v: f32) -> f32 {
        let (t, sign) = if v < 1.0 { (v, 1.0) } else { (v - 1.0, -1.0) };
        sign * 2.0 * self.crest * (1.0 - 2.0 * t) / self.half()
    }

    /// Where `x` points along the path is on the wave's own scale: one for each half wave, and back to 0 after two.
    fn scale(self, x: Value) -> Value {
        x.scaled(2.0 / self.length).modulo(2.0)
    }
}

/// How high the wave is above the indicator's middle and how steeply it goes, under a point of a window.
#[derive(Debug, Clone, Copy)]
struct End {
    /// Where the window ends, from the left.
    x: Value,
    /// Where the wave is at `x`, from the top.
    y: Value,
    /// The wave's slope at `x`.
    slope: Value,
}

/// What a wave is doing: how high it is, and how far it has moved.
#[derive(Debug, Clone, Copy)]
pub(super) struct Motion {
    /// 0 for a flat line, 1 for the wave at its height.
    pub(super) amplitude: Value,
    /// How far the wave has moved from where it starts, in points, from 0 to a wavelength.
    pub(super) shift: Value,
}

impl Motion {
    /// The motion of `wave` for an `amplitude` node and a `phase` node (a fraction of a wavelength, which counts up
    /// forever), or, where the wave does not travel (`travels` is false), held at where it starts: androidx's wave
    /// offset is 0 until its animation moves it (`LinearWavyProgressModifiers.kt`, line 329).
    pub(super) fn of(
        amplitude: AnimatedNode,
        phase: AnimatedNode,
        wave: Wave,
        travels: bool,
    ) -> Self {
        Self {
            amplitude: Value::Moving(amplitude),
            shift: if travels {
                Value::Moving(phase.modulo(1.0).mul(wave.length))
            } else {
                Value::Fixed(0.0)
            },
        }
    }

    /// The window's end at `x`.
    fn end(self, wave: Wave, x: Value) -> End {
        let v = wave.scale(x.plus(self.shift));
        let table: Vec<f32> = (0..=2 * HEIGHT_STEPS)
            .map(|i| f32::from(i) / f32::from(HEIGHT_STEPS))
            .collect();
        End {
            x,
            y: self
                .amplitude
                .times(v.through(|v| wave.height(v), &table))
                .shifted(HEIGHT.0 / 2.0),
            // The slope is a straight line in each half wave, so a point at each end of one gives it exactly.
            slope: self
                .amplitude
                .times(v.through(|v| wave.slope(v), &[0.0, 1.0, 2.0])),
        }
    }
}

/// The amplitude `WavyProgressIndicatorDefaults.indicatorAmplitude` gives a determinate indicator at `progress`: 0 at or
/// below a tenth and at or above 95 %, and 1 between (`WavyProgressIndicator.kt`, lines 493 to 500). The progress is
/// held to 0 to 1 first, and a progress that is not finite counts as 0 (`determinateProgress`,
/// `LinearWavyProgressModifiers.kt`, lines 442 to 445).
pub(super) fn amplitude_at(progress: f32) -> f32 {
    if has_waves(progress) { 1.0 } else { 0.0 }
}

/// Whether the wave has any height at `progress`.
pub(super) fn has_waves(progress: f32) -> bool {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    progress > FLAT_AT_OR_BELOW && progress < FLAT_AT_OR_ABOVE
}

// ---- the track and the lines ---------------------------------------------------------------------------------------

/// A line along the middle of the indicator: where it starts and ends, in points from the left, and how much of it is
/// there (`shown` is 1 where androidx draws it and 0 where it does not).
#[derive(Debug, Clone, Copy)]
pub(super) struct Piece {
    pub(super) from: Value,
    pub(super) to: Value,
    pub(super) shown: Value,
}

/// What a wavy indicator draws: the track's lines, and the windows of the wave the active lines are cut to, both in
/// androidx's order.
#[derive(Debug, Clone, Default)]
pub(super) struct Layout {
    /// The pieces of the track path.
    pub(super) track: Vec<Piece>,
    /// The windows of the lines.
    pub(super) lines: Vec<Piece>,
}

/// How far the strokes' caps reach past the ends of a line, for an indicator `width` points wide: half the thicker
/// stroke, but nothing where the indicator is narrower than it is tall (`LinearWavyProgressModifiers.kt`, lines 826 to
/// 835). Both strokes are round.
pub(super) fn cap_width(width: f32) -> f32 {
    if HEIGHT.0 > width {
        0.0
    } else {
        (INDICATOR_THICKNESS / 2.0).max(TRACK_THICKNESS / 2.0)
    }
}

/// The track and the lines for `pairs` of fractions of the width, each pair the start and the end of a line, as
/// `LinearProgressDrawingCache.updateDrawPaths` makes its paths (`LinearWavyProgressModifiers.kt`, lines 913 to 1052).
///
/// The track is one path there, drawn from the right to the left: a `lineTo` for each line, if there is room for it, and
/// a `moveTo` past the line, if it has a length. Each `lineTo` is a piece here, from where the path was to where it goes,
/// shown where androidx's condition holds; a gate says whether it does. The path's current point is a number, selected by
/// the same conditions.
pub(super) fn layout(pairs: &[(Value, Value)], width: f32) -> Layout {
    let c = cap_width(width);
    let far = width - c;
    // No number of the track is above the end of the track and the widest a line and its gap reach past it. The bound
    // only gives the graph's tables a range to hold the numbers in: a piece is drawn only where it starts above where
    // it ends, inside the track, so what lies above `far` is never drawn.
    let ceiling = width + 2.0 * (GAP + 2.0 * c) + 1.0;
    let floor = -(GAP + 2.0 * c) - 1.0;
    let mut layout = Layout::default();
    let mut next_end = Value::Fixed(far);
    let mut at = next_end;
    let mut gap = Value::Fixed(GAP);
    let mut visible = Value::Fixed(0.0);
    for (i, &(start, end)) in pairs.iter().enumerate() {
        let tail = start.scaled(width);
        let head = end.scaled(width);
        if i == 0 {
            // "Potentially shorten the gap and insets when the progress bar just enters the track."
            gap = Value::select(
                head.lt(Value::Fixed(c)),
                Value::Fixed(0.0),
                head.shifted(-c).at_most(GAP, floor),
            );
            visible = head.ge(Value::Fixed(c));
        }
        let adjusted_head = head.clamped(c, far);
        let adjusted_tail = tail.clamped(c, far);
        layout.lines.push(Piece {
            from: adjusted_tail,
            to: adjusted_head,
            shown: end.minus(start).differs(),
        });
        let spacing = gap.plus(visible.scaled(2.0 * c));
        let lead = adjusted_head.plus(spacing);
        let reaches = next_end.gt(lead);
        let to = lead.at_least(c, ceiling);
        layout.track.push(Piece {
            from: at,
            to,
            shown: reaches,
        });
        at = Value::select(reaches, to, at);
        let moves = head.gt(tail);
        let past = adjusted_tail.minus(spacing).at_least(c, ceiling);
        next_end = Value::select(moves, past, next_end);
        at = Value::select(moves, past, at);
    }
    layout.track.push(Piece {
        from: at,
        to: Value::Fixed(c),
        shown: next_end.gt(Value::Fixed(c)),
    });
    layout
}

/// The stop of `drawStopIndicator` for a determinate indicator `width` points wide at `progress` (a fraction held to 0 to
/// 1): a circle against the end of the track that shrinks when the line's end nears it, until it is gone
/// (`LinearWavyProgressModifiers.kt`, lines 641 to 689).
pub(super) fn stop(progress: f32, width: f32) -> Option<Stop> {
    stop_of(progress, width, TRACK_THICKNESS, STOP_SIZE)
}

/// [`stop`] for a track `track` points thick and a stop of at most `largest` points. The tokens make both 4, which
/// leaves the offset for a smaller stop unused; the arguments let a test reach it.
pub(super) fn stop_of(progress: f32, width: f32, track: f32, largest: f32) -> Option<Stop> {
    let mut diameter = track.min(largest);
    // "An additional offset in case the height of the progress bar is greater than the height of the indicator."
    let offset = if diameter < track { track / 4.0 } else { 0.0 };
    let mut x = width - diameter - offset;
    let reach = width * progress + cap_width(width);
    if x <= reach {
        diameter = (diameter - (reach - x)).max(0.0);
        x = reach;
    }
    (diameter > 0.0).then(|| Stop {
        x: x + diameter / 2.0,
        y: HEIGHT.0 / 2.0,
        radius: diameter / 2.0,
    })
}

// ---- drawing -------------------------------------------------------------------------------------------------------

/// What a drawing needs to know about the indicator: how wide it is, and which way it goes.
#[derive(Debug, Clone, Copy)]
pub(super) struct Frame {
    pub(super) width: f32,
    pub(super) ltr: bool,
}

impl Frame {
    /// The transform the drawing starts from: none from the left, and a half turn about the middle in a right-to-left
    /// layout, as androidx's `rotate(180f)` does (`LinearWavyProgressModifiers.kt`, line 551): the wave is turned over as
    /// well as mirrored.
    pub(super) fn base(self) -> [f32; 6] {
        if self.ltr {
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        } else {
            [-1.0, 0.0, 0.0, -1.0, self.width, HEIGHT.0]
        }
    }

    /// The painter set to the base transform.
    fn start(self, painter: Painter) -> Painter {
        painter.set_transform(self.base().map(CanvasArg::from))
    }
}

/// `painter`'s state for drawing the track: round lines as thick as the track.
pub(super) fn track(painter: Painter, layout: &Layout, frame: Frame) -> Painter {
    let y = HEIGHT.0 / 2.0;
    layout.track.iter().fold(
        frame
            .start(painter)
            .line_cap(LineCap::Round)
            .line_width(TRACK_THICKNESS),
        |painter, piece| {
            painter
                .global_alpha(piece.shown)
                .begin_path()
                .move_to(piece.from, y)
                .line_to(piece.to, y)
                .stroke()
        },
    )
}

/// The wave's lines, in `painter`, one for each window of `layout`, and the stop over them where there is one.
pub(super) fn lines(
    painter: Painter,
    layout: &Layout,
    wave: Wave,
    motion: Motion,
    stop: Option<Stop>,
    frame: Frame,
) -> Painter {
    let painter = layout.lines.iter().fold(painter, |painter, window| {
        line(painter, window, wave, motion, frame)
    });
    match stop {
        Some(dot) => frame
            .start(painter)
            .global_alpha(1.0)
            .begin_path()
            .arc(dot.x, dot.y, dot.radius, 0.0, TAU)
            .fill(),
        None => painter,
    }
}

/// How far a clip's half plane reaches from the edge, in points, for an indicator `width` points wide. A point of the
/// wave that is `dx` along and `dy` across from the end of a window is `(dx + s · dy) / (1 + s²)` along the clip's frame
/// and `(dy - s · dx) / (1 + s²)` across it, `s` being the wave's slope there. `|s| / (1 + s²)` is at most a half whatever
/// the slope, so the point is at most `width` and the height away along it and half the width and the height away across
/// it, and twice the width and twice the height reach every point of the window at every angle.
pub(super) fn half_plane(width: f32) -> f32 {
    2.0 * width + 2.0 * HEIGHT.0
}

/// The frame of a clip at `end`: a rotation, to within a scale, that turns the x axis to the wave's tangent there.
fn frame_at(end: &End) -> [CanvasArg; 6] {
    [
        CanvasArg::from(1.0),
        end.slope.into(),
        end.slope.scaled(-1.0).into(),
        CanvasArg::from(1.0),
        end.x.into(),
        end.y.into(),
    ]
}

/// A line of the wave: the wave, cut where `window` ends square to the wave, and a circle at each end.
fn line(painter: Painter, window: &Piece, wave: Wave, motion: Motion, frame: Frame) -> Painter {
    let tail = motion.end(wave, window.from);
    let head = motion.end(wave, window.to);
    let reach = half_plane(frame.width);
    let behind = Bounds::new(
        point(px(-reach), px(-reach)),
        size(px(reach), px(2.0 * reach)),
    );
    let ahead = Bounds::new(point(px(0.0), px(-reach)), size(px(reach), px(2.0 * reach)));
    // The base transform is set before the state is saved, so that what is drawn after the restore (the circles) is
    // turned as well.
    let painter = frame
        .start(painter)
        .global_alpha(window.shown)
        .save()
        .transform(frame_at(&head))
        .begin_path()
        .rect(behind)
        .clip();
    let painter = frame
        .start(painter)
        .transform(frame_at(&tail))
        .begin_path()
        .rect(ahead)
        .clip();
    // The path starts at the left of the indicator, `shift` before the window, which androidx's `translate` also does.
    let painter = frame
        .start(painter)
        .translate(motion.shift.scaled(-1.0), HEIGHT.0 / 2.0)
        .line_cap(LineCap::Butt)
        .line_width(INDICATOR_THICKNESS);
    let radius = INDICATOR_THICKNESS / 2.0;
    quads(painter, wave, motion.amplitude, frame.width)
        .stroke()
        .restore()
        .begin_path()
        .arc(tail.x, tail.y, radius, 0.0, TAU)
        .fill()
        .begin_path()
        .arc(head.x, head.y, radius, 0.0, TAU)
        .fill()
}

/// The path of the wave: a chain of quadratic curves, each half a wave long, from the left, as far as the indicator and
/// two wavelengths more (`updateFullPaths`, `LinearWavyProgressModifiers.kt`, lines 838 to 875). Each curve's control
/// point is the amplitude times `crest` above or below the line, one after the other.
fn quads(painter: Painter, wave: Wave, amplitude: Value, width: f32) -> Painter {
    let mut painter = painter.begin_path().move_to(0.0, 0.0);
    let half = wave.half();
    let reach = width + wave.length * 2.0;
    if half > 0.0 && reach.is_finite() {
        let up = painter.num(amplitude.scaled(wave.crest));
        let down = painter.num(amplitude.scaled(-wave.crest));
        let (mut anchor, mut control) = (half, half / 2.0);
        let mut upward = true;
        while anchor <= reach {
            painter = painter.op(CanvasOp::Path(PathOp::QuadTo([
                Num::Const(control),
                if upward { up } else { down },
                Num::Const(anchor),
                Num::Const(0.0),
            ])));
            anchor += half;
            control += half;
            upward = !upward;
        }
    }
    painter
}

// ---- what moves the wave -------------------------------------------------------------------------------------------

/// What the wave's travel is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Travel {
    /// Not moving: the indicator is flat, or the user asked for less motion.
    Rest,
    /// Moving, one wavelength a second, for as long as the wave has a height.
    Loop,
    /// Moving on while the amplitude shrinks to nothing, which is for the length of its transition, and then at rest: there
    /// is nothing left to see move.
    Settle,
}

/// What the wave's travel does for an indicator that is wavy (or not), that may move (or not), in `mode`.
pub(super) fn travel(wavy: bool, plays: bool, mode: Mode) -> Travel {
    if !wavy || !plays {
        return Travel::Rest;
    }
    match mode {
        Mode::Determinate(progress) if !has_waves(progress) => Travel::Settle,
        Mode::Determinate(_) | Mode::Moving | Mode::Still => Travel::Loop,
    }
}

/// The amplitude a wavy indicator in `mode` is heading for: its function of the progress when determinate, and 1 while
/// indeterminate (`targetAmplitude`: the function gets a NaN, which is above neither threshold,
/// `LinearWavyProgressModifiers.kt`, lines 447 to 450).
pub(super) fn target(mode: Mode) -> f32 {
    match mode {
        Mode::Determinate(progress) => amplitude_at(progress),
        Mode::Moving | Mode::Still => 1.0,
    }
}

/// The two values that move a wave: its amplitude, and how far it has travelled.
#[derive(Debug)]
pub(super) struct Waving {
    amplitude: AnimatedValue,
    /// The travelled fraction of a wavelength is this value and its offset, which counts up for ever: the offset holds
    /// what the value ran through before the loop that is on it now began, so that a loop that starts again goes on from
    /// where the wave was.
    phase: AnimatedValue,
    /// The loop that moves `phase` from 0 to 1, over and over.
    run: Animation,
    /// What the travel is doing now.
    travel: Cell<Travel>,
    /// The amplitude the value is at or on its way to, where the indicator is wavy.
    heading: Cell<Option<f32>>,
}

impl Waving {
    /// The values of an indicator that starts with `amplitude` (wavy, or flat).
    pub(super) fn new(amplitude: f32, wavy: bool) -> Self {
        let phase = animated_value(0.0);
        Self {
            amplitude: animated_value(amplitude),
            run: Animation::timing(&phase, 1.0, Timing::new(TRAVEL_MS, Easing::Linear))
                .looped(None),
            phase,
            travel: Cell::new(Travel::Rest),
            heading: Cell::new(wavy.then_some(amplitude)),
        }
    }

    /// The amplitude's node.
    pub(super) fn amplitude(&self) -> AnimatedNode {
        *self.amplitude.node()
    }

    /// The phase's node.
    pub(super) fn phase(&self) -> AnimatedNode {
        *self.phase.node()
    }

    /// The amplitude of a wave that is shown, or not (`wavy`), is to be `target`, which it takes at once or in its
    /// transition (`updateAmplitudeAnimation`, `LinearWavyProgressModifiers.kt`, lines 417 to 440): the wave grows along
    /// one curve and shrinks along another, and a user who asked for less motion gets it at once. An indicator that is
    /// flat draws no wave and does nothing, so one that turns flat while the amplitude is on its way stops it where it
    /// is: nothing draws it any more; when it turns wavy its amplitude is set to where it is to be.
    pub(super) fn reach(&self, wavy: bool, plays: bool, target: f32) {
        if !wavy {
            if self.heading.replace(None).is_some() {
                self.amplitude.stop();
            }
            return;
        }
        let was = self.heading.replace(Some(target));
        match was {
            Some(was) if was.to_bits() == target.to_bits() => {}
            Some(was) if plays => {
                let curve = if target > was { GROW } else { SHRINK };
                let _ = self
                    .amplitude
                    .timing_with(target, Timing::new(AMPLITUDE_MS, easing(curve)));
            }
            _ => self.amplitude.set(target),
        }
    }

    /// The travel is `now`.
    pub(super) fn go(&self, now: Travel) {
        let before = self.travel.replace(now);
        if before == now {
            return;
        }
        match (before, now) {
            // The wave was not moving and has no height to show now either, or it goes on settling.
            (Travel::Rest | Travel::Settle, Travel::Settle) => {}
            (_, Travel::Loop) => {
                // What the value has run through goes to its offset, so that the loop from 0 goes on from there. A
                // settling travel is a timing on the value itself, which would go on moving a value that has just been
                // emptied: it stops where it is first.
                self.phase.stop();
                self.phase.extract_offset();
                let _ = self.run.start();
            }
            (Travel::Loop, Travel::Settle) => {
                // The wave goes on at its speed for as long as the amplitude takes to reach nothing, a share of a wavelength.
                self.run.stop();
                self.phase.extract_offset();
                let _ = self
                    .phase
                    .timing(AMPLITUDE_MS / TRAVEL_MS, AMPLITUDE_MS, Easing::Linear);
            }
            (_, Travel::Rest) => {
                self.run.stop();
                self.phase.stop();
            }
        }
    }
}
