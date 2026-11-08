//! The Material 3 Expressive filter chip: a label in a flat container that is selected or not, with an optional icon
//! before it and after it, the state layer of hover, focus and press, and the shape it morphs into when it is
//! selected.
//!
//! The filter chip follows androidx's Expressive `FilterChip(selected, onClick, label, shapes, …)` in `Chip.kt` at the
//! pinned commit (line 730), as [ADR 0012] says; every line number below is a line of
//! `reference/androidx-material3-components/Chip.kt`. It reads the `FilterChipTokens` the way `FilterChipDefaults`
//! does: `defaultFilterChipColors` (lines 2336 to 2368), `filterChipBorder` (2423 to 2443), `Height` and `IconSize`
//! (2229 and 2232), and the corners from `ChipsTokens` (`FilterChipDefaults.shapes()`, 2590 to 2621).
//!
//! # Where the component overrides the tokens
//!
//! - The Expressive `FilterChip` defaults to `tonalFilterChipColors()` (line 739): the flat colors, with the unselected
//!   leading icon in `ChipsTokens.UnselectedLeadingIconColor` (on surface variant, line 2374), where
//!   `FilterChipTokens.UnselectedLeadingIconColor` is the primary color.
//! - Its corners are `ChipsTokens`' (`UnselectedShape` 12 points, `SelectedShape` fully rounded, `PressedShape` 8
//!   points), where `FilterChipTokens.ContainerShape` (8 points) is the shape of the overload with no `ChipShapes`.
//! - A selected chip has a `Color.Transparent` border (line 2427), which draws nothing: it has no outline, whatever
//!   width `FlatSelectedOutlineWidth` gives. The unselected chip's disabled outline fades `FlatDisabledUnselectedOutlineColor`
//!   by `FlatDisabledUnselectedOutlineOpacity` (2429), not `ChipsTokens.UnselectedDisabledOutlineOpacity`.
//! - The colors androidx gives to hover, focus and press are the state's own (the `Hover`, `Focus` and `Pressed`
//!   label and icon tokens name the same roles): `Chip.kt` reads none of them.
//!
//! # The shape's morph
//!
//! The chip's shape is its unselected corners, its selected corners, or its pressed corners (`shapeByInteraction`,
//! lines 4269 to 4289), moved by `MotionSchemeKeyTokens.FastSpatial` (line 3312, marked "TODO: Replace with correct
//! animation tokens" in the file). The chip springs from its unselected to its selected corners and back when
//! `selected` changes, from where the corners are and as fast as they move; with reduce motion on they change at once
//! (nothing moves, but the shape shows the state).
//!
//! # What NSH could not do, and what is left out
//!
//! - The pressed corners: `nsh-base`'s `Toggle`, which gives this chip its behavior, reports no press going down or
//!   ending (`toggle.rs`, `ToggleEvent` has `Change` only), so the corners cannot spring to `PressedShape` while a
//!   press is down. They are given to the container already; wiring them is two lines when the `Toggle` reports a
//!   press (the input chip, built on `Button`, has them).
//! - Role and state for screen readers: androidx gives the chip `Role.Checkbox` and `selected` (lines 3258 and 3317).
//!   A `Toggle` is a button with a pressed state, which is what a filter chip is: a toggle button.
//! - The icons' expand and fade (`AnimatingChipContent`, lines 3423 to 3515): it animates the chip's width, a layout per
//!   frame. An icon comes and goes at once. The selection icon (a check mark, "may visually indicate that the chip is
//!   selected", line 805) is the caller's `leading_icon`.
//! - The elevated filter chip (`ElevatedFilterChip`, line 910): elevation is decided by ADR 0018 and written by its own
//!   change; this chip has no shadow code.
//! - A focus indicator: `Chip.kt` reads none (`FilterChipTokens.FocusIndicatorColor` has no reader).
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::chip::{
    CONTENT_PADDING, Chip, Leading, Look, Metrics, Spaces, compact_arrangement, touch_slop,
};
use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::motion::SpringToken;
use crate::tint::Tint;
use crate::tokens::Shape;
use crate::tokens::androidx::{chips_tokens, filter_chip_tokens};

/// The spring that moves the filter chip's corners: `MotionSchemeKeyTokens.FastSpatial` (`Chip.kt`, line 3312).
const SHAPE_SPRING: SpringToken = SpringToken::FastSpatial;

/// The corners at rest: `ChipsTokens.SelectedShape` selected, `UnselectedShape` not (`FilterChipDefaults.shapes()`,
/// lines 2612 to 2621).
fn rest_shape(selected: bool) -> Shape {
    if selected {
        chips_tokens::SELECTED_SHAPE
    } else {
        chips_tokens::UNSELECTED_SHAPE
    }
}

/// What the filter chip shows in each state, from `defaultTonalFilterChipColors` (lines 2370 to 2376, which is
/// `defaultFilterChipColors`, 2336 to 2368, with the unselected leading icon from `ChipsTokens`) and `filterChipBorder`
/// (2423 to 2443).
fn look(selected: bool, disabled: bool) -> Look {
    let disabled_label = Tint::faded(
        filter_chip_tokens::DISABLED_LABEL_TEXT_COLOR,
        filter_chip_tokens::DISABLED_LABEL_TEXT_OPACITY,
    );
    let disabled_leading = Tint::faded(
        filter_chip_tokens::DISABLED_LEADING_ICON_COLOR,
        filter_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
    );
    let disabled_trailing = Tint::faded(
        filter_chip_tokens::DISABLED_TRAILING_ICON_COLOR,
        filter_chip_tokens::DISABLED_TRAILING_ICON_OPACITY,
    );
    match (selected, disabled) {
        (false, false) => Look {
            container: None,
            label: Tint::solid(filter_chip_tokens::UNSELECTED_LABEL_TEXT_COLOR),
            leading: Tint::solid(chips_tokens::UNSELECTED_LEADING_ICON_COLOR),
            trailing: Tint::solid(filter_chip_tokens::UNSELECTED_TRAILING_ICON_COLOR),
            outline: Some((
                filter_chip_tokens::FLAT_UNSELECTED_OUTLINE_WIDTH,
                Tint::solid(filter_chip_tokens::FLAT_UNSELECTED_OUTLINE_COLOR),
            )),
        },
        (true, false) => Look {
            container: Some(Tint::solid(
                filter_chip_tokens::FLAT_SELECTED_CONTAINER_COLOR,
            )),
            label: Tint::solid(filter_chip_tokens::SELECTED_LABEL_TEXT_COLOR),
            leading: Tint::solid(filter_chip_tokens::SELECTED_LEADING_ICON_COLOR),
            trailing: Tint::solid(filter_chip_tokens::SELECTED_TRAILING_ICON_COLOR),
            outline: None,
        },
        (false, true) => Look {
            container: None,
            label: disabled_label,
            leading: disabled_leading,
            trailing: disabled_trailing,
            outline: Some((
                filter_chip_tokens::FLAT_UNSELECTED_OUTLINE_WIDTH,
                Tint::faded(
                    filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_COLOR,
                    filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY,
                ),
            )),
        },
        (true, true) => Look {
            container: Some(Tint::faded(
                filter_chip_tokens::FLAT_DISABLED_SELECTED_CONTAINER_COLOR,
                filter_chip_tokens::FLAT_DISABLED_SELECTED_CONTAINER_OPACITY,
            )),
            label: disabled_label,
            leading: disabled_leading,
            trailing: disabled_trailing,
            outline: None,
        },
    }
}

/// The space around the label: `FilterChipDefaults.ContentPadding` (line 2269) and the arrangement the Expressive
/// `FilterChip` takes by default (`horizontalArrangement(leadingIcon != null, trailingIcon != null)`, lines 742 and
/// 743).
fn spaces(has_leading: bool, has_trailing: bool) -> Spaces {
    Spaces::new(
        (CONTENT_PADDING, CONTENT_PADDING),
        compact_arrangement(has_leading, has_trailing),
        has_leading,
    )
}

/// The filter chip's sizes: `FilterChipDefaults.Height` and `IconSize`, and the label style `SelectableChip` gets
/// (`FilterChipTokens.LabelTextFont`, line 753).
fn metrics() -> Metrics {
    Metrics {
        height: filter_chip_tokens::CONTAINER_HEIGHT,
        leading_icon: filter_chip_tokens::ICON_SIZE,
        trailing_icon: filter_chip_tokens::ICON_SIZE,
        // A filter chip has no avatar.
        avatar: Points(0.0),
        label: filter_chip_tokens::LABEL_TEXT_FONT,
    }
}

/// The props of a [`FilterChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct FilterChipProps {
    /// What the chip says: screen readers read it as the chip's name.
    pub label: Prop<SharedString>,
    /// Whether it is selected. The app owns the value: a press reports the next one with
    /// [`FilterChipEvent::Change`], and the chip shows the value the app passes back.
    #[prop(default)]
    pub selected: Prop<bool>,
    /// An icon before the label, which may show that the chip is selected (a check mark).
    pub leading_icon: Prop<Option<Icon>>,
    /// An icon after the label.
    pub trailing_icon: Prop<Option<Icon>>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`FilterChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum FilterChipEvent {
    /// The filter chip was pressed: whether it asks to be selected, the opposite of `selected`. It runs before
    /// anything changes: the chip shows the `selected` the app passes back.
    #[args(selected)]
    Change(bool),
}

/// A Material 3 Expressive filter chip: a label that filters content when it is selected, in a flat container, with
/// an optional icon before and after it.
///
/// It has the behavior of `nsh-base`'s toggle (presses, focus, the disabled state, a toggle button's role and pressed
/// state for screen readers, named by its label) and the look of androidx's Expressive flat filter chip. Selected, it
/// has the secondary container's color and no outline, and its corners spring from the unselected to the selected
/// shape. Hover, focus and press lay the label's color over it at the state layer opacities of `StateTokens`, as
/// androidx's ripple does, and change nothing else, so nothing is laid out again. It is 32 points tall, with a
/// 48-point touch target around it.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::filter_chip::filter_chip;
///
/// fn favorites() -> impl IntoElement {
///     let selected = signal(false);
///     filter_chip()
///         .label("Favorites")
///         .selected(move || selected.get())
///         .on_change(move |next: &bool| selected.set(*next))
/// }
/// ```
#[component]
pub fn FilterChip(props: FilterChipProps, cx: Scope<FilterChipEvent>) -> impl IntoElement {
    let metrics = metrics();
    // Before its first layout, the container is taken to be as wide as it is tall; its layout corrects that.
    let drawn = untracked(|| {
        DrawnShape::new(
            corners(rest_shape(props.selected.get())),
            corners(chips_tokens::PRESSED_SHAPE),
            Size::new(metrics.height, metrics.height),
        )
    })
    .with_spring(SHAPE_SPRING);
    effect(move || drawn.rest_to(corners(rest_shape(props.selected.get()))));
    let has_leading = computed(move || props.leading_icon.get().is_some());
    let has_trailing = computed(move || props.trailing_icon.get().is_some());
    let chip = Chip {
        drawn,
        metrics,
        label: props.label,
        disabled: props.disabled,
        look: computed(move || look(props.selected.get(), props.disabled.get())),
        spaces: computed(move || spaces(has_leading.get(), has_trailing.get())),
        leading: computed(move || props.leading_icon.get().map(Leading::Icon)),
        trailing: props.trailing_icon,
    };
    chip.assemble(
        nsh_base::toggle()
            .pressed(move || props.selected.get())
            .disabled(move || props.disabled.get())
            .on_change(move |next: &bool| cx.emit(FilterChipEvent::Change(*next)))
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
        ALPHA_12, ALPHA_38, CHIP_KT, Config, Kind, Mounted, Painted, cites, faded, morph_in, mount,
        painted, press_down, press_items, solid, start,
    };
    use crate::color::ColorRole;
    use crate::morph::Corners;
    use crate::morph_probe::set_values;

    /// What a filter chip paints, with both icons, in the baseline schemes: androidx `FilterChipTokens` with the
    /// tonal colors the Expressive `FilterChip` defaults to (`tonalFilterChipColors`: the unselected leading icon is
    /// on surface variant, where the token is primary). Unselected: transparent, an outline variant outline 1 point
    /// wide, the label and icons on surface variant. Selected: a secondary container, no outline, the label and icons
    /// on secondary container. Disabled: on surface at 38 % for the label and icons, at 12 % for the outline (the
    /// unselected chip) or the container (the selected one).
    fn expected(selected: bool, disabled: bool, dark: bool) -> Painted {
        let none = Color(0);
        let on_surface = |alpha| faded(ColorRole::OnSurface, alpha, dark);
        match (selected, disabled) {
            (false, false) => {
                let content = solid(ColorRole::OnSurfaceVariant, dark);
                Painted {
                    container: none,
                    outline_width: 1.0,
                    outline: solid(ColorRole::OutlineVariant, dark),
                    layer: content,
                    label: content,
                    leading: content,
                    trailing: content,
                }
            }
            (true, false) => {
                let content = solid(ColorRole::OnSecondaryContainer, dark);
                Painted {
                    container: solid(ColorRole::SecondaryContainer, dark),
                    outline_width: 0.0,
                    outline: none,
                    layer: content,
                    label: content,
                    leading: content,
                    trailing: content,
                }
            }
            (false, true) => Painted {
                container: none,
                outline_width: 1.0,
                outline: on_surface(ALPHA_12),
                layer: on_surface(ALPHA_38),
                label: on_surface(ALPHA_38),
                leading: on_surface(ALPHA_38),
                trailing: on_surface(ALPHA_38),
            },
            (true, true) => Painted {
                container: on_surface(ALPHA_12),
                outline_width: 0.0,
                outline: none,
                layer: on_surface(ALPHA_38),
                label: on_surface(ALPHA_38),
                leading: on_surface(ALPHA_38),
                trailing: on_surface(ALPHA_38),
            },
        }
    }

    #[test]
    fn it_has_its_colors_selected_or_not_enabled_and_disabled_light_and_dark() {
        for selected in [false, true] {
            for disabled in [false, true] {
                for dark in [false, true] {
                    let mounted = mount(
                        Config::of(Kind::Filter)
                            .selected(selected)
                            .disabled(disabled)
                            .dark(dark)
                            .leading(true)
                            .trailing(true),
                    );
                    assert_eq!(
                        painted(&mounted),
                        expected(selected, disabled, dark),
                        "selected: {selected} disabled: {disabled} dark: {dark}"
                    );
                }
            }
        }
        assert_ne!(expected(false, false, false), expected(false, false, true));
        // The two states are told apart by more than a color: the container and the outline.
        assert_ne!(expected(false, false, false), expected(true, false, false));
    }

    #[test]
    fn its_unselected_leading_icon_is_the_chips_token_not_the_filter_chips() {
        // `FilterChipTokens.UnselectedLeadingIconColor` is primary; the tonal colors the Expressive `FilterChip`
        // defaults to take `ChipsTokens.UnselectedLeadingIconColor`, on surface variant (ADR 0012). Only the leading
        // icon differs: the trailing one is on surface variant in both.
        assert_eq!(
            filter_chip_tokens::UNSELECTED_LEADING_ICON_COLOR,
            ColorRole::Primary
        );
        assert_eq!(
            chips_tokens::UNSELECTED_LEADING_ICON_COLOR,
            ColorRole::OnSurfaceVariant
        );
        let mounted = mount(Config::of(Kind::Filter).leading(true).trailing(true));
        let painted = painted(&mounted);
        assert_eq!(painted.leading, solid(ColorRole::OnSurfaceVariant, false));
        assert_ne!(painted.leading, solid(ColorRole::Primary, false));
        assert_eq!(painted.leading, painted.trailing);
    }

    #[test]
    fn its_corners_are_the_chips_not_the_filter_chips_container_shape() {
        // `ChipsTokens` (`FilterChipDefaults.shapes()`): 12 points unselected, fully rounded selected, 8 pressed. The
        // 8 points of `FilterChipTokens.ContainerShape` are the shape of the overload that takes no `ChipShapes`
        // (ADR 0012); here they are not used.
        assert_eq!(
            Corners::of(chips_tokens::UNSELECTED_SHAPE),
            Some(Corners::Radius(Points(12.0)))
        );
        assert_eq!(
            Corners::of(chips_tokens::SELECTED_SHAPE),
            Some(Corners::Full)
        );
        assert_eq!(
            Corners::of(chips_tokens::PRESSED_SHAPE),
            Some(Corners::Radius(Points(8.0)))
        );
        assert_eq!(
            Corners::of(filter_chip_tokens::CONTAINER_SHAPE),
            Some(Corners::Radius(Points(8.0)))
        );
        assert_eq!(rest_shape(false), chips_tokens::UNSELECTED_SHAPE);
        assert_eq!(rest_shape(true), chips_tokens::SELECTED_SHAPE);
        // Every shape the chip draws can morph: none has uneven corners, which would fall back to full corners.
        for shape in [
            chips_tokens::UNSELECTED_SHAPE,
            chips_tokens::SELECTED_SHAPE,
            chips_tokens::PRESSED_SHAPE,
        ] {
            assert!(Corners::of(shape).is_some(), "{shape:?}");
        }
    }

    #[test]
    fn its_corners_move_with_fast_spatial() {
        // `Chip.kt` animates a chip's shape with `FastSpatial` (stiffness 800, damping ratio 0.6), not the button's
        // `DefaultEffects`: the spring that starts is checked against that token, not the button's.
        assert_eq!(SHAPE_SPRING, SpringToken::FastSpatial);
        let spec = SpringToken::FastSpatial.spec();
        assert_eq!(spec.stiffness, 800.0);
        assert_eq!(spec.damping_ratio, 0.6);
        assert_ne!(spec, SpringToken::DefaultEffects.spec());
    }

    #[test]
    fn a_press_asks_for_the_opposite_and_the_chip_springs_to_the_shape_it_is_given() {
        let (mut app, recording) = start(Config::of(Kind::Filter));
        assert!(!Mounted::of(&recording.items()).state.selected);
        let items = press_items(&mut app, &recording, "Chip");
        assert!(app.shows("presses: 1"));
        // The app passes `selected` back: one spring, to the selected corners, and the corners were at rest until then.
        let _ = morph_in(&recording, &items, 16.0);
        let items = press_items(&mut app, &recording, "Chip");
        assert!(app.shows("presses: 2"));
        let _ = morph_in(&recording, &items, 12.0);
    }

    #[test]
    fn a_disabled_filter_chip_keeps_its_corners_under_a_press() {
        for selected in [false, true] {
            let (mut app, recording) =
                start(Config::of(Kind::Filter).selected(selected).disabled(true));
            let before = recording.items().len();
            let _ = app.press("Chip");
            assert!(app.shows("presses: 0"), "selected: {selected}");
            let items = recording.items().split_off(before);
            assert_eq!(started_springs(&items), Vec::new(), "selected: {selected}");
            assert_eq!(set_values(&items), 0, "selected: {selected}");
        }
    }

    #[test]
    fn it_has_no_pressed_corners_because_the_toggle_reports_no_press_down() {
        // What NSH could not do (the module's documentation): `nsh-base`'s `Toggle` reports a change, not a press that
        // goes down. A finger that lands and stays springs nothing; the change comes when it lifts. When the `Toggle`
        // reports presses, this test is the one to change, with the wiring.
        let (mut app, recording) = start(Config::of(Kind::Filter));
        let step = press_down(&mut app);
        let down = recording.in_step(&step);
        assert_eq!(started_springs(&down), Vec::new());
        assert_eq!(set_values(&down), 0);
        assert!(app.shows("presses: 0"));
    }

    /// The vendored `Chip.kt`'s lines that the filter chip's colors and border are made of (ADR 0012).
    #[test]
    fn every_color_and_border_taken_from_androidx_is_still_there() {
        // The Expressive `FilterChip` takes the shapes, and the tonal colors by default.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun FilterChip(",
                "selected: Boolean,",
                "onClick: () -> Unit,",
                "label: @Composable () -> Unit,",
                "shapes: ChipShapes,",
                "modifier: Modifier = Modifier,",
                "enabled: Boolean = true,",
                "leadingIcon: @Composable (() -> Unit)? = null,",
                "trailingIcon: @Composable (() -> Unit)? = null,",
                "colors: SelectableChipColors = FilterChipDefaults.tonalFilterChipColors(),",
            ]
        ));
        // The flat colors, and the tonal override of the unselected leading icon.
        assert!(cites(
            CHIP_KT,
            &[
                "containerColor = Color.Transparent,",
                "labelColor = fromToken(FilterChipTokens.UnselectedLabelTextColor),",
                "leadingIconColor = fromToken(FilterChipTokens.UnselectedLeadingIconColor),",
                "trailingIconColor = fromToken(FilterChipTokens.UnselectedTrailingIconColor),",
                "disabledContainerColor = Color.Transparent,",
                "disabledLabelColor =",
                "fromToken(FilterChipTokens.DisabledLabelTextColor)",
                ".copy(alpha = FilterChipTokens.DisabledLabelTextOpacity),",
                "disabledLeadingIconColor =",
                "fromToken(FilterChipTokens.DisabledLeadingIconColor)",
                ".copy(alpha = FilterChipTokens.DisabledLeadingIconOpacity),",
                "disabledTrailingIconColor =",
                "fromToken(FilterChipTokens.DisabledTrailingIconColor)",
                ".copy(alpha = FilterChipTokens.DisabledTrailingIconOpacity),",
                "selectedContainerColor =",
                "fromToken(FilterChipTokens.FlatSelectedContainerColor),",
                "disabledSelectedContainerColor =",
                "fromToken(FilterChipTokens.FlatDisabledSelectedContainerColor)",
                ".copy(",
                "alpha = FilterChipTokens.FlatDisabledSelectedContainerOpacity",
                "),",
                "selectedLabelColor = fromToken(FilterChipTokens.SelectedLabelTextColor),",
                "selectedLeadingIconColor =",
                "fromToken(FilterChipTokens.SelectedLeadingIconColor),",
                "selectedTrailingIconColor =",
                "fromToken(FilterChipTokens.SelectedTrailingIconColor),",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "?: defaultFilterChipColors",
                ".copy(leadingIconColor = fromToken(ChipsTokens.UnselectedLeadingIconColor))",
                ".also { defaultTonalFilterChipColorsCached = it }",
            ]
        ));
        // The border: transparent when selected, whatever width the token gives, and the unselected disabled outline's
        // own opacity, not `ChipsTokens`'.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun filterChipBorder(",
                "enabled: Boolean,",
                "selected: Boolean,",
                "borderColor: Color = FilterChipTokens.FlatUnselectedOutlineColor.value,",
                "selectedBorderColor: Color = Color.Transparent,",
                "disabledBorderColor: Color =",
                "FilterChipTokens.FlatDisabledUnselectedOutlineColor.value.copy(",
                "alpha = FilterChipTokens.FlatDisabledUnselectedOutlineOpacity",
                "),",
                "disabledSelectedBorderColor: Color = Color.Transparent,",
                "borderWidth: Dp = FilterChipTokens.FlatUnselectedOutlineWidth,",
                "selectedBorderWidth: Dp = FilterChipTokens.FlatSelectedOutlineWidth,",
            ]
        ));
        assert_eq!(
            filter_chip_tokens::FLAT_UNSELECTED_OUTLINE_WIDTH,
            Points(1.0)
        );
        assert_eq!(
            filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY.to_bits(),
            0.12_f32.to_bits()
        );
        assert_ne!(
            filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY.to_bits(),
            chips_tokens::UNSELECTED_DISABLED_OUTLINE_OPACITY.to_bits()
        );
    }

    /// The vendored `Chip.kt`'s lines that the filter chip's shapes, morph and spaces are made of (ADR 0012).
    #[test]
    fn every_shape_morph_and_space_taken_from_androidx_is_still_there() {
        // The shapes, and what moves them: the shape by interaction, with `FastSpatial`.
        assert!(cites(
            CHIP_KT,
            &[
                "shape = fromToken(ChipsTokens.UnselectedShape),",
                "selectedShape = fromToken(ChipsTokens.SelectedShape),",
                "pressedShape = fromToken(ChipsTokens.PressedShape),",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "// TODO: Replace with correct animation tokens",
                "val defaultAnimationSpec = MotionSchemeKeyTokens.FastSpatial.value<Float>()",
                "val chipShape = shapeByInteraction(shapes, selected, isPressed, defaultAnimationSpec)",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "val shape =",
                "if (pressed) {",
                "shapes.pressedShape",
                "} else if (selected) {",
                "shapes.selectedShape",
                "} else {",
                "shapes.shape",
                "}",
            ]
        ));
        // The spaces: 8 points of padding, and the arrangement that is compact next to an icon.
        assert!(cites(
            CHIP_KT,
            &[
                "public val HorizontalSpacing: Dp = 8.dp",
                "",
                "/** The compact horizontal spacing between the icon and label of a filter chip. */",
                "public val CompactHorizontalSpacing: Dp = 4.dp",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "return if (hasLeadingIcon && hasTrailingIcon) ChipArrangement(CompactHorizontalSpacing)",
                "else if (hasLeadingIcon) ChipArrangement(CompactHorizontalSpacing, HorizontalSpacing)",
                "else if (hasTrailingIcon) ChipArrangement(HorizontalSpacing, CompactHorizontalSpacing)",
                "else DefaultHorizontalArrangement",
            ]
        ));
        assert_eq!(CONTENT_PADDING, Points(8.0));
        // A line that is not there is not found.
        assert!(!cites(
            CHIP_KT,
            &["val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultEffects.value<Float>()"]
        ));
    }
}
