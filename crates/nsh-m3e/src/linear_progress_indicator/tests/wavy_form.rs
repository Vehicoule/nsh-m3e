//! The tests of the wavy form (`wavy`): its geometry as pure functions, worked out by hand from androidx's Kotlin; what
//! its canvases draw, decoded back into lines, waves and circles and checked against the wave's closed form; and the
//! measurements of ADR 0010 for its two animations, the wave's travel and its amplitude.

use super::super::wavy::{
    AMPLITUDE_MS, DETERMINATE_WAVELENGTH, FLAT_AT_OR_ABOVE, FLAT_AT_OR_BELOW, GAP, HEIGHT,
    HEIGHT_STEPS, INDETERMINATE_WAVELENGTH, INDICATOR_THICKNESS, Layout, Piece, STOP_SIZE,
    TRACK_THICKNESS, TRAVEL_MS, Travel, Wave, amplitude_at, cap_width, half_plane, has_waves,
    layout, stop, stop_of, target, travel,
};
use nsh_core::animation::{Composite, Driver};

use super::*;
use crate::androidx_probe::{LINEAR_WAVY_MODIFIERS_KT, WAVY_PROGRESS_INDICATOR_KT};

/// The lines of `file` that a citation points at: each is the text it must have, so a bump of the pinned commit that
/// moves one fails here.
fn pinned(file: &str, lines: &[(usize, &str)]) {
    for &(number, text) in lines {
        assert_eq!(kotlin_line(file, number), text, "line {number}");
    }
}

// ---- the literals, each at its line in androidx -----------------------------------------------------------------

#[test]
fn the_sizes_of_the_wavy_form_are_the_tokens_and_their_lines_in_androidx() {
    // The sizes are the tokens', which `WavyProgressIndicatorDefaults` reads (`WavyProgressIndicator.kt`).
    assert_eq!(HEIGHT, Points(10.0));
    assert_eq!(tokens::WAVE_HEIGHT, Points(10.0));
    pinned(
        WAVY_PROGRESS_INDICATOR_KT,
        &[
            (
                466,
                "public val LinearContainerHeight: Dp = LinearProgressIndicatorTokens.WaveHeight",
            ),
            (
                459,
                "public val LinearDeterminateWavelength: Dp = LinearProgressIndicatorTokens.ActiveWaveWavelength",
            ),
            (462, "public val LinearIndeterminateWavelength: Dp ="),
            (
                463,
                "LinearProgressIndicatorTokens.IndeterminateActiveWaveWavelength",
            ),
            (
                472,
                "public val LinearTrackStopIndicatorSize: Dp = LinearProgressIndicatorTokens.StopSize",
            ),
            (
                484,
                "public val LinearIndicatorTrackGapSize: Dp = LinearProgressIndicatorTokens.TrackActiveSpace",
            ),
            (417, "LinearProgressIndicatorTokens.ActiveThickness.toPx()"),
            (419, "cap = StrokeCap.Round,"),
            (441, "LinearProgressIndicatorTokens.TrackThickness.toPx()"),
            (443, "cap = StrokeCap.Round,"),
        ],
    );
    assert_eq!(DETERMINATE_WAVELENGTH.to_bits(), 40.0_f32.to_bits());
    assert_eq!(tokens::ACTIVE_WAVE_WAVELENGTH, Points(40.0));
    assert_eq!(INDETERMINATE_WAVELENGTH.to_bits(), 20.0_f32.to_bits());
    assert_eq!(tokens::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH, Points(20.0));
    assert_eq!(INDICATOR_THICKNESS.to_bits(), 4.0_f32.to_bits());
    assert_eq!(tokens::ACTIVE_THICKNESS, Points(4.0));
    assert_eq!(TRACK_THICKNESS.to_bits(), 4.0_f32.to_bits());
    assert_eq!(tokens::TRACK_THICKNESS, Points(4.0));
    assert_eq!(GAP.to_bits(), 4.0_f32.to_bits());
    assert_eq!(tokens::TRACK_ACTIVE_SPACE, Points(4.0));
    assert_eq!(STOP_SIZE.to_bits(), 4.0_f32.to_bits());
    assert_eq!(tokens::STOP_SIZE, Points(4.0));
}

#[test]
fn the_thresholds_and_the_transitions_of_the_amplitude_are_their_lines_in_androidx() {
    // The amplitude is 1 between a tenth and 95 % of the progress.
    assert_eq!(FLAT_AT_OR_BELOW.to_bits(), 0.1_f32.to_bits());
    assert_eq!(FLAT_AT_OR_ABOVE.to_bits(), 0.95_f32.to_bits());
    pinned(
        WAVY_PROGRESS_INDICATOR_KT,
        &[
            (
                494,
                "// Sets the amplitude to the max on 10%, and back to zero on 95% of the progress.",
            ),
            (495, "if (progress <= 0.1f || progress >= 0.95f) {"),
            (496, "0f"),
            (498, "1f"),
        ],
    );
    // The amplitude changes in `DurationLong2`, 500 ms, along `EasingStandardCubicBezier` as it grows and
    // `EasingEmphasizedAccelerateCubicBezier` as it shrinks.
    assert_eq!(AMPLITUDE_MS.to_bits(), 500.0_f32.to_bits());
    pinned(
        WAVY_PROGRESS_INDICATOR_KT,
        &[
            (
                504,
                "internal val IncreasingAmplitudeAnimationSpec: AnimationSpec<Float> =",
            ),
            (506, "durationMillis = MotionTokens.DurationLong2.toInt(),"),
            (507, "easing = MotionTokens.EasingStandardCubicBezier,"),
            (
                511,
                "internal val DecreasingAmplitudeAnimationSpec: AnimationSpec<Float> =",
            ),
            (513, "durationMillis = MotionTokens.DurationLong2.toInt(),"),
            (
                514,
                "easing = MotionTokens.EasingEmphasizedAccelerateCubicBezier,",
            ),
        ],
    );
    pinned(
        MOTION_TOKENS_KT,
        &[
            (29, "const val DurationLong2 = 500.0"),
            (44, "get() = CubicBezierEasing(0.3f, 0.0f, 0.8f, 0.15f)"),
            (62, "get() = CubicBezierEasing(0.2f, 0.0f, 0.0f, 1.0f)"),
        ],
    );
}

#[test]
fn the_travel_is_one_wavelength_a_second_and_turns_with_the_layout_direction_in_androidx() {
    // One wavelength a second: the speed is the wavelength, so the duration is `wavelength / speed * 1000`.
    assert_eq!(TRAVEL_MS.to_bits(), 1000.0_f32.to_bits());
    pinned(
        WAVY_PROGRESS_INDICATOR_KT,
        &[
            (
                130,
                "waveSpeed: Dp = wavelength, // Match to 1 wavelength per second",
            ),
            (
                203,
                "waveSpeed: Dp = wavelength, // Match to 1 wavelength per second",
            ),
        ],
    );
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[
            (103, "indeterminateWaveSpeed: Dp = indeterminateWavelength,"),
            (
                352,
                "get() = if (indeterminate) indeterminateWaveSpeed else waveSpeed",
            ),
            (388, "((currentWavelength / currentWaveSpeed) * 1000)"),
            (395, "val endValue = startValue + 1f"),
            (
                401,
                "animation = tween(durationMillis, easing = LinearEasing),",
            ),
            (402, "repeatMode = RepeatMode.Restart,"),
            (405, "waveOffset.floatValue = value % 1f"),
            // The wave's offset is 0 until its animation moves it.
            (329, "private val waveOffset = mutableFloatStateOf(0f)"),
        ],
    );
    // A turn of 180 degrees in a right-to-left layout, the wave with it.
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[(
            551,
            "rotate(if (layoutDirection == LayoutDirection.Ltr) 0f else 180f) {",
        )],
    );
}

#[test]
fn the_path_of_the_wave_is_its_lines_in_androidx() {
    // The path of the wave: quadratic curves a half wave long, their control points the container's height less the
    // stroke high, up and down in turn, as far as the width and two wavelengths.
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[
            (845, "val halfWavelengthPx = wavelength / 2f"),
            (848, "var controlX = halfWavelengthPx / 2f"),
            (857, "var controlY = height - stroke.width"),
            (864, "val widthWithExtraPhase = width + wavelength * 2"),
            (866, "while (anchorX <= widthWithExtraPhase) {"),
            (
                867,
                "fullProgressPath.quadraticTo(controlX, controlY, anchorX, anchorY)",
            ),
            (870, "controlY *= -1f"),
            (
                875,
                "fullProgressPath.translate(Offset(x = 0f, y = height / 2f))",
            ),
        ],
    );
}

#[test]
fn the_track_and_the_lines_are_laid_out_as_their_lines_in_androidx_say() {
    // The room the round caps take, and what the track and the lines do with the progress.
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[
            (
                829,
                "(stroke.cap == StrokeCap.Butt && trackStroke.cap == StrokeCap.Butt) ||",
            ),
            (830, "height > width"),
            (834, "max(stroke.width / 2, trackStroke.width / 2)"),
            (
                948,
                "var nextEndTrackOffset = width - currentStrokeCapWidth",
            ),
            (968, "if (barHead < currentStrokeCapWidth) {"),
            (972, "barHead - currentStrokeCapWidth,"),
            (
                973,
                "currentIndicatorTrackGapSize, /*+ currentStrokeCapWidth * 2*/",
            ),
            (
                976,
                "activeIndicatorVisible = barHead >= currentStrokeCapWidth",
            ),
            (
                981,
                "barHead.fastCoerceIn(currentStrokeCapWidth, width - currentStrokeCapWidth)",
            ),
            (
                986,
                "if (abs(endProgressFraction - startProgressFraction) > 0) {",
            ),
            (1023, "adjustedTrackGapSize + currentStrokeCapWidth * 2"),
            (
                1027,
                "if (nextEndTrackOffset > adjustedBarHead + adaptiveTrackSpacing) {",
            ),
            (
                1029,
                "x = max(currentStrokeCapWidth, adjustedBarHead + adaptiveTrackSpacing),",
            ),
            (1034, "if (barHead > barTail) {"),
            (
                1037,
                "max(currentStrokeCapWidth, adjustedBarTail - adaptiveTrackSpacing)",
            ),
            (1044, "if (nextEndTrackOffset > currentStrokeCapWidth) {"),
        ],
    );
}

#[test]
fn the_stop_and_the_amplitude_and_progress_it_reads_are_their_lines_in_androidx() {
    // The stop.
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[
            (
                649,
                "var stopIndicatorSize = min(trackStroke.width, maxStopIndicatorSize.toPx())",
            ),
            (653, "if (stopIndicatorSize == trackStroke.width) {"),
            (656, "trackStroke.width / 4f"),
            (
                659,
                "var indicatorX = progressIndicatorSize.width - stopIndicatorSize - indicatorXOffset",
            ),
            (
                660,
                "val progressX = progressIndicatorSize.width * progressEnd + horizontalInsets",
            ),
            (661, "if (indicatorX <= progressX) {"),
            (
                662,
                "stopIndicatorSize = max(0f, stopIndicatorSize - (progressX - indicatorX))",
            ),
            (666, "if (stopIndicatorSize > 0) {"),
            (673, "x = indicatorX + stopIndicatorSize / 2f,"),
            (674, "y = progressIndicatorSize.height / 2f,"),
        ],
    );
    // What the amplitude and the progress are, determinate and not.
    pinned(
        LINEAR_WAVY_MODIFIERS_KT,
        &[
            (
                444,
                "return if (!currentProgress.isFinite()) 0f else currentProgress.fastCoerceIn(0f, 1f)",
            ),
            (
                448,
                "val value = amplitude(if (indeterminate) Float.NaN else determinateProgress())",
            ),
            (
                449,
                "return if (value.isNaN()) 1f else value.fastCoerceIn(0f, 1f)",
            ),
            (
                425,
                "if (isAttached && currentTargetAmplitude != targetAmplitude) {",
            ),
            (
                432,
                "if (currentAmplitudeAnimatable.value < targetAmplitude) {",
            ),
            (433, "IncreasingAmplitudeAnimationSpec"),
            (435, "DecreasingAmplitudeAnimationSpec"),
        ],
    );
}

// ---- the wave -----------------------------------------------------------------------------------------------------

/// The point of the quadratic Bézier curve from `from`, by `control`, to `to`, at `t`: `(1-t)² P0 + 2(1-t)t P1 + t² P2`.
fn quadratic(from: (f64, f64), control: (f64, f64), to: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let blend = |a: f64, b: f64, c: f64| u * u * a + 2.0 * u * t * b + t * t * c;
    (
        blend(from.0, control.0, to.0),
        blend(from.1, control.1, to.1),
    )
}

#[test]
fn a_half_wave_is_the_quadratic_curve_androidx_plots_and_the_wave_peaks_at_the_tokens_amplitude() {
    // `controlY = height - stroke.width` = 10 - 4 = 6, "the height of the control point ... will plot a wave that
    // peaks at half the height" (`LinearWavyProgressModifiers.kt`, lines 850 to 857): 3, the token
    // `ActiveWaveAmplitude`.
    let wave = Wave::new(40.0);
    assert_eq!(wave.crest.to_bits(), 6.0_f32.to_bits());
    assert_eq!(wave.length.to_bits(), 40.0_f32.to_bits());
    assert_eq!(tokens::ACTIVE_WAVE_AMPLITUDE, Points(3.0));
    assert_eq!(
        wave.height(0.5).to_bits(),
        tokens::ACTIVE_WAVE_AMPLITUDE.0.to_bits()
    );
    assert_eq!(
        wave.height(1.5).to_bits(),
        (-tokens::ACTIVE_WAVE_AMPLITUDE.0).to_bits()
    );
    assert_eq!(Wave::new(20.0).crest.to_bits(), 6.0_f32.to_bits());
    // The ends of every half wave are on the line; the wave goes down first (a positive control height is below the
    // line on a canvas) and then up, and the next wave starts as this one did.
    for v in [0.0, 1.0, 2.0] {
        assert!(wave.height(v).abs() < 1e-6, "{v}");
    }
    // Each half wave is exactly the curve `quadraticTo(controlX, controlY, anchorX, 0)` draws: x is linear in the
    // parameter (the control is half way between the anchors), so the height at `v` of a half wave is the curve's at
    // `t = v`. Compared against the Bézier curve evaluated as it is defined, for both wavelengths and both signs.
    for length in [40.0_f32, 20.0] {
        let wave = Wave::new(length);
        let half = f64::from(length) / 2.0;
        for (index, sign) in [(0.0_f32, 1.0_f64), (1.0, -1.0)] {
            for step in 0..=64_u16 {
                let t = f64::from(step) / 64.0;
                let (x, y) = quadratic((0.0, 0.0), (half / 2.0, sign * 6.0), (half, 0.0), t);
                // x is linear in t: the half wave's own scale is x / half, which is t.
                assert!((x - half * t).abs() < 1e-9, "{length} {t}");
                let v = index + f32::from(step) / 64.0;
                assert!(
                    (f64::from(wave.height(v)) - y).abs() < 1e-4,
                    "{length}, half wave {index}, t {t}: {} is not {y}",
                    wave.height(v)
                );
                // The slope is the curve's derivative along x: dy/dt over dx/dt.
                let (dx, dy) = (
                    2.0 * (1.0 - t) * (half / 2.0) + 2.0 * t * (half - half / 2.0),
                    2.0 * (1.0 - t) * (sign * 6.0) + 2.0 * t * (-sign * 6.0),
                );
                assert!(
                    (f64::from(wave.slope(v)) - dy / dx).abs() < 1e-4,
                    "{length}, half wave {index}, t {t}: slope {} is not {}",
                    wave.slope(v),
                    dy / dx
                );
            }
        }
    }
}

#[test]
fn the_table_the_graph_reads_the_wave_from_is_off_by_less_than_three_thousandths_of_a_point() {
    // `Motion::end` gives a node of the graph the height of the wave from a table of `HEIGHT_STEPS` points to each
    // half wave, joined by straight lines. A parabola of height 12 t (1 - t) is off by at most 24/8 of the square of
    // the step.
    assert_eq!(HEIGHT_STEPS, 32);
    let wave = Wave::new(40.0);
    let step = 1.0 / f32::from(HEIGHT_STEPS);
    let mut worst = 0.0_f32;
    for fine in 0..=4096_u16 {
        let v = f32::from(fine) / 2048.0;
        let low = (v / step).floor() * step;
        let share = (v - low) / step;
        let (a, b) = (wave.height(low), wave.height((low + step).min(2.0)));
        // The point after the last of a half wave starts the next, so the end of the second is at 2.
        let interpolated = a + (b - a) * share;
        // The two sides of an anchor have the same height, 0, so the table has no jump there.
        worst = worst.max((interpolated - wave.height(v)).abs());
    }
    assert!(worst < 0.003, "{worst}");
    assert!(worst > 0.002, "{worst}: the bound is not slack");
}

#[test]
fn the_amplitude_is_1_between_a_tenth_and_95_percent_of_the_progress_and_0_outside_it() {
    let past_a_tenth = f32::from_bits(0.1_f32.to_bits() + 1);
    let short_of_95 = f32::from_bits(0.95_f32.to_bits() - 1);
    for (progress, amplitude, what) in [
        (0.0, 0.0, "none"),
        (0.05, 0.0, "a twentieth"),
        // Strict: the thresholds themselves are flat.
        (0.1, 0.0, "a tenth"),
        (past_a_tenth, 1.0, "just past a tenth"),
        (0.5, 1.0, "half"),
        (short_of_95, 1.0, "just short of 95 %"),
        (0.95, 0.0, "95 %"),
        (0.99, 0.0, "99 %"),
        (1.0, 0.0, "all"),
        // Held to 0 to 1 first, and not finite counts as none.
        (1.5, 0.0, "more than all"),
        (-0.5, 0.0, "less than none"),
        (f32::NAN, 0.0, "not a number"),
        (f32::INFINITY, 0.0, "infinite"),
        (f32::NEG_INFINITY, 0.0, "minus infinite"),
    ] {
        assert_eq!(
            amplitude_at(progress).to_bits(),
            f32::to_bits(amplitude),
            "{what}"
        );
        assert_eq!(has_waves(progress), amplitude > 0.5, "{what}");
    }
}

#[test]
fn the_amplitude_an_indicator_heads_for_is_its_function_of_the_progress_and_1_while_indeterminate()
{
    for (mode, expected) in [
        (Mode::Determinate(0.5), 1.0_f32),
        (Mode::Determinate(0.05), 0.0),
        (Mode::Determinate(0.99), 0.0),
        (Mode::Determinate(0.0), 0.0),
        // "The function gets a NaN, which is above neither threshold": the full wave.
        (Mode::Moving, 1.0),
        (Mode::Still, 1.0),
    ] {
        assert_eq!(target(mode).to_bits(), expected.to_bits(), "{mode:?}");
    }
}

#[test]
fn the_wave_travels_while_it_has_a_height_to_show_and_settles_when_it_loses_it() {
    use Travel::{Loop, Rest, Settle};
    for (wavy, plays, mode, expected) in [
        // A flat indicator and one told to hold still do not move the wave.
        (false, true, Mode::Moving, Rest),
        (false, true, Mode::Determinate(0.5), Rest),
        (false, false, Mode::Determinate(0.5), Rest),
        (true, false, Mode::Determinate(0.5), Rest),
        (true, false, Mode::Moving, Rest),
        (true, false, Mode::Still, Rest),
        // A wave with a height goes on.
        (true, true, Mode::Determinate(0.5), Loop),
        (true, true, Mode::Moving, Loop),
        (true, true, Mode::Still, Loop),
        // A wave that is to lose its height goes on while it does, then rests.
        (true, true, Mode::Determinate(0.0), Settle),
        (true, true, Mode::Determinate(0.05), Settle),
        (true, true, Mode::Determinate(0.1), Settle),
        (true, true, Mode::Determinate(0.95), Settle),
        (true, true, Mode::Determinate(1.0), Settle),
    ] {
        assert_eq!(
            travel(wavy, plays, mode),
            expected,
            "wavy {wavy}, plays {plays}, {mode:?}"
        );
    }
}

// ---- the track and the lines, from androidx's arithmetic ---------------------------------------------------------

/// The number a fixed value is.
fn fixed(value: Value) -> f32 {
    match value {
        Value::Fixed(number) => number,
        Value::Moving(_) => panic!("a node of the graph where a fixed number was laid out"),
    }
}

/// The from and to of pieces, in points.
type Spans = Vec<(f32, f32)>;

/// From and to of the pieces of `layout`'s track that are drawn, and of its lines.
fn drawn_pieces(layout: &Layout) -> (Spans, Spans) {
    let pieces = |pieces: &[Piece]| {
        pieces
            .iter()
            .filter(|piece| {
                let shown = fixed(piece.shown);
                assert!(
                    shown.to_bits() == 0 || shown.to_bits() == 1.0_f32.to_bits(),
                    "a fixed gate is 0 or 1, not {shown}"
                );
                shown > 0.5
            })
            .map(|piece| (fixed(piece.from), fixed(piece.to)))
            .collect::<Vec<_>>()
    };
    (pieces(&layout.track), pieces(&layout.lines))
}

/// `found` and `expected` are the same pieces, to a thousandth of a point.
fn assert_pieces(found: &[(f32, f32)], expected: &[(f32, f32)], what: &str) {
    assert_eq!(
        found.len(),
        expected.len(),
        "{what}: {found:?} is not {expected:?}"
    );
    for (found, expected) in found.iter().zip(expected) {
        assert!(
            near(found.0, expected.0) && near(found.1, expected.1),
            "{what}: {found:?} is not {expected:?}"
        );
    }
}

/// The layout of a determinate indicator `width` points wide at `fraction`.
fn determinate_layout(fraction: f32, width: f32) -> Layout {
    layout(&[(Value::Fixed(0.0), Value::Fixed(fraction))], width)
}

#[test]
fn a_determinate_wavy_indicator_lays_out_the_track_and_the_line_as_androidx_does() {
    // Width 240, round caps of 2 points, a gap of 4. Each case is androidx's `updateDrawPaths`
    // (`LinearWavyProgressModifiers.kt`, lines 913 to 1052) worked out by hand: the track is drawn from the right,
    // 238, to where the line's head and its spacing are; the line is from 2 (the tail, held inside by the cap) to the
    // head.
    for (fraction, track, line, what) in [
        // Nothing done: the whole track, 238 to 2 and its last piece, a dot at 2 (`lineTo(c)` from where the path
        // is); and no line, as its length is 0.
        (0.0, vec![(238.0, 2.0), (2.0, 2.0)], vec![], "0"),
        // A head of 0.96 points is under the cap: the gap is 0, the line is as long as its cap, and the track
        // reaches it.
        (0.004, vec![(238.0, 2.0)], vec![(2.0, 2.0)], "0.004"),
        // A head of 4.8: the gap is shortened to 4.8 - 2 = 2.8, so the spacing is 2.8 + 4 = 6.8 and the track
        // starts at 4.8 + 6.8 = 11.6.
        (0.02, vec![(238.0, 11.6)], vec![(2.0, 4.8)], "0.02"),
        // 24 + 8 = 32.
        (0.1, vec![(238.0, 32.0)], vec![(2.0, 24.0)], "0.1"),
        // 120 + 8 = 128: 4 points between the caps (122 and 126).
        (0.5, vec![(238.0, 128.0)], vec![(2.0, 120.0)], "0.5"),
        (0.9, vec![(238.0, 224.0)], vec![(2.0, 216.0)], "0.9"),
        // 228 + 8 = 236 is still before 238.
        (0.95, vec![(238.0, 236.0)], vec![(2.0, 228.0)], "0.95"),
        // 230.4 + 8 = 238.4 is not: the track ends where it starts, and `nextEndTrackOffset > ...` fails.
        (0.96, vec![], vec![(2.0, 230.4)], "0.96"),
        (1.0, vec![], vec![(2.0, 238.0)], "1"),
    ] {
        let (found_track, found_line) = drawn_pieces(&determinate_layout(fraction, 240.0));
        assert_pieces(&found_track, &track, &format!("{what}: track"));
        assert_pieces(&found_line, &line, &format!("{what}: line"));
    }
}

#[test]
fn a_wavy_track_is_made_of_one_piece_per_line_and_one_after_them() {
    // A `lineTo` for each line and the last `lineTo`, each shown or not, as the conditions say.
    for lines in 1..4_u8 {
        let pairs: Vec<(Value, Value)> = (0..lines)
            .map(|i| {
                (
                    Value::Fixed(f32::from(i) * 0.3),
                    Value::Fixed(f32::from(i) * 0.3 + 0.2),
                )
            })
            .collect();
        let layout = layout(&pairs, 240.0);
        assert_eq!(layout.lines.len(), usize::from(lines));
        assert_eq!(layout.track.len(), usize::from(lines) + 1);
    }
}

#[test]
fn a_head_exactly_at_the_cap_s_width_makes_the_line_visible_and_one_short_of_it_does_not() {
    // `activeIndicatorVisible = barHead >= currentStrokeCapWidth` (`LinearWavyProgressModifiers.kt`, line 972), and
    // the gap is `min(barHead - cap, gap)`, 0 at the cap. On a track 64 points wide a fraction of 1/32 is 2 points
    // exactly, the cap's width. The line shows, so the track keeps off it by the two caps that meet there, 2 + 2, on
    // top of its gap of 0: it starts at 2 + 4 = 6.
    let (track, line) = drawn_pieces(&determinate_layout(1.0 / 32.0, 64.0));
    assert_pieces(&track, &[(62.0, 6.0)], "at the cap, track");
    assert_pieces(&line, &[(2.0, 2.0)], "at the cap, line");
    // Under the cap the line is not visible: no room kept for its caps, no gap, and the track reaches the line.
    let (track, line) = drawn_pieces(&determinate_layout(0.031, 64.0));
    assert_pieces(&track, &[(62.0, 2.0)], "under the cap, track");
    assert_pieces(&line, &[(2.0, 2.0)], "under the cap, line");
    // Just past it the gap has begun to open, 0.0032 of a point.
    let (track, _) = drawn_pieces(&determinate_layout(0.0313, 64.0));
    assert_pieces(&track, &[(62.0, 6.0064)], "past the cap, track");
}

#[test]
fn a_head_past_the_end_of_the_track_is_held_inside_it_by_the_cap() {
    // A head past the end of the track is held inside it by the cap (`fastCoerceIn(c, width - c)`, line 981), so a
    // fraction of 1 and one a quarter more lay out the same.
    let (track, line) = drawn_pieces(&layout(&[(Value::Fixed(0.0), Value::Fixed(1.25))], 240.0));
    assert_eq!(track, vec![]);
    assert_pieces(&line, &[(2.0, 238.0)], "past the end");
}

#[test]
fn an_indicator_narrower_than_it_is_tall_keeps_no_room_for_caps() {
    // 10 points is as wide as it is tall, so the caps fit, and 2 points are left either side.
    assert_eq!(cap_width(10.0).to_bits(), 2.0_f32.to_bits());
    assert_eq!(cap_width(240.0).to_bits(), 2.0_f32.to_bits());
    // Narrower: `height > width`, no room for the caps, and none left for them (`LinearWavyProgressModifiers.kt`,
    // lines 826 to 835).
    assert_eq!(cap_width(9.9).to_bits(), 0.0_f32.to_bits());
    assert_eq!(cap_width(8.0).to_bits(), 0.0_f32.to_bits());
    // Width 10, half done: a head of 5 is past the cap, the gap is 5 - 2 = 3 and the spacing 3 + 4 = 7, so the track
    // would start at 12, past its end (8): none. The line is from 2 to 5.
    let (track, line) = drawn_pieces(&determinate_layout(0.5, 10.0));
    assert_eq!(track, vec![]);
    assert_pieces(&line, &[(2.0, 5.0)], "10 wide");
    // Width 8, half done: no caps, so the head is at 4, the gap min(4 - 0, 4) = 4 and the spacing 4, so the track
    // would start at 8, which is not before its end. The line is from 0 to 4.
    let (track, line) = drawn_pieces(&determinate_layout(0.5, 8.0));
    assert_eq!(track, vec![]);
    assert_pieces(&line, &[(0.0, 4.0)], "8 wide");
    // Width 100, half done: a gap of 4, spacing 8, the line from 2 to 50 and the track from 98 to 58.
    let (track, line) = drawn_pieces(&determinate_layout(0.5, 100.0));
    assert_pieces(&track, &[(98.0, 58.0)], "100 wide, track");
    assert_pieces(&line, &[(2.0, 50.0)], "100 wide, line");
}

#[test]
fn the_stop_sits_against_the_end_of_the_track_and_shrinks_as_the_line_reaches_it() {
    let at = |fraction: f32, width: f32| stop(fraction, width).map(|s| (s.x, s.y, s.radius));
    let same =
        |found: Option<(f32, f32, f32)>, expected: Option<(f32, f32, f32)>, what: &str| match (
            found, expected,
        ) {
            (Some(found), Some(expected)) => assert!(
                near(found.0, expected.0) && near(found.1, expected.1) && near(found.2, expected.2),
                "{what}: {found:?} is not {expected:?}"
            ),
            (None, None) => {}
            (found, expected) => panic!("{what}: {found:?} is not {expected:?}"),
        };
    // 4 points in 4: no room to spare, the dot is 4 wide against the end (240 - 4 = 236 to 240) at the middle of
    // the 10 points.
    same(at(0.0, 240.0), Some((238.0, 5.0, 2.0)), "none");
    same(at(0.5, 240.0), Some((238.0, 5.0, 2.0)), "half");
    // The line's end and its cap reach 232.4 (0.96) and 234.8 (0.97): short of the dot's 236.
    same(at(0.96, 240.0), Some((238.0, 5.0, 2.0)), "0.96");
    same(at(0.97, 240.0), Some((238.0, 5.0, 2.0)), "0.97");
    // 0.98: the reach is 237.2, 1.2 past the dot's start: the dot is 4 - 1.2 = 2.8 wide from 237.2.
    same(at(0.98, 240.0), Some((238.6, 5.0, 1.4)), "0.98");
    // 0.99: 239.6, 3.6 past: 0.4 wide.
    same(at(0.99, 240.0), Some((239.8, 5.0, 0.2)), "0.99");
    // 1: 242, 6 past: gone (`max(0f, ...)`, and no dot where its size is not above 0).
    same(at(1.0, 240.0), None, "all");
    // An indicator without caps (narrower than tall) has no room kept for them at the end of the line.
    same(at(0.5, 8.0), Some((6.0, 5.0, 2.0)), "8 wide, half");
    same(at(0.5, 10.0), Some((8.5, 5.0, 1.5)), "10 wide, half");
}

#[test]
fn a_stop_smaller_than_the_track_is_thick_is_moved_in_by_a_quarter_of_the_track() {
    // `drawStopIndicator` (`LinearWavyProgressModifiers.kt`, lines 650 to 664): a stop of 2 on a track 4 thick is 2
    // wide, and `indicatorXOffset` is 4 / 4 = 1, so on a track 240 wide it starts at 240 - 2 - 1 = 237 and its centre
    // is at 238. The tokens make the stop as thick as the track, so only these arguments reach that branch.
    let at = |progress: f32, track: f32, largest: f32| {
        stop_of(progress, 240.0, track, largest).map(|s| (s.x, s.y, s.radius))
    };
    assert_eq!(at(0.0, 4.0, 2.0), Some((238.0, 5.0, 1.0)));
    // No offset where the stop is as thick as the track, or the track is the thinner (the stop is held to it).
    assert_eq!(at(0.0, 4.0, 4.0), Some((238.0, 5.0, 2.0)));
    assert_eq!(at(0.0, 4.0, 8.0), Some((238.0, 5.0, 2.0)));
    // The line's end reaches into it as it does a full-size one: 240 * 0.98 + 2 = 237.2, 0.2 past 237, so the stop is
    // 1.8 wide from 237.2, and the one that is 2.6 past is gone.
    let near_end = at(0.98, 4.0, 2.0).expect("a stop 1.8 wide");
    assert!(
        near(near_end.0, 238.1) && near(near_end.2, 0.9),
        "{near_end:?}"
    );
    assert_eq!(at(0.99, 4.0, 2.0), None);
}

#[test]
fn the_still_pose_of_a_wavy_indeterminate_indicator_is_laid_out_as_androidx_lays_the_lines() {
    // The pose at 1000 ms (the flat form's, with the first head at the end, the first tail at 0.40559, the second head
    // at 0.10063 and its tail at 0.00767: 97.34052, 24.15111 and 1.84 points of 240), laid out by `updateDrawPaths`:
    // line 1 from its tail to the end (238, held inside by the cap), line 2 from its tail (held to 2) to its head,
    // and the track between them: from the tail less the spacing, 89.34, to the head and the spacing, 32.15.
    let pose = still_pose();
    let pairs = [
        (Value::Fixed(pose.first_tail), Value::Fixed(pose.first_head)),
        (
            Value::Fixed(pose.second_tail),
            Value::Fixed(pose.second_head),
        ),
    ];
    let (track, lines) = drawn_pieces(&layout(&pairs, 240.0));
    assert_pieces(&track, &[(89.340_52, 32.151_11)], "track");
    assert_pieces(&lines, &[(97.340_52, 238.0), (2.0, 24.151_11)], "lines");
}

#[test]
fn a_line_of_no_length_is_not_shown_and_a_line_that_is_one_is() {
    // `abs(end - start) > 0` (line 986): a gate of the fractions, not of the points the caps hold them to.
    let shown = |start: f32, end: f32| {
        let layout = layout(&[(Value::Fixed(start), Value::Fixed(end))], 240.0);
        fixed(layout.lines[0].shown) > 0.5
    };
    assert!(!shown(0.3, 0.3));
    assert!(shown(0.3, 0.30001));
    assert!(shown(0.3, 0.2));
    assert!(!shown(0.0, 0.0));
}

// ---- what the canvases draw, decoded ------------------------------------------------------------------------------

/// A canvas transform as `[a, b, c, d, e, f]`: `x' = a x + c y + e` and `y' = b x + d y + f`.
type Matrix = [f64; 6];

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// The transform that does `then` after `first`: what `transform(...)` makes of the current transform and a matrix.
fn compose(first: Matrix, then: Matrix) -> Matrix {
    let (m, n) = (first, then);
    [
        m[0] * n[0] + m[2] * n[1],
        m[1] * n[0] + m[3] * n[1],
        m[0] * n[2] + m[2] * n[3],
        m[1] * n[2] + m[3] * n[3],
        m[0] * n[4] + m[2] * n[5] + m[4],
        m[1] * n[4] + m[3] * n[5] + m[5],
    ]
}

/// `point` under `matrix`.
fn apply(matrix: Matrix, point: (f64, f64)) -> (f64, f64) {
    (
        matrix[0] * point.0 + matrix[2] * point.1 + matrix[4],
        matrix[1] * point.0 + matrix[3] * point.1 + matrix[5],
    )
}

/// The transform that undoes `matrix`.
fn invert(matrix: Matrix) -> Matrix {
    let det = matrix[0] * matrix[3] - matrix[1] * matrix[2];
    assert!(
        det.abs() > 1e-9,
        "a transform that cannot be undone: {matrix:?}"
    );
    let (a, b, c, d) = (
        matrix[3] / det,
        -matrix[1] / det,
        -matrix[2] / det,
        matrix[0] / det,
    );
    [
        a,
        b,
        c,
        d,
        -(a * matrix[4] + c * matrix[5]),
        -(b * matrix[4] + d * matrix[5]),
    ]
}

/// A rectangle that a clip keeps: in its own coordinates, and the transform it was made under.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Region {
    frame: Matrix,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// How a stroke or a fill is drawn: how much of it shows, how thick it is, how it ends, and its color.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Brush {
    alpha: f64,
    width: f64,
    cap: LineCap,
    color: Color,
}

/// A straight line of the track, in the view.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rule {
    from: (f64, f64),
    to: (f64, f64),
    brush: Brush,
}

/// A quadratic curve in the view: from, by a control point, to.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Curve {
    from: (f64, f64),
    control: (f64, f64),
    to: (f64, f64),
}

/// The wave: a chain of curves stroked under the clips of a window.
#[derive(Debug, Clone, PartialEq)]
struct Chain {
    curves: Vec<Curve>,
    clips: Vec<Region>,
    brush: Brush,
}

/// A filled circle, in the view.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Disc {
    center: (f64, f64),
    radius: f64,
    brush: Brush,
}

/// What a canvas draws, in order.
#[derive(Debug, Default)]
struct Decoded {
    rules: Vec<Rule>,
    chains: Vec<Chain>,
    discs: Vec<Disc>,
}

/// One piece of the path being built.
#[derive(Debug, Clone, Copy)]
enum Segment {
    Move((f64, f64)),
    Line((f64, f64)),
    Quad { control: (f64, f64), to: (f64, f64) },
    Rect(Region),
    Circle { center: (f64, f64), radius: f64 },
}

/// The state a `save` keeps.
#[derive(Debug, Clone)]
struct Pen {
    frame: Matrix,
    clips: Vec<Region>,
    brush: Brush,
    fill: Color,
}

/// What decoding a drawing keeps: the state of the pen, the states a `save` set aside, the path so far and what has been
/// drawn.
struct Decoder<'a> {
    drawing: &'a Drawing,
    replay: &'a Replay,
    pen: Pen,
    saved: Vec<Pen>,
    path: Vec<Segment>,
    decoded: Decoded,
}

impl Decoder<'_> {
    /// The number a canvas number is now.
    fn number(&self, num: Num) -> f64 {
        f64::from(num.get(|slot| self.replay.value(self.drawing.slots[usize::from(slot)])))
    }

    /// The point `(x, y)` is, under the transform now.
    fn point(&self, x: Num, y: Num) -> (f64, f64) {
        apply(self.pen.frame, (self.number(x), self.number(y)))
    }

    /// The segment a path step adds.
    fn segment(&self, step: &PathOp) -> Segment {
        match step {
            PathOp::MoveTo(x, y) => Segment::Move(self.point(*x, *y)),
            PathOp::LineTo(x, y) => Segment::Line(self.point(*x, *y)),
            PathOp::QuadTo([cx, cy, x, y]) => Segment::Quad {
                control: self.point(*cx, *cy),
                to: self.point(*x, *y),
            },
            PathOp::Rect([x, y, width, height]) => Segment::Rect(Region {
                frame: self.pen.frame,
                x: self.number(*x),
                y: self.number(*y),
                width: self.number(*width),
                height: self.number(*height),
            }),
            PathOp::Arc {
                x,
                y,
                radius,
                start,
                end,
                ccw: false,
            } => {
                assert_eq!(
                    (self.number(*start), self.number(*end)),
                    (0.0, f64::from(TAU))
                );
                Segment::Circle {
                    center: self.point(*x, *y),
                    radius: self.number(*radius),
                }
            }
            other => panic!("a path step the wavy indicator does not draw with: {other:?}"),
        }
    }

    /// Strokes the path: a rule if it is one line, a chain of curves if it is a move and curves.
    fn stroke(&mut self) {
        match self.path[..] {
            [Segment::Move(from), Segment::Line(to)] => self.decoded.rules.push(Rule {
                from,
                to,
                brush: self.pen.brush,
            }),
            [Segment::Move(start), ref rest @ ..] => {
                let mut at = start;
                let curves = rest
                    .iter()
                    .map(|segment| {
                        let Segment::Quad { control, to } = *segment else {
                            panic!("a chain of curves: {:?}", self.path);
                        };
                        let curve = Curve {
                            from: at,
                            control,
                            to,
                        };
                        at = to;
                        curve
                    })
                    .collect();
                self.decoded.chains.push(Chain {
                    curves,
                    clips: self.pen.clips.clone(),
                    brush: self.pen.brush,
                });
            }
            _ => panic!("a stroke of a line or a chain: {:?}", self.path),
        }
    }

    /// Does one operation.
    fn run(&mut self, op: &CanvasOp) {
        match op {
            CanvasOp::Save => self.saved.push(self.pen.clone()),
            CanvasOp::Restore => self.pen = self.saved.pop().expect("a restore after a save"),
            CanvasOp::Translate(x, y) => {
                let by = [1.0, 0.0, 0.0, 1.0, self.number(*x), self.number(*y)];
                self.pen.frame = compose(self.pen.frame, by);
            }
            CanvasOp::Transform(m) => {
                let by = m.map(|num| self.number(num));
                self.pen.frame = compose(self.pen.frame, by);
            }
            // Replaces the transform, and leaves the clip as it is.
            CanvasOp::SetTransform(m) => self.pen.frame = m.map(|num| self.number(num)),
            CanvasOp::GlobalAlpha(alpha) => self.pen.brush.alpha = self.number(*alpha),
            CanvasOp::LineWidth(width) => self.pen.brush.width = self.number(*width),
            CanvasOp::LineCap(cap) => self.pen.brush.cap = *cap,
            CanvasOp::StrokeStyle(canvas::Paint::Color(color)) => self.pen.brush.color = *color,
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => self.pen.fill = *color,
            CanvasOp::BeginPath => self.path.clear(),
            CanvasOp::Path(step) => {
                let segment = self.segment(step);
                self.path.push(segment);
            }
            CanvasOp::Clip(None, _) => {
                let [Segment::Rect(region)] = self.path[..] else {
                    panic!("a clip of one rectangle: {:?}", self.path);
                };
                self.pen.clips.push(region);
            }
            CanvasOp::Stroke(None) => self.stroke(),
            CanvasOp::Fill(None, _) => {
                let [Segment::Circle { center, radius }] = self.path[..] else {
                    panic!("a fill of one circle: {:?}", self.path);
                };
                self.decoded.discs.push(Disc {
                    center,
                    radius,
                    brush: Brush {
                        color: self.pen.fill,
                        ..self.pen.brush
                    },
                });
            }
            other => panic!("an operation the wavy indicator does not draw with: {other:?}"),
        }
    }
}

/// Decodes `drawing` as the animation graph `replay` holds its numbers now: the transforms, clips and path of every
/// stroke and fill, in the view, as an HTML canvas applies them (a path point is moved by the transform it is added
/// under; a clip keeps the rectangle under the transform it was made under).
fn decode(drawing: &Drawing, replay: &Replay) -> Decoded {
    let mut decoder = Decoder {
        drawing,
        replay,
        pen: Pen {
            frame: IDENTITY,
            clips: Vec::new(),
            brush: Brush {
                alpha: 1.0,
                width: 1.0,
                cap: LineCap::Butt,
                color: Color(0),
            },
            fill: Color(0),
        },
        saved: Vec::new(),
        path: Vec::new(),
        decoded: Decoded::default(),
    };
    for op in &drawing.ops {
        decoder.run(op);
    }
    assert!(decoder.saved.is_empty(), "every save is restored");
    decoder.decoded
}

/// The decoded canvas `id` of `mounted`, with the numbers of `replay`.
fn decoded_by(mounted: &Mounted, id: NodeId, replay: &Replay) -> Decoded {
    decode(mounted.drawing(id), replay)
}

/// A mounted indicator and the numbers its animations hold, as they were when it was mounted.
fn mounted_with_graph(config: Config) -> (Mounted, Replay) {
    let (_app, recording) = start(config);
    let items = recording.items();
    let mut replay = Replay::default();
    replay.apply(&items);
    (Mounted::of(&items), replay)
}

// ---- the wave's closed form ---------------------------------------------------------------------------------------

/// How high a wave of amplitude 1 is above the indicator's middle, `along` points from where its path starts, for a
/// wave `wavelength` long: each half wave a parabola that peaks at the token's amplitude (`ActiveWaveAmplitude`, 3
/// points), up (down on a canvas) for the first half and the other way for the next.
fn wave_at(along: f64, wavelength: f64) -> f64 {
    let half = wavelength / 2.0;
    let index = (along / half).floor();
    let t = along / half - index;
    let sign = if index.rem_euclid(2.0) < 0.5 {
        1.0
    } else {
        -1.0
    };
    sign * 4.0 * f64::from(tokens::ACTIVE_WAVE_AMPLITUDE.0) * t * (1.0 - t)
}

/// The wave's slope along the indicator, by a central difference.
fn wave_slope(along: f64, wavelength: f64) -> f64 {
    let h = 1e-4;
    (wave_at(along + h, wavelength) - wave_at(along - h, wavelength)) / (2.0 * h)
}

/// The indicator being decoded: how wide it is, which way it goes, and its wavelength.
#[derive(Debug, Clone, Copy)]
struct Scene {
    width: f64,
    ltr: bool,
    wavelength: f64,
}

impl Scene {
    /// A point of the view as the indicator's own drawing, which goes from the left: mirrored and turned over in a
    /// right-to-left layout, where androidx rotates the drawing by 180 degrees.
    fn base(self, point: (f64, f64)) -> (f64, f64) {
        if self.ltr {
            point
        } else {
            (self.width - point.0, f64::from(HEIGHT.0) - point.1)
        }
    }

    /// Where the wave of `amplitude`, `shift` points along, is `x` points from the left, from the top.
    fn wave_y(self, x: f64, amplitude: f64, shift: f64) -> f64 {
        f64::from(HEIGHT.0) / 2.0 + amplitude * wave_at(x + shift, self.wavelength)
    }
}

/// What a window of the wave should be: where it starts and ends, from the left of the indicator, how high the wave is,
/// how far along it has gone, and how much of it shows.
#[derive(Debug, Clone, Copy)]
struct Window {
    from: f64,
    to: f64,
    amplitude: f64,
    shift: f64,
    alpha: f64,
}

/// The points the clips of `chain` keep, found by walking each curve in small steps.
fn kept(chain: &Chain) -> Vec<(f64, f64)> {
    const STEPS: u16 = 400;
    let regions: Vec<(Region, Matrix)> = chain
        .clips
        .iter()
        .map(|region| (*region, invert(region.frame)))
        .collect();
    let mut points = Vec::new();
    for curve in &chain.curves {
        for step in 0..=STEPS {
            let at = quadratic(
                curve.from,
                curve.control,
                curve.to,
                f64::from(step) / f64::from(STEPS),
            );
            let inside = regions.iter().all(|(region, inverse)| {
                let (x, y) = apply(*inverse, at);
                x >= region.x
                    && x <= region.x + region.width
                    && y >= region.y
                    && y <= region.y + region.height
            });
            if inside {
                points.push(at);
            }
        }
    }
    points
}

/// `chain` and the two circles at its ends (`tail` and `head`) draw `window` of the wave of `scene` in `color`: the
/// curves are the wave, the clips keep what lies between the window's ends and are square to the wave there, and
/// the circles are at the ends, on the wave.
fn assert_window(
    chain: &Chain,
    (tail, head): (&Disc, &Disc),
    scene: Scene,
    window: Window,
    color: Color,
    what: &str,
) {
    let brush = Brush {
        alpha: window.alpha,
        width: f64::from(INDICATOR_THICKNESS),
        cap: LineCap::Butt,
        color,
    };
    assert!(
        (chain.brush.alpha - brush.alpha).abs() < 1e-3
            && chain.brush.width.to_bits() == brush.width.to_bits()
            && chain.brush.cap == brush.cap
            && chain.brush.color == brush.color,
        "{what}: the stroke {:?} is not {brush:?}",
        chain.brush
    );
    // The curves start `shift` before the window's origin: androidx's `translate(-waveShift, ...)`.
    let origin = scene.base(chain.curves[0].from);
    assert!(
        (origin.0 + window.shift).abs() < 1e-3
            && (origin.1 - f64::from(HEIGHT.0) / 2.0).abs() < 1e-3,
        "{what}: the path starts at {origin:?}, not at ({}, {})",
        -window.shift,
        f64::from(HEIGHT.0) / 2.0
    );
    // What the clips keep is the wave, from the window's start to its end. A step along a curve is a 400th of a half
    // wave (0.05 points at most), so a window as long as a few steps has its ends found to that; a shorter one, or one
    // that is not there (no length, or backwards: the two half planes meet in a line or not at all), keeps nothing
    // outside it.
    let points: Vec<(f64, f64)> = kept(chain).into_iter().map(|p| scene.base(p)).collect();
    if window.to - window.from > 0.2 {
        let (low, high) = points.iter().fold((f64::MAX, f64::MIN), |(low, high), p| {
            (low.min(p.0), high.max(p.0))
        });
        assert!(
            (low - window.from).abs() < 0.1 && (high - window.to).abs() < 0.1,
            "{what}: the clips keep {low} to {high}, not {} to {}",
            window.from,
            window.to
        );
    } else {
        assert!(
            points
                .iter()
                .all(|p| p.0 >= window.from - 1e-2 && p.0 <= window.to + 1e-2),
            "{what}: the clips keep {points:?}, outside {} to {}",
            window.from,
            window.to
        );
    }
    for (x, y) in &points {
        let expected = scene.wave_y(*x, window.amplitude, window.shift);
        assert!(
            (y - expected).abs() < 2e-3,
            "{what}: at {x} the wave is at {y}, not {expected}"
        );
    }
    // The clips: the half plane behind the head and the one ahead of the tail, each in a frame at the window's end
    // that is turned to the wave's tangent there.
    // Twice the width and twice the height, past the end of the wave in each direction (`half_plane`, pinned below).
    let reach = 2.0 * scene.width + 2.0 * f64::from(HEIGHT.0);
    let [behind, ahead] = chain.clips[..] else {
        panic!("{what}: two clips, not {:?}", chain.clips);
    };
    for (region, (x, width), end, at) in [
        (behind, (-reach, reach), window.to, head),
        (ahead, (0.0, reach), window.from, tail),
    ] {
        assert!(
            (region.x - x).abs() < 1e-3
                && (region.width - width).abs() < 1e-3
                && (region.y + reach).abs() < 1e-3
                && (region.height - 2.0 * reach).abs() < 1e-3,
            "{what}: the clip {region:?}"
        );
        // The frame's origin is on the wave at the window's end, and its x axis along the tangent there.
        let origin = scene.base((region.frame[4], region.frame[5]));
        let expected_y = scene.wave_y(end, window.amplitude, window.shift);
        assert!(
            (origin.0 - end).abs() < 1e-3 && (origin.1 - expected_y).abs() < 4e-3,
            "{what}: a clip's frame is at {origin:?}, not at ({end}, {expected_y})"
        );
        let slope = region.frame[1] / region.frame[0];
        let expected_slope = window.amplitude * wave_slope(end + window.shift, scene.wavelength);
        assert!(
            (slope - expected_slope).abs() < 1e-2,
            "{what}: a clip is turned to a slope of {slope}, not {expected_slope}"
        );
        // The frame is square: its y axis is its x axis turned a quarter.
        assert!(
            (region.frame[2] + region.frame[1]).abs() < 1e-6
                && (region.frame[3] - region.frame[0]).abs() < 1e-6,
            "{what}: the frame {:?} is not square",
            region.frame
        );
        // The circle at this end of the window.
        let center = scene.base(at.center);
        assert!(
            (center.0 - end).abs() < 1e-3 && (center.1 - expected_y).abs() < 4e-3,
            "{what}: the circle is at {center:?}, not at ({end}, {expected_y})"
        );
        assert_eq!(
            (at.radius.to_bits(), at.brush.color),
            (f64::from(INDICATOR_THICKNESS / 2.0).to_bits(), color),
            "{what}"
        );
        assert!(
            (at.brush.alpha - window.alpha).abs() < 1e-3,
            "{what}: the circle shows {} of itself, not {}",
            at.brush.alpha,
            window.alpha
        );
    }
}

/// The pieces of the track in `rules` that show, as `(from, to)` in the indicator's own drawing.
fn visible_rules(rules: &[Rule], scene: Scene) -> Vec<(f64, f64)> {
    rules
        .iter()
        .filter(|rule| rule.brush.alpha > 0.5)
        .map(|rule| {
            let (from, to) = (scene.base(rule.from), scene.base(rule.to));
            assert!(
                (from.1 - 5.0).abs() < 1e-6 && (to.1 - 5.0).abs() < 1e-6,
                "a piece along the middle: {rule:?}"
            );
            (from.0, to.0)
        })
        .collect()
}

/// `found` and `expected` are the same pieces to a thousandth of a point.
fn assert_rules(found: &[(f64, f64)], expected: &[(f64, f64)], what: &str) {
    assert_eq!(
        found.len(),
        expected.len(),
        "{what}: {found:?} is not {expected:?}"
    );
    for (found, expected) in found.iter().zip(expected) {
        assert!(
            (found.0 - expected.0).abs() < 1e-3 && (found.1 - expected.1).abs() < 1e-3,
            "{what}: {found:?} is not {expected:?}"
        );
    }
}

// ---- the component, determinate -----------------------------------------------------------------------------------

fn wavy(progress: Progress) -> Config {
    Config {
        progress,
        shape: ProgressShape::Wavy,
        ..Config::default()
    }
}

fn wavy_at(fraction: f32) -> Config {
    wavy(Progress::Determinate(percentage(fraction)))
}

fn wavy_indeterminate() -> Config {
    wavy(Progress::Indeterminate)
}

/// The scene of a wavy indicator `width` points wide.
fn scene(width: f32, rtl: bool, wavelength: f32) -> Scene {
    Scene {
        width: f64::from(width),
        ltr: !rtl,
        wavelength: f64::from(wavelength),
    }
}

#[test]
fn a_wavy_indicator_is_240_by_10_points_unless_it_is_given_a_width() {
    let mounted = mount(wavy_at(0.5));
    assert_eq!(mounted.size(), Size::new(Points(240.0), Points(10.0)));
    // The three canvases cover it.
    for id in [mounted.first, mounted.track, mounted.indicator] {
        assert_eq!(
            mounted.frames[&id],
            Bounds::new(point(px(0.0), px(0.0)), mounted.size())
        );
    }
    let narrow = mount(Config {
        width: Points(100.0),
        ..wavy_at(0.5)
    });
    assert_eq!(narrow.size(), Size::new(Points(100.0), Points(10.0)));
    // The flat form is as tall as its stroke, which the shape decides.
    let flat = mount(Config::default());
    assert_eq!(flat.size(), Size::new(Points(240.0), Points(4.0)));
}

/// What a determinate indicator draws at a fraction: the fraction, the amplitude it gives, the pieces of the track, the
/// window of the wave (None for a line of no length) and the stop (None once the line has reached it).
type DrawnCase = (
    f32,
    f64,
    Vec<(f64, f64)>,
    Option<(f64, f64)>,
    Option<(f64, f64)>,
);

/// Checks what a determinate wavy indicator draws at `case`'s fraction, in the light or dark scheme.
fn assert_drawn(case: DrawnCase, dark: bool) {
    let (fraction, amplitude, track, window, stop) = case;
    let (track_color, primary) = (
        scheme_color(ColorRole::SecondaryContainer, dark),
        scheme_color(ColorRole::Primary, dark),
    );
    let what = format!("{fraction}, dark {dark}");
    let (mounted, replay) = mounted_with_graph(Config {
        dark,
        ..wavy_at(fraction)
    });
    let scene = scene(240.0, false, 40.0);
    // Nothing at the back: a wavy indicator draws its lines together, at the front.
    let first = decoded_by(&mounted, mounted.first, &replay);
    assert!(first.rules.is_empty() && first.chains.is_empty() && first.discs.is_empty());
    // The track, in the container color, along the middle: round, 4 points thick.
    let drawn = decoded_by(&mounted, mounted.track, &replay);
    assert_rules(
        &visible_rules(&drawn.rules, scene),
        &track,
        &format!("{what}: track"),
    );
    for rule in &drawn.rules {
        assert_eq!(
            (rule.brush.width.to_bits(), rule.brush.cap, rule.brush.color),
            (4.0_f64.to_bits(), LineCap::Round, track_color),
            "{what}"
        );
        assert!(
            rule.brush.alpha.abs() < 1e-6 || (rule.brush.alpha - 1.0).abs() < 1e-6,
            "{what}: a piece shows {} of itself",
            rule.brush.alpha
        );
    }
    let lines = decoded_by(&mounted, mounted.indicator, &replay);
    assert_eq!(lines.rules.len(), 0, "{what}");
    let [chain] = &lines.chains[..] else {
        panic!("{what}: one wave, not {:?}", lines.chains);
    };
    let (stop_disc, ends) = match (stop, &lines.discs[..]) {
        (Some(_), [tail, head, stop]) => (Some(*stop), (tail, head)),
        (None, [tail, head]) => (None, (tail, head)),
        (_, discs) => panic!("{what}: the circles {discs:?}"),
    };
    // A line of no length is there, at no strength; a line that is, whole.
    let (from, to) = window.unwrap_or((2.0, 2.0));
    assert_window(
        chain,
        ends,
        scene,
        Window {
            from,
            to,
            amplitude,
            shift: 0.0,
            alpha: f64::from(u8::from(window.is_some())),
        },
        primary,
        &what,
    );
    // The circle is there exactly where the layout has a stop.
    if let (Some((x, y)), Some(disc)) = (stop, stop_disc) {
        assert!(
            (disc.center.0 - x).abs() < 1e-3 && (disc.center.1 - 5.0).abs() < 1e-3,
            "{what}: the stop {disc:?}"
        );
        assert_eq!(
            (disc.radius.to_bits(), disc.brush.color),
            (y.to_bits(), primary),
            "{what}"
        );
        assert!((disc.brush.alpha - 1.0).abs() < 1e-6, "{what}");
    }
}

#[test]
fn a_determinate_wavy_indicator_draws_the_track_the_wave_and_the_stop_in_the_token_colors() {
    for dark in [false, true] {
        // The fraction, the amplitude it gives, the track and the window of the wave (None for a line of no length),
        // and the stop (None once the line has reached it), from the hand-worked layouts above.
        let cases: [DrawnCase; 7] = [
            (
                0.0,
                0.0,
                vec![(238.0, 2.0), (2.0, 2.0)],
                None,
                Some((238.0, 2.0)),
            ),
            (
                0.05,
                0.0,
                vec![(238.0, 20.0)],
                Some((2.0, 12.0)),
                Some((238.0, 2.0)),
            ),
            (
                0.5,
                1.0,
                vec![(238.0, 128.0)],
                Some((2.0, 120.0)),
                Some((238.0, 2.0)),
            ),
            (
                0.9,
                1.0,
                vec![(238.0, 224.0)],
                Some((2.0, 216.0)),
                Some((238.0, 2.0)),
            ),
            (
                0.95,
                0.0,
                vec![(238.0, 236.0)],
                Some((2.0, 228.0)),
                Some((238.0, 2.0)),
            ),
            (0.97, 0.0, vec![], Some((2.0, 232.8)), Some((238.0, 2.0))),
            (1.0, 0.0, vec![], Some((2.0, 238.0)), None),
        ];
        for case in cases {
            assert_drawn(case, dark);
        }
    }
}

#[test]
fn right_to_left_turns_the_whole_drawing_by_a_half_turn() {
    let (mounted, replay) = mounted_with_graph(Config {
        rtl: true,
        ..wavy_at(0.5)
    });
    let scene = scene(240.0, true, 40.0);
    let (track_color, primary) = (
        scheme_color(ColorRole::SecondaryContainer, false),
        scheme_color(ColorRole::Primary, false),
    );
    // In the view the track goes from 2 to 112 (240 - 238 and 240 - 128): the right-to-left layout's track is drawn
    // from the left and the line from 120 to 238; in the indicator's own drawing, which is turned, it is the same as
    // from the left.
    let drawn = decoded_by(&mounted, mounted.track, &replay);
    let in_view: Vec<(f64, f64)> = drawn
        .rules
        .iter()
        .filter(|rule| rule.brush.alpha > 0.5)
        .map(|rule| (rule.from.0, rule.to.0))
        .collect();
    assert_rules(&in_view, &[(2.0, 112.0)], "the track in the view");
    assert_rules(
        &visible_rules(&drawn.rules, scene),
        &[(238.0, 128.0)],
        "the track in the drawing",
    );
    let lines = decoded_by(&mounted, mounted.indicator, &replay);
    let [chain] = &lines.chains[..] else {
        panic!("one wave, not {:?}", lines.chains);
    };
    let [tail, head, stop] = &lines.discs[..] else {
        panic!("three circles, not {:?}", lines.discs);
    };
    // The wave is turned over as well as mirrored: it is the same wave, seen from the other side.
    assert_window(
        chain,
        (tail, head),
        scene,
        Window {
            from: 2.0,
            to: 120.0,
            amplitude: 1.0,
            shift: 0.0,
            alpha: 1.0,
        },
        primary,
        "rtl",
    );
    // In the view the line is from 120 to 238, and the circle at its tail is at 238, with the wave turned over: where
    // the wave is 1.08 below the middle at x = 2 from the left (6.08), it is above it in the view (3.92).
    assert!((tail.center.0 - 238.0).abs() < 1e-3 && (head.center.0 - 120.0).abs() < 1e-3);
    assert!((tail.center.1 - (10.0 - 6.08)).abs() < 4e-3, "{tail:?}");
    // The stop is at the left end.
    assert!(
        (stop.center.0 - 2.0).abs() < 1e-3 && (stop.center.1 - 5.0).abs() < 1e-3,
        "{stop:?}"
    );
    assert_eq!(
        (stop.radius.to_bits(), stop.brush.color),
        (2.0_f64.to_bits(), primary)
    );
    let _ = track_color;
}

#[test]
fn a_wavy_indicator_of_another_width_is_laid_out_for_it() {
    for (width, track, window, stop_x) in [
        (100.0_f32, vec![(98.0, 58.0)], (2.0, 50.0), 98.0),
        (1000.0, vec![(998.0, 508.0)], (2.0, 500.0), 998.0),
        // 3000 points: a window much longer than the half planes of a fixed size would reach.
        (3000.0, vec![(2998.0, 1508.0)], (2.0, 1500.0), 2998.0),
    ] {
        let (mounted, replay) = mounted_with_graph(Config {
            width: Points(width),
            ..wavy_at(0.5)
        });
        assert_eq!(mounted.size(), Size::new(Points(width), Points(10.0)));
        let scene = scene(width, false, 40.0);
        let what = format!("{width} wide");
        let drawn = decoded_by(&mounted, mounted.track, &replay);
        assert_rules(&visible_rules(&drawn.rules, scene), &track, &what);
        let lines = decoded_by(&mounted, mounted.indicator, &replay);
        let [chain] = &lines.chains[..] else {
            panic!("{what}: one wave");
        };
        let [tail, head, stop] = &lines.discs[..] else {
            panic!("{what}: three circles");
        };
        assert_window(
            chain,
            (tail, head),
            scene,
            Window {
                from: window.0,
                to: window.1,
                amplitude: 1.0,
                shift: 0.0,
                alpha: 1.0,
            },
            scheme_color(ColorRole::Primary, false),
            &what,
        );
        assert!((stop.center.0 - stop_x).abs() < 1e-3, "{what}: {stop:?}");
        // The half planes reach twice the width and the height past each end.
        assert!((half_plane(width) - (2.0 * width + 20.0)).abs() < 1e-3);
    }
}

#[test]
fn a_wavy_indicator_narrower_than_it_is_tall_has_no_caps_and_no_room_for_them() {
    // 8 points: no room for caps (`height > width`), so the line is from 0 to 4 and the track is gone.
    let (mounted, replay) = mounted_with_graph(Config {
        width: Points(8.0),
        ..wavy_at(0.5)
    });
    let scene = scene(8.0, false, 40.0);
    let drawn = decoded_by(&mounted, mounted.track, &replay);
    assert_rules(&visible_rules(&drawn.rules, scene), &[], "track");
    let lines = decoded_by(&mounted, mounted.indicator, &replay);
    let [chain] = &lines.chains[..] else {
        panic!("one wave");
    };
    let [tail, head, stop] = &lines.discs[..] else {
        panic!("three circles");
    };
    assert_window(
        chain,
        (tail, head),
        scene,
        Window {
            from: 0.0,
            to: 4.0,
            amplitude: 1.0,
            shift: 0.0,
            alpha: 1.0,
        },
        scheme_color(ColorRole::Primary, false),
        "8 wide",
    );
    assert!((stop.center.0 - 6.0).abs() < 1e-3, "{stop:?}");
}

#[test]
fn screen_readers_get_the_same_progress_bar_from_a_wavy_indicator() {
    let reading = |progress: Progress| {
        let mounted = mount(wavy(progress));
        assert_eq!(mounted.value.as_ref().and_then(|v| v.text.clone()), None);
        range(&mounted)
    };
    let determinate = |fraction: f32| reading(Progress::Determinate(percentage(fraction)));
    assert_eq!(determinate(0.4), Some((Some(0.0), Some(1.0), Some(0.4))));
    assert_eq!(determinate(0.0), Some((Some(0.0), Some(1.0), Some(0.0))));
    // Out of range, held to it.
    assert_eq!(determinate(3.0), Some((Some(0.0), Some(1.0), Some(1.0))));
    assert_eq!(determinate(-3.0), Some((Some(0.0), Some(1.0), Some(0.0))));
    // Not finite: indeterminate, as androidx's wavy indicator is while the progress is `!isFinite()` (the flat one
    // reads 0).
    for unknown in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(determinate(unknown), Some((Some(0.0), Some(1.0), None)));
    }
    assert_eq!(
        reading(Progress::Indeterminate),
        Some((Some(0.0), Some(1.0), None))
    );
    let labelled = mount(Config {
        label: Some("Loading"),
        ..wavy_at(0.5)
    });
    assert_eq!(labelled.label.as_deref(), Some("Loading"));
    assert_eq!(mount(wavy_at(0.5)).label, None);
}

#[test]
fn a_wavy_indicator_whose_progress_is_not_finite_draws_as_an_indeterminate_one() {
    let (mounted, replay) = mounted_with_graph(wavy(Progress::Determinate(percentage(f32::NAN))));
    let lines = decoded_by(&mounted, mounted.indicator, &replay);
    // Two lines, four circles at their ends, and no stop.
    assert_eq!(lines.chains.len(), 2, "{:?}", lines.chains);
    assert_eq!(lines.discs.len(), 4, "{:?}", lines.discs);
    let (amplitude, _) = read_wave(&mounted, &replay);
    assert!((amplitude - 1.0).abs() < 1e-6);
}

// ---- the animations the wavy form starts --------------------------------------------------------------------------

/// A loop of one value that goes from where it is to `to`, over and over, as the wave's travel is started.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Travelling {
    node: NodeId,
    to: f32,
    duration_ms: f32,
    easing: Easing,
    delay_ms: f32,
    frame_rate: Option<f32>,
    iterations: Option<u32>,
    reset: bool,
}

/// The wave loops among the composites `items` start, in order.
fn wave_loops(items: &[MountItem]) -> Vec<Travelling> {
    items
        .iter()
        .filter_map(|item| match item {
            MountItem::Animation(AnimationOp::StartComposite {
                spec:
                    Composite::Loop {
                        child,
                        iterations,
                        reset,
                    },
                ..
            }) => match &**child {
                Composite::Run {
                    node,
                    driver:
                        Driver::Timing {
                            to,
                            duration_ms,
                            easing,
                            delay_ms,
                            frame_rate,
                        },
                } => Some(Travelling {
                    node: *node,
                    to: *to,
                    duration_ms: *duration_ms,
                    easing: *easing,
                    delay_ms: *delay_ms,
                    frame_rate: *frame_rate,
                    iterations: *iterations,
                    reset: *reset,
                }),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// A value that goes to `to` in `duration_ms` along `easing`, from where it is.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tween {
    node: NodeId,
    to: f32,
    duration_ms: f32,
    easing: Easing,
    delay_ms: f32,
    frame_rate: Option<f32>,
}

/// The timings `items` start on a value, in order.
fn started_tweens(items: &[MountItem]) -> Vec<Tween> {
    items
        .iter()
        .filter_map(|item| match item {
            MountItem::Animation(AnimationOp::Start {
                node,
                driver:
                    Driver::Timing {
                        to,
                        duration_ms,
                        easing,
                        delay_ms,
                        frame_rate,
                    },
                ..
            }) => Some(Tween {
                node: *node,
                to: *to,
                duration_ms: *duration_ms,
                easing: *easing,
                delay_ms: *delay_ms,
                frame_rate: *frame_rate,
            }),
            _ => None,
        })
        .collect()
}

/// Whether `op` moves, sets or stops a value or an animation: any operation but the creation of a node (the wavy form
/// builds nodes of the graph that read its values, and they are made whether anything moves or not).
fn is_motion(op: &AnimationOp) -> bool {
    !matches!(
        op,
        AnimationOp::CreateValue { .. }
            | AnimationOp::CreateInterpolate { .. }
            | AnimationOp::CreateDiffClamp { .. }
            | AnimationOp::CreateMath { .. }
            | AnimationOp::SetOutput { .. }
            | AnimationOp::ReleaseComposite { .. }
            | AnimationOp::Release { .. }
            | AnimationOp::Read { .. }
    )
}

/// The four sweeps of the indeterminate motion among `items`, as the flat indicator starts them.
fn course_tracks(items: &[MountItem]) -> Vec<Track> {
    let course: Vec<MountItem> = items
        .iter()
        .filter(|item| match item {
            MountItem::Animation(AnimationOp::StartComposite { spec, .. }) => {
                !matches!(spec, Composite::Loop { child, .. } if matches!(**child, Composite::Run { .. }))
            }
            _ => true,
        })
        .cloned()
        .collect();
    four_tracks(&course)
}

/// How high the wave of the indicator's last canvas is, and how far it has moved, as `replay` holds them now: the
/// control height of its first curve over the crest, and the translation of its path.
fn read_wave(mounted: &Mounted, replay: &Replay) -> (f64, f64) {
    let drawing = mounted.drawing(mounted.indicator);
    let number =
        |num: Num| f64::from(num.get(|slot| replay.value(drawing.slots[usize::from(slot)])));
    let shift = drawing
        .ops
        .iter()
        .find_map(|op| match op {
            CanvasOp::Translate(x, _) => Some(-number(*x)),
            _ => None,
        })
        .expect("a translation of the wave's path");
    let amplitude = drawing
        .ops
        .iter()
        .find_map(|op| match op {
            CanvasOp::Path(PathOp::QuadTo([_, y, _, _])) => Some(number(*y) / 6.0),
            _ => None,
        })
        .expect("a curve of the wave");
    (amplitude, shift)
}

/// `a` and `b` are the same shift of a wave of this wavelength, to a thousandth of a point, a shift of a whole wavelength
/// being no shift.
fn same_shift(a: f64, b: f64, wavelength: f64) -> bool {
    let difference = (a - b).rem_euclid(wavelength);
    difference < 1e-3 || wavelength - difference < 1e-3
}

/// The amplitude `ms` into a tween from `from` to `to` along the curve whose control points are `curve`.
fn tweened(from: f64, to: f64, curve: ((f64, f64), (f64, f64)), ms: f64) -> f64 {
    from + (to - from) * cubic_bezier(curve.0, curve.1, (ms / 500.0).clamp(0.0, 1.0))
}

/// `EasingStandardCubicBezier` (`MotionTokens.kt`, lines 61 and 62), the curve an amplitude that grows follows.
const STANDARD: ((f64, f64), (f64, f64)) = ((0.2, 0.0), (0.0, 1.0));

#[test]
fn a_wavy_determinate_indicator_starts_one_loop_that_moves_the_wave_a_wavelength_a_second() {
    let (_app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    assert_eq!(started_composites(&items), 1, "{items:?}");
    let loops = wave_loops(&items);
    let [travelling] = &loops[..] else {
        panic!("one loop of one value: {loops:?}");
    };
    // From 0 to 1 in 1000 ms, along a straight line, until stopped, going back to 0 each time, and at the screen's
    // highest rate (ADR 0010, check 4).
    assert_eq!(travelling.to.to_bits(), 1.0_f32.to_bits());
    assert_eq!(travelling.duration_ms.to_bits(), 1000.0_f32.to_bits());
    assert_eq!(travelling.easing, Easing::Linear);
    assert_eq!(travelling.delay_ms.to_bits(), 0.0_f32.to_bits());
    assert_eq!(travelling.frame_rate, None);
    assert_eq!(travelling.iterations, None);
    assert!(travelling.reset);
    // The amplitude is where it is to be, so nothing moves it: no timing on any value.
    assert_eq!(started_tweens(&items), Vec::new());
}

#[test]
fn the_graph_moves_the_wave_one_wavelength_a_second_at_120_hz() {
    let (_app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let loops = wave_loops(&items);
    let [travelling] = &loops[..] else {
        panic!("one loop: {loops:?}");
    };
    let mut replay = Replay::default();
    replay.apply(&items);
    let scene = scene(240.0, false, 40.0);
    let primary = scheme_color(ColorRole::Primary, false);
    // Three seconds, a frame at a time.
    for frame in 0_u32..360 {
        replay.run(travelling.node, 1);
        let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
        // The frame on the end of a wavelength may be on either side of the jump back to the start.
        if (ms % 1000.0).min(1000.0 - ms % 1000.0) < 1e-6 {
            continue;
        }
        let expected = ms % 1000.0 / 1000.0 * 40.0;
        let (amplitude, shift) = read_wave(&mounted, &replay);
        assert!((amplitude - 1.0).abs() < 1e-6, "{ms} ms: {amplitude}");
        assert!(
            same_shift(shift, expected, 40.0),
            "{ms} ms: the wave is {shift} along, not {expected}"
        );
        if frame % 17 == 0 {
            let lines = decoded_by(&mounted, mounted.indicator, &replay);
            let [chain] = &lines.chains[..] else {
                panic!("one wave");
            };
            let [tail, head, _] = &lines.discs[..] else {
                panic!("three circles");
            };
            assert_window(
                chain,
                (tail, head),
                scene,
                Window {
                    from: 2.0,
                    to: 120.0,
                    amplitude: 1.0,
                    shift: expected,
                    alpha: 1.0,
                },
                primary,
                &format!("{ms} ms"),
            );
        }
    }
}

#[test]
fn a_wavy_indicator_sends_nothing_while_the_wave_travels() -> Result<(), String> {
    for config in [wavy_at(0.5), wavy_indeterminate()] {
        let (mut app, recording) = start(config);
        let before = recording.items().len();
        // Two and a half cycles of the indeterminate motion on the manual clock: no batch at all, so no layout, no
        // text and no drawing again.
        app.run_for(std::time::Duration::from_millis(4375))?;
        assert_eq!(recording.items().len(), before);
    }
    Ok(())
}

#[test]
fn a_wave_with_no_height_to_show_starts_nothing() -> Result<(), String> {
    for fraction in [0.0, 0.05, 0.1, 0.95, 1.0] {
        let (mut app, recording) = start(wavy_at(fraction));
        app.step("settle");
        app.run_for(std::time::Duration::from_secs(2))?;
        let items = recording.items();
        assert_eq!(operations(&items, is_motion), 0, "{fraction}");
        assert!(!replay(&recording).is_animating(), "{fraction}");
        assert!(recording.in_step("settle").is_empty(), "{fraction}");
        // And it is flat: no control point is off the line.
        let mounted = Mounted::of(&items);
        let mut graph = Replay::default();
        graph.apply(&items);
        let (amplitude, _) = read_wave(&mounted, &graph);
        assert!(amplitude.abs() < 1e-9, "{fraction}: {amplitude}");
    }
    Ok(())
}

#[test]
fn the_flat_form_starts_none_of_the_waves_animations() {
    for config in [Config::default(), determinate_at(0.5)] {
        let (_app, recording) = start(config);
        let items = recording.items();
        assert_eq!(operations(&items, is_motion), 0, "{items:?}");
        assert!(!replay(&recording).is_animating());
    }
}

#[test]
fn the_wave_is_there_only_while_the_indicator_is() -> Result<(), String> {
    // Taking it off the screen releases its animations; one frame later nothing is left animating.
    let (mut app, recording) = start(wavy_at(0.5));
    assert!(replay(&recording).is_animating());
    app.press("Remove")?;
    let items = recording.in_step("press(Remove)");
    assert!(operations(&items, is_release) >= 1, "{items:?}");
    let mut gone = replay(&recording);
    gone.run(1, 1);
    assert!(!gone.is_animating(), "{items:?}");
    // The same for an indeterminate one, which has the four sweeps as well.
    let (mut app, recording) = start(wavy_indeterminate());
    assert!(replay(&recording).is_animating());
    app.press("Remove")?;
    let mut gone = replay(&recording);
    gone.run(1, 1);
    assert!(!gone.is_animating());
    Ok(())
}

#[test]
fn becoming_flat_stops_the_wave_and_becoming_wavy_starts_it() -> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.5));
    assert!(replay(&recording).is_animating());
    app.press("Straight")?;
    let items = recording.in_step("press(Straight)");
    assert_eq!(operations(&items, is_stop), 1, "{items:?}");
    assert_eq!(started_composites(&items), 0, "{items:?}");
    assert!(!replay(&recording).is_animating());
    // The canvases are drawn again as the flat form draws them: a straight line, 4 points tall.
    let mounted = Mounted::of(&recording.items());
    assert_eq!(mounted.size(), Size::new(Points(240.0), Points(4.0)));
    let drawn = drawn_by(&mounted);
    assert_line(
        &drawn.indicator,
        2.0,
        120.0,
        scheme_color(ColorRole::Primary, false),
        "flat",
    );
    // Wavy again: the wave starts at once, with its height as it is to be and no tween on it.
    app.press("Waves")?;
    let items = recording.in_step("press(Waves)");
    assert_eq!(started_composites(&items), 1, "{items:?}");
    assert_eq!(wave_loops(&items).len(), 1, "{items:?}");
    assert_eq!(started_tweens(&items), Vec::new(), "{items:?}");
    assert!(replay(&recording).is_animating());
    let mounted = Mounted::of(&recording.items());
    assert_eq!(mounted.size(), Size::new(Points(240.0), Points(10.0)));
    Ok(())
}

/// Whether `items` stop the timing on `node`, where it is.
fn stops_value(items: &[MountItem], node: NodeId) -> bool {
    items.iter().any(|item| {
        matches!(item, MountItem::Animation(AnimationOp::Stop { node: stopped }) if *stopped == node)
    })
}

#[test]
fn a_wave_that_goes_flat_in_mid_settle_leaves_nothing_moving() -> Result<(), String> {
    // A settling wave moves two values, its travel and its amplitude. A flat indicator draws neither, so when it turns
    // flat before they have arrived both stop where they are (ADR 0010: nothing animates without a visible consumer),
    // and neither moves on behind the straight line.
    let (mut app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    let travel = wave_loops(&items)[0].node;
    let mut replay = Replay::default();
    replay.apply(&items);
    replay.run(travel, 84);
    app.press("Low")?;
    let low = recording.in_step("press(Low)");
    let amplitude = started_tweens(&low)
        .iter()
        .map(|tween| tween.node)
        .find(|node| *node != travel)
        .ok_or("no tween of the amplitude")?;
    replay.apply(&low);
    // A quarter of a second in: both are on their way.
    replay.run(travel, 30);
    assert!(replay.is_animating());
    assert!(replay.value(amplitude) > 0.1 && replay.value(amplitude) < 0.99);
    app.press("Straight")?;
    let flat = recording.in_step("press(Straight)");
    assert!(stops_value(&flat, travel), "the travel goes on: {flat:?}");
    assert!(
        stops_value(&flat, amplitude),
        "the amplitude goes on: {flat:?}"
    );
    replay.apply(&flat);
    replay.run(travel, 1);
    assert!(!replay.is_animating());
    let held = (replay.value(travel), replay.value(amplitude));
    replay.run(travel, 90);
    assert_eq!(replay.value(travel).to_bits(), held.0.to_bits());
    assert_eq!(replay.value(amplitude).to_bits(), held.1.to_bits());
    Ok(())
}

#[test]
fn a_wave_that_goes_flat_in_mid_growth_stops_growing() -> Result<(), String> {
    // From a flat wave to a full one: the amplitude grows over 500 ms, on a travel that loops.
    let (mut app, recording) = start(wavy_at(0.05));
    app.press("Half")?;
    let grow = recording.in_step("press(Half)");
    let amplitude = started_tweens(&grow)
        .first()
        .map(|tween| tween.node)
        .ok_or("no tween of the amplitude")?;
    let mut replay = replay(&recording);
    replay.run(amplitude, 30);
    assert!(replay.is_animating());
    app.press("Straight")?;
    let flat = recording.in_step("press(Straight)");
    assert!(
        stops_value(&flat, amplitude),
        "the amplitude goes on: {flat:?}"
    );
    replay.apply(&flat);
    replay.run(amplitude, 1);
    assert!(!replay.is_animating());
    Ok(())
}

// ---- the amplitude and the travel, changed in mid-flight ----------------------------------------------------------

/// The numbers `replay` holds for the wave of `mounted`, `ms` into a stretch that started with its tick 0: how high the
/// wave is, and how far along. Compared, at every frame, with what a closed form says.
struct Stretch<'a> {
    mounted: &'a Mounted,
    replay: &'a mut Replay,
    node: NodeId,
}

impl Stretch<'_> {
    /// Ticks the graph a frame and reads the wave.
    fn tick(&mut self) -> (f64, f64) {
        self.replay.run(self.node, 1);
        read_wave(self.mounted, self.replay)
    }
}

#[test]
fn a_wave_that_loses_its_height_shrinks_along_the_accelerating_curve_while_it_goes_on_for_half_a_wavelength()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let loops = wave_loops(&items);
    let [travelling] = &loops[..] else {
        panic!("one loop: {loops:?}");
    };
    let node = travelling.node;
    let mut replay = Replay::default();
    replay.apply(&items);
    // 84 frames: the last at 83/120 s, 0.69167 of a wavelength along.
    replay.run(node, 84);
    let before = 83.0 / REPLAY_HZ;
    let (amplitude, shift) = read_wave(&mounted, &replay);
    assert!((amplitude - 1.0).abs() < 1e-6 && same_shift(shift, before * 40.0, 40.0));
    // The progress falls under a tenth: the amplitude is to be 0.
    app.press("Low")?;
    let step = recording.in_step("press(Low)");
    assert_eq!(layout_items(&step), 0, "{step:?}");
    // The amplitude's tween and the travel's, which is on the loop's own value; in whichever order they are sent.
    let tweens = started_tweens(&step);
    let [shrink, settle] = &tweens[..] else {
        panic!("a tween of the amplitude and one of the travel: {tweens:?}");
    };
    let (shrink, settle) = if settle.node == node {
        (shrink, settle)
    } else {
        (settle, shrink)
    };
    // The amplitude: to 0 in 500 ms along `EasingEmphasizedAccelerateCubicBezier`, at the screen's highest rate.
    assert_eq!(shrink.to.to_bits(), 0.0_f32.to_bits());
    assert_eq!(shrink.duration_ms.to_bits(), 500.0_f32.to_bits());
    assert_eq!(shrink.easing, Easing::Bezier(0.3, 0.0, 0.8, 0.15));
    assert_eq!(
        (shrink.delay_ms.to_bits(), shrink.frame_rate),
        (0.0_f32.to_bits(), None)
    );
    // The travel: the wave goes on at its speed, half a wavelength in the 500 ms, on a straight line.
    assert_eq!(settle.node, node);
    assert_eq!(settle.to.to_bits(), 0.5_f32.to_bits());
    assert_eq!(settle.duration_ms.to_bits(), 500.0_f32.to_bits());
    assert_eq!(settle.easing, Easing::Linear);
    assert_eq!(
        (settle.delay_ms.to_bits(), settle.frame_rate),
        (0.0_f32.to_bits(), None)
    );
    assert_eq!(operations(&step, is_stop), 1, "the loop stops: {step:?}");
    assert_eq!(started_composites(&step), 0, "{step:?}");
    replay.apply(&step);
    let mut stretch = Stretch {
        mounted: &mounted,
        replay: &mut replay,
        node,
    };
    for frame in 0_u32..90 {
        let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
        let (amplitude, shift) = stretch.tick();
        let expected = tweened(1.0, 0.0, ACCELERATE, ms);
        assert!(
            (amplitude - expected).abs() < 2e-3,
            "{ms} ms: the amplitude is {amplitude}, not {expected}"
        );
        // One wavelength a second for half a second, and then still: no jump where the loop stopped.
        let travelled = (ms / 1000.0).min(0.5);
        assert!(
            same_shift(shift, (before + travelled) * 40.0, 40.0),
            "{ms} ms: the wave is {shift} along, not {}",
            (before + travelled) * 40.0
        );
    }
    // Nothing is left animating, and the wave is flat.
    assert!(!replay.is_animating());
    let (amplitude, _) = read_wave(&mounted, &replay);
    assert!(amplitude.abs() < 1e-6, "{amplitude}");
    Ok(())
}

#[test]
fn a_wave_that_gets_its_height_back_in_mid_settle_goes_on_from_where_it_was_with_no_jump()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let loops = wave_loops(&items);
    let node = loops[0].node;
    let mut replay = Replay::default();
    replay.apply(&items);
    replay.run(node, 84);
    let before = 83.0 / REPLAY_HZ;
    app.press("Low")?;
    replay.apply(&recording.in_step("press(Low)"));
    // A quarter of a second into the settle: the amplitude is part way down and the travel is part way through its
    // half wavelength. 30 frames, the last at 29/120 s.
    replay.run(node, 30);
    let settled = 29.0 / REPLAY_HZ;
    let (amplitude_then, shift_then) = read_wave(&mounted, &replay);
    let expected_then = tweened(1.0, 0.0, ACCELERATE, settled * 1000.0);
    assert!(
        (amplitude_then - expected_then).abs() < 2e-3
            && amplitude_then > 0.1
            && amplitude_then < 0.99,
        "{amplitude_then} is not {expected_then}"
    );
    assert!(same_shift(shift_then, (before + settled) * 40.0, 40.0));
    // The progress is back: half. The amplitude is to be 1 again, along the standard curve, from where it is.
    app.press("Half")?;
    let step = recording.in_step("press(Half)");
    let tweens = started_tweens(&step);
    let [grow] = &tweens[..] else {
        panic!("a tween of the amplitude: {tweens:?}");
    };
    assert_eq!(grow.to.to_bits(), 1.0_f32.to_bits());
    assert_eq!(grow.duration_ms.to_bits(), 500.0_f32.to_bits());
    assert_eq!(grow.easing, Easing::Bezier(0.2, 0.0, 0.0, 1.0));
    assert_eq!(
        (grow.delay_ms.to_bits(), grow.frame_rate),
        (0.0_f32.to_bits(), None)
    );
    // The loop starts again, once.
    assert_eq!(started_composites(&step), 1, "{step:?}");
    let restarted = wave_loops(&step);
    assert_eq!(restarted.len(), 1, "{restarted:?}");
    assert_eq!(restarted[0].node, node, "the same value");
    replay.apply(&step);
    let travelled_so_far = before + settled;
    let mut stretch = Stretch {
        mounted: &mounted,
        replay: &mut replay,
        node,
    };
    for frame in 0_u32..130 {
        let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
        let (amplitude, shift) = stretch.tick();
        let expected = tweened(expected_then, 1.0, STANDARD, ms);
        assert!(
            (amplitude - expected).abs() < 2e-3,
            "{ms} ms: the amplitude is {amplitude}, not {expected}"
        );
        // The wave is where it was and goes on at one wavelength a second: the settle's tween on the value stops
        // where it is and does not go on moving the value that the loop starts from.
        assert!(
            same_shift(shift, (travelled_so_far + ms / 1000.0) * 40.0, 40.0),
            "{ms} ms: the wave is {shift} along, not {}",
            (travelled_so_far + ms / 1000.0) * 40.0
        );
    }
    assert!(replay.is_animating());
    Ok(())
}

#[test]
fn a_wave_that_starts_flat_grows_along_the_standard_curve_and_starts_to_travel_when_the_progress_passes_a_tenth()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.05));
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let mut replay = Replay::default();
    replay.apply(&items);
    app.press("Half")?;
    let step = recording.in_step("press(Half)");
    let tweens = started_tweens(&step);
    let [grow] = &tweens[..] else {
        panic!("a tween of the amplitude: {tweens:?}");
    };
    assert_eq!(grow.to.to_bits(), 1.0_f32.to_bits());
    assert_eq!(grow.easing, Easing::Bezier(0.2, 0.0, 0.0, 1.0));
    let loops = wave_loops(&step);
    let [travelling] = &loops[..] else {
        panic!("one loop: {loops:?}");
    };
    replay.apply(&step);
    let mut stretch = Stretch {
        mounted: &mounted,
        replay: &mut replay,
        node: travelling.node,
    };
    for frame in 0_u32..100 {
        let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
        let (amplitude, shift) = stretch.tick();
        let expected = tweened(0.0, 1.0, STANDARD, ms);
        assert!(
            (amplitude - expected).abs() < 2e-3,
            "{ms} ms: the amplitude is {amplitude}, not {expected}"
        );
        assert!(
            same_shift(shift, ms / 1000.0 * 40.0, 40.0),
            "{ms} ms: the wave is {shift} along"
        );
    }
    Ok(())
}

#[test]
fn the_wave_goes_on_when_the_progress_stops_being_known_and_when_it_is_known_again()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.5));
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let node = wave_loops(&items)[0].node;
    let mut replay = Replay::default();
    replay.apply(&items);
    replay.run(node, 30);
    // Not a number: indeterminate. The two lines' sweeps start, and the wave's loop is not started again.
    app.press("Nan")?;
    let step = recording.in_step("press(Nan)");
    assert_eq!(started_composites(&step), 1, "{step:?}");
    assert_eq!(wave_loops(&step), Vec::new(), "{step:?}");
    assert_eq!(operations(&step, is_stop), 0, "{step:?}");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    replay.apply(&step);
    for frame in 0_u32..60 {
        let (amplitude, shift) = {
            replay.run(node, 1);
            read_wave(&mounted, &replay)
        };
        let ms = 29.0 * 1000.0 / REPLAY_HZ + f64::from(frame) * 1000.0 / REPLAY_HZ;
        assert!((amplitude - 1.0).abs() < 1e-6, "{ms} ms");
        let _ = shift;
    }
    // Known again: half. The sweeps stop; the wave goes on.
    app.press("Half")?;
    let step = recording.in_step("press(Half)");
    assert_eq!(operations(&step, is_stop), 1, "{step:?}");
    assert_eq!(started_composites(&step), 0, "{step:?}");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    Ok(())
}

// ---- the component, indeterminate ---------------------------------------------------------------------------------

#[test]
fn a_wavy_indeterminate_indicator_starts_the_four_sweeps_and_the_loop_of_the_wave() {
    let (_app, recording) = start(wavy_indeterminate());
    let items = recording.items();
    assert_eq!(started_composites(&items), 2, "{items:?}");
    // The wave: a wavelength a second, as the determinate one (the indeterminate wavelength and speed are equal).
    let loops = wave_loops(&items);
    let [travelling] = &loops[..] else {
        panic!("one loop of one value: {loops:?}");
    };
    assert_eq!(
        (travelling.to.to_bits(), travelling.duration_ms.to_bits()),
        (1.0_f32.to_bits(), 1000.0_f32.to_bits())
    );
    assert_eq!(
        (
            travelling.easing,
            travelling.frame_rate,
            travelling.iterations
        ),
        (Easing::Linear, None, None)
    );
    // The sweeps are the ones the flat indicator plays: the same steps in the same cycle.
    let (_flat, flat_recording) = start(indeterminate_config());
    let flat = four_tracks(&flat_recording.items());
    let wavy = course_tracks(&items);
    assert_eq!(wavy.len(), 4);
    for (wavy, flat) in wavy.iter().zip(&flat) {
        assert_eq!(
            (&wavy.steps, wavy.iterations),
            (&flat.steps, flat.iterations)
        );
    }
    // The wave is as high as it is to be from the start: no tween.
    assert_eq!(started_tweens(&items), Vec::new());
}

/// `at` as the layout of the wavy form lays out the lines and the track.
fn wavy_layout(at: Course<f32>, width: f32) -> Layout {
    layout(
        &[
            (Value::Fixed(at.first_tail), Value::Fixed(at.first_head)),
            (Value::Fixed(at.second_tail), Value::Fixed(at.second_head)),
        ],
        width,
    )
}

/// Which pieces of `layout` show: its lines, and then its track.
fn decisions(layout: &Layout) -> Vec<bool> {
    layout
        .lines
        .iter()
        .chain(&layout.track)
        .map(|piece| fixed(piece.shown) > 0.5)
        .collect()
}

/// Whether a frame with the values `at` is at one of androidx's conditions, where its `if` is a step and the drawing's
/// gate is a ramp, so that the pose of a moment a millisecond earlier or later is as right as the one androidx has:
///
/// - a value that is a hair off 0 is the start of a rise, where androidx's value is exactly 0 and the graph's is the
///   curve's first microseconds (1e-16, 8e-33), which the zero ramp reads as 0 or as a part of a step, whichever it
///   lands in;
/// - moving any one of the values that are in motion by a hundred thousandth, either way, changes which pieces show.
fn near_a_decision(at: Course<f32>, width: f32) -> bool {
    let values = [at.first_head, at.first_tail, at.second_head, at.second_tail];
    if values.iter().any(|v| v.to_bits() != 0 && v.abs() < 1e-9) {
        return true;
    }
    let base = decisions(&wavy_layout(at, width));
    let at_rest = |value: f32| value.to_bits() == 0 || value.to_bits() == 1.0_f32.to_bits();
    (0..4).filter(|i| !at_rest(values[*i])).any(|i| {
        [-1e-5, 1e-5].into_iter().any(|by| {
            let mut moved = values;
            moved[i] += by;
            let moved = course(moved[0], moved[1], moved[2], moved[3]);
            decisions(&wavy_layout(moved, width)) != base
        })
    })
}

/// One frame of a wavy indeterminate indicator that is checked: what the graph holds then and what the layout says for it.
struct FrameCheck<'a> {
    mounted: &'a Mounted,
    replay: &'a Replay,
    scene: Scene,
    width: f32,
    rtl: bool,
    frame: u16,
    at: Course<f32>,
    expected: Layout,
    /// How far along the wave is, in points.
    shift: f64,
    /// The frame on the end of a wavelength may be on either side of the jump back to the start.
    on_a_wrap: bool,
}

impl FrameCheck<'_> {
    /// Where a failure is, and what it is about.
    fn what(&self, name: &str) -> String {
        format!(
            "{} wide, rtl {}, frame {}, {name}: {:?}",
            self.width, self.rtl, self.frame, self.at
        )
    }

    /// The track's pieces are where and as shown as the layout says.
    fn check_track(&self) {
        let container = scheme_color(ColorRole::SecondaryContainer, false);
        let tracks = decoded_by(self.mounted, self.mounted.track, self.replay);
        assert_eq!(
            tracks.rules.len(),
            self.expected.track.len(),
            "{}",
            self.what("track")
        );
        for (i, (rule, piece)) in tracks.rules.iter().zip(&self.expected.track).enumerate() {
            let shown = fixed(piece.shown) > 0.5;
            assert_eq!(
                (rule.brush.width.to_bits(), rule.brush.cap, rule.brush.color),
                (4.0_f64.to_bits(), LineCap::Round, container),
                "{}",
                self.what("track")
            );
            assert!(
                (rule.brush.alpha - f64::from(u8::from(shown))).abs() < 1e-3,
                "{}: piece {i} shows {} of itself",
                self.what("track"),
                rule.brush.alpha
            );
            if shown {
                let (from, to) = (self.scene.base(rule.from).0, self.scene.base(rule.to).0);
                assert!(
                    (from - f64::from(fixed(piece.from))).abs() < 5e-3
                        && (to - f64::from(fixed(piece.to))).abs() < 5e-3,
                    "{}: piece {i} is {from} to {to}, not {} to {}",
                    self.what("track"),
                    fixed(piece.from),
                    fixed(piece.to)
                );
            }
        }
    }

    /// The two lines' waves, windows and circles are where and as shown as the layout says.
    fn check_lines(&self) {
        let primary = scheme_color(ColorRole::Primary, false);
        let lines = decoded_by(self.mounted, self.mounted.indicator, self.replay);
        assert_eq!(lines.chains.len(), 2, "{}", self.what("lines"));
        assert_eq!(lines.discs.len(), 4, "{}: no stop", self.what("lines"));
        for (i, window) in self.expected.lines.iter().enumerate() {
            let shown = fixed(window.shown) > 0.5;
            let (from, to) = (f64::from(fixed(window.from)), f64::from(fixed(window.to)));
            let alpha = f64::from(u8::from(shown));
            let (tail, head) = (&lines.discs[2 * i], &lines.discs[2 * i + 1]);
            assert!(
                (lines.chains[i].brush.alpha - alpha).abs() < 1e-3,
                "{}: line {i} shows {} of itself",
                self.what("lines"),
                lines.chains[i].brush.alpha
            );
            if shown && !self.on_a_wrap {
                for (disc, x) in [(tail, from), (head, to)] {
                    let center = self.scene.base(disc.center);
                    assert!(
                        (center.0 - x).abs() < 5e-3
                            && (center.1 - self.scene.wave_y(x, 1.0, self.shift)).abs() < 4e-3,
                        "{}: line {i}'s circle is at {center:?}, not at {x}",
                        self.what("lines")
                    );
                }
            }
            // Every so often, all of it: the curves, the clips and the circles of a whole window.
            if self.frame.is_multiple_of(41) && !self.on_a_wrap {
                assert_window(
                    &lines.chains[i],
                    (tail, head),
                    self.scene,
                    Window {
                        from,
                        to,
                        amplitude: 1.0,
                        shift: self.shift,
                        alpha,
                    },
                    primary,
                    &self.what(&format!("line {i}")),
                );
            }
        }
    }
}

/// Plays a wavy indeterminate indicator of this size for two and a half cycles, a 120 Hz frame at a time, and checks at
/// every frame that what the canvases draw is what the layout says for the values the graph holds then, and the wave
/// is where its loop has taken it.
fn assert_the_wavy_canvases_follow_the_layout(width: f32, rtl: bool) {
    let (_app, recording) = start(Config {
        width: Points(width),
        rtl,
        ..wavy_indeterminate()
    });
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let nodes: Vec<NodeId> = course_tracks(&items)
        .iter()
        .map(|track| track.node)
        .collect();
    let mut replay = Replay::default();
    replay.apply(&items);
    let scene = scene(width, rtl, 20.0);
    let mut undecided = 0_u32;
    for frame in 0_u16..525 {
        replay.run(nodes[0], 1);
        let ms = f64::from(frame) * 1000.0 / REPLAY_HZ;
        let value = |i: usize| replay.value(nodes[i]);
        let at = course(value(0), value(1), value(2), value(3));
        if near_a_decision(at, width) {
            undecided += 1;
            continue;
        }
        // The wave: as high as it is, a wavelength of 20 a second.
        let (amplitude, shift) = read_wave(&mounted, &replay);
        let check = FrameCheck {
            mounted: &mounted,
            replay: &replay,
            scene,
            width,
            rtl,
            frame,
            at,
            expected: wavy_layout(at, width),
            shift,
            on_a_wrap: (ms % 1000.0).min(1000.0 - ms % 1000.0) < 1e-6,
        };
        assert!(
            (amplitude - 1.0).abs() < 1e-6,
            "{}",
            check.what("amplitude")
        );
        if !check.on_a_wrap {
            assert!(
                same_shift(shift, ms % 1000.0 / 1000.0 * 20.0, 20.0),
                "{}: the wave is {shift} along",
                check.what("wave")
            );
        }
        check.check_track();
        check.check_lines();
    }
    // The frames that are left out are those at a condition. 3 of 525 for most widths: frames 78, 288 and 498, where the
    // second head's rise starts, and the graph's value is the curve's first microseconds where androidx's is exactly 0.
    // 5 for 8 points, where a line is shorter than the caps of the others.
    assert!(
        undecided <= 5,
        "{width} wide, rtl {rtl}: {undecided} of 525 frames left out"
    );
}

#[test]
fn what_the_wavy_canvases_draw_at_every_frame_is_what_the_layout_says() {
    // 240 is the default; 100 has the tracks and the lines near each other; 30 is narrow; 10 is as wide as it is tall;
    // 8 is narrower, so without caps.
    for width in [240.0, 100.0, 30.0, 10.0, 8.0] {
        for rtl in [false, true] {
            assert_the_wavy_canvases_follow_the_layout(width, rtl);
        }
    }
}

#[test]
fn a_wavy_indeterminate_indicator_has_the_range_and_no_value_for_screen_readers() {
    let mounted = mount(wavy_indeterminate());
    assert_eq!(range(&mounted), Some((Some(0.0), Some(1.0), None)));
    assert_eq!(mounted.size(), Size::new(Points(240.0), Points(10.0)));
}

#[test]
fn a_wavy_indeterminate_indicator_draws_no_stop() {
    let (mounted, replay) = mounted_with_graph(wavy_indeterminate());
    let lines = decoded_by(&mounted, mounted.indicator, &replay);
    // Two lines, each with a circle at either end; none at the end of the track.
    assert_eq!(lines.discs.len(), 4);
    assert_eq!(lines.chains.len(), 2);
    assert_eq!(lines.rules.len(), 0);
}

// ---- reduce motion ------------------------------------------------------------------------------------------------

fn still_wavy(rtl: bool) -> Config {
    Config {
        reduce_motion: true,
        rtl,
        ..wavy_indeterminate()
    }
}

#[test]
fn with_reduce_motion_a_wavy_indeterminate_indicator_holds_the_still_pose_and_the_wave_holds_still()
-> Result<(), String> {
    for rtl in [false, true] {
        let (mut app, recording) = start(still_wavy(rtl));
        app.run_for(std::time::Duration::from_secs(3))?;
        let items = recording.items();
        // Nothing starts, nothing is set, and nothing is left running.
        assert_eq!(started_composites(&items), 0);
        assert_eq!(operations(&items, is_motion), 0, "{items:?}");
        assert!(!replay(&recording).is_animating());
        let mounted = Mounted::of(&items);
        let mut graph = Replay::default();
        graph.apply(&items);
        let scene = scene(240.0, rtl, 20.0);
        // The pose at 1000 ms, which the flat form's numbers give: line 1 from 97.34052 to the end, line 2 from the
        // start to 24.15111, and the track between them from 89.34052 to 32.15111; a wave of full height at the
        // start of its travel.
        let drawn = decoded_by(&mounted, mounted.track, &graph);
        assert_rules(
            &visible_rules(&drawn.rules, scene),
            &[(89.340_52, 32.151_11)],
            "the track between",
        );
        let lines = decoded_by(&mounted, mounted.indicator, &graph);
        let [first, second] = &lines.chains[..] else {
            panic!("two lines: {:?}", lines.chains);
        };
        let [tail_1, head_1, tail_2, head_2] = &lines.discs[..] else {
            panic!("four circles: {:?}", lines.discs);
        };
        let primary = scheme_color(ColorRole::Primary, false);
        for (chain, ends, (from, to), what) in [
            (first, (tail_1, head_1), (97.340_52, 238.0), "line 1"),
            (second, (tail_2, head_2), (2.0, 24.151_11), "line 2"),
        ] {
            assert_window(
                chain,
                ends,
                scene,
                Window {
                    from,
                    to,
                    amplitude: 1.0,
                    shift: 0.0,
                    alpha: 1.0,
                },
                primary,
                what,
            );
        }
        // The same element for screen readers as the moving one.
        assert_eq!(range(&mounted), Some((Some(0.0), Some(1.0), None)));
    }
    Ok(())
}

#[test]
fn the_still_pose_of_the_wave_is_the_one_the_layout_gives_for_the_pose() {
    let pose = still_pose();
    let expected = wavy_layout(pose, 240.0);
    let (mounted, replay) = mounted_with_graph(still_wavy(false));
    let drawn = decoded_by(&mounted, mounted.track, &replay);
    let (track, lines) = drawn_pieces(&expected);
    let scene = scene(240.0, false, 20.0);
    let found = visible_rules(&drawn.rules, scene);
    assert_eq!(found.len(), track.len());
    for (found, expected) in found.iter().zip(&track) {
        assert!(
            (found.0 - f64::from(expected.0)).abs() < 1e-3
                && (found.1 - f64::from(expected.1)).abs() < 1e-3
        );
    }
    let decoded = decoded_by(&mounted, mounted.indicator, &replay);
    assert_eq!(decoded.chains.len(), lines.len());
}

#[test]
fn with_reduce_motion_a_determinate_wavy_indicator_holds_its_wave_still_at_its_start()
-> Result<(), String> {
    for (fraction, amplitude, window) in [
        (0.5, 1.0, (2.0, 120.0)),
        (0.05, 0.0, (2.0, 12.0)),
        (0.97, 0.0, (2.0, 232.8)),
    ] {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..wavy_at(fraction)
        });
        app.run_for(std::time::Duration::from_secs(2))?;
        let items = recording.items();
        assert_eq!(started_composites(&items), 0, "{fraction}");
        assert_eq!(operations(&items, is_motion), 0, "{fraction}");
        assert!(!replay(&recording).is_animating(), "{fraction}");
        let mounted = Mounted::of(&items);
        let mut graph = Replay::default();
        graph.apply(&items);
        let lines = decoded_by(&mounted, mounted.indicator, &graph);
        let [chain] = &lines.chains[..] else {
            panic!("one line");
        };
        let [tail, head, _] = &lines.discs[..] else {
            panic!("three circles");
        };
        assert_window(
            chain,
            (tail, head),
            scene(240.0, false, 40.0),
            Window {
                from: window.0,
                to: window.1,
                amplitude,
                shift: 0.0,
                alpha: 1.0,
            },
            scheme_color(ColorRole::Primary, false),
            &format!("{fraction}"),
        );
    }
    Ok(())
}

#[test]
fn with_reduce_motion_a_change_of_height_is_at_once_and_does_not_tween() -> Result<(), String> {
    let (mut app, recording) = start(Config {
        reduce_motion: true,
        ..wavy_at(0.5)
    });
    let items = recording.items();
    let mounted = Mounted::of(&items);
    let mut graph = Replay::default();
    graph.apply(&items);
    app.press("Low")?;
    let step = recording.in_step("press(Low)");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    assert_eq!(started_composites(&step), 0, "{step:?}");
    graph.apply(&step);
    // At once, with no frame played.
    let (amplitude, shift) = read_wave(&mounted, &graph);
    assert!(
        amplitude.abs() < 1e-9 && shift.abs() < 1e-9,
        "{amplitude} {shift}"
    );
    assert!(!graph.is_animating());
    app.press("Half")?;
    let step = recording.in_step("press(Half)");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    graph.apply(&step);
    let (amplitude, _) = read_wave(&mounted, &graph);
    assert!((amplitude - 1.0).abs() < 1e-9, "{amplitude}");
    assert!(!graph.is_animating());
    Ok(())
}

#[test]
fn the_wave_follows_the_reduce_motion_setting_when_it_changes() -> Result<(), String> {
    let (mut app, recording) = start(wavy_at(0.5));
    assert!(replay(&recording).is_animating());
    app.set_environment("reduceMotion", |environment| {
        environment.reduce_motion = true;
    })?;
    let step = recording.in_step("reduceMotion");
    assert_eq!(operations(&step, is_stop), 1, "{step:?}");
    assert_eq!(layout_items(&step), 0, "{step:?}");
    assert!(!replay(&recording).is_animating());
    // The wave is drawn at the start of its travel.
    let mounted = Mounted::of(&recording.items());
    let mut graph = replay(&recording);
    let (amplitude, shift) = read_wave(&mounted, &graph);
    assert!(
        (amplitude - 1.0).abs() < 1e-6 && shift.abs() < 1e-9,
        "{amplitude} {shift}"
    );
    graph.run(1, 1);
    // And when motion is allowed again it travels.
    app.set_environment("motion", |environment| {
        environment.reduce_motion = false;
    })?;
    let step = recording.in_step("motion");
    assert_eq!(started_composites(&step), 1, "{step:?}");
    assert!(replay(&recording).is_animating());
    Ok(())
}

#[test]
fn the_sweeps_and_the_wave_both_stop_when_an_indeterminate_wavy_indicator_is_told_to_hold_still()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_indeterminate());
    assert!(replay(&recording).is_animating());
    app.set_environment("reduceMotion", |environment| {
        environment.reduce_motion = true;
    })?;
    let step = recording.in_step("reduceMotion");
    // The sweeps' composite and the wave's loop.
    assert_eq!(operations(&step, is_stop), 2, "{step:?}");
    assert!(!replay(&recording).is_animating());
    Ok(())
}

// ---- the shape and the progress, changed while it plays ------------------------------------------------------------

#[test]
fn a_flat_indeterminate_indicator_that_becomes_wavy_gets_a_wave_and_keeps_its_sweeps()
-> Result<(), String> {
    let (mut app, recording) = start(indeterminate_config());
    app.press("Waves")?;
    let step = recording.in_step("press(Waves)");
    // The wave's loop starts, with the amplitude at its height at once (no tween); the sweeps go on, and are not
    // started again.
    assert_eq!(started_composites(&step), 1, "{step:?}");
    assert_eq!(wave_loops(&step).len(), 1, "{step:?}");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    assert_eq!(operations(&step, is_stop), 0, "{step:?}");
    assert!(replay(&recording).is_animating());
    let mounted = Mounted::of(&recording.items());
    assert_eq!(mounted.size(), Size::new(Points(240.0), Points(10.0)));
    // Flat again: the wave stops and the sweeps go on.
    app.press("Straight")?;
    let step = recording.in_step("press(Straight)");
    assert_eq!(operations(&step, is_stop), 1, "{step:?}");
    assert_eq!(started_composites(&step), 0, "{step:?}");
    assert!(replay(&recording).is_animating());
    Ok(())
}

#[test]
fn a_wavy_indeterminate_indicator_that_gets_its_progress_stops_its_sweeps_and_goes_on_waving()
-> Result<(), String> {
    let (mut app, recording) = start(wavy_indeterminate());
    app.press("Half")?;
    let step = recording.in_step("press(Half)");
    assert_eq!(operations(&step, is_stop), 1, "the sweeps: {step:?}");
    assert_eq!(started_composites(&step), 0, "{step:?}");
    assert_eq!(started_tweens(&step), Vec::new(), "{step:?}");
    assert!(replay(&recording).is_animating(), "the wave goes on");
    // With a progress under a tenth the wave loses its height instead, and settles.
    let (mut app, recording) = start(wavy_indeterminate());
    app.press("Low")?;
    let step = recording.in_step("press(Low)");
    assert_eq!(
        operations(&step, is_stop),
        2,
        "the sweeps and the loop: {step:?}"
    );
    let tweens = started_tweens(&step);
    assert_eq!(tweens.len(), 2, "{tweens:?}");
    let mut graph = replay(&recording);
    for _ in 0..90 {
        graph.run(1, 1);
    }
    assert!(!graph.is_animating());
    Ok(())
}
