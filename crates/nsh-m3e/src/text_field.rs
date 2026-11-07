//! The Material 3 Expressive text field: the filled and the outlined one, with a label that floats up when the field
//! has the focus or a value, an optional placeholder, leading and trailing icon, prefix and suffix, and supporting
//! text below.
//!
//! The text field follows androidx's `TextField` and `OutlinedTextField` in `TextField.kt`, `OutlinedTextField.kt`,
//! `TextFieldDefaults.kt` and `internal/TextFieldImpl.kt` at the pinned commit, as [ADR 0012] says: it reads the
//! tokens they read (`FilledTextFieldTokens`, `OutlinedTextFieldTokens`), and the values they write in place of a
//! token, each cited where it is used and pinned by `every_choice_taken_from_androidx_is_still_there`. The input is
//! `nsh-base`'s `Input`, the native text field; this module gives it the look.
//!
//! # What it does differently from androidx
//!
//! - **One status, not three booleans.** androidx's `enabled`, `readOnly` and `isError` are not independent: a
//!   disabled field shows its disabled colors whether or not it is in error, and read-only changes no color. A
//!   [`TextFieldStatus`] is one of the five combinations that mean something.
//! - **A label is required.** androidx's label is optional; Material 3 and `docs/app-requirements.md` ask for a
//!   label on every control, and it is the field's name for screen readers, so there is no field without one.
//! - **Colors change at once.** androidx animates the colors of the label, the container and the indicator with the
//!   effects spring. NSH's animated color nodes keep the colors they were made with, so a change of status cannot be
//!   animated from one color to another; the colors switch at once. What androidx moves with a spring moves here:
//!   the label (the fast spatial spring), the indicator's and the outline's thickness (the same one), and the
//!   placeholder's and the prefix and suffix's opacity (the fast and slow effects springs), all through transform and
//!   opacity, so the field is laid out again for none of them.
//! - **Hover changes nothing.** `FilledTextFieldTokens` and `OutlinedTextFieldTokens` have hover colors, but
//!   `TextFieldColors` resolves its colors by enabled, error and focus only, so androidx's text field shows no hover
//!   (ADR 0012: the component wins).
//! - **The default container is the disabled one too.** `FilledTextFieldTokens.DisabledContainerColor` (on surface at
//!   4 %) is not used by androidx, which draws the container color in every state.
//! - **Reduce motion.** With reduce motion on, nothing springs: the label and the indicator jump to where they go,
//!   and the label, the placeholder and the prefix and suffix fade in over [`REDUCED_MOTION_FADE`] instead.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::prelude::*;
use nsh_core::{AccessibilityRole, LineHeight, PointerEvents, SvgPaint, TransformKind};

use crate::icon::Icon;
use crate::motion::{SPRING_MASS, SpringToken};
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::androidx::{
    filled_text_field_tokens as filled, motion_tokens, outlined_text_field_tokens as outlined,
    small_icon_button_tokens, typography_tokens,
};
use crate::tokens::{RoundedCorners, Shape, TypeStyle};

/// Which text field it is.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum TextFieldStyle {
    /// A container in the surface container highest color, with an indicator line along its bottom edge and square
    /// bottom corners: androidx `TextField`, its default.
    #[default]
    Filled,
    /// No container, an outline all around it with a notch in it where the label floats: androidx
    /// `OutlinedTextField`.
    Outlined,
}

/// Whether a [`TextField`] can be edited, and whether it shows an error.
///
/// androidx has three booleans for this, `enabled`, `readOnly` and `isError`, that are not independent: a disabled
/// field takes the colors of a disabled one whatever else it is, and `readOnly` changes no color. The five
/// combinations that mean something are the five values here, so a field that is disabled and read-only, or
/// disabled and in error, cannot be written.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum TextFieldStatus {
    /// Editable, with no error: androidx's default.
    #[default]
    Enabled,
    /// The text can be selected and copied but not changed (`readOnly`). Nothing shows it: the colors are the
    /// enabled ones.
    ReadOnly,
    /// Editable, and the value is not valid (`isError`): the error colors, and the supporting text says why.
    Error,
    /// The text can be selected and copied but not changed, and the value is not valid: the error colors.
    ReadOnlyError,
    /// No typing and no focus (`enabled = false`): the disabled colors.
    Disabled,
}

impl TextFieldStatus {
    /// Whether the value is shown as not valid.
    fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::ReadOnlyError)
    }

    /// Whether the text can not be changed, by a field that is read-only or disabled.
    fn is_read_only(self) -> bool {
        matches!(self, Self::ReadOnly | Self::ReadOnlyError | Self::Disabled)
    }

    /// Whether it takes no focus.
    fn is_disabled(self) -> bool {
        self == Self::Disabled
    }
}

/// How many lines a [`TextField`] shows.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum TextFieldLines {
    /// One line: return submits (androidx `singleLine`).
    #[default]
    Single,
    /// Several lines: it grows with its text and return adds a line.
    Multiple,
}

/// What a text field looks like, by androidx's order of precedence: `TextFieldColors` takes the disabled colors when
/// the field is not enabled, else the error colors, else the focused ones, else the unfocused ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Resting,
    Focused,
    Error,
    Disabled,
}

impl Look {
    fn of(status: TextFieldStatus, focused: bool) -> Self {
        if status.is_disabled() {
            Self::Disabled
        } else if status.is_error() {
            Self::Error
        } else if focused {
            Self::Focused
        } else {
            Self::Resting
        }
    }
}

/// A color in each [`Look`].
#[derive(Debug, Clone, Copy, PartialEq)]
struct ByLook {
    resting: Tint,
    focused: Tint,
    error: Tint,
    disabled: Tint,
}

impl ByLook {
    /// The same color whatever the focus: androidx gives some a color for each of focused and unfocused that are one
    /// token.
    const fn unfocused(resting: Tint, error: Tint, disabled: Tint) -> Self {
        Self {
            resting,
            focused: resting,
            error,
            disabled,
        }
    }

    fn of(self, look: Look) -> Tint {
        match look {
            Look::Resting => self.resting,
            Look::Focused => self.focused,
            Look::Error => self.error,
            Look::Disabled => self.disabled,
        }
    }
}

/// What a style gives a text field, in each look.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Colors {
    label: ByLook,
    leading_icon: ByLook,
    trailing_icon: ByLook,
    supporting: ByLook,
    input: ByLook,
    placeholder: ByLook,
    prefix: ByLook,
    suffix: ByLook,
    /// The filled field's active indicator, the outlined field's outline.
    line: ByLook,
    /// The container: a color for the filled field in every look (androidx `defaultTextFieldColors`), none for the
    /// outlined one (`Color.Transparent`).
    container: Option<Tint>,
    caret: Tint,
    error_caret: Tint,
}

/// The colors of `style`: `defaultTextFieldColors` and `defaultOutlinedTextFieldColors` in `TextFieldDefaults.kt`.
fn colors(style: TextFieldStyle) -> Colors {
    match style {
        TextFieldStyle::Filled => filled_colors(),
        TextFieldStyle::Outlined => outlined_colors(),
    }
}

/// `defaultTextFieldColors`: `FilledTextFieldTokens`.
fn filled_colors() -> Colors {
    Colors {
        label: ByLook {
            resting: Tint::solid(filled::LABEL_COLOR),
            focused: Tint::solid(filled::FOCUS_LABEL_COLOR),
            error: Tint::solid(filled::ERROR_LABEL_COLOR),
            disabled: Tint::faded(filled::DISABLED_LABEL_COLOR, filled::DISABLED_LABEL_OPACITY),
        },
        leading_icon: ByLook {
            resting: Tint::solid(filled::LEADING_ICON_COLOR),
            focused: Tint::solid(filled::FOCUS_LEADING_ICON_COLOR),
            error: Tint::solid(filled::ERROR_LEADING_ICON_COLOR),
            disabled: Tint::faded(
                filled::DISABLED_LEADING_ICON_COLOR,
                filled::DISABLED_LEADING_ICON_OPACITY,
            ),
        },
        trailing_icon: ByLook {
            resting: Tint::solid(filled::TRAILING_ICON_COLOR),
            focused: Tint::solid(filled::FOCUS_TRAILING_ICON_COLOR),
            error: Tint::solid(filled::ERROR_TRAILING_ICON_COLOR),
            disabled: Tint::faded(
                filled::DISABLED_TRAILING_ICON_COLOR,
                filled::DISABLED_TRAILING_ICON_OPACITY,
            ),
        },
        supporting: ByLook {
            resting: Tint::solid(filled::SUPPORTING_COLOR),
            focused: Tint::solid(filled::FOCUS_SUPPORTING_COLOR),
            error: Tint::solid(filled::ERROR_SUPPORTING_COLOR),
            disabled: Tint::faded(
                filled::DISABLED_SUPPORTING_COLOR,
                filled::DISABLED_SUPPORTING_OPACITY,
            ),
        },
        input: ByLook {
            resting: Tint::solid(filled::INPUT_COLOR),
            focused: Tint::solid(filled::FOCUS_INPUT_COLOR),
            error: Tint::solid(filled::ERROR_INPUT_COLOR),
            disabled: Tint::faded(filled::DISABLED_INPUT_COLOR, filled::DISABLED_INPUT_OPACITY),
        },
        placeholder: ByLook::unfocused(
            Tint::solid(filled::INPUT_PLACEHOLDER_COLOR),
            Tint::solid(filled::INPUT_PLACEHOLDER_COLOR),
            // androidx fades the input color for a disabled placeholder: the tokens name no such color.
            Tint::faded(filled::DISABLED_INPUT_COLOR, filled::DISABLED_INPUT_OPACITY),
        ),
        prefix: ByLook::unfocused(
            Tint::solid(filled::INPUT_PREFIX_COLOR),
            Tint::solid(filled::INPUT_PREFIX_COLOR),
            // androidx fades the prefix's own color, not the input's: `disabledPrefixColor`.
            Tint::faded(filled::INPUT_PREFIX_COLOR, filled::DISABLED_INPUT_OPACITY),
        ),
        suffix: ByLook::unfocused(
            Tint::solid(filled::INPUT_SUFFIX_COLOR),
            Tint::solid(filled::INPUT_SUFFIX_COLOR),
            Tint::faded(filled::INPUT_SUFFIX_COLOR, filled::DISABLED_INPUT_OPACITY),
        ),
        line: ByLook {
            resting: Tint::solid(filled::ACTIVE_INDICATOR_COLOR),
            focused: Tint::solid(filled::FOCUS_ACTIVE_INDICATOR_COLOR),
            error: Tint::solid(filled::ERROR_ACTIVE_INDICATOR_COLOR),
            disabled: Tint::faded(
                filled::DISABLED_ACTIVE_INDICATOR_COLOR,
                filled::DISABLED_ACTIVE_INDICATOR_OPACITY,
            ),
        },
        // The container color in every state, the disabled one included: `FilledTextFieldTokens
        // .DisabledContainerColor` (on surface at 4 %) is not read (ADR 0012).
        container: Some(Tint::solid(filled::CONTAINER_COLOR)),
        caret: Tint::solid(filled::CARET_COLOR),
        error_caret: Tint::solid(filled::ERROR_FOCUS_CARET_COLOR),
    }
}

/// `defaultOutlinedTextFieldColors`: `OutlinedTextFieldTokens`.
fn outlined_colors() -> Colors {
    Colors {
        label: ByLook {
            resting: Tint::solid(outlined::LABEL_COLOR),
            focused: Tint::solid(outlined::FOCUS_LABEL_COLOR),
            error: Tint::solid(outlined::ERROR_LABEL_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_LABEL_COLOR,
                outlined::DISABLED_LABEL_OPACITY,
            ),
        },
        leading_icon: ByLook {
            resting: Tint::solid(outlined::LEADING_ICON_COLOR),
            focused: Tint::solid(outlined::FOCUS_LEADING_ICON_COLOR),
            error: Tint::solid(outlined::ERROR_LEADING_ICON_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_LEADING_ICON_COLOR,
                outlined::DISABLED_LEADING_ICON_OPACITY,
            ),
        },
        trailing_icon: ByLook {
            resting: Tint::solid(outlined::TRAILING_ICON_COLOR),
            focused: Tint::solid(outlined::FOCUS_TRAILING_ICON_COLOR),
            error: Tint::solid(outlined::ERROR_TRAILING_ICON_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_TRAILING_ICON_COLOR,
                outlined::DISABLED_TRAILING_ICON_OPACITY,
            ),
        },
        supporting: ByLook {
            resting: Tint::solid(outlined::SUPPORTING_COLOR),
            focused: Tint::solid(outlined::FOCUS_SUPPORTING_COLOR),
            error: Tint::solid(outlined::ERROR_SUPPORTING_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_SUPPORTING_COLOR,
                outlined::DISABLED_SUPPORTING_OPACITY,
            ),
        },
        input: ByLook {
            resting: Tint::solid(outlined::INPUT_COLOR),
            focused: Tint::solid(outlined::FOCUS_INPUT_COLOR),
            error: Tint::solid(outlined::ERROR_INPUT_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_INPUT_COLOR,
                outlined::DISABLED_INPUT_OPACITY,
            ),
        },
        placeholder: ByLook::unfocused(
            Tint::solid(outlined::INPUT_PLACEHOLDER_COLOR),
            Tint::solid(outlined::INPUT_PLACEHOLDER_COLOR),
            Tint::faded(
                outlined::DISABLED_INPUT_COLOR,
                outlined::DISABLED_INPUT_OPACITY,
            ),
        ),
        prefix: ByLook::unfocused(
            Tint::solid(outlined::INPUT_PREFIX_COLOR),
            Tint::solid(outlined::INPUT_PREFIX_COLOR),
            Tint::faded(
                outlined::INPUT_PREFIX_COLOR,
                outlined::DISABLED_INPUT_OPACITY,
            ),
        ),
        suffix: ByLook::unfocused(
            Tint::solid(outlined::INPUT_SUFFIX_COLOR),
            Tint::solid(outlined::INPUT_SUFFIX_COLOR),
            Tint::faded(
                outlined::INPUT_SUFFIX_COLOR,
                outlined::DISABLED_INPUT_OPACITY,
            ),
        ),
        line: ByLook {
            resting: Tint::solid(outlined::OUTLINE_COLOR),
            focused: Tint::solid(outlined::FOCUS_OUTLINE_COLOR),
            error: Tint::solid(outlined::ERROR_OUTLINE_COLOR),
            disabled: Tint::faded(
                outlined::DISABLED_OUTLINE_COLOR,
                outlined::DISABLED_OUTLINE_OPACITY,
            ),
        },
        container: None,
        caret: Tint::solid(outlined::CARET_COLOR),
        error_caret: Tint::solid(outlined::ERROR_FOCUS_CARET_COLOR),
    }
}

/// The thickness of the indicator line or the outline: 2 points while the field has the focus, 1 otherwise, and 1
/// while it is disabled whatever the focus, an error or not (androidx `animateBorderStrokeAsState`, with
/// `TextFieldDefaults.FocusedIndicatorThickness`, `UnfocusedIndicatorThickness` and the outlined `*BorderThickness`,
/// which are the tokens' heights).
fn thickness(style: TextFieldStyle, status: TextFieldStatus, phase: Phase) -> Points {
    let (resting, focus, off) = match style {
        TextFieldStyle::Filled => (
            filled::ACTIVE_INDICATOR_HEIGHT,
            filled::FOCUS_ACTIVE_INDICATOR_HEIGHT,
            filled::DISABLED_ACTIVE_INDICATOR_HEIGHT,
        ),
        TextFieldStyle::Outlined => (
            outlined::OUTLINE_WIDTH,
            outlined::FOCUS_OUTLINE_WIDTH,
            outlined::DISABLED_OUTLINE_WIDTH,
        ),
    };
    if status.is_disabled() {
        off
    } else if phase == Phase::Focused {
        focus
    } else {
        resting
    }
}

/// The thickest the indicator or the outline gets.
const MAX_THICKNESS: Points = filled::FOCUS_ACTIVE_INDICATOR_HEIGHT;

/// The smallest height of a text field: androidx `TextFieldDefaults.MinHeight` (`TextFieldDefaults.kt` line 83) and
/// `OutlinedTextFieldDefaults.MinHeight` (line 1009), the outlined container's height token.
const MIN_HEIGHT: Points = Points(56.0);
/// The smallest width: androidx `TextFieldDefaults.MinWidth` (`TextFieldDefaults.kt` line 89) and
/// `OutlinedTextFieldDefaults.MinWidth` (line 1015).
const MIN_WIDTH: Points = Points(280.0);
/// The space between the container's edge, or an icon, and the text: androidx `TextFieldPadding`
/// (`TextFieldImpl.kt` line 2235).
const TEXT_FIELD_PADDING: Points = Points(16.0);
/// The space above the label and below the text of a filled field: androidx `TextFieldWithLabelVerticalPadding`
/// (`TextField.kt` line 867).
const FILLED_VERTICAL_PADDING: Points = Points(8.0);
/// The space above the supporting text: androidx `SupportingTopPadding` (`TextFieldImpl.kt` line 2241).
const SUPPORTING_TOP_PADDING: Points = Points(4.0);
/// The space between a prefix or a suffix and the text: androidx `PrefixSuffixTextPadding` (`TextFieldImpl.kt` line
/// 2243).
const PREFIX_SUFFIX_PADDING: Points = Points(2.0);
/// The smallest height of the text's line: androidx `MinTextLineHeight` (`TextFieldImpl.kt` line 2245).
const MIN_TEXT_LINE_HEIGHT: Points = Points(24.0);
/// The smallest height of the supporting text's line: androidx `MinSupportingTextLineHeight` (`TextFieldImpl.kt`
/// line 2249).
const MIN_SUPPORTING_LINE_HEIGHT: Points = Points(16.0);
/// How far the label's notch reaches past the label on each side: androidx `OutlinedTextFieldInnerPadding`
/// (`OutlinedTextField.kt` line 619).
const NOTCH_PADDING: Points = Points(4.0);
/// The corner radius of the outlined field: its container shape token, `CornerExtraSmall`.
const OUTLINED_RADIUS: Points = crate::tokens::androidx::shape_tokens::CORNER_VALUE_EXTRA_SMALL;

/// The typography of the text, the label, the placeholder, the prefix and the suffix (`InputFont`, `LabelFont`:
/// body large) and of the supporting text (`SupportingFont`: body small). Both field styles name the same ones.
const INPUT_FONT: TypeStyle = filled::INPUT_FONT;
const LABEL_FONT: TypeStyle = filled::LABEL_FONT;
const SUPPORTING_FONT: TypeStyle = filled::SUPPORTING_FONT;
/// The icons' glyph size: `LeadingIconSize` and `TrailingIconSize`, which both field styles give the same.
const ICON_SIZE: Points = filled::LEADING_ICON_SIZE;
/// The label's type style when it floats: body small.
const FLOATING_LABEL_FONT: TypeStyle = typography_tokens::BODY_SMALL;

/// How tall the input is: one line, 24 points, however tall the platform's text is; the height of its text, at least
/// one line, for several lines (androidx `MinTextLineHeight`, and a text field that grows with its lines).
fn input_height(lines: TextFieldLines, content: Points) -> Points {
    match lines {
        TextFieldLines::Single => MIN_TEXT_LINE_HEIGHT,
        _ => Points(content.0.max(MIN_TEXT_LINE_HEIGHT.0)),
    }
}

/// How a text field's label is laid out, in one style and with or without a leading icon.
#[derive(Debug, Clone, Copy, PartialEq)]
struct LabelGeometry {
    /// How far the floating label is above the one at rest, in points (negative): androidx's label goes from the
    /// middle of the field (`CenterVertically`) to `topPadding` for the filled field, and to half its height above the
    /// outline for the outlined one.
    rise: f32,
    /// How far the floating label is to the right of the one at rest (negative to the left): the outlined field's
    /// label moves over its leading icon, the filled field's does not move across.
    slide: f32,
}

/// The space before the text, from the container's edge or an icon: the padding less what an icon's box already
/// holds around its glyph (`textFieldHorizontalIconPadding`, `TextFieldImpl.kt` line 2170).
fn side_padding(icon: bool) -> Points {
    if icon {
        Points((TEXT_FIELD_PADDING.0 - icon_padding().0).max(0.0))
    } else {
        TEXT_FIELD_PADDING
    }
}

/// How far the label, resting, is from the container's edge on the side of an icon, or of none: the icon's box, then
/// the space before the text.
fn label_inset(icon: bool) -> Points {
    Points(if icon { MIN_TOUCH_TARGET.0 } else { 0.0 } + side_padding(icon).0)
}

/// An icon's box is as wide as the touch target and its glyph is 24 points: half the difference is space the icon
/// brings with it (`textFieldHorizontalIconPadding`).
fn icon_padding() -> Points {
    Points(((MIN_TOUCH_TARGET.0 - small_icon_button_tokens::ICON_SIZE.0) / 2.0).max(0.0))
}

/// The label's top edge, at rest, from the field's top: centered in the minimum height.
fn label_top() -> f32 {
    (MIN_HEIGHT.0 - LABEL_FONT.line_height.0) / 2.0
}

/// The size of the floating label's text over the size at rest.
fn label_scale() -> f32 {
    FLOATING_LABEL_FONT.size.0 / LABEL_FONT.size.0
}

fn label_geometry(style: TextFieldStyle, leading_icon: bool) -> LabelGeometry {
    // The label's middle, at rest and floating, from the field's top.
    let at_rest = MIN_HEIGHT.0 / 2.0;
    let floating = match style {
        // `topPadding`, then half the floating label's line.
        TextFieldStyle::Filled => {
            FILLED_VERTICAL_PADDING.0 + FLOATING_LABEL_FONT.line_height.0 / 2.0
        }
        // On the outline: the label's middle is the top edge.
        TextFieldStyle::Outlined => 0.0,
    };
    let slide = match style {
        TextFieldStyle::Filled => 0.0,
        // `startX` and `endX` of `placeWithLabel`: the label goes from after the icon to the padding.
        TextFieldStyle::Outlined => TEXT_FIELD_PADDING.0 - label_inset(leading_icon).0,
    };
    LabelGeometry {
        rise: floating - at_rest,
        slide,
    }
}

/// Where the notch in the outline begins along the top edge: `TextFieldPadding` less the notch's own padding
/// (androidx `outlineCutout`).
fn notch_left() -> f32 {
    TEXT_FIELD_PADDING.0 - NOTCH_PADDING.0
}

/// How far along the top edge the notch can reach: the straight part of the edge ends where the corner begins.
fn notch_reach(width: Points) -> f32 {
    width.0 - OUTLINED_RADIUS.0
}

/// How far the outlined field's container sticks out above its first line: the label floats half its height above the
/// outline (`topPaddingForLabelCutout`: `minimizedLabelHalfHeight`).
fn label_overhang() -> Points {
    Points(FLOATING_LABEL_FONT.line_height.0 / 2.0)
}

/// The props of a [`TextField`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct TextFieldProps {
    /// The text (controlled): the app owns it and sets the one `Change` reports; whenever it sets another, the field
    /// shows it.
    pub value: Prop<SharedString>,
    /// What the field asks for, and its name for screen readers. It rests in the middle of the field, over the text,
    /// and floats up when the field has the focus or a value.
    pub label: Prop<SharedString>,
    /// Which text field it is.
    #[prop(default)]
    pub style: Prop<TextFieldStyle>,
    /// Whether it can be edited, and whether it shows an error.
    #[prop(default)]
    pub status: Prop<TextFieldStatus>,
    /// How many lines it shows.
    #[prop(default)]
    pub lines: Prop<TextFieldLines>,
    /// Shown while the field has the focus and no text (the app's words).
    pub placeholder: Prop<Option<SharedString>>,
    /// An icon before the text, 24 points in a 48-point box. It is not a button.
    pub leading_icon: Prop<Option<Icon>>,
    /// An icon after the text, 24 points in a 48-point box. It is not a button.
    pub trailing_icon: Prop<Option<Icon>>,
    /// Text before the input, shown while the field has the focus or a value (a currency sign).
    pub prefix: Prop<Option<SharedString>>,
    /// Text after the input, shown while the field has the focus or a value (a unit).
    pub suffix: Prop<Option<SharedString>>,
    /// A line under the field: what to type, or, in error, what is wrong. Screen readers say it as the field's hint.
    pub supporting_text: Prop<Option<SharedString>>,
}

/// The events of a [`TextField`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum TextFieldEvent {
    /// The user changed the text: `value` is the text now.
    #[args(value)]
    Change(String),
    /// The user pressed return in a single-line field: `value` is the text.
    #[args(value)]
    Submit(String),
    /// The field gained the focus.
    Focus,
    /// The field lost the focus.
    Blur,
}

/// What a text field is doing, from its focus and its text: androidx `InputPhase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Focused,
    UnfocusedEmpty,
    UnfocusedNotEmpty,
}

/// How far the label has floated, as the springs move it: 0 at rest in the middle, 1 floating.
fn label_target(phase: Phase) -> f32 {
    match phase {
        Phase::UnfocusedEmpty => 0.0,
        Phase::Focused | Phase::UnfocusedNotEmpty => 1.0,
    }
}

/// How opaque the placeholder is (androidx `placeholderOpacity`): only while the field has the focus.
fn placeholder_target(phase: Phase) -> f32 {
    match phase {
        Phase::Focused => 1.0,
        Phase::UnfocusedEmpty | Phase::UnfocusedNotEmpty => 0.0,
    }
}

/// How opaque the prefix and the suffix are (androidx `affixOpacity`): hidden while the label rests over their place.
fn affix_target(phase: Phase) -> f32 {
    match phase {
        Phase::UnfocusedEmpty => 0.0,
        Phase::Focused | Phase::UnfocusedNotEmpty => 1.0,
    }
}

/// The spring the placeholder fades with when the field goes `from` one phase `to` another, as androidx's
/// `placeholderOpacity` chooses it: slow when the placeholder comes in as the field is focused or emptied, fast
/// otherwise (androidx names the move from focused to empty as fast too, which is its default).
fn placeholder_spring(from: Phase, to: Phase) -> SpringToken {
    match (from, to) {
        (Phase::UnfocusedEmpty, Phase::Focused)
        | (Phase::UnfocusedNotEmpty, Phase::UnfocusedEmpty) => SpringToken::SlowEffects,
        _ => SpringToken::FastEffects,
    }
}

/// The spring of a token, as ADR 0004 converts it. Every field is written out, none taken from NSH's defaults: it
/// starts from the speed the value already has, at the screen's own rate (ADR 0010).
fn spring_of(token: SpringToken) -> Spring {
    let spec = token.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// The fade that replaces a spring when the user asks for less motion: androidx has none, and ADR 0010 and ADR 0011
/// leave its duration to the first component that needs it. This is the shortest of the standard durations that is
/// not a blink, with the standard easing, both from the tokens (`MotionTokens.DurationShort2`,
/// `EasingStandardCubicBezier`).
pub const REDUCED_MOTION_FADE: Timing = Timing::new(
    motion_tokens::DURATION_SHORT2,
    Easing::Bezier(
        motion_tokens::EASING_STANDARD_CUBIC_BEZIER.x1,
        motion_tokens::EASING_STANDARD_CUBIC_BEZIER.y1,
        motion_tokens::EASING_STANDARD_CUBIC_BEZIER.x2,
        motion_tokens::EASING_STANDARD_CUBIC_BEZIER.y2,
    ),
);

/// How an animated value reaches a new target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// With a spring; with reduce motion, at once.
    Move,
    /// With a spring; with reduce motion, with the fade.
    Fade,
}

/// An animated value and where it is going. A `Copy` handle, like the signals it holds. It is created where it rests
/// and moves only when its target changes, so mounting animates nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Follower {
    value: AnimatedValue,
    target: Signal<f32>,
}

impl Follower {
    fn new(at: f32) -> Self {
        Self {
            value: animated_value(at),
            target: signal(at),
        }
    }

    /// Goes to `to` with `spring`, from where it is and as fast as it moves. With reduce motion on nothing springs:
    /// a [`Reach::Move`] arrives at once, a [`Reach::Fade`] fades there over [`REDUCED_MOTION_FADE`]. Nothing is sent
    /// when the target stays the same. It reports whether it started to go.
    fn go(self, to: f32, spring: SpringToken, reach: Reach) -> bool {
        untracked(|| {
            if self.target.get().to_bits() == to.to_bits() {
                return false;
            }
            self.target.set(to);
            if !reduce_motion() {
                let _ = self.value.spring(to, spring_of(spring));
            } else if reach == Reach::Fade {
                let _ = self.value.timing_with(to, REDUCED_MOTION_FADE);
            } else {
                self.value.set(to);
            }
            true
        })
    }

    /// Goes to `to` at once, stopping what moves it.
    fn jump(self, to: f32) {
        untracked(|| {
            if self.target.get().to_bits() == to.to_bits() {
                return;
            }
            self.target.set(to);
            self.value.set(to);
        });
    }
}

/// The path of the outlined field's outline: a rounded rectangle `thickness / 2` inside the container's edge, open
/// where the label's notch is, from `gap_left` to `gap_right` along the top edge. It starts at the notch's right end
/// and goes clockwise, so the stroke ends at its left end. The corners' centers stay where they are while the outline
/// thickens: only the radius and the edges follow `thickness` (ADR 0011's drawn container, with a thickness that moves
/// where its corners did).
fn outline_path(
    painter: Painter,
    size: Size<Points>,
    thickness: &AnimatedValue,
    gap_left: f32,
    gap_right: &AnimatedNode,
) -> Painter {
    use std::f32::consts::{FRAC_PI_2, PI};
    let radius = OUTLINED_RADIUS.0;
    let (width, height) = (size.width.0, size.height.0);
    // The path's edge inside the container's, and the corners' radius inside it.
    let half: AnimatedNode = thickness.mul(0.5);
    let r: AnimatedNode = thickness.mul(-0.5).add(radius);
    let right: AnimatedNode = thickness.mul(-0.5).add(width);
    let bottom: AnimatedNode = thickness.mul(-0.5).add(height);
    painter
        .begin_path()
        .move_to(gap_right, &half)
        .line_to(width - radius, &half)
        .arc(width - radius, radius, &r, -FRAC_PI_2, 0.0)
        .line_to(&right, height - radius)
        .arc(width - radius, height - radius, &r, 0.0, FRAC_PI_2)
        .line_to(radius, &bottom)
        .arc(radius, height - radius, &r, FRAC_PI_2, PI)
        .line_to(&half, radius)
        .arc(radius, radius, &r, PI, PI + FRAC_PI_2)
        .line_to(gap_left, &half)
}

/// The text of a type style on `text`, in `color`.
fn typeset<E: Styled>(element: E, font: TypeStyle) -> E {
    element
        .text_size(font.size)
        .line_height(LineHeight::Points(font.line_height))
        .letter_spacing(font.tracking)
        .style_with(move |s| s.font_weight(font.weight))
}

/// A Material 3 Expressive text field: a native text input in the filled or outlined container, with a label that
/// floats, and the icons, prefix, suffix and supporting text around it.
///
/// The app owns the text: the field reports what the user typed with [`TextFieldEvent::Change`] and shows the
/// `value` it is given. Pressing anywhere in the container focuses the input. The focus, the status and the text
/// change colors, move the label and thicken the indicator or outline, and lay nothing out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::text_field::{TextFieldStatus, TextFieldStyle, text_field};
///
/// fn email() -> impl IntoElement {
///     let value = signal(String::new());
///     text_field()
///         .value(move || value.get())
///         .label("Email")
///         .style(TextFieldStyle::Outlined)
///         .status(move || {
///             if value.get().contains('@') {
///                 TextFieldStatus::Enabled
///             } else {
///                 TextFieldStatus::Error
///             }
///         })
///         .supporting_text("Needs an @")
///         .on_change(move |text: &String| value.set(text.clone()))
/// }
/// ```
#[component]
pub fn TextField(props: TextFieldProps, cx: Scope<TextFieldEvent>) -> impl IntoElement {
    let style = move || props.style.get();
    let status = move || props.status.get();
    let outlined_field = move || style() == TextFieldStyle::Outlined;
    let has_leading = move || props.leading_icon.get().is_some();
    let has_trailing = move || props.trailing_icon.get().is_some();

    // The focus is the native input's: this copy is made from its events, for what the springs need to start.
    let focused = signal(false);
    let phase = computed(move || {
        // A disabled field takes no focus, whatever the platform reports of it.
        if focused.get() && !status().is_disabled() {
            Phase::Focused
        } else if props.value.get().is_empty() {
            Phase::UnfocusedEmpty
        } else {
            Phase::UnfocusedNotEmpty
        }
    });
    let look = move || Look::of(status(), focused.get());
    let palette = move || colors(style());
    let field = element_ref();

    // The animated values, each created where it rests.
    let rest = phase.get_untracked();
    let thick = untracked(|| thickness(style(), status(), rest)).0;
    let label_progress = Follower::new(label_target(rest));
    let label_fade = animated_value(1.0);
    let placeholder_alpha = Follower::new(placeholder_target(rest));
    let affix_alpha = Follower::new(affix_target(rest));
    let line_thickness = Follower::new(thick);
    // Where the floating label is, from where it rests, and the width of the label: set when the style or the icon
    // changes, and the width when the label is laid out, never while the label moves.
    let geometry = computed(move || label_geometry(style(), has_leading()));
    let placed = signal(untracked(|| geometry.get()));
    let rise = animated_value(untracked(|| placed.get().rise));
    let slide = animated_value(untracked(|| placed.get().slide));
    let label_width = animated_value(0.0);
    let previous = signal(rest);
    effect(move || {
        let (to, status) = (phase.get(), status());
        let from = previous.get_untracked();
        previous.set(to);
        if label_progress.go(label_target(to), SpringToken::FastSpatial, Reach::Move)
            && reduce_motion()
        {
            // The label arrives at once, and fades in where it is.
            label_fade.set(0.0);
            let _ = label_fade.timing_with(1.0, REDUCED_MOTION_FADE);
        }
        placeholder_alpha.go(
            placeholder_target(to),
            placeholder_spring(from, to),
            Reach::Fade,
        );
        affix_alpha.go(affix_target(to), SpringToken::FastEffects, Reach::Fade);
        let thickness = thickness(style(), status, to).0;
        if status.is_disabled() {
            // androidx snaps the indicator's thickness while the field is disabled.
            line_thickness.jump(thickness);
        } else {
            line_thickness.go(thickness, SpringToken::FastSpatial, Reach::Move);
        }
    });
    effect(move || {
        let now = geometry.get();
        untracked(|| {
            if placed.get() != now {
                placed.set(now);
                rise.set(now.rise);
                slide.set(now.slide);
            }
        });
    });

    // The label's three transforms follow its progress.
    let label_scale_node = label_progress.value.mul(label_scale() - 1.0).add(1.0);
    let label_rise = label_progress.value.mul(&rise);
    let label_slide = label_progress.value.mul(&slide);
    // The notch in the outline, from where the label floats to its end, grown with the label (androidx
    // `outlineCutout`): the width of the label at its size now, times how far it has floated, and the notch's
    // padding on both sides once the label has begun to float.
    let progress = label_progress.value.interpolate(&[0.0, 1.0], &[0.0, 1.0]);
    let began = label_progress.value.interpolate(&[0.0, 0.01], &[0.0, 1.0]);
    let notch_width: AnimatedNode = progress
        .mul(label_scale() - 1.0)
        .add(1.0)
        .mul(&progress)
        .mul(&label_width)
        .add(&began.mul(2.0 * NOTCH_PADDING.0));
    let outline_size = signal(Size::new(MIN_WIDTH, MIN_HEIGHT));
    let content_height = signal(MIN_TEXT_LINE_HEIGHT);

    let input = nsh_base::input()
        .element_ref(&field)
        .value(move || props.value.get().to_string())
        .label(move || props.label.get())
        .hint(move || props.supporting_text.get().unwrap_or_default())
        .disabled(move || status().is_disabled())
        .readonly(move || status().is_read_only())
        .multiline(move || props.lines.get() == TextFieldLines::Multiple)
        .caret_color(move || {
            let colors = palette();
            color(tint_color(Some(if status().is_error() {
                colors.error_caret
            } else {
                colors.caret
            })))
        })
        .text_color(move || tint_color(Some(palette().input.of(look()))))
        .on_change(move |text: &str| cx.emit(TextFieldEvent::Change(text.to_owned())))
        .on_submit(move |text: &str| cx.emit(TextFieldEvent::Submit(text.to_owned())))
        .on_focus(move || {
            focused.set(true);
            cx.emit(TextFieldEvent::Focus);
        })
        .on_blur(move || {
            focused.set(false);
            cx.emit(TextFieldEvent::Blur);
        })
        .on_content_size_change(move |size: &Size<Points>| content_height.set(size.height))
        .w_full()
        .h(move || input_height(props.lines.get(), content_height.get()));
    let input = typeset(input, INPUT_FONT);

    // The filled container: its color, square at the bottom, with the indicator line along its bottom edge.
    let filled_container = move || {
        let corners = container_corners();
        [
            view()
                .absolute()
                .inset(px(0.0))
                .rounded_ss(corners.top_start)
                .rounded_se(corners.top_end)
                .rounded_ee(corners.bottom_end)
                .rounded_es(corners.bottom_start)
                .bg(move || tint_color(colors(TextFieldStyle::Filled).container)),
            view()
                .absolute()
                .inset(px(0.0))
                .flex_col()
                .justify_end()
                .pointer_events(PointerEvents::None)
                .child(
                    // As thick as it gets, scaled down from its bottom edge to the thickness it has now.
                    view()
                        .h(MAX_THICKNESS)
                        .origin_bottom()
                        .bind_transform(
                            TransformKind::ScaleY,
                            &line_thickness.value.div(MAX_THICKNESS.0),
                        )
                        .bg(move || {
                            tint_color(Some(colors(TextFieldStyle::Filled).line.of(look())))
                        }),
                ),
        ]
    };
    // The outlined container: the outline, on a canvas so that its notch can follow the label.
    let outlined_container = move || {
        canvas(move || {
            let size = outline_size.get();
            // The notch goes no further than the top edge's straight part.
            let reach = notch_reach(size.width);
            outline_path(
                Painter::new(),
                size,
                &line_thickness.value,
                notch_left(),
                &notch_width
                    .add(notch_left())
                    .interpolate(&[0.0, reach], &[0.0, reach]),
            )
            .stroke_style_from_view(SvgPaint::None)
            .line_width(&line_thickness.value)
            .stroke()
        })
        .absolute()
        .inset(px(0.0))
        .stroke_color(move || tint_color(Some(colors(TextFieldStyle::Outlined).line.of(look()))))
        .on_layout(move |bounds: &Bounds<Points>| outline_size.set(bounds.size))
    };

    let icon_box = move |icon: Prop<Option<Icon>>, by_look: fn(Colors) -> ByLook| {
        view()
            .w(MIN_TOUCH_TARGET)
            .h(MIN_TOUCH_TARGET)
            .self_center()
            .items_center()
            .justify_center()
            .child(
                canvas(move || icon.get().unwrap_or_default().drawing(ICON_SIZE))
                    .w(ICON_SIZE)
                    .h(ICON_SIZE)
                    .fill_color(move || tint_color(Some(by_look(palette()).of(look())))),
            )
    };

    let prefix_or_suffix =
        move |text_of: Prop<Option<SharedString>>, by_look: fn(Colors) -> ByLook, before: bool| {
            let word = text(move || text_of.get().unwrap_or_default())
                .text_color(move || tint_color(Some(by_look(palette()).of(look()))))
                .bind_opacity(&affix_alpha.value)
                .accessibility_elements_hidden(true);
            let word = typeset(word, INPUT_FONT);
            if before {
                word.pe(PREFIX_SUFFIX_PADDING)
            } else {
                word.ps(PREFIX_SUFFIX_PADDING)
            }
        };

    view()
        .flex_col()
        .min_w(MIN_WIDTH)
        .pt(move || {
            if outlined_field() {
                label_overhang()
            } else {
                px(0.0)
            }
        })
        .child(
            view()
                .accessible(false)
                .flex_row()
                .items_stretch()
                .min_h(MIN_HEIGHT)
                .on_press(move || field.focus())
                .child(when(move || !outlined_field(), filled_container))
                .child(when(outlined_field, outlined_container))
                .child(when(has_leading, move || {
                    icon_box(props.leading_icon, |c| c.leading_icon)
                }))
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .ps(move || side_padding(has_leading()))
                        .pe(move || side_padding(has_trailing()))
                        .pt(move || {
                            if outlined_field() {
                                TEXT_FIELD_PADDING
                            } else {
                                Points(
                                    FILLED_VERTICAL_PADDING.0 + FLOATING_LABEL_FONT.line_height.0,
                                )
                            }
                        })
                        .pb(move || {
                            if outlined_field() {
                                TEXT_FIELD_PADDING
                            } else {
                                FILLED_VERTICAL_PADDING
                            }
                        })
                        .child(
                            view()
                                .flex_row()
                                .items_start()
                                .child(when(
                                    move || props.prefix.get().is_some(),
                                    move || prefix_or_suffix(props.prefix, |c| c.prefix, true),
                                ))
                                .child(
                                    view()
                                        .flex_1()
                                        .child(when(
                                            move || props.placeholder.get().is_some(),
                                            move || {
                                                let hint = text(move || {
                                                    props.placeholder.get().unwrap_or_default()
                                                })
                                                .absolute()
                                                .inset(px(0.0))
                                                .pointer_events(PointerEvents::None)
                                                .text_color(move || {
                                                    tint_color(Some(
                                                        palette().placeholder.of(look()),
                                                    ))
                                                })
                                                .bind_opacity(&placeholder_alpha.value)
                                                .accessibility_elements_hidden(true)
                                                .style_with(move |s| {
                                                    // Only an empty field has a placeholder; it keeps its place, so
                                                    // that the text coming and going lays nothing out.
                                                    if props.value.get().is_empty() {
                                                        s
                                                    } else {
                                                        s.invisible()
                                                    }
                                                });
                                                typeset(hint, INPUT_FONT)
                                            },
                                        ))
                                        .child(input),
                                )
                                .child(when(
                                    move || props.suffix.get().is_some(),
                                    move || prefix_or_suffix(props.suffix, |c| c.suffix, false),
                                )),
                        ),
                )
                .child(when(has_trailing, move || {
                    icon_box(props.trailing_icon, |c| c.trailing_icon)
                }))
                .child(
                    // The label, over everything: it rests where the text will be and floats up to the top.
                    view()
                        .absolute()
                        .inset(px(0.0))
                        .flex_col()
                        .items_start()
                        .pt(px(label_top()))
                        .ps(move || label_inset(has_leading()))
                        .pe(move || label_inset(has_trailing()))
                        .pointer_events(PointerEvents::None)
                        .child({
                            let label = text(move || props.label.get())
                                .text_color(move || tint_color(Some(palette().label.of(look()))))
                                .origin_left()
                                .bind_translate_x(&label_slide)
                                .bind_translate_y(&label_rise)
                                .bind_scale(&label_scale_node)
                                .bind_opacity(&label_fade)
                                .accessibility_elements_hidden(true)
                                .on_layout(move |bounds: &Bounds<Points>| {
                                    label_width.set(bounds.size.width.0);
                                });
                            typeset(label, LABEL_FONT)
                        }),
                ),
        )
        .child(when(
            move || props.supporting_text.get().is_some(),
            move || {
                let line = text(move || props.supporting_text.get().unwrap_or_default())
                    .text_color(move || tint_color(Some(palette().supporting.of(look()))))
                    .ps(TEXT_FIELD_PADDING)
                    .pe(TEXT_FIELD_PADDING)
                    .pt(SUPPORTING_TOP_PADDING)
                    .min_h(MIN_SUPPORTING_LINE_HEIGHT)
                    .role(move || {
                        if status().is_error() {
                            AccessibilityRole::Alert
                        } else {
                            AccessibilityRole::Text
                        }
                    });
                typeset(line, SUPPORTING_FONT)
            },
        ))
}

/// The container's corners: the filled field's `CornerExtraSmallTop`, which is square at the bottom. A shape that is
/// not a set of rounded corners has square ones.
fn container_corners() -> RoundedCorners {
    match filled::CONTAINER_SHAPE {
        Shape::Rounded(corners) => corners,
        _ => RoundedCorners::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Write as _;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{self, AnimationOp, AnimationProp, Driver, Operand};
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{
        AttributedText, Color, InputConfig, MountItem, NodeId, ROOT_ID, ViewKind, ViewProps,
        ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph_probe::{Step, assert_traces, parameters, paths, replay, tolerance};

    /// Material Symbols' `search` (google/material-design-icons at `10ecc147906`, `symbols/web/search/
    /// materialsymbolsoutlined/search_24px.svg`): test data, the `d` of its one path.
    const SEARCH: &str = "M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z";

    /// What the probe places: one text field with these props, lines that log its events and show its value, and
    /// lines that change its status, its style and its icon, and clear its text, when pressed.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        style: Prop<TextFieldStyle>,
        #[prop(default)]
        status: Prop<TextFieldStatus>,
        #[prop(default)]
        lines: Prop<TextFieldLines>,
        #[prop(default)]
        value: Prop<String>,
        #[prop(default)]
        leading: Prop<bool>,
        #[prop(default)]
        trailing: Prop<bool>,
        #[prop(default)]
        placeholder: Prop<bool>,
        #[prop(default)]
        affixes: Prop<bool>,
        #[prop(default)]
        supporting: Prop<bool>,
        #[prop(default)]
        width: Prop<Option<f32>>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let value = signal(SharedString::from(props.value.get_untracked()));
        let status = signal(props.status.get_untracked());
        let style = signal(props.style.get_untracked());
        let leading = signal(props.leading.get_untracked());
        let log = signal(String::new());
        let (trailing, placeholder, affixes, supporting) = (
            props.trailing.get_untracked(),
            props.placeholder.get_untracked(),
            props.affixes.get_untracked(),
            props.supporting.get_untracked(),
        );
        let entry = move |what: String| {
            log.update(|log| {
                // Writing into a `String` cannot fail.
                let _ = write!(log, "{what};");
            });
        };
        let field = text_field()
            .value(move || value.get())
            .label("Email")
            .style(move || style.get())
            .status(move || status.get())
            .lines(props.lines)
            .placeholder(placeholder.then(|| SharedString::from("name@example.com")))
            .leading_icon(move || leading.get().then(|| Icon::material_symbol(SEARCH)))
            .trailing_icon(trailing.then(|| Icon::material_symbol(SEARCH)))
            .prefix(affixes.then(|| SharedString::from("$")))
            .suffix(affixes.then(|| SharedString::from("kg")))
            .supporting_text(supporting.then(|| SharedString::from("Help")))
            .on_change(move |text: &String| {
                value.set(text.clone().into());
                entry(format!("change:{text}"));
            })
            .on_submit(move |text: &String| entry(format!("submit:{text}")))
            .on_focus(move |(): &()| entry("focus".to_owned()))
            .on_blur(move |(): &()| entry("blur".to_owned()));
        let frame = match props.width.get_untracked() {
            Some(width) => view().w(Points(width)),
            None => view(),
        };
        view()
            .child(frame.child(field))
            .child(text(move || format!("log: {}", log.get())))
            .child(text(move || format!("value: {}", value.get())))
            .child(text("Set error").on_press(move |(): &()| status.set(TextFieldStatus::Error)))
            .child(
                text("Set disabled").on_press(move |(): &()| status.set(TextFieldStatus::Disabled)),
            )
            .child(
                text("Set enabled").on_press(move |(): &()| status.set(TextFieldStatus::Enabled)),
            )
            .child(
                text("Set read-only")
                    .on_press(move |(): &()| status.set(TextFieldStatus::ReadOnly)),
            )
            .child(text("Toggle style").on_press(move |(): &()| {
                style.update(|style| {
                    *style = match *style {
                        TextFieldStyle::Filled => TextFieldStyle::Outlined,
                        _ => TextFieldStyle::Filled,
                    };
                });
            }))
            .child(text("Toggle icon").on_press(move |(): &()| leading.update(|on| *on = !*on)))
            .child(text("Clear").on_press(move |(): &()| value.set(SharedString::from(""))))
    }

    /// A part a text field can have besides its label and its text.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Part {
        Leading,
        Trailing,
        Placeholder,
        Affixes,
        Supporting,
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        style: TextFieldStyle,
        status: TextFieldStatus,
        lines: TextFieldLines,
        value: &'static str,
        parts: &'static [Part],
        width: Option<f32>,
        /// Focus the field before the test looks at it.
        focused: bool,
        dark: bool,
        reduce_motion: bool,
    }

    impl Config {
        /// Every part a text field has.
        fn full(style: TextFieldStyle) -> Self {
            Self {
                style,
                parts: EVERY_PART,
                ..Self::default()
            }
        }
    }

    /// Every part a field has.
    const EVERY_PART: &[Part] = &[
        Part::Leading,
        Part::Trailing,
        Part::Placeholder,
        Part::Affixes,
        Part::Supporting,
    ];

    /// The parts of a field with these icons.
    fn icons(leading: bool, trailing: bool) -> &'static [Part] {
        match (leading, trailing) {
            (true, true) => &[Part::Leading, Part::Trailing],
            (true, false) => &[Part::Leading],
            (false, true) => &[Part::Trailing],
            (false, false) => &[],
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        // Props are signals, which exist only on the engine thread, where `Root` builds them.
        let root = Root::new::<Probe>(move || {
            let has = |part| config.parts.contains(&part);
            ProbeProps {
                style: Prop::new(config.style),
                status: Prop::new(config.status),
                lines: Prop::new(config.lines),
                value: Prop::new(config.value.to_owned()),
                leading: Prop::new(has(Part::Leading)),
                trailing: Prop::new(has(Part::Trailing)),
                placeholder: Prop::new(has(Part::Placeholder)),
                affixes: Prop::new(has(Part::Affixes)),
                supporting: Prop::new(has(Part::Supporting)),
                width: Prop::new(config.width),
            }
        });
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
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
        app.wait_for("label:Email").unwrap();
        if config.focused {
            focus(&mut app);
        }
        (app, recording)
    }

    /// The step that gives the field the focus, as a keyboard user's tab does, typing nothing.
    const FOCUS: &str = "type(label:Email, \"\")";

    fn focus(app: &mut Headless) {
        app.type_text("label:Email", "").unwrap();
    }

    /// A rectangle in the screen's points.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Rect {
        x: Points,
        y: Points,
        width: Points,
        height: Points,
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x: Points(x),
            y: Points(y),
            width: Points(width),
            height: Points(height),
        }
    }

    /// What the text field mounted: its views, and the last style, frame, text and drawing each was sent.
    #[derive(Default)]
    struct Mounted {
        kinds: HashMap<NodeId, ViewKind>,
        parents: HashMap<NodeId, NodeId>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        props: HashMap<NodeId, ViewProps>,
        texts: HashMap<NodeId, Arc<AttributedText>>,
        contents: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        inputs: HashMap<NodeId, InputConfig>,
        bindings: HashMap<NodeId, Vec<(AnimationProp, animation::NodeId)>>,
        ops: HashMap<animation::NodeId, AnimationOp>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut mounted = Self::default();
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        mounted.kinds.insert(*id, *kind);
                    }
                    MountItem::Insert { parent, child, .. } => {
                        mounted.parents.insert(*child, *parent);
                    }
                    MountItem::Remove { id } => {
                        mounted.parents.remove(id);
                    }
                    MountItem::ViewProps { id, props } => {
                        mounted.props.insert(*id, (**props).clone());
                    }
                    MountItem::AccessibilityLabel { id, label } => {
                        if let Some(props) = mounted.props.get_mut(id) {
                            props.accessibility.label.clone_from(label);
                        }
                    }
                    MountItem::ViewStyle { id, style } => {
                        mounted.styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        mounted.frames.insert(*id, *frame);
                    }
                    MountItem::Text {
                        id, text, content, ..
                    } => {
                        mounted.texts.insert(*id, Arc::clone(text));
                        mounted.contents.insert(*id, *content);
                    }
                    MountItem::Canvas { id, drawing } => {
                        mounted.drawings.insert(*id, Arc::clone(drawing));
                    }
                    MountItem::Input { id, config } => {
                        mounted.inputs.insert(*id, (**config).clone());
                    }
                    MountItem::AnimationBindings { id, bindings } => {
                        mounted.bindings.insert(*id, bindings.clone());
                    }
                    MountItem::Animation(
                        op @ (AnimationOp::CreateValue { node, .. }
                        | AnimationOp::CreateMath { node, .. }
                        | AnimationOp::CreateInterpolate { node, .. }),
                    ) => {
                        mounted.ops.insert(*node, op.clone());
                    }
                    _ => {}
                }
            }
            mounted
        }

        /// Whether `id` is in the tree: inserted, and not removed since, under the root.
        fn live(&self, id: NodeId) -> bool {
            let mut at = id;
            loop {
                if at == ROOT_ID {
                    return true;
                }
                match self.parents.get(&at) {
                    Some(parent) => at = *parent,
                    None => return false,
                }
            }
        }

        /// The views of `kind` in the tree, by id.
        fn views(&self, kind: ViewKind) -> Vec<NodeId> {
            let mut ids: Vec<NodeId> = self
                .kinds
                .iter()
                .filter(|(id, k)| **k == kind && self.live(**id))
                .map(|(id, _)| *id)
                .collect();
            ids.sort_unstable();
            ids
        }

        /// The one text view that shows `content`, if any.
        fn text(&self, content: &str) -> Option<NodeId> {
            let found: Vec<NodeId> = self
                .texts
                .iter()
                .filter(|(id, text)| text.text == content && self.live(**id))
                .map(|(id, _)| *id)
                .collect();
            match found[..] {
                [id] => Some(id),
                [] => None,
                _ => panic!("several texts show {content:?}"),
            }
        }

        fn input(&self) -> NodeId {
            let [input] = self.views(ViewKind::Input)[..] else {
                panic!("one input");
            };
            input
        }

        /// The view that holds the container, the icons and the text: the one that hides itself from screen readers.
        fn field(&self) -> NodeId {
            let found: Vec<NodeId> = self
                .props
                .iter()
                .filter(|(id, props)| {
                    props.accessibility.accessible == Some(false) && self.live(**id)
                })
                .map(|(id, _)| *id)
                .collect();
            let [field] = found[..] else {
                panic!("one field view, not {found:?}");
            };
            field
        }

        /// `id`'s place on the screen.
        fn rect(&self, id: NodeId) -> Rect {
            let own = self.frames[&id];
            let (mut x, mut y) = (own.origin.x.0, own.origin.y.0);
            let mut at = id;
            while let Some(parent) = self.parents.get(&at) {
                if let Some(frame) = self.frames.get(parent) {
                    x += frame.origin.x.0;
                    y += frame.origin.y.0;
                }
                at = *parent;
            }
            rect(x, y, own.size.width.0, own.size.height.0)
        }

        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn props(&self, id: NodeId) -> ViewProps {
            self.props.get(&id).cloned().unwrap_or_default()
        }

        fn text_color(&self, content: &str) -> Color {
            let id = self
                .text(content)
                .unwrap_or_else(|| panic!("no text {content:?}"));
            self.texts[&id].spans[0].style.color
        }

        /// The canvases that draw an icon (they fill), left to right.
        fn icons(&self) -> Vec<NodeId> {
            let mut icons: Vec<NodeId> = self
                .views(ViewKind::Canvas)
                .into_iter()
                .filter(|id| {
                    self.drawings[id]
                        .ops
                        .iter()
                        .any(|op| matches!(op, CanvasOp::Fill(..)))
                })
                .collect();
            icons.sort_by(|a, b| self.rect(*a).x.0.total_cmp(&self.rect(*b).x.0));
            icons
        }

        /// The canvas that draws the outline (it strokes), if the field is outlined.
        fn outline(&self) -> Option<NodeId> {
            self.views(ViewKind::Canvas).into_iter().find(|id| {
                self.drawings[id]
                    .ops
                    .iter()
                    .any(|op| matches!(op, CanvasOp::Stroke(_)))
            })
        }

        /// The filled container: the view that paints the container color, with its corners.
        fn container(&self) -> Option<NodeId> {
            self.kinds
                .iter()
                .filter(|(id, kind)| **kind == ViewKind::View && self.live(**id))
                .map(|(id, _)| *id)
                .find(|id| {
                    let style = self.style(*id);
                    style.background_color != Color::TRANSPARENT && style.transform.ops().is_empty()
                })
        }

        /// The filled indicator: the view that scales along its height.
        fn indicator(&self) -> Option<NodeId> {
            self.kinds
                .iter()
                .filter(|(id, kind)| **kind == ViewKind::View && self.live(**id))
                .map(|(id, _)| *id)
                .find(|id| {
                    self.style(*id)
                        .transform
                        .ops()
                        .iter()
                        .any(|op| op.kind == TransformKind::ScaleY)
                })
        }

        /// The animation node `view`'s `prop` is bound to.
        fn bound(&self, view: NodeId, prop: AnimationProp) -> animation::NodeId {
            self.bindings
                .get(&view)
                .and_then(|bindings| {
                    bindings
                        .iter()
                        .find(|(bound, _)| *bound == prop)
                        .map(|(_, node)| *node)
                })
                .unwrap_or_else(|| panic!("{view} binds no {prop:?}"))
        }

        /// Whether `node` is computed from `on`, or is it.
        fn depends(&self, node: animation::NodeId, on: animation::NodeId) -> bool {
            if node == on {
                return true;
            }
            match self.ops.get(&node) {
                Some(AnimationOp::CreateMath { a, b, .. }) => [a, b]
                    .into_iter()
                    .any(|operand| matches!(operand, Operand::Node(n) if self.depends(*n, on))),
                Some(AnimationOp::CreateInterpolate { input, .. }) => self.depends(*input, on),
                _ => false,
            }
        }

        /// The animated value that is the thickness of the indicator or the outline.
        fn thickness_node(&self) -> animation::NodeId {
            if let Some(outline) = self.outline() {
                let drawing = &self.drawings[&outline];
                let slot = drawing
                    .ops
                    .iter()
                    .find_map(|op| match op {
                        CanvasOp::LineWidth(Num::Slot(slot)) => Some(*slot),
                        _ => None,
                    })
                    .expect("a line width that moves");
                drawing.slots[usize::from(slot)]
            } else {
                // The indicator's scale is the thickness over the thickest it gets.
                let indicator = self.indicator().expect("an indicator");
                let scale = self.bound(indicator, AnimationProp::Transform(0));
                match self.ops.get(&scale) {
                    Some(AnimationOp::CreateMath {
                        a: Operand::Node(thickness),
                        ..
                    }) => *thickness,
                    other => panic!("not a thickness over a constant: {other:?}"),
                }
            }
        }

        /// The label's text view.
        fn label(&self) -> NodeId {
            self.text("Email").expect("the label")
        }

        /// The node the label's size is bound to, its horizontal and its vertical translation: all of them follow
        /// how far the label has floated.
        fn label_nodes(&self) -> (animation::NodeId, animation::NodeId, animation::NodeId) {
            let label = self.label();
            (
                self.bound(label, AnimationProp::Transform(2)),
                self.bound(label, AnimationProp::Transform(0)),
                self.bound(label, AnimationProp::Transform(1)),
            )
        }
    }

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// NSH's color of `hex` (`0xRRGGBB`) at alpha `alpha` (0 to 255).
    fn color(hex: u32, alpha: u32) -> Color {
        Color((hex << 8) | alpha)
    }

    fn scheme_color(role: ColorRole, dark: bool) -> u32 {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        scheme.get(role).to_hex()
    }

    /// The 8-bit alphas NSH rounds the token opacities to: 0.38 of 255 and 0.12 of 255.
    const ALPHA_38: u32 = 97;
    const ALPHA_12: u32 = 31;

    /// The color `ops` fill with, or that they stroke with, if any.
    fn paint(ops: &[CanvasOp], stroke: bool) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) if !stroke => Some(*color),
            CanvasOp::StrokeStyle(canvas::Paint::Color(color)) if stroke => Some(*color),
            _ => None,
        })
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        assert_eq!(TextFieldStyle::default(), TextFieldStyle::Filled);
        assert_eq!(TextFieldStatus::default(), TextFieldStatus::Enabled);
        assert_eq!(TextFieldLines::default(), TextFieldLines::Single);
    }

    #[test]
    fn a_status_stands_for_the_androidx_flags_it_combines() {
        // (status, `isError`, not editable, `enabled = false`)
        let table = [
            (TextFieldStatus::Enabled, false, false, false),
            (TextFieldStatus::ReadOnly, false, true, false),
            (TextFieldStatus::Error, true, false, false),
            (TextFieldStatus::ReadOnlyError, true, true, false),
            (TextFieldStatus::Disabled, false, true, true),
        ];
        for (status, error, read_only, disabled) in table {
            assert_eq!(status.is_error(), error, "{status:?}");
            assert_eq!(status.is_read_only(), read_only, "{status:?}");
            assert_eq!(status.is_disabled(), disabled, "{status:?}");
        }
    }

    #[test]
    fn what_a_field_looks_like_follows_the_order_of_androidx_text_field_colors() {
        use TextFieldStatus::{Disabled, Enabled, Error, ReadOnly, ReadOnlyError};
        // `!enabled`, then `isError`, then `focused`, then the rest: the focus changes nothing in the first two.
        let table = [
            (Enabled, false, Look::Resting),
            (Enabled, true, Look::Focused),
            (ReadOnly, false, Look::Resting),
            (ReadOnly, true, Look::Focused),
            (Error, false, Look::Error),
            (Error, true, Look::Error),
            (ReadOnlyError, false, Look::Error),
            (ReadOnlyError, true, Look::Error),
            (Disabled, false, Look::Disabled),
            (Disabled, true, Look::Disabled),
        ];
        for (status, focused, look) in table {
            assert_eq!(Look::of(status, focused), look, "{status:?} {focused}");
        }
    }

    #[test]
    fn the_spaces_around_the_text_are_the_ones_androidx_gives_it() {
        // Half what the 48-point box leaves around the 24-point glyph.
        assert_eq!(icon_padding(), Points(12.0));
        // `TextFieldPadding`, less what an icon's box already holds.
        assert_eq!(side_padding(false), Points(16.0));
        assert_eq!(side_padding(true), Points(4.0));
        // The icon's box and that padding, on the label's side.
        assert_eq!(label_inset(false), Points(16.0));
        assert_eq!(label_inset(true), Points(52.0));
        // The label's line centered in 56 points, 24 points high.
        assert_eq!(label_top().to_bits(), 16.0_f32.to_bits());
        assert_eq!(label_scale().to_bits(), 0.75_f32.to_bits());
        assert_eq!(label_overhang(), Points(8.0));
        assert_eq!(MAX_THICKNESS, Points(2.0));
        assert_eq!(OUTLINED_RADIUS, Points(4.0));
    }

    #[test]
    fn both_styles_name_the_same_type_and_icon_tokens() {
        assert_eq!(filled::INPUT_FONT, outlined::INPUT_FONT);
        assert_eq!(filled::LABEL_FONT, outlined::LABEL_FONT);
        assert_eq!(filled::SUPPORTING_FONT, outlined::SUPPORTING_FONT);
        for size in [
            filled::TRAILING_ICON_SIZE,
            outlined::LEADING_ICON_SIZE,
            outlined::TRAILING_ICON_SIZE,
        ] {
            assert_eq!(size, ICON_SIZE);
        }
        assert_eq!(ICON_SIZE, Points(24.0));
        assert_eq!(filled::FOCUS_ACTIVE_INDICATOR_HEIGHT, MAX_THICKNESS);
        assert_eq!(outlined::FOCUS_OUTLINE_WIDTH, MAX_THICKNESS);
        assert_eq!(outlined::CONTAINER_HEIGHT, MIN_HEIGHT);
    }

    #[test]
    fn the_label_floats_to_where_androidx_puts_it() {
        let table = [
            (TextFieldStyle::Filled, false, -12.0, 0.0),
            (TextFieldStyle::Filled, true, -12.0, 0.0),
            (TextFieldStyle::Outlined, false, -28.0, 0.0),
            (TextFieldStyle::Outlined, true, -28.0, -36.0),
        ];
        for (style, icon, rise, slide) in table {
            assert_eq!(
                label_geometry(style, icon),
                LabelGeometry { rise, slide },
                "{style:?} {icon}"
            );
        }
    }

    #[test]
    fn the_indicator_is_thick_with_the_focus_and_thin_when_disabled_whatever_else() {
        use TextFieldStatus::{Disabled, Enabled, Error, ReadOnly, ReadOnlyError};
        let table = [
            (Enabled, Phase::UnfocusedEmpty, 1.0),
            (Enabled, Phase::UnfocusedNotEmpty, 1.0),
            (Enabled, Phase::Focused, 2.0),
            (ReadOnly, Phase::Focused, 2.0),
            // An error does not change it.
            (Error, Phase::UnfocusedEmpty, 1.0),
            (Error, Phase::Focused, 2.0),
            (ReadOnlyError, Phase::Focused, 2.0),
            (Disabled, Phase::UnfocusedEmpty, 1.0),
            (Disabled, Phase::Focused, 1.0),
        ];
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            for (status, phase, points) in table {
                assert_eq!(
                    thickness(style, status, phase),
                    Points(points),
                    "{style:?} {status:?} {phase:?}"
                );
            }
        }
    }

    #[test]
    fn the_input_is_one_line_high_or_as_high_as_its_lines() {
        for content in [0.0, 10.0, 24.0, 100.0] {
            assert_eq!(
                input_height(TextFieldLines::Single, Points(content)),
                Points(24.0)
            );
        }
        let lines = [(0.0, 24.0), (10.0, 24.0), (24.0, 24.0), (100.0, 100.0)];
        for (content, height) in lines {
            assert_eq!(
                input_height(TextFieldLines::Multiple, Points(content)),
                Points(height)
            );
        }
    }

    #[test]
    fn what_each_phase_shows_is_what_androidx_shows() {
        // (phase, label floats, placeholder opacity, prefix and suffix opacity)
        let table = [
            (Phase::Focused, 1.0, 1.0, 1.0),
            (Phase::UnfocusedEmpty, 0.0, 0.0, 0.0),
            (Phase::UnfocusedNotEmpty, 1.0, 0.0, 1.0),
        ];
        for (phase, label, placeholder, affix) in table {
            assert_eq!(
                label_target(phase).to_bits(),
                f32::to_bits(label),
                "{phase:?}"
            );
            assert_eq!(
                placeholder_target(phase).to_bits(),
                f32::to_bits(placeholder),
                "{phase:?}"
            );
            assert_eq!(
                affix_target(phase).to_bits(),
                f32::to_bits(affix),
                "{phase:?}"
            );
        }
    }

    #[test]
    fn the_placeholder_comes_in_slowly_and_goes_quickly() {
        let phases = [
            Phase::Focused,
            Phase::UnfocusedEmpty,
            Phase::UnfocusedNotEmpty,
        ];
        for from in phases {
            for to in phases {
                let slow = matches!(
                    (from, to),
                    (Phase::UnfocusedEmpty, Phase::Focused)
                        | (Phase::UnfocusedNotEmpty, Phase::UnfocusedEmpty)
                );
                let expected = if slow {
                    SpringToken::SlowEffects
                } else {
                    SpringToken::FastEffects
                };
                assert_eq!(placeholder_spring(from, to), expected, "{from:?} to {to:?}");
            }
        }
    }

    #[test]
    fn a_spring_is_a_token_converted_with_nothing_left_to_nsh_defaults() {
        for token in [
            SpringToken::FastSpatial,
            SpringToken::FastEffects,
            SpringToken::SlowEffects,
        ] {
            let spec = token.spec();
            let spring = spring_of(token);
            assert_eq!(spring.stiffness.to_bits(), spec.stiffness.to_bits());
            assert_eq!(
                spring.damping.to_bits(),
                spec.damping_coefficient().to_bits()
            );
            assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
            assert_eq!(spring.velocity, None);
            assert_eq!(spring.frame_rate, None);
        }
    }

    #[test]
    fn the_reduced_motion_fade_is_the_standard_short_fade() {
        assert_eq!(
            REDUCED_MOTION_FADE.duration_ms.to_bits(),
            motion_tokens::DURATION_SHORT2.to_bits()
        );
        let curve = motion_tokens::EASING_STANDARD_CUBIC_BEZIER;
        assert_eq!(
            REDUCED_MOTION_FADE.easing,
            Easing::Bezier(curve.x1, curve.y1, curve.x2, curve.y2)
        );
        assert_eq!(REDUCED_MOTION_FADE.delay_ms.to_bits(), 0.0_f32.to_bits());
        assert_eq!(REDUCED_MOTION_FADE.frame_rate, None);
    }

    /// What a text field paints, each in the color it should have.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Painted {
        label: Color,
        leading_icon: Color,
        trailing_icon: Color,
        supporting: Color,
        input: Color,
        caret: Option<Color>,
        placeholder: Color,
        prefix: Color,
        suffix: Color,
        /// The indicator of a filled field, the outline of an outlined one.
        line: Color,
        container: Option<Color>,
    }

    fn painted(mounted: &Mounted) -> Painted {
        let config = &mounted.inputs[&mounted.input()];
        let icons = mounted.icons();
        let [leading, trailing] = icons[..] else {
            panic!("two icons, not {icons:?}");
        };
        let fill = |id: NodeId| paint(&mounted.drawings[&id].ops, false).expect("an icon's fill");
        let (line, container) = match (mounted.outline(), mounted.indicator()) {
            (Some(outline), None) => (
                paint(&mounted.drawings[&outline].ops, true).expect("a stroke color"),
                None,
            ),
            (None, Some(indicator)) => (
                mounted.style(indicator).background_color,
                mounted
                    .container()
                    .map(|id| mounted.style(id).background_color),
            ),
            other => panic!("an outline or an indicator: {other:?}"),
        };
        Painted {
            label: mounted.text_color("Email"),
            leading_icon: fill(leading),
            trailing_icon: fill(trailing),
            supporting: mounted.text_color("Help"),
            input: config.style.color,
            caret: config.caret_color,
            placeholder: mounted.text_color("name@example.com"),
            prefix: mounted.text_color("$"),
            suffix: mounted.text_color("kg"),
            line,
            container,
        }
    }

    /// How a text field looks, in the words of the tokens and not of the code.
    #[derive(Debug, Clone, Copy)]
    enum Seen {
        Resting,
        Focused,
        Error,
        Disabled,
    }

    /// The colors of `FilledTextFieldTokens` and `OutlinedTextFieldTokens` that `defaultTextFieldColors` and
    /// `defaultOutlinedTextFieldColors` read, by look.
    fn expected(style: TextFieldStyle, seen: Seen, error: bool, dark: bool) -> Painted {
        use ColorRole as R;
        let solid = |role| color(scheme_color(role, dark), 255);
        let faded = |role| color(scheme_color(role, dark), ALPHA_38);
        let by = |resting: Color, focused: Color, error: Color, disabled: Color| match seen {
            Seen::Resting => resting,
            Seen::Focused => focused,
            Seen::Error => error,
            Seen::Disabled => disabled,
        };
        let variant = solid(R::OnSurfaceVariant);
        let caret = if error { R::Error } else { R::Primary };
        Painted {
            label: by(
                variant,
                solid(R::Primary),
                solid(R::Error),
                faded(R::OnSurface),
            ),
            leading_icon: by(variant, variant, variant, faded(R::OnSurface)),
            trailing_icon: by(variant, variant, solid(R::Error), faded(R::OnSurface)),
            supporting: by(variant, variant, solid(R::Error), faded(R::OnSurface)),
            input: by(
                solid(R::OnSurface),
                solid(R::OnSurface),
                solid(R::OnSurface),
                faded(R::OnSurface),
            ),
            caret: Some(solid(caret)),
            placeholder: by(variant, variant, variant, faded(R::OnSurface)),
            prefix: by(variant, variant, variant, faded(R::OnSurfaceVariant)),
            suffix: by(variant, variant, variant, faded(R::OnSurfaceVariant)),
            line: by(
                match style {
                    TextFieldStyle::Filled => variant,
                    TextFieldStyle::Outlined => solid(R::Outline),
                },
                solid(R::Primary),
                solid(R::Error),
                match style {
                    // `FilledTextFieldTokens.DisabledActiveIndicatorOpacity` and `OutlinedTextFieldTokens
                    // .DisabledOutlineOpacity`.
                    TextFieldStyle::Filled => faded(R::OnSurface),
                    TextFieldStyle::Outlined => color(scheme_color(R::OnSurface, dark), ALPHA_12),
                },
            ),
            // The filled container is the container color in every look, the disabled one included.
            container: match style {
                TextFieldStyle::Filled => Some(solid(R::SurfaceContainerHighest)),
                TextFieldStyle::Outlined => None,
            },
        }
    }

    #[test]
    fn each_look_has_its_colors_in_both_styles_light_and_dark() {
        use TextFieldStatus::{Disabled, Enabled, Error, ReadOnly, ReadOnlyError};
        // (status, focused, how it looks, whether the caret is the error's)
        let looks = [
            (Enabled, false, Seen::Resting, false),
            (Enabled, true, Seen::Focused, false),
            (ReadOnly, false, Seen::Resting, false),
            (Error, false, Seen::Error, true),
            (Error, true, Seen::Error, true),
            (ReadOnlyError, false, Seen::Error, true),
            (Disabled, false, Seen::Disabled, false),
        ];
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            for (status, focused, seen, error) in looks {
                for dark in [false, true] {
                    let what = format!("{style:?} {status:?} focused: {focused} dark: {dark}");
                    let mounted = mount(Config {
                        status,
                        focused,
                        dark,
                        ..Config::full(style)
                    });
                    assert_eq!(
                        painted(&mounted),
                        expected(style, seen, error, dark),
                        "{what}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_filled_container_has_its_corners_and_the_outlined_one_has_none() {
        let filled = mount(Config::default());
        let container = filled.container().expect("a container");
        let corners = filled.style(container).corner_radius;
        assert_eq!(
            (
                corners.top_left,
                corners.top_right,
                corners.bottom_right,
                corners.bottom_left
            ),
            (
                Some(Points(4.0)),
                Some(Points(4.0)),
                Some(Points(0.0)),
                Some(Points(0.0))
            )
        );
        let outlined = mount(Config {
            style: TextFieldStyle::Outlined,
            ..Config::default()
        });
        assert_eq!(outlined.container(), None);
        assert_eq!(outlined.indicator(), None);
        assert!(outlined.outline().is_some());
        assert_eq!(filled.outline(), None);
    }

    /// A text's `Rect` and the part of it the text itself takes, inside its paddings.
    fn content(mounted: &Mounted, id: NodeId) -> Rect {
        let content = mounted.contents[&id];
        rect(
            content.origin.x.0,
            content.origin.y.0,
            content.size.width.0,
            content.size.height.0,
        )
    }

    #[test]
    fn a_filled_field_is_56_points_high_with_the_text_where_androidx_puts_it() {
        let mounted = mount(Config {
            parts: &[Part::Supporting],
            ..Config::default()
        });
        assert_eq!(mounted.rect(mounted.field()), rect(0.0, 0.0, 390.0, 56.0));
        let container = mounted.container().expect("a container");
        assert_eq!(mounted.rect(container), rect(0.0, 0.0, 390.0, 56.0));
        // The indicator is as thick as it gets and scales down from its bottom edge.
        let indicator = mounted.indicator().expect("an indicator");
        assert_eq!(mounted.rect(indicator), rect(0.0, 54.0, 390.0, 2.0));
        assert_eq!(
            Some(mounted.style(indicator).transform_origin),
            nsh_core::TransformOrigin::parse("50% 100%")
        );
        // 8 points, the label's 16, then the line; 8 points below it.
        assert_eq!(mounted.rect(mounted.input()), rect(16.0, 24.0, 358.0, 24.0));
        let label = mounted.rect(mounted.label());
        assert_eq!((label.x, label.y), (Points(16.0), Points(16.0)));
        // The supporting text, under the field, 16 points in from each side and 4 below it.
        let help = mounted.text("Help").expect("the supporting text");
        let line = mounted.rect(help);
        assert_eq!(
            (line.x, line.y, line.width),
            (Points(0.0), Points(56.0), Points(390.0))
        );
        let inner = content(&mounted, help);
        assert_eq!(
            (inner.x, inner.y, inner.width),
            (Points(16.0), Points(4.0), Points(358.0))
        );
    }

    #[test]
    fn an_outlined_field_has_the_label_stick_out_above_its_outline() {
        let mounted = mount(Config {
            style: TextFieldStyle::Outlined,
            parts: &[Part::Supporting],
            ..Config::default()
        });
        // Half the floating label's line, which is centered on the outline's top edge.
        assert_eq!(mounted.rect(mounted.field()), rect(0.0, 8.0, 390.0, 56.0));
        let outline = mounted.outline().expect("an outline");
        assert_eq!(mounted.rect(outline), rect(0.0, 8.0, 390.0, 56.0));
        assert_eq!(mounted.rect(mounted.input()), rect(16.0, 24.0, 358.0, 24.0));
        let label = mounted.rect(mounted.label());
        assert_eq!((label.x, label.y), (Points(16.0), Points(24.0)));
        let help = mounted.text("Help").expect("the supporting text");
        let line = mounted.rect(help);
        assert_eq!(
            (line.x, line.y, line.width),
            (Points(0.0), Points(64.0), Points(390.0))
        );
    }

    #[test]
    fn icons_sit_in_48_point_boxes_and_the_text_moves_over() {
        for (style, top) in [
            (TextFieldStyle::Filled, 0.0),
            (TextFieldStyle::Outlined, 8.0),
        ] {
            let what = format!("{style:?}");
            let table = [
                // (leading, trailing, the input's left, the input's width)
                (true, true, 52.0, 286.0),
                (true, false, 52.0, 322.0),
                (false, true, 16.0, 322.0),
                (false, false, 16.0, 358.0),
            ];
            for (leading, trailing, left, width) in table {
                let what = format!("{what} {leading} {trailing}");
                let mounted = mount(Config {
                    style,
                    parts: icons(leading, trailing),
                    ..Config::default()
                });
                assert_eq!(
                    mounted.rect(mounted.input()),
                    rect(left, 24.0, width, 24.0),
                    "{what}"
                );
                let label = mounted.rect(mounted.label());
                assert_eq!(
                    (label.x, label.y),
                    (Points(left), Points(top + 16.0)),
                    "{what}"
                );
                let mut at = Vec::new();
                for icon in mounted.icons() {
                    let glyph = mounted.rect(icon);
                    let boxed = mounted.rect(mounted.parents[&icon]);
                    at.push((glyph, boxed));
                }
                let mut wanted = Vec::new();
                if leading {
                    wanted.push((
                        rect(12.0, top + 16.0, 24.0, 24.0),
                        rect(0.0, top + 4.0, 48.0, 48.0),
                    ));
                }
                if trailing {
                    wanted.push((
                        rect(354.0, top + 16.0, 24.0, 24.0),
                        rect(342.0, top + 4.0, 48.0, 48.0),
                    ));
                }
                assert_eq!(at, wanted, "{what}");
            }
        }
    }

    #[test]
    fn affixes_sit_either_side_of_the_input_with_two_points_between() {
        let mounted = mount(Config {
            parts: &[Part::Affixes],
            ..Config::default()
        });
        let (prefix, suffix) = (
            mounted.text("$").expect("a prefix"),
            mounted.text("kg").expect("a suffix"),
        );
        let (before, input, after) = (
            mounted.rect(prefix),
            mounted.rect(mounted.input()),
            mounted.rect(suffix),
        );
        // The prefix starts the line, the input follows it, the suffix follows the input and ends the line.
        assert_eq!((before.x, before.y), (Points(16.0), Points(24.0)));
        assert_eq!(input.x.0, before.x.0 + before.width.0);
        assert_eq!(after.x.0, input.x.0 + input.width.0);
        assert_eq!(after.x.0 + after.width.0, 374.0);
        assert_eq!((input.y, after.y), (Points(24.0), Points(24.0)));
        // 2 points of the prefix's box are space after its text, 2 of the suffix's before its text.
        let (inner_before, inner_after) = (content(&mounted, prefix), content(&mounted, suffix));
        assert_eq!(
            (inner_before.x.0, before.width.0 - inner_before.width.0),
            (0.0, 2.0)
        );
        assert_eq!(
            (inner_after.x.0, after.width.0 - inner_after.width.0),
            (2.0, 2.0)
        );
    }

    #[test]
    fn a_field_is_at_least_280_points_wide_and_no_more_than_its_place_allows() {
        for (given, wanted) in [(200.0, 280.0), (280.0, 280.0), (300.0, 300.0)] {
            for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
                let mounted = mount(Config {
                    style,
                    width: Some(given),
                    ..Config::default()
                });
                assert_eq!(
                    mounted.rect(mounted.field()).width,
                    Points(wanted),
                    "{style:?} {given}"
                );
            }
        }
    }

    #[test]
    fn a_multi_line_field_grows_with_its_text_and_a_single_line_one_does_not() {
        let (mut app, recording) = start(Config {
            lines: TextFieldLines::Multiple,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        assert!(mounted.inputs[&mounted.input()].multiline);
        assert_eq!(mounted.rect(mounted.field()).height, Points(56.0));
        app.type_text("label:Email", "a\nb\nc").unwrap();
        let mounted = Mounted::of(&recording.items());
        let input = mounted.rect(mounted.input());
        // Three lines are more than the one of a single-line field, and the field is as high as the text, with the
        // label's room above it and 8 points below.
        assert!(input.height.0 > 2.0 * 24.0, "{input:?}");
        assert_eq!(
            mounted.rect(mounted.field()).height,
            Points(24.0 + input.height.0 + 8.0)
        );

        let (mut app, recording) = start(Config::default());
        app.type_text("label:Email", &"long ".repeat(40)).unwrap();
        let mounted = Mounted::of(&recording.items());
        assert!(!mounted.inputs[&mounted.input()].multiline);
        assert_eq!(mounted.rect(mounted.input()).height, Points(24.0));
        assert_eq!(mounted.rect(mounted.field()).height, Points(56.0));
    }

    #[test]
    fn screen_readers_name_the_input_by_its_label_and_read_the_supporting_text_as_its_hint() {
        let mounted = mount(Config {
            parts: &[Part::Supporting],
            ..Config::default()
        });
        let input = mounted.input();
        let accessibility = mounted.props(input).accessibility;
        assert_eq!(accessibility.label.as_deref(), Some("Email"));
        assert_eq!(accessibility.hint.as_deref(), Some("Help"));
        assert!(!accessibility.state.disabled);
        let bare = mount(Config::default());
        assert_eq!(
            bare.props(bare.input()).accessibility.hint.as_deref(),
            Some("")
        );

        // The words drawn over and beside the input are not read a second time; the container is not an element.
        let full = mount(Config::full(TextFieldStyle::Filled));
        for hidden in ["Email", "name@example.com", "$", "kg"] {
            let id = full.text(hidden).expect(hidden);
            assert!(full.props(id).accessibility.hidden, "{hidden}");
        }
        // The label and the placeholder, drawn over the input, take no press: the input does.
        let label = full.label();
        assert_eq!(
            full.props(full.parents[&label]).pointer_events,
            PointerEvents::None
        );
        let placeholder = full.text("name@example.com").unwrap();
        assert_eq!(full.props(placeholder).pointer_events, PointerEvents::None);
        assert_eq!(
            full.props(full.field()).accessibility.accessible,
            Some(false)
        );
        // The supporting text is read as text; an error is an alert.
        let help = full.text("Help").expect("the supporting text");
        assert!(!full.props(help).accessibility.hidden);
        assert_eq!(full.props(help).accessibility.role, AccessibilityRole::Text);
        for status in [TextFieldStatus::Error, TextFieldStatus::ReadOnlyError] {
            let errored = mount(Config {
                status,
                ..Config::full(TextFieldStyle::Filled)
            });
            let help = errored.text("Help").expect("the supporting text");
            assert_eq!(
                errored.props(help).accessibility.role,
                AccessibilityRole::Alert,
                "{status:?}"
            );
        }
        let disabled = mount(Config {
            status: TextFieldStatus::Disabled,
            ..Config::full(TextFieldStyle::Filled)
        });
        let help = disabled.text("Help").expect("the supporting text");
        assert_eq!(
            disabled.props(help).accessibility.role,
            AccessibilityRole::Text
        );
    }

    #[test]
    fn the_input_is_editable_read_only_or_disabled_as_the_status_says() {
        // (status, editable, disabled for screen readers)
        let table = [
            (TextFieldStatus::Enabled, true, false),
            (TextFieldStatus::Error, true, false),
            (TextFieldStatus::ReadOnly, false, false),
            (TextFieldStatus::ReadOnlyError, false, false),
            (TextFieldStatus::Disabled, false, true),
        ];
        for (status, editable, disabled) in table {
            let (mut app, recording) = start(Config {
                status,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let input = mounted.input();
            assert_eq!(mounted.inputs[&input].editable, editable, "{status:?}");
            assert_eq!(
                mounted.props(input).accessibility.state.disabled,
                disabled,
                "{status:?}"
            );
            // Nothing can be typed in what is not editable.
            assert_eq!(
                app.type_text("label:Email", "x").is_ok(),
                editable,
                "{status:?}"
            );
        }
    }

    #[test]
    fn the_text_is_the_apps_and_the_events_say_what_happened() {
        let (mut app, _recording) = start(Config::default());
        app.type_text("label:Email", "ab").unwrap();
        assert!(app.shows("log: focus;change:a;change:ab;"));
        assert!(app.shows("value: ab"));
        assert_eq!(app.screen().input_text("label:Email"), Ok(Some("ab")));
        app.submit("label:Email").unwrap();
        assert!(app.shows("log: focus;change:a;change:ab;submit:ab;blur;"));
        // The app sets another text: the input shows it, and nothing is reported.
        app.press("Clear").unwrap();
        assert!(!app.shows("value: ab"));
        assert_eq!(app.screen().input_text("label:Email"), Ok(Some("")));
        assert!(app.shows("log: focus;change:a;change:ab;submit:ab;blur;"));
    }

    #[test]
    fn a_press_anywhere_in_the_container_focuses_the_input() {
        let (mut app, recording) = start(Config {
            parts: &[Part::Affixes],
            ..Config::default()
        });
        let input = Mounted::of(&recording.items()).input();
        app.press("$").unwrap();
        let items = recording.in_step("press($)");
        assert!(
            items.iter().any(|item| matches!(
                item,
                MountItem::Command { id, command: nsh_core::Command::Focus } if *id == input
            )),
            "{items:?}"
        );
    }

    #[test]
    fn the_placeholder_shows_only_while_the_text_is_empty() {
        for (value, hidden) in [("", false), ("x", true)] {
            let mounted = mount(Config {
                value,
                parts: &[Part::Placeholder],
                ..Config::default()
            });
            let placeholder = mounted.text("name@example.com").expect("a placeholder");
            assert_eq!(mounted.style(placeholder).hidden, hidden, "{value:?}");
            // It is a text of its own, over the input, so that its opacity can follow the springs.
            assert_eq!(
                mounted.inputs[&mounted.input()].placeholder,
                "",
                "{value:?}"
            );
            assert_eq!(
                mounted.rect(placeholder),
                mounted.rect(mounted.input()),
                "{value:?}"
            );
        }
        let (mut app, recording) = start(Config {
            parts: &[Part::Placeholder],
            ..Config::default()
        });
        app.type_text("label:Email", "a").unwrap();
        let mounted = Mounted::of(&recording.items());
        assert!(
            mounted
                .style(mounted.text("name@example.com").unwrap())
                .hidden
        );
        app.press("Clear").unwrap();
        let mounted = Mounted::of(&recording.items());
        assert!(
            !mounted
                .style(mounted.text("name@example.com").unwrap())
                .hidden
        );
    }

    /// The `Set` operations of `items`: the value node and what it is set to.
    fn sets(items: &[MountItem]) -> Vec<(animation::NodeId, f32)> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::Animation(AnimationOp::Set { node, value }) => Some((*node, *value)),
                _ => None,
            })
            .collect()
    }

    /// The timing animations `items` start: the value node, where it goes, and how.
    fn timings(items: &[MountItem]) -> Vec<(animation::NodeId, f32, f32, Easing, f32)> {
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
                            frame_rate: None,
                        },
                    ..
                }) => Some((*node, *to, *duration_ms, *easing, *delay_ms)),
                _ => None,
            })
            .collect()
    }

    /// How many of `items` lay a view out again.
    fn frames(items: &[MountItem]) -> usize {
        items
            .iter()
            .filter(|item| matches!(item, MountItem::Frame { .. }))
            .count()
    }

    #[test]
    fn mounting_starts_nothing_and_sets_only_the_width_of_the_label() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            for status in [
                TextFieldStatus::Enabled,
                TextFieldStatus::Error,
                TextFieldStatus::Disabled,
            ] {
                for (value, leading) in [("", false), ("x", true)] {
                    let what = format!("{style:?} {status:?} {value:?} {leading}");
                    let (_app, recording) = start(Config {
                        status,
                        value,
                        parts: if leading {
                            EVERY_PART
                        } else {
                            &EVERY_PART[1..]
                        },
                        ..Config::full(style)
                    });
                    let items = recording.items();
                    assert_eq!(started_springs(&items), Vec::new(), "{what}");
                    assert_eq!(timings(&items), Vec::new(), "{what}");
                    // The label's width, when the label is laid out; nothing else is set.
                    let set = sets(&items);
                    let mounted = Mounted::of(&items);
                    let width = mounted.rect(mounted.label()).width.0;
                    assert_eq!(set.len(), 1, "{what}: {set:?}");
                    assert_eq!(set[0].1.to_bits(), width.to_bits(), "{what}: {set:?}");
                    assert!(!replay(&recording).is_animating(), "{what}");
                }
            }
        }
    }

    /// The spring of `items` that moves the label.
    fn label_spring(mounted: &Mounted, items: &[MountItem]) -> StartedSpring {
        let (scale, ..) = mounted.label_nodes();
        started_springs(items)
            .into_iter()
            .find(|spring| mounted.depends(scale, spring.node))
            .expect("the label's spring")
    }

    /// What the label's bindings give with the graph as it is: its scale, and its translation along x and along y.
    fn label_now(mounted: &Mounted, replay: &crate::animation_probe::Replay) -> (f32, f32, f32) {
        let (scale, x, y) = mounted.label_nodes();
        (replay.value(scale), replay.value(x), replay.value(y))
    }

    #[test]
    fn the_label_rests_over_the_text_and_floats_where_androidx_floats_it() {
        // (style, a leading icon, where the floating label's left edge is, where its middle is)
        let table = [
            (TextFieldStyle::Filled, false, 16.0, 16.0),
            (TextFieldStyle::Filled, true, 52.0, 16.0),
            (TextFieldStyle::Outlined, false, 16.0, 8.0),
            (TextFieldStyle::Outlined, true, 16.0, 8.0),
        ];
        for (style, leading, left, middle) in table {
            let what = format!("{style:?} {leading}");
            let (mut app, recording) = start(Config {
                style,
                parts: icons(leading, false),
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let mut graph = replay(&recording);
            assert_eq!(label_now(&mounted, &graph), (1.0, 0.0, 0.0), "{what}");
            // At rest the label's line is centered in the 56 points of the field.
            let rest = mounted.rect(mounted.label());
            let field = mounted.rect(mounted.field());
            assert_eq!(
                rest.y.0 + LABEL_FONT.line_height.0 / 2.0,
                field.y.0 + 28.0,
                "{what}"
            );

            focus(&mut app);
            graph.apply(&recording.in_step(FOCUS));
            let node = label_spring(&mounted, &recording.in_step(FOCUS)).node;
            let _ = graph.run(node, 600);
            let (scale, x, y) = label_now(&mounted, &graph);
            assert_eq!(scale.to_bits(), 0.75_f32.to_bits(), "{what}");
            // The label scales about its left edge and its middle: where they end up is where its box was, moved.
            assert_eq!(rest.x.0 + x, left, "{what}");
            assert_eq!(
                rest.y.0 + LABEL_FONT.line_height.0 / 2.0 + y,
                middle,
                "{what}"
            );
        }
    }

    #[test]
    fn a_field_with_a_value_mounts_with_its_label_floating() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            let mounted_and_graph = |value| {
                let (_app, recording) = start(Config {
                    style,
                    value,
                    ..Config::full(style)
                });
                (Mounted::of(&recording.items()), replay(&recording))
            };
            let (mounted, graph) = mounted_and_graph("x");
            let geometry = label_geometry(style, true);
            assert_eq!(
                label_now(&mounted, &graph),
                (0.75, geometry.slide, geometry.rise),
                "{style:?}"
            );
            let (mounted, graph) = mounted_and_graph("");
            assert_eq!(label_now(&mounted, &graph), (1.0, 0.0, 0.0), "{style:?}");
        }
    }

    /// The springs of one step, by what each moves.
    #[derive(Debug, Default)]
    struct Moves {
        label: Option<StartedSpring>,
        placeholder: Option<StartedSpring>,
        affix: Option<StartedSpring>,
        thickness: Option<StartedSpring>,
    }

    fn moves(mounted: &Mounted, items: &[MountItem]) -> Moves {
        let (scale, ..) = mounted.label_nodes();
        let placeholder = mounted.bound(
            mounted.text("name@example.com").expect("a placeholder"),
            AnimationProp::Opacity,
        );
        let affix = mounted.bound(mounted.text("$").expect("a prefix"), AnimationProp::Opacity);
        let thickness = mounted.thickness_node();
        let mut moves = Moves::default();
        for spring in started_springs(items) {
            let slot = if mounted.depends(scale, spring.node) {
                &mut moves.label
            } else if mounted.depends(placeholder, spring.node) {
                &mut moves.placeholder
            } else if mounted.depends(affix, spring.node) {
                &mut moves.affix
            } else if mounted.depends(thickness, spring.node) {
                &mut moves.thickness
            } else {
                panic!("a spring that moves none of them: {spring:?}");
            };
            assert!(slot.is_none(), "two springs on one value: {spring:?}");
            *slot = Some(spring);
        }
        moves
    }

    /// `spring` is `token`'s spring, converted as ADR 0004 says, to `to`: from where the value is, as fast as it
    /// moves, at the screen's own rate.
    fn assert_spring(spring: Option<StartedSpring>, token: SpringToken, to: f32, what: &str) {
        let spring = spring.unwrap_or_else(|| panic!("{what}: no spring"));
        let spec = token.spec();
        assert_eq!(spring.to.to_bits(), to.to_bits(), "{what}: {spring:?}");
        assert_eq!(
            spring.stiffness.to_bits(),
            spec.stiffness.to_bits(),
            "{what}"
        );
        assert_eq!(
            spring.damping.to_bits(),
            spec.damping_coefficient().to_bits(),
            "{what}"
        );
        assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits(), "{what}");
        assert_eq!(spring.velocity, None, "{what}");
        assert_eq!(spring.frame_rate, None, "{what}");
    }

    /// Nothing but springs came in `items`: no value set, no timing, no view laid out again.
    fn assert_only_springs(items: &[MountItem], what: &str) {
        assert_eq!(sets(items), Vec::new(), "{what}");
        assert_eq!(timings(items), Vec::new(), "{what}");
        assert_eq!(frames(items), 0, "{what}");
    }

    #[test]
    fn the_focus_springs_the_label_the_placeholder_the_affixes_and_the_indicator() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            let what = format!("{style:?}");
            let (mut app, recording) = start(Config::full(style));
            let mounted = Mounted::of(&recording.items());
            focus(&mut app);
            let items = recording.in_step(FOCUS);
            let m = moves(&mounted, &items);
            assert_spring(m.label, SpringToken::FastSpatial, 1.0, &what);
            assert_spring(m.placeholder, SpringToken::SlowEffects, 1.0, &what);
            assert_spring(m.affix, SpringToken::FastEffects, 1.0, &what);
            assert_spring(m.thickness, SpringToken::FastSpatial, 2.0, &what);
            assert_eq!(started_springs(&items).len(), 4, "{what}");
            assert_only_springs(&items, &what);
            // The same prefix and suffix opacity moves both.
            let suffix = mounted.bound(mounted.text("kg").unwrap(), AnimationProp::Opacity);
            let prefix = mounted.bound(mounted.text("$").unwrap(), AnimationProp::Opacity);
            assert_eq!(suffix, prefix, "{what}");

            // The indicator or the outline ends 2 points thick, the label floating, the placeholder shown.
            let mut graph = replay(&recording);
            let _ = graph.run(m.label.unwrap().node, 600);
            assert!(!graph.is_animating(), "{what}");
            assert_eq!(
                graph.value(mounted.thickness_node()).to_bits(),
                2.0_f32.to_bits()
            );
            assert_eq!(graph.value(prefix).to_bits(), 1.0_f32.to_bits(), "{what}");
            let placeholder = mounted.text("name@example.com").unwrap();
            let shown = mounted.bound(placeholder, AnimationProp::Opacity);
            assert_eq!(graph.value(shown).to_bits(), 1.0_f32.to_bits(), "{what}");
        }
    }

    #[test]
    fn leaving_the_field_springs_everything_back_from_where_it_is() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            let what = format!("{style:?}");
            let (mut app, recording) = start(Config {
                focused: true,
                ..Config::full(style)
            });
            let mounted = Mounted::of(&recording.items());
            app.submit("label:Email").unwrap();
            let items = recording.in_step("submit(label:Email)");
            let m = moves(&mounted, &items);
            assert_spring(m.label, SpringToken::FastSpatial, 0.0, &what);
            // An empty field's placeholder goes out quickly.
            assert_spring(m.placeholder, SpringToken::FastEffects, 0.0, &what);
            assert_spring(m.affix, SpringToken::FastEffects, 0.0, &what);
            assert_spring(m.thickness, SpringToken::FastSpatial, 1.0, &what);
            assert_eq!(started_springs(&items).len(), 4, "{what}");
            assert_only_springs(&items, &what);
        }
    }

    #[test]
    fn with_a_value_only_what_the_focus_changes_moves() {
        let (mut app, recording) = start(Config {
            value: "x",
            ..Config::full(TextFieldStyle::Filled)
        });
        let mounted = Mounted::of(&recording.items());
        focus(&mut app);
        let items = recording.in_step(FOCUS);
        let m = moves(&mounted, &items);
        // The label and the affixes are where the value keeps them; the placeholder (under the text, hidden) fades
        // in with the fast spring: it comes from a field that has text.
        assert!(m.label.is_none() && m.affix.is_none(), "{m:?}");
        assert_spring(m.placeholder, SpringToken::FastEffects, 1.0, "placeholder");
        assert_spring(m.thickness, SpringToken::FastSpatial, 2.0, "thickness");
        assert_eq!(started_springs(&items).len(), 2);

        app.submit("label:Email").unwrap();
        let items = recording.in_step("submit(label:Email)");
        let m = moves(&mounted, &items);
        assert!(m.label.is_none() && m.affix.is_none(), "{m:?}");
        assert_spring(m.placeholder, SpringToken::FastEffects, 0.0, "placeholder");
        assert_spring(m.thickness, SpringToken::FastSpatial, 1.0, "thickness");
    }

    #[test]
    fn emptying_an_unfocused_field_brings_the_label_down() {
        let (mut app, recording) = start(Config {
            value: "x",
            ..Config::full(TextFieldStyle::Filled)
        });
        let mounted = Mounted::of(&recording.items());
        app.press("Clear").unwrap();
        let items = recording.in_step("press(Clear)");
        let m = moves(&mounted, &items);
        assert_spring(m.label, SpringToken::FastSpatial, 0.0, "label");
        assert_spring(m.affix, SpringToken::FastEffects, 0.0, "affix");
        // The placeholder stays out: it was out, and still is.
        assert!(m.placeholder.is_none() && m.thickness.is_none(), "{m:?}");
        assert_only_springs(&items, "clear");
    }

    #[test]
    fn typing_in_a_focused_field_moves_nothing() {
        let (mut app, recording) = start(Config {
            focused: true,
            ..Config::full(TextFieldStyle::Outlined)
        });
        app.type_text("label:Email", "abc").unwrap();
        let items = recording.in_step("type(label:Email, \"abc\")");
        assert_eq!(started_springs(&items), Vec::new());
        assert_only_springs(&items, "typing");
    }

    #[test]
    fn a_status_changes_the_colors_and_moves_nothing() {
        for reduce_motion in [false, true] {
            let (mut app, recording) = start(Config {
                reduce_motion,
                ..Config::full(TextFieldStyle::Filled)
            });
            for step in ["Set error", "Set read-only", "Set enabled"] {
                app.press(step).unwrap();
                let items = recording.in_step(&format!("press({step})"));
                assert_eq!(
                    started_springs(&items),
                    Vec::new(),
                    "{step} {reduce_motion}"
                );
                assert_only_springs(&items, step);
            }
        }
    }

    #[test]
    fn the_curve_of_the_label_is_the_fast_spatial_spring_of_the_token() {
        let (mut app, recording) = start(Config::full(TextFieldStyle::Filled));
        let mounted = Mounted::of(&recording.items());
        focus(&mut app);
        let spring = moves(&mounted, &recording.in_step(FOCUS)).label.unwrap();
        let mut graph = replay(&recording);
        let samples = graph.run(spring.node, 180);
        // The closed-form spring from rest at 0, within 0.1 % of the travel (ADR 0010).
        let gap = worst_gap(&samples, 0.0, &spring);
        assert!(gap < tolerance(0.0, 1.0), "{gap}");
        // Underdamped: it goes past 1 and settles on it exactly.
        assert!(samples.iter().any(|v| *v > 1.001), "{samples:?}");
        let settled = graph.run(spring.node, 600);
        assert!(!graph.is_animating());
        assert_eq!(settled.last().map(|v| v.to_bits()), Some(1.0_f32.to_bits()));
    }

    #[test]
    fn leaving_while_the_label_still_goes_up_continues_from_its_value_and_speed() {
        let (mut app, recording) = start(Config::full(TextFieldStyle::Filled));
        let mounted = Mounted::of(&recording.items());
        focus(&mut app);
        let up = moves(&mounted, &recording.in_step(FOCUS)).label.unwrap();
        let mut graph = replay(&recording);
        // 50 ms in, the label is still rising.
        let before = graph.run(up.node, 6);
        app.submit("label:Email").unwrap();
        let items = recording.in_step("submit(label:Email)");
        let down = moves(&mounted, &items).label.unwrap();
        assert_eq!(down.node, up.node);
        graph.apply(&items);
        let after = graph.run(down.node, 180);

        // The graph last ticked 5 frames into the rise: the way back starts at that value and that speed.
        let rise = parameters(&up);
        let at = 5.0 / REPLAY_HZ;
        let position = |t: f64| spring_position(0.0, 1.0, 0.0, rise, t);
        let from = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed > 1.0, "{speed}");
        let gap = |velocity: f64| {
            after
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                    let closed = spring_position(from, 0.0, velocity, parameters(&down), t);
                    (f64::from(value) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        // 0.1 % of the label's whole travel, 0 to 1 (ADR 0010): the graph's own speed is a finite difference.
        let tolerance = tolerance(0.0, 1.0);
        assert!(gap(speed) < tolerance, "{}", gap(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(gap(0.0) > 10.0 * tolerance, "{}", gap(0.0));
    }

    #[test]
    fn nothing_is_sent_while_the_springs_play() {
        let (mut app, recording) = start(Config::full(TextFieldStyle::Outlined));
        focus(&mut app);
        app.step("run");
        app.run_for(Duration::from_secs(1)).unwrap();
        assert!(recording.in_step("run").is_empty());
    }

    #[test]
    fn with_reduce_motion_nothing_springs_and_what_appears_fades_in() {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::full(TextFieldStyle::Filled)
        });
        let mounted = Mounted::of(&recording.items());
        let label = mounted.label();
        let fade = mounted.bound(label, AnimationProp::Opacity);
        let (scale, ..) = mounted.label_nodes();
        let placeholder = mounted.bound(
            mounted.text("name@example.com").unwrap(),
            AnimationProp::Opacity,
        );
        let affix = mounted.bound(mounted.text("$").unwrap(), AnimationProp::Opacity);
        let thickness = mounted.thickness_node();

        focus(&mut app);
        let items = recording.in_step(FOCUS);
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(frames(&items), 0);
        // The label arrives at once, the indicator too, and the label comes in again from nothing.
        let set = sets(&items);
        assert_eq!(set.len(), 3, "{set:?}");
        let value_of =
            |node: animation::NodeId| set.iter().find(|(n, _)| *n == node).map(|(_, v)| *v);
        assert_eq!(value_of(thickness), Some(2.0));
        assert_eq!(value_of(fade), Some(0.0));
        let progress = set
            .iter()
            .find(|(node, value)| {
                *node != fade && *node != thickness && value.to_bits() == 1.0_f32.to_bits()
            })
            .expect("the label's progress");
        assert!(mounted.depends(scale, progress.0));
        // The label, the placeholder and the affixes fade to where they go, the short standard fade.
        let timed = timings(&items);
        assert_eq!(timed.len(), 3, "{timed:?}");
        for node in [fade, placeholder, affix] {
            let found = timed.iter().find(|(n, ..)| *n == node);
            let Some((_, to, duration, easing, delay)) = found else {
                panic!("{node} does not fade: {timed:?}");
            };
            assert_eq!(to.to_bits(), 1.0_f32.to_bits());
            assert_eq!(
                duration.to_bits(),
                REDUCED_MOTION_FADE.duration_ms.to_bits()
            );
            assert_eq!(*easing, REDUCED_MOTION_FADE.easing);
            assert_eq!(delay.to_bits(), 0.0_f32.to_bits());
        }

        // Replayed: the label is up at the first frame, and its fade takes the 100 ms of the token.
        let mut graph = replay(&recording);
        let fading = graph.run(fade, 20);
        assert_eq!(fading[0].to_bits(), 0.0_f32.to_bits());
        assert!(fading[1] > 0.0 && fading[1] < 0.5, "{fading:?}");
        assert!(fading[6] > 0.5 && fading[6] < 1.0, "{fading:?}");
        assert_eq!(fading[19].to_bits(), 1.0_f32.to_bits());
        assert_eq!(graph.value(scale).to_bits(), 0.75_f32.to_bits());
        assert_eq!(graph.value(thickness).to_bits(), 2.0_f32.to_bits());
        assert!(!graph.is_animating());

        // Typing does not fade again, and leaving fades what leaves.
        app.type_text("label:Email", "a").unwrap();
        assert_eq!(
            timings(&recording.in_step("type(label:Email, \"a\")")),
            Vec::new()
        );
        app.press("Clear").unwrap();
        app.submit("label:Email").unwrap();
        let items = recording.in_step("submit(label:Email)");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(sets(&items).len(), 3, "{:?}", sets(&items));
        assert_eq!(timings(&items).len(), 3);
    }

    #[test]
    fn a_disabled_field_takes_the_disabled_look_at_once_and_keeps_no_focus() {
        let (mut app, recording) = start(Config {
            focused: true,
            ..Config::full(TextFieldStyle::Filled)
        });
        let mounted = Mounted::of(&recording.items());
        let thickness = mounted.thickness_node();
        app.press("Set disabled").unwrap();
        let items = recording.in_step("press(Set disabled)");
        // The indicator is 1 point thick at once (androidx does not animate it for a disabled field); the label
        // comes down, as the field of no focus does.
        assert_eq!(sets(&items), vec![(thickness, 1.0)]);
        let m = moves(&mounted, &items);
        assert!(m.thickness.is_none(), "{m:?}");
        assert_spring(m.label, SpringToken::FastSpatial, 0.0, "label");
        assert_spring(m.placeholder, SpringToken::FastEffects, 0.0, "placeholder");
        assert_spring(m.affix, SpringToken::FastEffects, 0.0, "affix");
        assert_eq!(frames(&items), 0);
        // And it is the disabled look.
        let after = Mounted::of(&recording.items());
        assert_eq!(
            after.text_color("Email"),
            expected(TextFieldStyle::Filled, Seen::Disabled, false, false).label
        );
    }

    #[test]
    fn a_new_style_or_icon_puts_the_label_where_it_floats_in_that_style() {
        // What the app's change sets, but for the label's width, which the layout reports again when the label moves.
        let geometry = |recording: &Recording, since: usize| -> Vec<f32> {
            let mounted = Mounted::of(&recording.items());
            let width = mounted.rect(mounted.label()).width.0;
            sets(&recording.items()[since..])
                .into_iter()
                .map(|(_, value)| value)
                .filter(|value| value.to_bits() != width.to_bits())
                .collect()
        };
        let (mut app, recording) = start(Config::default());
        let press = |app: &mut Headless, what: &str| {
            let since = recording.items().len();
            app.press(what).unwrap();
            geometry(&recording, since)
        };
        // Outlined: 28 points up, none across.
        assert_eq!(press(&mut app, "Toggle style"), [-28.0, 0.0]);
        assert_eq!(press(&mut app, "Toggle icon"), [-28.0, -36.0]);
        assert_eq!(press(&mut app, "Toggle style"), [-12.0, 0.0]);
        // The same geometry sets nothing again: the filled label is where it was, with or without an icon.
        assert_eq!(press(&mut app, "Toggle icon"), Vec::<f32>::new());
        assert_eq!(press(&mut app, "Toggle icon"), Vec::<f32>::new());

        // Focused, the outlined label with an icon floats over the icon, to the padding.
        let (mut app, recording) = start(Config {
            style: TextFieldStyle::Outlined,
            ..Config::default()
        });
        app.press("Toggle icon").unwrap();
        let mounted = Mounted::of(&recording.items());
        focus(&mut app);
        let mut graph = replay(&recording);
        let node = label_spring(&mounted, &recording.in_step(FOCUS)).node;
        let _ = graph.run(node, 600);
        let (.., x, y) = {
            let (scale, x, y) = mounted.label_nodes();
            (scale, graph.value(x), graph.value(y))
        };
        let label = Mounted::of(&recording.items()).rect(mounted.label());
        assert_eq!((label.x.0 + x, y), (16.0, -28.0));
    }

    /// The outline's path, with the graph where it is.
    fn outline(mounted: &Mounted, graph: &crate::animation_probe::Replay) -> Vec<Step> {
        let drawing = &mounted.drawings[&mounted.outline().expect("an outline")];
        let mut paths = paths(drawing, graph);
        assert_eq!(paths.len(), 1);
        paths.remove(0)
    }

    /// The path of an outline of `size` that `thickness` thick, open from `left` to `right` along its top: androidx's
    /// outline with a notch in it, as a stroke inset by half its width.
    fn notched(size: Size<Points>, thickness: f32, left: f32, right: f32) -> Vec<Step> {
        use std::f32::consts::{FRAC_PI_2, PI};
        let (w, h) = (size.width.0, size.height.0);
        let (half, radius) = (thickness / 2.0, 4.0 - thickness / 2.0);
        let corner = |x, y, start: f32| Step::Arc {
            x,
            y,
            radius,
            start,
            end: start + FRAC_PI_2,
            ccw: false,
        };
        vec![
            Step::Move(right, half),
            Step::Line(w - 4.0, half),
            corner(w - 4.0, 4.0, -FRAC_PI_2),
            Step::Line(w - half, h - 4.0),
            corner(w - 4.0, h - 4.0, 0.0),
            Step::Line(4.0, h - half),
            corner(4.0, h - 4.0, FRAC_PI_2),
            Step::Line(half, 4.0),
            corner(4.0, 4.0, PI),
            Step::Line(left, half),
        ]
    }

    #[test]
    fn the_outline_has_a_notch_that_opens_with_the_label_and_thickens_with_the_focus() {
        let (mut app, recording) = start(Config {
            style: TextFieldStyle::Outlined,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let size = Size::new(Points(390.0), Points(56.0));
        let label = mounted.contents[&mounted.label()].size.width.0;
        let mut graph = replay(&recording);
        // At rest: 1 point thick, the label down, no notch (it begins and ends 12 points in).
        assert_traces(
            &outline(&mounted, &graph),
            &notched(size, 1.0, 12.0, 12.0),
            "rest",
        );

        focus(&mut app);
        graph.apply(&recording.in_step(FOCUS));
        let nodes = moves_of(&recording, FOCUS);
        // On its way: the notch is as wide as the label is now, which is the label's width at its size now times how
        // far it has floated, with 4 points on each side once it has begun to float.
        let _ = graph.run(nodes.0, 5);
        let progress = graph.value(nodes.0);
        let thick = graph.value(nodes.1);
        assert!(progress > 0.1 && progress < 0.9, "{progress}");
        assert!(thick > 1.0 && thick < 2.0, "{thick}");
        let notch = (1.0 - 0.25 * progress) * progress * label + 8.0;
        assert_traces(
            &outline(&mounted, &graph),
            &notched(size, thick, 12.0, 12.0 + notch),
            "half way",
        );
        // Floating: the label at three quarters of its size and 4 points either side; the outline 2 points thick.
        let _ = graph.run(nodes.0, 600);
        assert_traces(
            &outline(&mounted, &graph),
            &notched(size, 2.0, 12.0, 12.0 + 0.75 * label + 8.0),
            "focused",
        );
    }

    /// The label's and the thickness's value nodes, from the springs of `step`.
    fn moves_of(recording: &Recording, step: &str) -> (animation::NodeId, animation::NodeId) {
        let springs = started_springs(&recording.in_step(step));
        let mounted = Mounted::of(&recording.items());
        let (scale, ..) = mounted.label_nodes();
        let thickness = mounted.thickness_node();
        let label = springs
            .iter()
            .find(|spring| mounted.depends(scale, spring.node))
            .expect("the label's spring");
        let line = springs
            .iter()
            .find(|spring| mounted.depends(thickness, spring.node))
            .expect("the outline's spring");
        (label.node, line.node)
    }

    #[test]
    fn the_notch_goes_no_further_than_the_straight_part_of_the_edge() {
        assert_eq!(notch_reach(Points(390.0)).to_bits(), 386.0_f32.to_bits());
        assert_eq!(notch_reach(Points(280.0)).to_bits(), 276.0_f32.to_bits());
        assert_eq!(notch_left().to_bits(), 12.0_f32.to_bits());
    }

    #[test]
    fn the_indicator_scales_from_its_bottom_edge_with_its_thickness() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let indicator = mounted.indicator().expect("an indicator");
        let scale = mounted.bound(indicator, AnimationProp::Transform(0));
        let mut graph = replay(&recording);
        // 1 point of the 2 it is as thick as it gets.
        assert_eq!(graph.value(scale).to_bits(), 0.5_f32.to_bits());
        focus(&mut app);
        graph.apply(&recording.in_step(FOCUS));
        let _ = graph.run(mounted.thickness_node(), 600);
        assert_eq!(graph.value(scale).to_bits(), 1.0_f32.to_bits());
    }

    /// The vendored androidx files this component follows, at the pinned commit (ADR 0012).
    const TEXT_FIELD_KT: &str =
        include_str!("../../../reference/androidx-material3-components/TextField.kt");
    const OUTLINED_TEXT_FIELD_KT: &str =
        include_str!("../../../reference/androidx-material3-components/OutlinedTextField.kt");
    const TEXT_FIELD_DEFAULTS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/TextFieldDefaults.kt");
    const TEXT_FIELD_IMPL_KT: &str =
        include_str!("../../../reference/androidx-material3-components/TextFieldImpl.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_androidx_is_still_there() {
        // The minimum size, for both styles.
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &[
                "* The default min height applied to a [TextField]. Note that you can override it by applying",
                "* Modifier.heightIn directly on a text field.",
                "*/",
                "public val MinHeight: Dp = 56.dp",
                "",
                "/**",
                "* The default min width applied to a [TextField]. Note that you can override it by applying",
                "* Modifier.widthIn directly on a text field.",
                "*/",
                "public val MinWidth: Dp = 280.dp",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &[
                "* The default min height applied to an [OutlinedTextField]. Note that you can override it by",
                "* applying Modifier.heightIn directly on a text field.",
                "*/",
                "public val MinHeight: Dp = 56.dp",
                "",
                "/**",
                "* The default min width applied to an [OutlinedTextField]. Note that you can override it by",
                "* applying Modifier.widthIn directly on a text field.",
                "*/",
                "public val MinWidth: Dp = 280.dp",
            ]
        ));
        assert_eq!((MIN_HEIGHT, MIN_WIDTH), (Points(56.0), Points(280.0)));
        // The spaces.
        for (name, value) in [
            ("TextFieldPadding", "16.dp"),
            ("SupportingTopPadding", "4.dp"),
            ("PrefixSuffixTextPadding", "2.dp"),
            ("MinTextLineHeight", "24.dp"),
            ("MinSupportingTextLineHeight", "16.dp"),
        ] {
            assert!(
                cites(
                    TEXT_FIELD_IMPL_KT,
                    &[&format!("internal val {name}"), &format!("get() = {value}")]
                ),
                "{name}"
            );
        }
        assert_eq!(
            (
                TEXT_FIELD_PADDING,
                SUPPORTING_TOP_PADDING,
                PREFIX_SUFFIX_PADDING,
                MIN_TEXT_LINE_HEIGHT,
                MIN_SUPPORTING_LINE_HEIGHT
            ),
            (
                Points(16.0),
                Points(4.0),
                Points(2.0),
                Points(24.0),
                Points(16.0)
            )
        );
        assert!(cites(
            TEXT_FIELD_KT,
            &[
                "internal val TextFieldWithLabelVerticalPadding",
                "get() = 8.dp"
            ]
        ));
        assert_eq!(FILLED_VERTICAL_PADDING, Points(8.0));
        assert!(cites(
            OUTLINED_TEXT_FIELD_KT,
            &["private val OutlinedTextFieldInnerPadding", "get() = 4.dp"]
        ));
        assert_eq!(NOTCH_PADDING, Points(4.0));
        // The padding of an icon: what the 48-point touch target leaves around the 24-point glyph.
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "return ((interactiveSize - SmallIconButtonTokens.IconSize) / 2).coerceAtLeast(0.dp)"
            ]
        ));
        assert_eq!(small_icon_button_tokens::ICON_SIZE, Points(24.0));
        assert_eq!(MIN_TOUCH_TARGET, Points(48.0));
        // A line that is not there is not found.
        assert!(!cites(
            TEXT_FIELD_IMPL_KT,
            &["internal val TextFieldPadding", "get() = 17.dp"]
        ));
    }

    #[test]
    fn every_choice_of_color_taken_from_androidx_is_still_there() {
        // Disabled, then error, then focused, then the rest, for every color a text field has.
        for (function, name) in [
            ("leadingIconColor", "LeadingIconColor"),
            ("trailingIconColor", "TrailingIconColor"),
            ("indicatorColor", "IndicatorColor"),
            ("placeholderColor", "PlaceholderColor"),
            ("labelColor", "LabelColor"),
            ("textColor", "TextColor"),
            ("supportingTextColor", "SupportingTextColor"),
            ("prefixColor", "PrefixColor"),
            ("suffixColor", "SuffixColor"),
        ] {
            let lines = [
                format!(
                    "public fun {function}(enabled: Boolean, isError: Boolean, focused: Boolean): Color ="
                ),
                "when {".to_owned(),
                format!("!enabled -> disabled{name}"),
                format!("isError -> error{name}"),
                format!("focused -> focused{name}"),
                format!("else -> unfocused{name}"),
            ];
            let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
            assert!(cites(TEXT_FIELD_DEFAULTS_KT, &lines), "{function}");
        }
        // The caret is the error's in error, whatever else.
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &[
                "public fun cursorColor(isError: Boolean): Color = if (isError) errorCursorColor else cursorColor"
            ]
        ));
        for tokens in ["FilledTextFieldTokens", "OutlinedTextFieldTokens"] {
            assert!(cites(
                TEXT_FIELD_DEFAULTS_KT,
                &[&format!(
                    "errorCursorColor = fromToken({tokens}.ErrorFocusCaretColor),"
                )]
            ));
            // A disabled placeholder is the input's color, a disabled prefix and suffix their own, all at the input's
            // disabled opacity.
            for (what, source) in [
                ("Placeholder", "DisabledInputColor"),
                ("Prefix", "InputPrefixColor"),
                ("Suffix", "InputSuffixColor"),
            ] {
                assert!(cites(
                    TEXT_FIELD_DEFAULTS_KT,
                    &[
                        &format!("disabled{what}Color ="),
                        &format!("fromToken({tokens}.{source})"),
                        &format!(".copy(alpha = {tokens}.DisabledInputOpacity),"),
                    ]
                ));
            }
        }
        // The filled container is the container color in every state; the disabled container token is not read.
        for state in ["focused", "unfocused", "disabled", "error"] {
            assert!(cites(
                TEXT_FIELD_DEFAULTS_KT,
                &[&format!(
                    "{state}ContainerColor = fromToken(FilledTextFieldTokens.ContainerColor),"
                )]
            ));
        }
        assert!(!TEXT_FIELD_DEFAULTS_KT.contains("FilledTextFieldTokens.DisabledContainerColor"));
        assert!(!TEXT_FIELD_DEFAULTS_KT.contains("OutlinedTextFieldTokens.DisabledContainerColor"));
        // The outlined field has no container.
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &["disabledContainerColor = Color.Transparent,"]
        ));
        // Hover is in the tokens and in no color of the text field.
        assert!(!TEXT_FIELD_DEFAULTS_KT.contains("hovered"));
        assert!(!TEXT_FIELD_DEFAULTS_KT.contains("Hovered"));
    }

    #[test]
    fn every_motion_taken_from_androidx_is_still_there() {
        // The thickness of the indicator and of the outline: 1 point, 2 when focused, 1 when disabled; the fast
        // spatial spring, not for a disabled field.
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &[
                "/** The default thickness of the indicator line in [TextField] in unfocused state. */",
                "public val UnfocusedIndicatorThickness: Dp = 1.dp",
                "",
                "/** The default thickness of the indicator line in [TextField] in focused state. */",
                "public val FocusedIndicatorThickness: Dp = 2.dp",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_DEFAULTS_KT,
            &[
                "/** The default thickness of the border in [OutlinedTextField] in unfocused state. */",
                "public val UnfocusedBorderThickness: Dp = 1.dp",
                "",
                "/** The default thickness of the border in [OutlinedTextField] in focused state. */",
                "public val FocusedBorderThickness: Dp = 2.dp",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "val thicknessAnimationSpec = MotionSchemeKeyTokens.FastSpatial.value<Dp>()",
                "val thickness =",
                "if (enabled) {",
                "val targetThickness = if (focused) focusedBorderThickness else unfocusedBorderThickness",
                "animateDpAsState(targetThickness, thicknessAnimationSpec)",
                "} else {",
                "rememberUpdatedState(unfocusedBorderThickness)",
                "}",
            ]
        ));
        // The label, the placeholder and the prefix and suffix.
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "val labelTransitionSpec = MotionSchemeKeyTokens.FastSpatial.value<Float>()",
                "return animateFloat(label = \"LabelProgress\", transitionSpec = { labelTransitionSpec }) {",
                "when (it) {",
                "InputPhase.Focused -> 1f",
                "InputPhase.UnfocusedEmpty -> if (showExpandedLabel) 0f else 1f",
                "InputPhase.UnfocusedNotEmpty -> 1f",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "val fastOpacityTransitionSpec = MotionSchemeKeyTokens.FastEffects.value<Float>()",
                "val slowOpacityTransitionSpec = MotionSchemeKeyTokens.SlowEffects.value<Float>()",
                "return animateFloat(",
                "label = \"PlaceholderOpacity\",",
                "transitionSpec = {",
                "if (InputPhase.Focused isTransitioningTo InputPhase.UnfocusedEmpty) {",
                "fastOpacityTransitionSpec",
                "} else if (",
                "InputPhase.UnfocusedEmpty isTransitioningTo InputPhase.Focused ||",
                "InputPhase.UnfocusedNotEmpty isTransitioningTo InputPhase.UnfocusedEmpty",
                ") {",
                "slowOpacityTransitionSpec",
                "} else {",
                "fastOpacityTransitionSpec",
                "}",
                "},",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "InputPhase.Focused -> 1f",
                "InputPhase.UnfocusedEmpty -> if (showExpandedLabel) 0f else 1f",
                "InputPhase.UnfocusedNotEmpty -> 0f",
            ]
        ));
        assert!(cites(
            TEXT_FIELD_IMPL_KT,
            &[
                "label = \"PrefixSuffixOpacity\",",
                "transitionSpec = { fastOpacityTransitionSpec },",
            ]
        ));
    }
}
