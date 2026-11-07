//! The Material 3 Expressive assist chip: a label in a flat, outlined container, with an optional icon before it and
//! after it, and the state layer of hover, focus and press.
//!
//! The assist chip follows androidx's `AssistChip(onClick, label, …)` in `Chip.kt` at the pinned commit, as [ADR 0012]
//! says; every line number below is a line of `reference/androidx-material3-components/Chip.kt`. It reads the
//! `AssistChipTokens` the way `AssistChipDefaults` does: `assistChipColors` (lines 2020 to 2040), `assistChipBorder`
//! (2080 to 2088), the `Height` and `IconSize` (1950 and 1953). Its container is the token's `ContainerShape` and does
//! not change: `AssistChip` has no `ChipShapes`, so it does not morph (only `FilterChip` and `InputChip` take shapes).
//! Its container is still drawn by a canvas, as every chip's is, so that the outline and the state layer share its
//! corners ([ADR 0011]).
//!
//! The icons are the caller's: androidx takes a composable for each, sized by the caller with `AssistChipDefaults.
//! IconSize`; here an [`Icon`] is drawn at that size in the icon color of the chip's state.
//!
//! # What is left out
//!
//! - The elevated assist chip (`ElevatedAssistChip`, line 377): elevation is decided by ADR 0018 and written by its own
//!   change; this chip has no shadow code.
//! - The icons' expand and fade (`AnimatingChipContent`): the assist chip does not have it either (`Chip`, line 3186,
//!   places its content with `ChipContent`), so nothing is missing.
//! - A focus indicator: `Chip.kt` reads none (`AssistChipTokens.FocusIndicatorColor` has no reader).
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::chip::{CONTENT_PADDING, Chip, Leading, Look, Metrics, SPACING, Spaces, touch_slop};
use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::tint::Tint;
use crate::tokens::androidx::assist_chip_tokens;

/// What the assist chip shows disabled or not, from `AssistChipDefaults.assistChipColors` (lines 2020 to 2040) and
/// `assistChipBorder` (2080 to 2088): a transparent container, an outline in `FlatOutlineColor` that fades to
/// `FlatDisabledOutlineColor` at its opacity, and the label and the icons faded by their disabled opacities.
fn look(disabled: bool) -> Look {
    if disabled {
        let icon = Tint::faded(
            assist_chip_tokens::DISABLED_ICON_COLOR,
            assist_chip_tokens::DISABLED_ICON_OPACITY,
        );
        Look {
            container: None,
            label: Tint::faded(
                assist_chip_tokens::DISABLED_LABEL_TEXT_COLOR,
                assist_chip_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            leading: icon,
            trailing: icon,
            outline: Some((
                assist_chip_tokens::FLAT_OUTLINE_WIDTH,
                Tint::faded(
                    assist_chip_tokens::FLAT_DISABLED_OUTLINE_COLOR,
                    assist_chip_tokens::FLAT_DISABLED_OUTLINE_OPACITY,
                ),
            )),
        }
    } else {
        let icon = Tint::solid(assist_chip_tokens::ICON_COLOR);
        Look {
            container: None,
            label: Tint::solid(assist_chip_tokens::LABEL_TEXT_COLOR),
            leading: icon,
            trailing: icon,
            outline: Some((
                assist_chip_tokens::FLAT_OUTLINE_WIDTH,
                Tint::solid(assist_chip_tokens::FLAT_OUTLINE_COLOR),
            )),
        }
    }
}

/// The space around the label: `AssistChipDefaults.ContentPadding` and `horizontalArrangement()` (lines 1962 and 1965),
/// which are the same with every icon there is.
fn spaces(has_leading: bool) -> Spaces {
    Spaces::new(
        (CONTENT_PADDING, CONTENT_PADDING),
        (SPACING, SPACING),
        has_leading,
    )
}

/// The assist chip's sizes: `AssistChipDefaults.Height` and `IconSize`, and the label style `Chip` gives it
/// (`AssistChipTokens.LabelTextFont`, line 155).
fn metrics() -> Metrics {
    Metrics {
        height: assist_chip_tokens::CONTAINER_HEIGHT,
        leading_icon: assist_chip_tokens::ICON_SIZE,
        trailing_icon: assist_chip_tokens::ICON_SIZE,
        // An assist chip has no avatar.
        avatar: Points(0.0),
        label: assist_chip_tokens::LABEL_TEXT_FONT,
    }
}

/// The props of an [`AssistChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct AssistChipProps {
    /// What the chip says: screen readers read it as the chip's name.
    pub label: Prop<SharedString>,
    /// An icon before the label.
    pub leading_icon: Prop<Option<Icon>>,
    /// An icon after the label.
    pub trailing_icon: Prop<Option<Icon>>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of an [`AssistChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum AssistChipEvent {
    /// The assist chip was pressed.
    Press,
}

/// A Material 3 Expressive assist chip: a label that starts an action, in a flat container with an outline, with an
/// optional icon before and after it.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, the button role, named by its
/// label for screen readers) and the look of androidx's flat assist chip. Hover, focus and press lay the label's color
/// over it at the state layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so
/// nothing is laid out again. It is 32 points tall, with a 48-point touch target around it.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::assist_chip::assist_chip;
/// use nsh_m3e::icon::Icon;
///
/// fn directions() -> impl IntoElement {
///     assist_chip()
///         .label("Directions")
///         // Material Symbols' `add`.
///         .leading_icon(Icon::material_symbol(
///             "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z",
///         ))
///         .on_press(|| println!("directions"))
/// }
/// ```
#[component]
pub fn AssistChip(props: AssistChipProps, cx: Scope<AssistChipEvent>) -> impl IntoElement {
    let metrics = metrics();
    // Before its first layout, the container is taken to be as wide as it is tall; its layout corrects that.
    let drawn = DrawnShape::new(
        corners(assist_chip_tokens::CONTAINER_SHAPE),
        corners(assist_chip_tokens::CONTAINER_SHAPE),
        Size::new(metrics.height, metrics.height),
    );
    let has_leading = computed(move || props.leading_icon.get().is_some());
    let chip = Chip {
        drawn,
        metrics,
        label: props.label,
        disabled: props.disabled,
        look: computed(move || look(props.disabled.get())),
        spaces: computed(move || spaces(has_leading.get())),
        leading: computed(move || props.leading_icon.get().map(Leading::Icon)),
        trailing: props.trailing_icon,
    };
    chip.assemble(
        nsh_base::button()
            .disabled(move || props.disabled.get())
            .on_press(move || cx.emit(AssistChipEvent::Press))
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

    /// What an assist chip paints, with both icons, in the baseline schemes: androidx `AssistChipTokens`. Flat and
    /// transparent; an outline variant outline 1 point wide; the label on surface; the icons primary. Disabled, the
    /// outline is on surface at 12 % and the label and icons on surface at 38 %.
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
                trailing: dim,
            }
        } else {
            Painted {
                container: none,
                outline_width: 1.0,
                outline: solid(ColorRole::OutlineVariant, dark),
                layer: solid(ColorRole::OnSurface, dark),
                label: solid(ColorRole::OnSurface, dark),
                leading: solid(ColorRole::Primary, dark),
                trailing: solid(ColorRole::Primary, dark),
            }
        }
    }

    #[test]
    fn it_has_its_colors_enabled_and_disabled_light_and_dark() {
        for disabled in [false, true] {
            for dark in [false, true] {
                let mounted = mount(
                    Config::of(Kind::Assist)
                        .leading(true)
                        .trailing(true)
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
        // Light and dark are not the same colors: the test would pass on one scheme twice otherwise.
        assert_ne!(expected(false, false), expected(false, true));
    }

    #[test]
    fn it_never_changes_its_corners() {
        // `AssistChipTokens.ContainerShape`, `CornerSmall`: 8 points, the same at each corner so that it can be
        // drawn (`AssistChip` takes a `Shape`, not `ChipShapes`: it does not morph).
        assert_eq!(
            Corners::of(assist_chip_tokens::CONTAINER_SHAPE),
            Some(Corners::Radius(Points(8.0)))
        );
        for reduce_motion in [false, true] {
            let what = format!("reduce motion: {reduce_motion}");
            let (mut app, recording) = start(
                Config::of(Kind::Assist)
                    .reduce_motion(reduce_motion)
                    .leading(true)
                    .trailing(true),
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

    #[test]
    fn it_has_no_selected_state_and_no_avatar() {
        // An assist chip has no props for an avatar or a selected state: what screen readers are told and its views
        // show neither, and it paints no icon it was not given.
        let mounted = mount(Config::of(Kind::Assist));
        assert!(!mounted.state.selected);
        assert_eq!(mounted.state.pressed, None);
        assert!(mounted.avatar.is_none());
        assert_eq!(painted(&mounted), expected(false, false).without_icons());
    }

    /// The vendored `Chip.kt`'s lines that the assist chip is made of (ADR 0012): the flat colors, the border, and the
    /// shape that does not morph.
    #[test]
    fn every_choice_taken_from_androidx_is_still_there() {
        // The colors: a transparent container, the label on surface (a token), the icons in the icon token's color,
        // and the disabled ones faded by the tokens' opacities.
        assert!(cites(
            CHIP_KT,
            &[
                "containerColor = Color.Transparent,",
                "labelColor = fromToken(AssistChipTokens.LabelTextColor),",
                "leadingIconContentColor = fromToken(AssistChipTokens.IconColor),",
                "trailingIconContentColor = fromToken(AssistChipTokens.IconColor),",
                "disabledContainerColor = Color.Transparent,",
                "disabledLabelColor =",
                "fromToken(AssistChipTokens.DisabledLabelTextColor)",
                ".copy(alpha = AssistChipTokens.DisabledLabelTextOpacity),",
                "disabledLeadingIconContentColor =",
                "fromToken(AssistChipTokens.DisabledIconColor)",
                ".copy(alpha = AssistChipTokens.DisabledIconOpacity),",
                "disabledTrailingIconContentColor =",
                "fromToken(AssistChipTokens.DisabledIconColor)",
                ".copy(alpha = AssistChipTokens.DisabledIconOpacity),",
            ]
        ));
        // The border: the flat outline, faded when disabled, at the flat outline's width.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun assistChipBorder(",
                "enabled: Boolean,",
                "borderColor: Color = AssistChipTokens.FlatOutlineColor.value,",
                "disabledBorderColor: Color =",
                "AssistChipTokens.FlatDisabledOutlineColor.value.copy(",
                "alpha = AssistChipTokens.FlatDisabledOutlineOpacity",
                "),",
                "borderWidth: Dp = AssistChipTokens.FlatOutlineWidth,",
                "): BorderStroke = BorderStroke(borderWidth, if (enabled) borderColor else disabledBorderColor)",
            ]
        ));
        assert_eq!(assist_chip_tokens::FLAT_OUTLINE_WIDTH, Points(1.0));
        // The shape is the token's, and `AssistChip` takes a `Shape`, not the `ChipShapes` that morph.
        assert!(cites(
            CHIP_KT,
            &[
                "public val shape: Shape",
                "@Composable get() = AssistChipTokens.ContainerShape.value",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &["shape: Shape = AssistChipDefaults.shape,"]
        ));
        // The sizes and spaces are the file's: 8 points around and between, with every icon there is.
        assert!(cites(
            CHIP_KT,
            &[
                "public val HorizontalSpacing: Dp = 8.dp",
                "",
                "/**",
                "* The padding around the content of the chip, including the leadingIcon, label, and",
                "* trailingIcon",
                "*/",
                "public val ContentPadding: PaddingValues = PaddingValues(horizontal = 8.dp)",
            ]
        ));
        assert_eq!(CONTENT_PADDING, Points(8.0));
        assert_eq!(SPACING, Points(8.0));
        // A line that is not there is not found.
        assert!(!cites(
            CHIP_KT,
            &["labelColor = fromToken(AssistChipTokens.IconColor),"]
        ));
    }
}
