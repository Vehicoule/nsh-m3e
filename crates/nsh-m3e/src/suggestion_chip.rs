//! The Material 3 Expressive suggestion chip: a label in a flat, outlined container, with an optional icon before it,
//! and the state layer of hover, focus and press.
//!
//! The suggestion chip follows androidx's `SuggestionChip(onClick, label, …)` in `Chip.kt` at the pinned commit, as
//! [ADR 0012] says; every line number below is a line of `reference/androidx-material3-components/Chip.kt`. It reads
//! the `SuggestionChipTokens` the way `SuggestionChipDefaults` does: `defaultSuggestionChipColors` (lines 3971 to
//! 3989), `suggestionChipBorder` (3011 to 3019), the `Height` and `IconSize` (2909 and 2912). Its container is the
//! token's `ContainerShape` and does not change: `SuggestionChip` has no `ChipShapes`, so it does not morph. Its
//! container is still drawn by a canvas, as every chip's is, so that the outline and the state layer share its
//! corners ([ADR 0011]).
//!
//! `SuggestionChip` has one icon, before the label, and no trailing one (`trailingIcon = null`, line 1703): its colors
//! name no trailing icon color either (3978).
//!
//! # What is left out
//!
//! - The elevated suggestion chip (`ElevatedSuggestionChip`, line 1761): elevation is decided by ADR 0018 and written
//!   by its own change; this chip has no shadow code.
//! - A focus indicator: `Chip.kt` reads none (`SuggestionChipTokens.FocusIndicatorColor` has no reader).
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::chip::{CONTENT_PADDING, Chip, Leading, Look, Metrics, SPACING, Spaces, touch_slop};
use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::tint::Tint;
use crate::tokens::androidx::suggestion_chip_tokens;

/// What the suggestion chip shows disabled or not, from `defaultSuggestionChipColors` (lines 3971 to 3989) and
/// `suggestionChipBorder` (3011 to 3019): a transparent container, an outline in `FlatOutlineColor` that fades to
/// `FlatDisabledOutlineColor` at its opacity, and the label and the icon faded by their disabled opacities.
fn look(disabled: bool) -> Look {
    if disabled {
        let icon = Tint::faded(
            suggestion_chip_tokens::DISABLED_LEADING_ICON_COLOR,
            suggestion_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
        );
        Look {
            container: None,
            label: Tint::faded(
                suggestion_chip_tokens::DISABLED_LABEL_TEXT_COLOR,
                suggestion_chip_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            leading: icon,
            trailing: icon,
            outline: Some((
                suggestion_chip_tokens::FLAT_OUTLINE_WIDTH,
                Tint::faded(
                    suggestion_chip_tokens::FLAT_DISABLED_OUTLINE_COLOR,
                    suggestion_chip_tokens::FLAT_DISABLED_OUTLINE_OPACITY,
                ),
            )),
        }
    } else {
        let icon = Tint::solid(suggestion_chip_tokens::LEADING_ICON_COLOR);
        Look {
            container: None,
            label: Tint::solid(suggestion_chip_tokens::LABEL_TEXT_COLOR),
            leading: icon,
            trailing: icon,
            outline: Some((
                suggestion_chip_tokens::FLAT_OUTLINE_WIDTH,
                Tint::solid(suggestion_chip_tokens::FLAT_OUTLINE_COLOR),
            )),
        }
    }
}

/// The space around the label: `SuggestionChipDefaults.ContentPadding` and `horizontalArrangement()` (lines 2915 and
/// 2921), which are the same with the icon or without it.
fn spaces(has_leading: bool) -> Spaces {
    Spaces::new(
        (CONTENT_PADDING, CONTENT_PADDING),
        (SPACING, SPACING),
        has_leading,
    )
}

/// The suggestion chip's sizes: `SuggestionChipDefaults.Height` and `IconSize`, and the label style `Chip` gives it
/// (`SuggestionChipTokens.LabelTextFont`, line 1700).
fn metrics() -> Metrics {
    Metrics {
        height: suggestion_chip_tokens::CONTAINER_HEIGHT,
        leading_icon: suggestion_chip_tokens::LEADING_ICON_SIZE,
        // A suggestion chip has no trailing icon.
        trailing_icon: Points(0.0),
        // A suggestion chip has no avatar.
        avatar: Points(0.0),
        label: suggestion_chip_tokens::LABEL_TEXT_FONT,
    }
}

/// The props of a [`SuggestionChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SuggestionChipProps {
    /// What the chip says: screen readers read it as the chip's name.
    pub label: Prop<SharedString>,
    /// An icon before the label.
    pub icon: Prop<Option<Icon>>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`SuggestionChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SuggestionChipEvent {
    /// The suggestion chip was pressed.
    Press,
}

/// A Material 3 Expressive suggestion chip: a label that offers a suggestion (a reply, a search), in a flat container
/// with an outline, with an optional icon before it.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, the button role, named by its
/// label for screen readers) and the look of androidx's flat suggestion chip. Hover, focus and press lay the label's
/// color over it at the state layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so
/// nothing is laid out again. It is 32 points tall, with a 48-point touch target around it.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::suggestion_chip::suggestion_chip;
///
/// fn reply() -> impl IntoElement {
///     suggestion_chip()
///         .label("Sounds good")
///         .on_press(|| println!("replied"))
/// }
/// ```
#[component]
pub fn SuggestionChip(
    props: SuggestionChipProps,
    cx: Scope<SuggestionChipEvent>,
) -> impl IntoElement {
    let metrics = metrics();
    // Before its first layout, the container is taken to be as wide as it is tall; its layout corrects that.
    let drawn = DrawnShape::new(
        corners(suggestion_chip_tokens::CONTAINER_SHAPE),
        corners(suggestion_chip_tokens::CONTAINER_SHAPE),
        Size::new(metrics.height, metrics.height),
    );
    let has_leading = computed(move || props.icon.get().is_some());
    let chip = Chip {
        drawn,
        metrics,
        label: props.label,
        disabled: props.disabled,
        look: computed(move || look(props.disabled.get())),
        spaces: computed(move || spaces(has_leading.get())),
        leading: computed(move || props.icon.get().map(Leading::Icon)),
        trailing: Prop::new(None),
    };
    chip.assemble(
        nsh_base::button()
            .disabled(move || props.disabled.get())
            .on_press(move || cx.emit(SuggestionChipEvent::Press))
            .hit_slop(move || touch_slop(drawn.size())),
    )
}

#[cfg(test)]
mod tests {
    use nsh_core::Color;
    use nsh_core::geometry::Points;

    use super::*;
    use crate::animation_probe::started_springs;
    use crate::chip_probe::{
        ALPHA_12, ALPHA_38, CHIP_KT, Config, Kind, Painted, cites, drawn, drop_items, faded, mount,
        painted, press_down, solid, start,
    };
    use crate::color::ColorRole;
    use crate::morph::Corners;
    use crate::morph_probe::{assert_traces, replay, rounded_rectangle, set_values};

    /// What a suggestion chip paints, with its icon, in the baseline schemes: androidx `SuggestionChipTokens`. Flat
    /// and transparent; an outline variant outline 1 point wide; the label on surface variant (the assist chip's is on
    /// surface); the icon primary. Disabled, the outline is on surface at 12 % and the label and icon on surface at
    /// 38 %.
    fn expected(disabled: bool, dark: bool) -> Painted {
        let none = Color(0);
        if disabled {
            let dim = faded(ColorRole::OnSurface, ALPHA_38, dark);
            Painted {
                container: none,
                outline_width: 1.0,
                outline: faded(ColorRole::OnSurface, ALPHA_12, dark),
                layer: dim,
                label: dim,
                leading: dim,
                trailing: none,
            }
        } else {
            Painted {
                container: none,
                outline_width: 1.0,
                outline: solid(ColorRole::OutlineVariant, dark),
                layer: solid(ColorRole::OnSurfaceVariant, dark),
                label: solid(ColorRole::OnSurfaceVariant, dark),
                leading: solid(ColorRole::Primary, dark),
                trailing: none,
            }
        }
    }

    #[test]
    fn it_has_its_colors_enabled_and_disabled_light_and_dark() {
        for disabled in [false, true] {
            for dark in [false, true] {
                let mounted = mount(
                    Config::of(Kind::Suggestion)
                        .leading(true)
                        .disabled(disabled)
                        .dark(dark),
                );
                assert_eq!(
                    painted(&mounted),
                    expected(disabled, dark),
                    "disabled: {disabled} dark: {dark}"
                );
            }
        }
        assert_ne!(expected(false, false), expected(false, true));
    }

    #[test]
    fn it_has_one_icon_before_the_label_and_nothing_else() {
        // A suggestion chip has no props for a trailing icon, an avatar or a selected state (`trailingIcon = null`),
        // so its views and what screen readers are told show none of them.
        let mounted = mount(Config::of(Kind::Suggestion).leading(true));
        assert!(mounted.leading.is_some());
        assert!(mounted.trailing.is_none());
        assert!(mounted.avatar.is_none());
        assert!(!mounted.state.selected);
        assert_eq!(mounted.state.pressed, None);
        assert_eq!(painted(&mounted), expected(false, false));
    }

    #[test]
    fn it_never_changes_its_corners() {
        // `SuggestionChipTokens.ContainerShape`, `CornerSmall`: 8 points, the same at each corner so that it can be
        // drawn (`SuggestionChip` takes a `Shape`, not `ChipShapes`: it does not morph).
        assert_eq!(
            Corners::of(suggestion_chip_tokens::CONTAINER_SHAPE),
            Some(Corners::Radius(Points(8.0)))
        );
        for reduce_motion in [false, true] {
            let what = format!("reduce motion: {reduce_motion}");
            let (mut app, recording) = start(
                Config::of(Kind::Suggestion)
                    .reduce_motion(reduce_motion)
                    .leading(true),
            );
            let at_rest = |recording: &crate::animation_probe::Recording| {
                let (container, layer, frame) = drawn(recording);
                assert_traces(&container[0], &rounded_rectangle(frame, 0.0, 8.0), &what);
                assert_traces(&layer[0], &rounded_rectangle(frame, 1.0, 7.0), &what);
            };
            at_rest(&recording);
            let step = press_down(&mut app);
            let pressed = recording.in_step(&step);
            // A press down starts no spring, sets no value and draws nothing again: the state layer only shows.
            assert_eq!(started_springs(&pressed), Vec::new(), "{what}");
            assert_eq!(set_values(&pressed), 0, "{what}");
            assert!(
                !pressed
                    .iter()
                    .any(|item| matches!(item, nsh_core::MountItem::Canvas { .. })),
                "{what}: {pressed:?}"
            );
            at_rest(&recording);
            let released = drop_items(&mut app, &recording);
            assert_eq!(started_springs(&released), Vec::new(), "{what}");
            assert_eq!(set_values(&released), 0, "{what}");
            at_rest(&recording);
            assert!(app.shows("presses: 1"), "{what}");
            assert!(!replay(&recording).is_animating(), "{what}");
        }
    }

    /// The vendored `Chip.kt`'s lines that the suggestion chip is made of (ADR 0012).
    #[test]
    fn every_choice_taken_from_androidx_is_still_there() {
        // The colors: a transparent container, the label and icon from their tokens, no trailing icon color, and the
        // disabled ones faded by the tokens' opacities.
        assert!(cites(
            CHIP_KT,
            &[
                "containerColor = Color.Transparent,",
                "labelColor = fromToken(SuggestionChipTokens.LabelTextColor),",
                "leadingIconContentColor = fromToken(SuggestionChipTokens.LeadingIconColor),",
                "trailingIconContentColor = Color.Unspecified,",
                "disabledContainerColor = Color.Transparent,",
                "disabledLabelColor =",
                "fromToken(SuggestionChipTokens.DisabledLabelTextColor)",
                ".copy(alpha = SuggestionChipTokens.DisabledLabelTextOpacity),",
                "disabledLeadingIconContentColor =",
                "fromToken(SuggestionChipTokens.DisabledLeadingIconColor)",
                ".copy(alpha = SuggestionChipTokens.DisabledLeadingIconOpacity),",
                "disabledTrailingIconContentColor = Color.Unspecified,",
            ]
        ));
        // The border: the flat outline, faded when disabled, at the flat outline's width.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun suggestionChipBorder(",
                "enabled: Boolean,",
                "borderColor: Color = SuggestionChipTokens.FlatOutlineColor.value,",
                "disabledBorderColor: Color =",
                "SuggestionChipTokens.FlatDisabledOutlineColor.value.copy(",
                "alpha = SuggestionChipTokens.FlatDisabledOutlineOpacity",
                "),",
                "borderWidth: Dp = SuggestionChipTokens.FlatOutlineWidth,",
                "): BorderStroke = BorderStroke(borderWidth, if (enabled) borderColor else disabledBorderColor)",
            ]
        ));
        assert_eq!(suggestion_chip_tokens::FLAT_OUTLINE_WIDTH, Points(1.0));
        // The shape is the token's, and no trailing icon is given to the chip.
        assert!(cites(
            CHIP_KT,
            &[
                "public val shape: Shape",
                "@Composable get() = SuggestionChipTokens.ContainerShape.value",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "leadingIcon = icon,",
                "trailingIcon = null,",
                "shape = shape,"
            ]
        ));
        // The spaces are the file's: 8 points around and between, with the icon or without it.
        assert!(cites(
            CHIP_KT,
            &[
                "public val ContentPadding: PaddingValues = PaddingValues(horizontal = 8.dp)",
                "",
                "/** Default spacing between elements in a suggestion chip. */",
                "public val HorizontalSpacing: Dp = 8.dp",
            ]
        ));
        assert_eq!(CONTENT_PADDING, Points(8.0));
        assert_eq!(SPACING, Points(8.0));
        // A line that is not there is not found.
        assert!(!cites(
            CHIP_KT,
            &["labelColor = fromToken(SuggestionChipTokens.LeadingIconColor),"]
        ));
    }
}
