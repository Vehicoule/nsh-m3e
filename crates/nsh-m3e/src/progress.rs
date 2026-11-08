//! What the progress indicators share: how far the work is ([`Progress`]), whether the indicator is flat or wavy
//! ([`ProgressShape`]), and the parts every indicator builds the same way (its accessibility, whether its looping
//! motion plays).
//!
//! The indicators follow androidx's `LinearProgressIndicator`, `LinearWavyProgressIndicator` and their circular
//! siblings in `ProgressIndicator.kt` and `WavyProgressIndicator.kt`, at the pinned commit, as [ADR 0012] says. Their
//! colors are the tokens of `ProgressIndicatorTokens`: the active indicator and the stop are the primary color, the
//! track the secondary container color. A progress indicator shows no text (ADR 0008).
//!
//! # Accessibility
//!
//! An indicator is one element for screen readers, with the progress bar role and the range 0 to 1, as androidx's
//! `ProgressBarRangeInfo(value, 0f..1f)`; while it is indeterminate it has the range and no value. It is not
//! `nsh-base`'s `progress()`: that one's range is 0 to 100, and it builds a track and an indicator out of views,
//! where these indicators draw both on canvases.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::AccessibilityValue;
use nsh_core::element::{JsHost, JsIn, JsOwned, JsValue, PropError, TsTypes};
use nsh_core::geometry::Percentage;
use nsh_core::prelude::*;

use crate::tint::{Tint, tint_color};
use crate::tokens::CubicBezier;
use crate::tokens::androidx::progress_indicator_tokens;

/// How far the work an indicator stands for has got.
///
/// ```
/// use nsh_core::geometry::percentage;
/// use nsh_m3e::progress::Progress;
///
/// assert_eq!(Progress::default(), Progress::Indeterminate);
/// // A fraction outside the range is held to it, as androidx coerces `progress` into 0 to 1.
/// assert_eq!(Progress::Determinate(percentage(1.5)).coerced(), Some(percentage(1.0)));
/// assert_eq!(Progress::Determinate(percentage(-0.5)).coerced(), Some(percentage(0.0)));
/// assert_eq!(Progress::Indeterminate.coerced(), None);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Progress {
    /// The work's extent is unknown (connecting, preparing): the indicator moves on its own, and screen readers hear a
    /// progress bar with no value.
    #[default]
    Indeterminate,
    /// The share of the work done, as a fraction: 0 is none and 1 is all of it, as androidx's `progress`.
    ///
    /// A fraction outside 0 to 1 is held to it (see [`Progress::coerced`]). A NaN draws no more than the stop of a
    /// flat indicator and is read as 0 by screen readers, as androidx's flat indicator does with one. A wavy
    /// indicator takes a fraction that is not finite (a NaN, an infinity) for no progress at all, as androidx's wavy
    /// indicator does ("Return a non-finite value ... while the progress is unknown"): it is indeterminate.
    Determinate(Percentage),
}

impl Progress {
    /// The fraction held to 0 to 1 (androidx `progress().fastCoerceIn(0f, 1f)`), or `None` while indeterminate. A NaN
    /// stays a NaN, as it does in androidx, which draws and reads it as its own case.
    #[must_use]
    pub fn coerced(self) -> Option<Percentage> {
        match self {
            Self::Indeterminate => None,
            Self::Determinate(fraction) => Some(Percentage(fraction.0.clamp(0.0, 1.0))),
        }
    }

    /// What screen readers get as the value, if anything: the coerced fraction, a NaN read as 0 as androidx does
    /// (`coercedProgress().takeUnless { it.isNaN() } ?: 0f`, "as the `ProgressBarRangeInfo` will throw an exception").
    pub(crate) fn reading(self) -> Option<f32> {
        self.coerced()
            .map(|fraction| if fraction.0.is_nan() { 0.0 } else { fraction.0 })
    }

    /// Whether an indicator of `shape` shows this as no progress: the work's extent is unknown, or, for a wavy
    /// indicator, the fraction is not finite (`!currentProgress.isFinite()` in `LinearWavyProgressIndicator`).
    pub(crate) fn is_indeterminate_in(self, shape: ProgressShape) -> bool {
        match self {
            Self::Indeterminate => true,
            Self::Determinate(fraction) => shape == ProgressShape::Wavy && !fraction.0.is_finite(),
        }
    }

    /// What screen readers get as the value of an indicator of `shape`: [`Progress::reading`], or nothing where the
    /// indicator is indeterminate in that shape (a wavy one with a fraction that is not finite).
    pub(crate) fn reading_in(self, shape: ProgressShape) -> Option<f32> {
        if self.is_indeterminate_in(shape) {
            None
        } else {
            self.reading()
        }
    }
}

impl From<Option<Percentage>> for Progress {
    /// No fraction is indeterminate.
    fn from(fraction: Option<Percentage>) -> Self {
        fraction.map_or(Self::Indeterminate, Self::Determinate)
    }
}

impl From<Progress> for Option<Percentage> {
    /// Indeterminate has no fraction.
    fn from(progress: Progress) -> Self {
        match progress {
            Progress::Indeterminate => None,
            Progress::Determinate(fraction) => Some(fraction),
        }
    }
}

/// NSH's props are values it can convert to and from JS, which this crate's components never involve: a progress is
/// a number, the fraction, or `null` while indeterminate, as an optional fraction is.
impl JsValue for Progress {
    fn ts_type(types: &mut TsTypes) -> String {
        <Option<Percentage>>::ts_type(types)
    }

    fn from_js(value: JsIn<'_>) -> Result<Self, PropError> {
        <Option<Percentage>>::from_js(value).map(Self::from)
    }

    fn to_js<'a>(&self, host: &'a dyn JsHost) -> Result<JsOwned<'a>, PropError> {
        Option::<Percentage>::from(*self).to_js(host)
    }
}

/// The shape of a progress indicator's active part.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ProgressShape {
    /// A straight line: androidx's `LinearProgressIndicator` and `CircularProgressIndicator`.
    #[default]
    Flat,
    /// A line that waves, which M3E added: androidx's `LinearWavyProgressIndicator` and
    /// `CircularWavyProgressIndicator`.
    Wavy,
}

/// The range screen readers get: 0 to 1, androidx `ProgressBarRangeInfo(_, 0f..1f)`.
const RANGE: (f32, f32) = (0.0, 1.0);

/// Makes `view` one element for screen readers: a progress bar, named by `label` when there is one, in the range 0 to
/// 1 with the value `reading` gives (none while indeterminate).
pub(crate) fn accessible(
    view: View,
    reading: impl Fn() -> Option<f32> + 'static,
    label: impl Fn() -> Option<SharedString> + 'static,
) -> View {
    view.accessible(true)
        .role(AccessibilityRole::Progressbar)
        .optional_label(label)
        .accessibility_value(move || {
            Some(AccessibilityValue {
                min: Some(RANGE.0),
                max: Some(RANGE.1),
                now: reading(),
                text: None,
            })
        })
}

/// Whether the looping motion of an indicator plays: not when the user asked for less motion (reduce motion), where an
/// indicator that would loop holds a still pose instead. Reading it subscribes, so an indicator that reads it in a
/// computation follows the setting when it changes.
pub(crate) fn plays_loops() -> bool {
    !reduce_motion()
}

/// `curve`, a Material 3 easing token, as the curve NSH's animations take.
pub(crate) fn easing(curve: CubicBezier) -> Easing {
    Easing::Bezier(curve.x1, curve.y1, curve.x2, curve.y2)
}

/// The color of the active indicator and of the stop: androidx `ProgressIndicatorTokens.ActiveIndicatorColor`, which
/// the default `drawStopIndicator` takes as its `color` (`StopColor` names the same role).
pub(crate) fn indicator_color() -> AdaptiveColor {
    tint_color(Some(Tint::solid(
        progress_indicator_tokens::ACTIVE_INDICATOR_COLOR,
    )))
}

/// The color of the track: androidx `ProgressIndicatorTokens.TrackColor`.
pub(crate) fn track_color() -> AdaptiveColor {
    tint_color(Some(Tint::solid(progress_indicator_tokens::TRACK_COLOR)))
}

#[cfg(test)]
mod tests {
    use nsh_core::geometry::percentage;

    use super::*;

    #[test]
    fn a_progress_is_indeterminate_until_it_is_given() {
        assert_eq!(Progress::default(), Progress::Indeterminate);
        assert_eq!(ProgressShape::default(), ProgressShape::Flat);
    }

    #[test]
    fn a_progress_is_an_optional_fraction() {
        use nsh_core::JsonValue;
        use nsh_core::element::from_json;

        let half = percentage(0.5);
        assert_eq!(Progress::from(None), Progress::Indeterminate);
        assert_eq!(Progress::from(Some(half)), Progress::Determinate(half));
        assert_eq!(Option::from(Progress::Indeterminate), None::<Percentage>);
        assert_eq!(Option::from(Progress::Determinate(half)), Some(half));
        // What a prop reads from JS: `null` is indeterminate, a number the fraction.
        assert_eq!(
            from_json::<Progress>(&JsonValue::Null).ok(),
            Some(Progress::Indeterminate)
        );
        assert_eq!(
            from_json::<Progress>(&JsonValue::from(0.5)).ok(),
            Some(Progress::Determinate(half))
        );
        assert!(from_json::<Progress>(&JsonValue::from("half")).is_err());
        assert_eq!(
            <Progress as JsValue>::ts_type(&mut TsTypes::default()),
            "number | null"
        );
    }

    #[test]
    fn a_fraction_outside_the_range_is_held_to_it() {
        let coerced = |fraction: f32| Progress::Determinate(percentage(fraction)).coerced();
        assert_eq!(coerced(-0.25), Some(percentage(0.0)));
        assert_eq!(coerced(0.0), Some(percentage(0.0)));
        assert_eq!(coerced(0.25), Some(percentage(0.25)));
        assert_eq!(coerced(1.0), Some(percentage(1.0)));
        assert_eq!(coerced(1.25), Some(percentage(1.0)));
        assert_eq!(coerced(f32::INFINITY), Some(percentage(1.0)));
        assert_eq!(coerced(f32::NEG_INFINITY), Some(percentage(0.0)));
        assert_eq!(Progress::Indeterminate.coerced(), None);
    }

    #[test]
    fn a_nan_stays_a_nan_in_the_coerced_fraction_and_screen_readers_read_it_as_zero() {
        // androidx: `NaN.fastCoerceIn(0f, 1f)` is NaN, and the range info takes 0 instead.
        let nan = Progress::Determinate(percentage(f32::NAN));
        assert!(nan.coerced().is_some_and(|fraction| fraction.0.is_nan()));
        assert_eq!(nan.reading(), Some(0.0));
    }

    #[test]
    fn only_a_wavy_indicator_takes_a_fraction_that_is_not_finite_for_no_progress() {
        use ProgressShape::{Flat, Wavy};

        // Unknown is unknown in both shapes.
        for shape in [Flat, Wavy] {
            assert!(Progress::Indeterminate.is_indeterminate_in(shape));
            assert_eq!(Progress::Indeterminate.reading_in(shape), None);
            // A finite fraction, in range or not, is known in both.
            for fraction in [-3.0, -0.0, 0.0, 0.4, 1.0, 3.0] {
                let progress = Progress::Determinate(percentage(fraction));
                assert!(!progress.is_indeterminate_in(shape), "{fraction}");
                assert_eq!(progress.reading_in(shape), progress.reading(), "{fraction}");
            }
        }
        // A fraction that is not finite: the flat indicator draws and reads it as its own case (NaN: 0, the infinities
        // held to the range), the wavy one has no progress to show and screen readers no value.
        for fraction in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let progress = Progress::Determinate(percentage(fraction));
            assert!(!progress.is_indeterminate_in(Flat), "{fraction}");
            assert!(progress.reading_in(Flat).is_some(), "{fraction}");
            assert!(progress.is_indeterminate_in(Wavy), "{fraction}");
            assert_eq!(progress.reading_in(Wavy), None, "{fraction}");
        }
        assert_eq!(
            Progress::Determinate(percentage(f32::NAN)).reading_in(Flat),
            Some(0.0)
        );
        assert_eq!(
            Progress::Determinate(percentage(f32::INFINITY)).reading_in(Flat),
            Some(1.0)
        );
        assert_eq!(
            Progress::Determinate(percentage(f32::NEG_INFINITY)).reading_in(Flat),
            Some(0.0)
        );
    }

    #[test]
    fn screen_readers_read_the_coerced_fraction_and_nothing_while_indeterminate() {
        let reading = |progress: Progress| progress.reading();
        assert_eq!(reading(Progress::Indeterminate), None);
        assert_eq!(reading(Progress::Determinate(percentage(0.4))), Some(0.4));
        assert_eq!(reading(Progress::Determinate(percentage(3.0))), Some(1.0));
        assert_eq!(reading(Progress::Determinate(percentage(-3.0))), Some(0.0));
    }

    #[test]
    fn a_curve_of_the_tokens_is_the_easing_nsh_takes_with_its_points_in_order() {
        use crate::tokens::androidx::motion_tokens;

        // Four different numbers, so that a swapped pair shows.
        assert_eq!(
            easing(CubicBezier::new(0.1, 0.2, 0.3, 0.4)),
            Easing::Bezier(0.1, 0.2, 0.3, 0.4)
        );
        assert_eq!(
            easing(motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER),
            Easing::Bezier(0.3, 0.0, 0.8, 0.15)
        );
    }

    #[test]
    fn the_indicator_and_the_stop_are_primary_and_the_track_the_secondary_container() {
        use crate::color::ColorRole;

        assert_eq!(
            progress_indicator_tokens::ACTIVE_INDICATOR_COLOR,
            ColorRole::Primary
        );
        // The default stop takes the indicator's color, which is the stop token's.
        assert_eq!(
            progress_indicator_tokens::STOP_COLOR,
            progress_indicator_tokens::ACTIVE_INDICATOR_COLOR
        );
        assert_eq!(
            progress_indicator_tokens::TRACK_COLOR,
            ColorRole::SecondaryContainer
        );
    }
}
