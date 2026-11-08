//! The Material 3 Expressive input chip: a label in a flat container that is selected or not, with an optional avatar
//! or icon before it and an icon after it, the state layer of hover, focus and press, and the shape it morphs into when
//! it is selected and while it is pressed.
//!
//! The input chip follows androidx's Expressive `InputChip(selected, onClick, label, shapes, …)` in `Chip.kt` at the
//! pinned commit (line 1298), as [ADR 0012] says; every line number below is a line of
//! `reference/androidx-material3-components/Chip.kt`. It reads the `InputChipTokens` the way `InputChipDefaults` does:
//! `defaultInputChipColors` (lines 2755 to 2784), `inputChipBorder` (2839 to 2859), `Height`, `IconSize` and
//! `AvatarSize` (2630 to 2636), and the corners from `ChipsTokens` (`InputChipDefaults.shapes()`, 2879 to 2900).
//!
//! # Where the component overrides the tokens
//!
//! - The Expressive `InputChip` defaults to `tonalInputChipColors()` (line 1308): the flat colors, with the unselected
//!   leading icon in `ChipsTokens.UnselectedLeadingIconColor` (line 2790). The two tokens name the same role (on
//!   surface variant) today.
//! - Its corners are `ChipsTokens`' (`UnselectedShape` 12 points, `SelectedShape` fully rounded, `PressedShape` 8
//!   points), where `InputChipTokens.ContainerShape` (8 points) is the shape of the overload with no `ChipShapes`.
//! - A selected chip has a `Color.Transparent` border (line 2848), which draws nothing: it has no outline.
//! - Its content padding and the spaces between its elements depend on its avatar, icon and trailing icon, and are
//!   literals of the file (`contentPadding`, lines 2678 to 2686: 4 or 8 points before, 8 or 4 after;
//!   `horizontalArrangement`, 2650 to 2661).
//! - An avatar takes precedence over a leading icon (`leadingContent`, line 3528). It is clipped to
//!   `InputChipTokens.AvatarShape` at `AvatarSize` and faded to `DisabledAvatarOpacity` when the chip is disabled
//!   (lines 1323 to 1341). The app fills it with `.avatar(..)`.
//!
//! # The shape's morph
//!
//! The chip's shape is its unselected corners, its selected corners, or its pressed corners (`shapeByInteraction`,
//! lines 4269 to 4289), moved by `MotionSchemeKeyTokens.FastSpatial` (line 3312, marked "TODO: Replace with correct
//! animation tokens" in the file). The chip springs to the pressed corners when a press goes down and back when it
//! ends, and between the unselected and the selected corners when `selected` changes, from where the corners are and
//! as fast as they move. With reduce motion on a press changes nothing and a change of `selected` changes the shape at
//! once.
//!
//! # What NSH could not do, and what is left out
//!
//! - Role and state for screen readers: androidx gives the chip `Role.Checkbox` and `selected` (lines 3258 and 3317).
//!   `nsh-base`'s `Button` has a `selected` state, which this chip sets, and the role of a button.
//! - The icons' expand and fade (`AnimatingChipContent`, lines 3423 to 3515): it animates the chip's width, a layout per
//!   frame. An icon or avatar comes and goes at once.
//! - A focus indicator: `Chip.kt` reads none (`InputChipTokens.FocusIndicatorColor` has no reader).
//! - There is no elevated input chip in androidx.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::chip::{
    COMPACT_SPACING, CONTENT_PADDING, Chip, Leading, Look, Metrics, Spaces, compact_arrangement,
    touch_slop,
};
use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::motion::SpringToken;
use crate::tint::Tint;
use crate::tokens::Shape;
use crate::tokens::androidx::{chips_tokens, input_chip_tokens};

/// The spring that moves the input chip's corners: `MotionSchemeKeyTokens.FastSpatial` (`Chip.kt`, line 3312).
const SHAPE_SPRING: SpringToken = SpringToken::FastSpatial;

/// The corners at rest: `ChipsTokens.SelectedShape` selected, `UnselectedShape` not (`InputChipDefaults.shapes()`,
/// lines 2891 to 2900).
fn rest_shape(selected: bool) -> Shape {
    if selected {
        chips_tokens::SELECTED_SHAPE
    } else {
        chips_tokens::UNSELECTED_SHAPE
    }
}

/// What the input chip shows in each state, from `defaultTonalInputChipColors` (lines 2786 to 2792, which is
/// `defaultInputChipColors`, 2755 to 2784, with the unselected leading icon from `ChipsTokens`) and `inputChipBorder`
/// (2839 to 2859).
fn look(selected: bool, disabled: bool) -> Look {
    let disabled_label = Tint::faded(
        input_chip_tokens::DISABLED_LABEL_TEXT_COLOR,
        input_chip_tokens::DISABLED_LABEL_TEXT_OPACITY,
    );
    let disabled_leading = Tint::faded(
        input_chip_tokens::DISABLED_LEADING_ICON_COLOR,
        input_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
    );
    let disabled_trailing = Tint::faded(
        input_chip_tokens::DISABLED_TRAILING_ICON_COLOR,
        input_chip_tokens::DISABLED_TRAILING_ICON_OPACITY,
    );
    match (selected, disabled) {
        (false, false) => Look {
            container: None,
            label: Tint::solid(input_chip_tokens::UNSELECTED_LABEL_TEXT_COLOR),
            leading: Tint::solid(chips_tokens::UNSELECTED_LEADING_ICON_COLOR),
            trailing: Tint::solid(input_chip_tokens::UNSELECTED_TRAILING_ICON_COLOR),
            outline: Some((
                input_chip_tokens::UNSELECTED_OUTLINE_WIDTH,
                Tint::solid(input_chip_tokens::UNSELECTED_OUTLINE_COLOR),
            )),
        },
        (true, false) => Look {
            container: Some(Tint::solid(input_chip_tokens::SELECTED_CONTAINER_COLOR)),
            label: Tint::solid(input_chip_tokens::SELECTED_LABEL_TEXT_COLOR),
            leading: Tint::solid(input_chip_tokens::SELECTED_LEADING_ICON_COLOR),
            trailing: Tint::solid(input_chip_tokens::SELECTED_TRAILING_ICON_COLOR),
            outline: None,
        },
        (false, true) => Look {
            container: None,
            label: disabled_label,
            leading: disabled_leading,
            trailing: disabled_trailing,
            outline: Some((
                input_chip_tokens::UNSELECTED_OUTLINE_WIDTH,
                Tint::faded(
                    input_chip_tokens::DISABLED_UNSELECTED_OUTLINE_COLOR,
                    input_chip_tokens::DISABLED_UNSELECTED_OUTLINE_OPACITY,
                ),
            )),
        },
        (true, true) => Look {
            container: Some(Tint::faded(
                input_chip_tokens::DISABLED_SELECTED_CONTAINER_COLOR,
                input_chip_tokens::DISABLED_SELECTED_CONTAINER_OPACITY,
            )),
            label: disabled_label,
            leading: disabled_leading,
            trailing: disabled_trailing,
            outline: None,
        },
    }
}

/// The padding before the content of an input chip with an avatar, a leading icon or neither: 4 points with an avatar
/// or no icon, 8 with an icon (`InputChipDefaults.contentPadding`, line 2683). A literal of the file; no token holds
/// it.
fn padding_start(has_avatar: bool, has_icon: bool) -> Points {
    if has_avatar || !has_icon {
        COMPACT_SPACING
    } else {
        CONTENT_PADDING
    }
}

/// The padding after the content: 8 points with a trailing icon, 4 without (`InputChipDefaults.contentPadding`,
/// line 2684). A literal of the file; no token holds it.
fn padding_end(has_trailing: bool) -> Points {
    if has_trailing {
        CONTENT_PADDING
    } else {
        COMPACT_SPACING
    }
}

/// The space around the label, for a chip with an avatar, a leading icon and a trailing icon or not (an avatar
/// takes the leading icon's place in the arrangement, `InputChipDefaults.horizontalArrangement`, lines 2650 to 2661).
fn spaces(has_avatar: bool, has_icon: bool, has_trailing: bool) -> Spaces {
    let has_leading = has_avatar || has_icon;
    Spaces::new(
        (
            padding_start(has_avatar, has_icon),
            padding_end(has_trailing),
        ),
        compact_arrangement(has_leading, has_trailing),
        has_leading,
    )
}

/// The input chip's sizes: `InputChipDefaults.Height`, `IconSize` and `AvatarSize`, the trailing icon's size, and the
/// label style `SelectableChip` gets (`InputChipTokens.LabelTextFont`, line 1348).
fn metrics() -> Metrics {
    Metrics {
        height: input_chip_tokens::CONTAINER_HEIGHT,
        leading_icon: input_chip_tokens::LEADING_ICON_SIZE,
        trailing_icon: input_chip_tokens::TRAILING_ICON_SIZE,
        avatar: input_chip_tokens::AVATAR_SIZE,
        label: input_chip_tokens::LABEL_TEXT_FONT,
    }
}

/// The props of an [`InputChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct InputChipProps {
    /// What the chip says: screen readers read it as the chip's name.
    pub label: Prop<SharedString>,
    /// Whether it is selected. The app owns the value and decides what a press does with it.
    #[prop(default)]
    pub selected: Prop<bool>,
    /// An icon before the label; an avatar takes its place.
    pub leading_icon: Prop<Option<Icon>>,
    /// An avatar before the label, filled with `.avatar(..)`: a picture, clipped to a circle.
    pub avatar: Prop<Option<Slot>>,
    /// An icon after the label.
    pub trailing_icon: Prop<Option<Icon>>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of an [`InputChip`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum InputChipEvent {
    /// The input chip was pressed.
    Press,
}

/// A Material 3 Expressive input chip: a piece of information the user entered (a person, a tag), in a flat
/// container, with an optional avatar or icon before it and an icon after it.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled and selected states, named by its label
/// for screen readers) and the look of androidx's Expressive input chip. Selected, it has the secondary container's
/// color and no outline; its corners spring to the selected shape, and to the pressed shape while a press is down.
/// Hover, focus and press lay the label's color over it at the state layer opacities of `StateTokens`, as androidx's
/// ripple does. It is 32 points tall, with a 48-point touch target around it.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::input_chip::input_chip;
///
/// fn recipient() -> impl IntoElement {
///     let selected = signal(false);
///     input_chip()
///         .label("Ada")
///         .selected(move || selected.get())
///         .on_press(move || selected.update(|selected| *selected = !*selected))
/// }
/// ```
#[component]
pub fn InputChip(props: InputChipProps, cx: Scope<InputChipEvent>) -> impl IntoElement {
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
    effect(move || {
        // A chip disabled while pressed may get no end of its press: it goes back to its shape at rest.
        if props.disabled.get() {
            drawn.press(false);
        }
    });
    let has_avatar = computed(move || props.avatar.get().is_some());
    let has_icon = computed(move || props.leading_icon.get().is_some());
    let has_trailing = computed(move || props.trailing_icon.get().is_some());
    let chip = Chip {
        drawn,
        metrics,
        label: props.label,
        disabled: props.disabled,
        look: computed(move || look(props.selected.get(), props.disabled.get())),
        spaces: computed(move || spaces(has_avatar.get(), has_icon.get(), has_trailing.get())),
        leading: computed(move || {
            props
                .avatar
                .get()
                .map(|slot| Leading::Avatar {
                    slot,
                    opacity: if props.disabled.get() {
                        input_chip_tokens::DISABLED_AVATAR_OPACITY
                    } else {
                        1.0
                    },
                })
                .or_else(|| props.leading_icon.get().map(Leading::Icon))
        }),
        trailing: props.trailing_icon,
    };
    chip.assemble(
        nsh_base::button()
            .disabled(move || props.disabled.get())
            .selected(move || props.selected.get())
            .on_press(move || cx.emit(InputChipEvent::Press))
            .on_press_down(move || drawn.press(true))
            .on_press_up(move || drawn.press(false))
            .hit_slop(move || touch_slop(drawn.size())),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nsh_core::geometry::Points;
    use nsh_core::{Color, MountItem};

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, spring_position, started_springs, worst_gap,
    };
    use crate::chip_probe::{
        ALPHA_12, ALPHA_38, CHIP_KT, Config, Kind, Mounted, Painted, cites, down_items, drawn,
        drop_items, faded, morph_in, mount, painted, press_items, solid, start,
    };
    use crate::color::ColorRole;
    use crate::morph::Corners;
    use crate::morph_probe::{
        assert_traces, parameters, paths, replay, rounded_rectangle, set_values, spring_in_with,
        tolerance,
    };

    /// What an input chip paints, with both icons, in the baseline schemes: androidx `InputChipTokens`. Unselected:
    /// transparent, an outline variant outline 1 point wide, the label and icons on surface variant. Selected: a
    /// secondary container, no outline, the label and the trailing icon on secondary container, and the leading icon
    /// primary (the filter chip's is on secondary container). Disabled: on surface at 38 % for the label and icons, at
    /// 12 % for the outline (unselected) or the container (selected).
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
                    leading: solid(ColorRole::Primary, dark),
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
                        Config::of(Kind::Input)
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
        assert_ne!(expected(false, false, false), expected(true, false, false));
    }

    #[test]
    fn its_selected_leading_icon_is_primary_where_the_filter_chips_is_not() {
        // `InputChipTokens.SelectedLeadingIconColor` is primary; `FilterChipTokens`' is on secondary container.
        assert_eq!(
            input_chip_tokens::SELECTED_LEADING_ICON_COLOR,
            ColorRole::Primary
        );
        assert_eq!(
            crate::tokens::androidx::filter_chip_tokens::SELECTED_LEADING_ICON_COLOR,
            ColorRole::OnSecondaryContainer
        );
        let mounted = mount(
            Config::of(Kind::Input)
                .selected(true)
                .leading(true)
                .trailing(true),
        );
        let painted = painted(&mounted);
        assert_eq!(painted.leading, solid(ColorRole::Primary, false));
        assert_ne!(painted.leading, painted.trailing);
    }

    #[test]
    fn its_corners_are_the_chips_not_the_input_chips_container_shape() {
        // `ChipsTokens` (`InputChipDefaults.shapes()`): 12 points unselected, fully rounded selected, 8 pressed. The
        // 8 points of `InputChipTokens.ContainerShape` are the shape of the overload that takes no `ChipShapes`
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
            Corners::of(input_chip_tokens::CONTAINER_SHAPE),
            Some(Corners::Radius(Points(8.0)))
        );
        assert_eq!(rest_shape(false), chips_tokens::UNSELECTED_SHAPE);
        assert_eq!(rest_shape(true), chips_tokens::SELECTED_SHAPE);
        for shape in [
            chips_tokens::UNSELECTED_SHAPE,
            chips_tokens::SELECTED_SHAPE,
            chips_tokens::PRESSED_SHAPE,
        ] {
            assert!(Corners::of(shape).is_some(), "{shape:?}");
        }
        assert_eq!(SHAPE_SPRING, SpringToken::FastSpatial);
        assert_ne!(
            SpringToken::FastSpatial.spec(),
            SpringToken::DefaultEffects.spec()
        );
    }

    /// The rest radius of the corners of a chip selected or not.
    fn rest_radius(selected: bool) -> f32 {
        if selected { 16.0 } else { 12.0 }
    }

    /// The pressed corners: `ChipsTokens.PressedShape`, 8 points.
    const PRESSED_RADIUS: f32 = 8.0;

    /// Nothing of `items` is drawn again or laid out: a press changes the corners and nothing else.
    fn only_the_corners_move(recording: &Recording, items: &[MountItem]) {
        assert!(
            !items
                .iter()
                .any(|item| matches!(item, MountItem::Canvas { .. })),
            "{items:?}"
        );
        assert_eq!(
            Mounted::of(&recording.items()).layout_items(items),
            0,
            "{items:?}"
        );
    }

    #[test]
    fn a_press_springs_the_corners_to_the_pressed_shape_and_the_drawing_follows() {
        for selected in [false, true] {
            let what = format!("selected: {selected}");
            let (mut app, recording) = start(Config::of(Kind::Input).selected(selected));
            let frame = Mounted::of(&recording.items()).size();
            let rest = rest_radius(selected);
            let items = down_items(&mut app, &recording);
            let spring = morph_in(&recording, &items, PRESSED_RADIUS);
            only_the_corners_move(&recording, &items);
            // While it plays, the engine sends nothing (ADR 0010).
            app.step("run");
            app.run_for(Duration::from_secs(1)).unwrap();
            assert!(recording.in_step("run").is_empty(), "{what}");

            let mut replay = replay(&recording);
            let mut samples = replay.run(spring.node, 3);
            // Early on, the container, its outline and the state layer follow the corners.
            let now = replay.value(spring.node);
            assert!(now < rest - 0.1 && now > PRESSED_RADIUS, "{what}: {now}");
            let mounted = Mounted::of(&recording.items());
            let container = paths(mounted.drawing(mounted.container), &replay);
            assert_eq!(container.len(), if selected { 1 } else { 2 }, "{what}");
            assert_traces(&container[0], &rounded_rectangle(frame, 0.0, now), &what);
            let inside = if selected { 0.0 } else { 1.0 };
            if !selected {
                assert_traces(
                    &container[1],
                    &rounded_rectangle(frame, 0.5, now - 0.5),
                    &what,
                );
            }
            let layer = paths(mounted.drawing(mounted.layer), &replay);
            assert_traces(
                &layer[0],
                &rounded_rectangle(frame, inside, now - inside),
                &what,
            );
            // The whole curve is the closed-form spring from the corners at rest, within 0.1 % of its travel.
            samples.extend(replay.run(spring.node, 177));
            let gap = worst_gap(&samples, f64::from(rest), &spring);
            assert!(
                gap < tolerance(f64::from(rest), f64::from(PRESSED_RADIUS)),
                "{what}: {gap}"
            );
            // It settles on the pressed corners exactly, and the graph stops.
            let settled = replay.run(spring.node, 600);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(PRESSED_RADIUS.to_bits()),
                "{what}"
            );
        }
    }

    #[test]
    fn a_release_springs_back_from_where_and_as_fast_as_the_corners_move() {
        for selected in [false, true] {
            let what = format!("selected: {selected}");
            let (mut app, recording) = start(Config::of(Kind::Input).selected(selected));
            let rest = rest_radius(selected);
            let items = down_items(&mut app, &recording);
            let down = morph_in(&recording, &items, PRESSED_RADIUS);
            let mut replay = replay(&recording);
            // The finger lifts 50 ms in, while the corners still move.
            let before = replay.run(down.node, 6);
            let released = drop_items(&mut app, &recording);
            let up = morph_in(&recording, &released, rest);
            only_the_corners_move(&recording, &released);
            assert_eq!(up.node, down.node, "{what}");
            assert!(app.shows("presses: 1"), "{what}");
            replay.apply(&released);
            let after = replay.run(up.node, 180);

            // The graph last ticked 5 frames into the press: the release starts there, at that value and speed.
            let press = parameters(&down);
            let at = 5.0 / REPLAY_HZ;
            let position =
                |t: f64| spring_position(f64::from(rest), f64::from(PRESSED_RADIUS), 0.0, press, t);
            let from = position(at);
            let h = 1e-6;
            let speed = (position(at + h) - position(at - h)) / (2.0 * h);
            assert!(
                (f64::from(before[5]) - from).abs() < 1e-3,
                "{what}: {before:?} {from}"
            );
            assert!(speed < -10.0, "{what}: {speed}");
            let expected = |velocity: f64| {
                after
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| {
                        let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                        let closed =
                            spring_position(from, f64::from(rest), velocity, parameters(&up), t);
                        (f64::from(value) - closed).abs()
                    })
                    .fold(0.0, f64::max)
            };
            let tolerance = tolerance(from, f64::from(rest));
            assert!(expected(speed) < tolerance, "{what}: {}", expected(speed));
            // Starting again from rest, as a new animation would, is another curve.
            assert!(
                expected(0.0) > 10.0 * tolerance,
                "{what}: {}",
                expected(0.0)
            );
            let settled = replay.run(up.node, 600);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(rest.to_bits()),
                "{what}"
            );
        }
    }

    #[test]
    fn pressing_again_and_again_ends_on_the_corners_at_rest_with_nothing_left_running() {
        let (mut app, recording) = start(Config::of(Kind::Input));
        let frame = Mounted::of(&recording.items()).size();
        let mut nodes = Vec::new();
        for _ in 0..5 {
            for items in [
                down_items(&mut app, &recording),
                drop_items(&mut app, &recording),
            ] {
                let springs = started_springs(&items);
                assert_eq!(springs.len(), 1, "{springs:?}");
                nodes.extend(springs.iter().map(|spring| spring.node));
            }
        }
        // One value holds the corners: every press and release moves it, none makes another.
        assert!(nodes.windows(2).all(|pair| pair[0] == pair[1]));
        assert!(app.shows("presses: 5"));
        let mut replay = replay(&recording);
        let settled = replay.run(nodes[0], 600);
        assert!(!replay.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(rest_radius(false).to_bits())
        );
        let mounted = Mounted::of(&recording.items());
        let container = paths(mounted.drawing(mounted.container), &replay);
        assert_traces(
            &container[0],
            &rounded_rectangle(frame, 0.0, rest_radius(false)),
            "container",
        );
    }

    #[test]
    fn with_reduce_motion_a_press_leaves_the_shape_as_it_is() {
        // ADR 0011, "Reduce motion": no spring, and no jump either.
        for selected in [false, true] {
            let what = format!("selected: {selected}");
            let (mut app, recording) = start(
                Config::of(Kind::Input)
                    .selected(selected)
                    .reduce_motion(true),
            );
            let (_, _, frame) = drawn(&recording);
            let at_rest = rounded_rectangle(frame, 0.0, rest_radius(selected));
            let pressed = down_items(&mut app, &recording);
            assert_eq!(started_springs(&pressed), Vec::new(), "{what}");
            assert_eq!(set_values(&pressed), 0, "{what}");
            let (container, _, _) = drawn(&recording);
            assert_traces(&container[0], &at_rest, &what);
            let released = drop_items(&mut app, &recording);
            assert_eq!(started_springs(&released), Vec::new(), "{what}");
            assert_eq!(set_values(&released), 0, "{what}");
            assert!(app.shows("presses: 1"), "{what}");
        }
    }

    #[test]
    fn selecting_during_a_press_leaves_the_pressed_corners_and_the_release_goes_to_the_new_ones() {
        let (mut app, recording) = start(Config::of(Kind::Input));
        let items = down_items(&mut app, &recording);
        let _ = morph_in(&recording, &items, PRESSED_RADIUS);
        // The chip gets selected with the finger down: the corners are the pressed ones and nothing moves.
        let items = press_items(&mut app, &recording, "Select");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 0, "{items:?}");
        // The release goes to the selected corners, not to the unselected ones the press began on.
        let released = drop_items(&mut app, &recording);
        let _ = morph_in(&recording, &released, rest_radius(true));
    }

    #[test]
    fn an_input_chip_disabled_while_pressed_goes_back_to_its_shape_at_rest() {
        // A disabled chip may get no end of its press: it springs back by itself.
        let (mut app, recording) = start(Config::of(Kind::Input));
        let items = down_items(&mut app, &recording);
        let _ = morph_in(&recording, &items, PRESSED_RADIUS);
        let items = press_items(&mut app, &recording, "Disable");
        let _ = spring_in_with(
            &items,
            "Disable",
            rest_radius(false),
            SpringToken::FastSpatial,
        );
        // The only view laid out again is the state layer, which a disabled chip hides.
        let mounted = Mounted::of(&recording.items());
        let laid_out: Vec<_> = items
            .iter()
            .filter_map(|item| match item {
                MountItem::Frame { id, .. } if mounted.frames.contains_key(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(laid_out, [mounted.layer]);
        // The finger lifts on a disabled chip: no press is reported, and nothing moves again.
        let released = drop_items(&mut app, &recording);
        assert_eq!(started_springs(&released), Vec::new());
        assert!(app.shows("presses: 0"));
    }

    #[test]
    fn an_avatar_is_clipped_to_a_circle_of_24_points_and_faded_when_the_chip_is_disabled() {
        // `InputChipTokens.AvatarSize`, `AvatarShape` (`CornerFull`) and `DisabledAvatarOpacity`.
        assert_eq!(input_chip_tokens::AVATAR_SIZE, Points(24.0));
        assert_eq!(
            Corners::of(input_chip_tokens::AVATAR_SHAPE),
            Some(Corners::Full)
        );
        for selected in [false, true] {
            for (disabled, opacity) in [(false, 1.0), (true, 0.38)] {
                let what = format!("selected: {selected} disabled: {disabled}");
                let mounted = mount(
                    Config::of(Kind::Input)
                        .selected(selected)
                        .disabled(disabled)
                        .avatar(true),
                );
                let id = mounted.avatar.expect("an avatar");
                let frame = mounted.frames[&id];
                assert_eq!(
                    (frame.size.width.0, frame.size.height.0),
                    (24.0, 24.0),
                    "{what}"
                );
                let style = mounted.style(id);
                assert!(style.overflow_hidden, "{what}");
                // Rounded as far as 24 points allow: a circle.
                assert!(style.border_radius.0 >= 12.0, "{what}");
                assert_eq!(style.opacity, opacity, "{what}");
            }
        }
    }

    #[test]
    fn an_avatar_takes_the_place_of_the_leading_icon() {
        // `leadingContent`: an avatar takes precedence over an icon, and the icon is not drawn at all.
        let with_both = mount(Config::of(Kind::Input).avatar(true).leading(true));
        assert!(with_both.avatar.is_some());
        assert!(with_both.leading.is_none());
        let icon_only = mount(Config::of(Kind::Input).leading(true));
        assert!(icon_only.avatar.is_none());
        assert!(icon_only.leading.is_some());
        // The spaces are the avatar's: 4 points before it (an icon's are 8), and the same chip width as an avatar alone.
        let avatar_only = mount(Config::of(Kind::Input).avatar(true));
        assert_eq!(with_both.size(), avatar_only.size());
        assert_ne!(with_both.size(), icon_only.size());
    }

    #[test]
    fn a_selected_input_chip_tells_screen_readers_it_is_selected() {
        for selected in [false, true] {
            let mounted = mount(Config::of(Kind::Input).selected(selected));
            assert_eq!(mounted.state.selected, selected);
        }
        // A press is reported, and what it does to `selected` is the app's: the chip shows what it is given.
        let (mut app, recording) = start(Config::of(Kind::Input));
        app.press("Chip").unwrap();
        assert!(app.shows("presses: 1"));
        assert!(!Mounted::of(&recording.items()).state.selected);
        app.press("Select").unwrap();
        assert!(Mounted::of(&recording.items()).state.selected);
    }

    /// The vendored `Chip.kt`'s lines that the input chip's colors and border are made of (ADR 0012).
    #[test]
    fn every_color_and_border_taken_from_androidx_is_still_there() {
        // The Expressive `InputChip` takes the shapes, and the tonal colors by default.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun InputChip(",
                "selected: Boolean,",
                "onClick: () -> Unit,",
                "label: @Composable () -> Unit,",
                "shapes: ChipShapes,",
                "modifier: Modifier = Modifier,",
                "enabled: Boolean = true,",
                "leadingIcon: @Composable (() -> Unit)? = null,",
                "avatar: @Composable (() -> Unit)? = null,",
                "trailingIcon: @Composable (() -> Unit)? = null,",
                "colors: SelectableChipColors = InputChipDefaults.tonalInputChipColors(),",
            ]
        ));
        // The flat colors, and the tonal override of the unselected leading icon.
        assert!(cites(
            CHIP_KT,
            &[
                "containerColor = Color.Transparent,",
                "labelColor = fromToken(InputChipTokens.UnselectedLabelTextColor),",
                "leadingIconColor = fromToken(InputChipTokens.UnselectedLeadingIconColor),",
                "trailingIconColor = fromToken(InputChipTokens.UnselectedTrailingIconColor),",
                "disabledContainerColor = Color.Transparent,",
                "disabledLabelColor =",
                "fromToken(InputChipTokens.DisabledLabelTextColor)",
                ".copy(alpha = InputChipTokens.DisabledLabelTextOpacity),",
                "disabledLeadingIconColor =",
                "fromToken(InputChipTokens.DisabledLeadingIconColor)",
                ".copy(alpha = InputChipTokens.DisabledLeadingIconOpacity),",
                "disabledTrailingIconColor =",
                "fromToken(InputChipTokens.DisabledTrailingIconColor)",
                ".copy(alpha = InputChipTokens.DisabledTrailingIconOpacity),",
                "selectedContainerColor = fromToken(InputChipTokens.SelectedContainerColor),",
                "disabledSelectedContainerColor =",
                "fromToken(InputChipTokens.DisabledSelectedContainerColor)",
                ".copy(alpha = InputChipTokens.DisabledSelectedContainerOpacity),",
                "selectedLabelColor = fromToken(InputChipTokens.SelectedLabelTextColor),",
                "selectedLeadingIconColor =",
                "fromToken(InputChipTokens.SelectedLeadingIconColor),",
                "selectedTrailingIconColor =",
                "fromToken(InputChipTokens.SelectedTrailingIconColor),",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "?: defaultInputChipColors",
                ".copy(leadingIconColor = fromToken(ChipsTokens.UnselectedLeadingIconColor))",
                ".also { defaultTonalInputChipColorsCached = it }",
            ]
        ));
        // The border: transparent when selected, and the unselected disabled outline at the input chip's opacity.
        assert!(cites(
            CHIP_KT,
            &[
                "public fun inputChipBorder(",
                "enabled: Boolean,",
                "selected: Boolean,",
                "borderColor: Color = InputChipTokens.UnselectedOutlineColor.value,",
                "selectedBorderColor: Color = Color.Transparent,",
                "disabledBorderColor: Color =",
                "InputChipTokens.DisabledUnselectedOutlineColor.value.copy(",
                "alpha = InputChipTokens.DisabledUnselectedOutlineOpacity",
                "),",
                "disabledSelectedBorderColor: Color = Color.Transparent,",
                "borderWidth: Dp = InputChipTokens.UnselectedOutlineWidth,",
                "selectedBorderWidth: Dp = InputChipTokens.SelectedOutlineWidth,",
            ]
        ));
        assert_eq!(input_chip_tokens::UNSELECTED_OUTLINE_WIDTH, Points(1.0));
        assert_eq!(
            input_chip_tokens::DISABLED_UNSELECTED_OUTLINE_OPACITY.to_bits(),
            0.12_f32.to_bits()
        );
    }

    /// The vendored `Chip.kt`'s lines that the input chip's shapes and morph are made of (ADR 0012).
    #[test]
    fn every_shape_and_morph_taken_from_androidx_is_still_there() {
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
        // A line that is not there is not found.
        assert!(!cites(
            CHIP_KT,
            &["val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultEffects.value<Float>()"]
        ));
    }

    /// The vendored `Chip.kt`'s lines that the input chip's avatar and spaces are made of (ADR 0012).
    #[test]
    fn every_avatar_and_space_taken_from_androidx_is_still_there() {
        // The avatar: faded when disabled, clipped to the avatar shape, and ahead of the leading icon.
        assert!(cites(
            CHIP_KT,
            &[
                "val avatarOpacity = if (enabled) 1f else InputChipTokens.DisabledAvatarOpacity",
                "val avatarShape = InputChipTokens.AvatarShape.value",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "this.alpha = avatarOpacity",
                "this.shape = avatarShape",
                "this.clip = true",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "avatar != null -> avatar // An avatar takes precedence",
                "leadingIcon != null -> {",
            ]
        ));
        // The spaces: the padding by what the chip has, and the arrangement that is compact next to an icon or avatar.
        assert!(cites(
            CHIP_KT,
            &[
                "val start = if (hasAvatar || !hasLeadingIcon) 4.dp else 8.dp",
                "val end = if (hasTrailingIcon) 8.dp else 4.dp",
                "return PaddingValues(start = start, end = end)",
            ]
        ));
        assert!(cites(
            CHIP_KT,
            &[
                "return if ((hasAvatar || hasLeadingIcon) && hasTrailingIcon)",
                "ChipArrangement(CompactHorizontalSpacing)",
                "else if (hasAvatar || hasLeadingIcon)",
                "ChipArrangement(CompactHorizontalSpacing, HorizontalSpacing)",
                "else if (hasTrailingIcon) ChipArrangement(HorizontalSpacing, CompactHorizontalSpacing)",
                "else DefaultHorizontalArrangement",
            ]
        ));
        assert_eq!(padding_start(true, true), Points(4.0));
        assert_eq!(padding_start(false, false), Points(4.0));
        assert_eq!(padding_start(false, true), Points(8.0));
        assert_eq!(padding_end(true), Points(8.0));
        assert_eq!(padding_end(false), Points(4.0));
        // A line that is not there is not found.
        assert!(!cites(
            CHIP_KT,
            &["val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultEffects.value<Float>()"]
        ));
    }
}
