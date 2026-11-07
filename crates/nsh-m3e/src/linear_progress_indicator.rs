//! The Material 3 Expressive linear progress indicator: a track and an active indicator along it, with a dot at the
//! end of the track (the stop), flat or wavy, that shows how far the work is or that it is under way.
//!
//! It follows androidx's `LinearProgressIndicator` in `ProgressIndicator.kt` at the pinned commit, as [ADR 0012] says,
//! and draws what it draws, on two canvases: the track under the active indicator and the stop. Where a value is not
//! a token it is a literal with its line in androidx: [`DEFAULT_WIDTH`] and the stop's trailing space.
//!
//! The indicator is as wide as [`LinearProgressIndicatorProps::width`] and as tall as its stroke
//! (`LinearProgressIndicatorTokens.Height`). It is drawn from the end it starts at: the left in a left-to-right layout,
//! the right in a right-to-left one, as androidx mirrors it. The direction is the app's
//! (`nsh_core::element::layout_rtl`), as it is for `nsh-base`'s sliders: a `direction` style on a part of the tree
//! does not turn it.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::SvgPaint;
use nsh_core::canvas::LineCap;
use nsh_core::element::layout_rtl;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::progress::{self, Progress, indicator_color, track_color};
use crate::tokens::androidx::linear_progress_indicator_tokens as tokens;

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
/// goes), or `None` where it draws nothing: a round line of no length, and a fraction that is not a number.
fn line(start: f32, end: f32, width: f32, height: f32, ltr: bool) -> Option<Line> {
    let from_start = (if ltr { start } else { 1.0 - end }) * width;
    let to_end = (if ltr { end } else { 1.0 - start }) * width;
    let cap = cap(width, height);
    let (from, to) = match cap {
        Cap::Butt => (from_start, to_end),
        Cap::Round => {
            // The caps reach half the stroke past the line, so its ends stay that far inside the indicator.
            let inset = height / 2.0;
            let inside = |at: f32| at.clamp(inset, width - inset);
            (inside(from_start), inside(to_end))
        }
    };
    let has_length = match cap {
        Cap::Butt => true,
        Cap::Round => (end - start).abs() > 0.0,
    };
    (has_length && from.is_finite() && to.is_finite()).then_some(Line { from, to, cap })
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
    // The caps eat into the gap, so a round line's gap is wider by the stroke (`adjustedGapSize`).
    let adjusted_gap = match cap(width, height) {
        Cap::Butt => gap,
        Cap::Round => gap + height,
    };
    let gap_fraction = adjusted_gap / width;
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

/// Adds to `painter` a stroke along `line`, `thickness` points thick, at height `y`.
fn stroke(painter: Painter, line: Line, y: f32, thickness: f32) -> Painter {
    painter
        .line_cap(match line.cap {
            Cap::Butt => LineCap::Butt,
            Cap::Round => LineCap::Round,
        })
        .line_width(thickness)
        .begin_path()
        .move_to(line.from, y)
        .line_to(line.to, y)
        .stroke()
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
    /// How wide it is: [`DEFAULT_WIDTH`] by default.
    #[prop(default = DEFAULT_WIDTH)]
    pub width: Prop<Points>,
}

/// A Material 3 Expressive linear progress indicator.
///
/// It is a progress bar for screen readers, in the range 0 to 1 with the progress as its value, and none while
/// indeterminate.
///
/// ```no_run
/// use nsh_core::geometry::percentage;
/// use nsh_core::prelude::*;
/// use nsh_m3e::linear_progress_indicator::linear_progress_indicator;
/// use nsh_m3e::progress::Progress;
///
/// fn download(done: f32) -> impl IntoElement {
///     linear_progress_indicator()
///         .progress(Progress::Determinate(percentage(done)))
///         .label("Downloading")
/// }
/// ```
#[component]
pub fn LinearProgressIndicator(
    props: LinearProgressIndicatorProps,
    _cx: Scope<NoEvent>,
) -> impl IntoElement {
    let height = tokens::HEIGHT;
    let size = move || props.width.get();
    let tracks = canvas(move || {
        let width = size().0;
        let mut painter = Painter::new().stroke_style_from_view(SvgPaint::None);
        if let Progress::Determinate(fraction) = props.progress.get() {
            let drawn = determinate(
                fraction.0,
                width,
                height.0,
                tokens::TRACK_ACTIVE_SPACE.0,
                !layout_rtl(),
            );
            if let Some(track) = drawn.track {
                painter = stroke(painter, track, height.0 / 2.0, height.0);
            }
        }
        painter
    })
    .stroke_color(track_color());
    let indicator = canvas(move || {
        let width = size().0;
        let ltr = !layout_rtl();
        let mut painter = Painter::new()
            .stroke_style_from_view(SvgPaint::None)
            .fill_style_from_view(SvgPaint::None);
        if let Progress::Determinate(fraction) = props.progress.get() {
            let drawn = determinate(
                fraction.0,
                width,
                height.0,
                tokens::TRACK_ACTIVE_SPACE.0,
                ltr,
            );
            if let Some(indicator) = drawn.indicator {
                painter = stroke(painter, indicator, height.0 / 2.0, height.0);
            }
        }
        let dot = stop(tokens::STOP_SIZE.0, width, height.0, ltr);
        painter
            .begin_path()
            .arc(dot.x, dot.y, dot.radius, 0.0, std::f32::consts::TAU)
            .fill()
    })
    .stroke_color(indicator_color())
    .fill_color(indicator_color());
    progress::accessible(
        view(),
        move || props.progress.get().reading(),
        move || props.label.get(),
    )
    .w(size)
    .h(height)
    .child(layer(tracks))
    .child(layer(indicator))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::f32::consts::TAU;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing, Num, PathOp};
    use nsh_core::geometry::{Bounds, Percentage, Size, percentage};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, AccessibilityValue, Color, MountItem, NodeId, ViewKind};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::androidx_probe::{PROGRESS_INDICATOR_KT, line as kotlin_line};
    use crate::animation_probe::{Recording, layout_items};
    use crate::color::{ColorRole, ColorScheme};
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
        #[prop(default = DEFAULT_WIDTH)]
        width: Prop<Points>,
        label: Prop<Option<SharedString>>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let progress = signal(props.progress.get_untracked());
        // A row, so that the indicator keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    linear_progress_indicator()
                        .progress(move || progress.get())
                        .width(props.width)
                        .label(props.label),
                ),
            )
            .child(text("Half").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(0.5)));
            }))
            .child(text("Full").on_press(move |(): &()| {
                progress.set(Progress::Determinate(percentage(1.0)));
            }))
            .child(text("ready"))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        progress: Progress,
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
        /// The canvas that draws the track.
        track: NodeId,
        /// The canvas that draws the indicator and the stop.
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
            // The track first, under the indicator; and nothing else in the progress bar.
            let [track, indicator] = canvases[..] else {
                panic!("two canvases in the progress bar: {items:?}");
            };
            assert_eq!(children.get(&root).map(Vec::len), Some(2), "{items:?}");
            Self {
                root,
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
        /// The track's lines.
        track: Vec<Stroked>,
        /// The indicator's lines.
        indicator: Vec<Stroked>,
        /// The dots over them.
        dots: Vec<Dot>,
    }

    fn drawn_by(mounted: &Mounted) -> Drawn {
        let (track, no_dots) = drawn(&mounted.drawing(mounted.track).ops);
        assert_eq!(no_dots, Vec::new(), "the track has no dot");
        let (indicator, dots) = drawn(&mounted.drawing(mounted.indicator).ops);
        Drawn {
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
                        && !matches!(
                            item,
                            MountItem::Animation(
                                nsh_core::animation::AnimationOp::CreateValue { .. }
                            )
                        )),
                "{fraction}: {items:?}"
            );
            assert!(!replay(&recording).is_animating(), "{fraction}");
            assert!(recording.in_step("settle").is_empty(), "{fraction}");
        }
    }

    #[test]
    fn the_indicator_is_one_element_with_nothing_else_in_it() {
        // `Mounted::of` insists on the progress bar's two canvases and nothing else; its role is the one element.
        let mounted = mount(Config::default());
        assert_ne!(mounted.track, mounted.indicator);
        let _: Percentage = percentage(0.0);
    }
}
