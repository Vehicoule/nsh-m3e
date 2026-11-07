//! What the four chips share: the drawn container and its outline, the state layer of hover, focus and press, the
//! leading icon or avatar, the label and the trailing icon, laid out as androidx's `Chip.kt` lays them out.
//!
//! Every line number below is a line of `reference/androidx-material3-components/Chip.kt` at the pinned commit
//! (ADR 0012).
//!
//! # Layout
//!
//! androidx lays a chip's content out in a `Row` of three elements (the leading icon or avatar, the label, the
//! trailing icon) with a `ChipArrangement(leading, trailing)` (`ChipArrangement`, lines 3147 to 3183) inside the content
//! padding (`ChipContent` line 3355, `AnimatingChipContent` line 3423). The arrangement puts the label `leading` after
//! the leading element, or, when there is none, `trailing` after the empty one (lines 3167 to 3170); the trailing
//! element is at the end, and so the label ends `trailing` before it when there is a leading element, `leading` before
//! it when there is none (the row's width adds both spaces once, and the label takes what the others leave). That is
//! [`Spaces`]: the content padding at each end, and the label's margin at each end.
//!
//! # What is left out
//!
//! - The elevated chips (`ElevatedAssistChip`, `ElevatedFilterChip`, `ElevatedSuggestionChip`): elevation is decided
//!   by ADR 0018 and written by its own change, not by this one, so these chips have no shadow code and none is
//!   elevated. The flat chips of every kind have no shadow (`FlatContainerElevation` is 0).
//! - The leading and trailing icons' animation (`AnimatingChipContent`, lines 3423 to 3515, expands and fades an icon
//!   in when it comes and goes): it animates the chip's width, which is a layout per frame. An icon comes and goes at
//!   once.
//! - A focus indicator: `Chip.kt` reads none of the `FocusIndicatorColor` tokens, so none is drawn.
//! - A color animation: `Chip.kt` gives its `Surface` the new container color at once.

use nsh_core::LineHeight;
use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::icon::Icon;
use crate::morph::DrawnShape;
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::TypeStyle;
use crate::tokens::androidx::state_tokens;

/// The space between a chip's elements and between them and its edge when nothing else says: `HorizontalSpacing = 8.dp`
/// of `AssistChipDefaults` (line 1956), `FilterChipDefaults` (2235), `InputChipDefaults` (2639) and
/// `SuggestionChipDefaults` (2918). No token holds it.
pub(crate) const SPACING: Points = Points(8.0);

/// The space a chip with an icon takes between the icon and the label: `CompactHorizontalSpacing = 4.dp` of
/// `FilterChipDefaults` (line 2238) and `InputChipDefaults` (2642).
pub(crate) const COMPACT_SPACING: Points = Points(4.0);

/// The padding at each end of a chip's content: `ContentPadding = PaddingValues(horizontal = 8.dp)` of
/// `AssistChipDefaults` (line 1962), `FilterChipDefaults` (2269) and `SuggestionChipDefaults` (2915).
pub(crate) const CONTENT_PADDING: Points = Points(8.0);

/// The arrangement of `FilterChipDefaults.horizontalArrangement(hasLeadingIcon, hasTrailingIcon)` (lines 2244 to 2252)
/// and `InputChipDefaults.horizontalArrangement(..)` (2650 to 2661), where the input chip's leading icon is its avatar
/// or its icon: the spacing after the leading element and the one before the trailing one.
pub(crate) fn compact_arrangement(has_leading: bool, has_trailing: bool) -> (Points, Points) {
    match (has_leading, has_trailing) {
        (true, true) => (COMPACT_SPACING, COMPACT_SPACING),
        (true, false) => (COMPACT_SPACING, SPACING),
        (false, true) => (SPACING, COMPACT_SPACING),
        (false, false) => (SPACING, SPACING),
    }
}

/// What a chip shows in its leading place: its icon, or its avatar, which takes precedence (`leadingContent`, lines
/// 3522 to 3538).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Leading {
    /// An icon, drawn at the leading icon's size in the leading icon color.
    Icon(Icon),
    /// The app's avatar, clipped to the avatar's shape and size, at `opacity` (`InputChip`, lines 1323 to 1341: faded
    /// to `DisabledAvatarOpacity` when the chip is disabled).
    Avatar {
        /// Where the app fills it.
        slot: Slot,
        /// How opaque it is.
        opacity: f32,
    },
}

/// The space around a chip's label, from its content padding and its arrangement (see the module's documentation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Spaces {
    /// The padding before the content.
    pub(crate) start: Points,
    /// The padding after the content.
    pub(crate) end: Points,
    /// The label's margin before it.
    pub(crate) label_start: Points,
    /// The label's margin after it.
    pub(crate) label_end: Points,
}

impl Spaces {
    /// The spaces of a chip with `padding` (before, after) and `ChipArrangement(leading, trailing)`, whose content
    /// has a leading element or not.
    pub(crate) fn new(
        padding: (Points, Points),
        arrangement: (Points, Points),
        has_leading: bool,
    ) -> Self {
        let (leading, trailing) = arrangement;
        let (label_start, label_end) = if has_leading {
            (leading, trailing)
        } else {
            (trailing, leading)
        };
        Self {
            start: padding.0,
            end: padding.1,
            label_start,
            label_end,
        }
    }
}

/// What a chip's container, label and icons are painted with in one state: enabled or disabled, selected or not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Look {
    /// The container's color: none is transparent.
    pub(crate) container: Option<Tint>,
    /// The label's color, and the state layer's: androidx's ripple takes the content color.
    pub(crate) label: Tint,
    /// The leading icon's color.
    pub(crate) leading: Tint,
    /// The trailing icon's color.
    pub(crate) trailing: Tint,
    /// The outline's width and color: none draws nothing, as a `Color.Transparent` border draws nothing.
    pub(crate) outline: Option<(Points, Tint)>,
}

/// The sizes a kind of chip gives the shared parts, from its token file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Metrics {
    /// The container's smallest height.
    pub(crate) height: Points,
    /// The leading icon's side.
    pub(crate) leading_icon: Points,
    /// The trailing icon's side.
    pub(crate) trailing_icon: Points,
    /// The avatar's side.
    pub(crate) avatar: Points,
    /// The label's style.
    pub(crate) label: TypeStyle,
}

/// How far presses also count around a container of `size`, so that a chip smaller than [`MIN_TOUCH_TARGET`] on a
/// side still has a 48-point target there (androidx `minimumInteractiveComponentSize`), without changing its M3 size
/// or the layout (NSH's accessibility guide: extend the touch area with the hit slop, not the size). A chip is 32
/// points tall, and as wide as its label: both sides are taken into account.
pub(crate) fn touch_slop(size: Size<Points>) -> EdgeInsets {
    // Half of what is missing on each side, before and after.
    let missing = |side: Points| px(((MIN_TOUCH_TARGET.0 - side.0) / 2.0).max(0.0));
    let (horizontal, vertical) = (missing(size.width), missing(size.height));
    EdgeInsets {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

/// A chip's parts and what they read, as `Copy` handles: the component makes them, [`Chip::assemble`] puts them on
/// the element that has the chip's behavior.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Chip {
    /// The container, and the corners it moves.
    pub(crate) drawn: DrawnShape,
    /// The kind's sizes.
    pub(crate) metrics: Metrics,
    /// What the chip says.
    pub(crate) label: Prop<SharedString>,
    /// Whether it takes no presses.
    pub(crate) disabled: Prop<bool>,
    /// The colors of the state it is in.
    pub(crate) look: Memo<Look>,
    /// The space around its label.
    pub(crate) spaces: Memo<Spaces>,
    /// What shows before the label.
    pub(crate) leading: Memo<Option<Leading>>,
    /// The icon after the label.
    pub(crate) trailing: Prop<Option<Icon>>,
}

impl Chip {
    /// Gives `base`, the element with the chip's behavior (a button or a toggle), the chip's look: the container and
    /// its outline, the state layer, the leading icon or avatar, the label and the trailing icon, in a row at least
    /// as tall as [`Metrics::height`].
    ///
    /// The state layer is the label's color over the container, inside its outline, in its shape (androidx's
    /// ripple, `RippleDefaults.RippleAlpha`, in the content color), shown by `base`'s own hover, focus and press with
    /// nothing running; a disabled chip takes no press and shows no state.
    pub(crate) fn assemble<E: ParentElement>(self, base: E) -> E {
        let Self {
            drawn,
            metrics,
            label,
            disabled,
            look,
            spaces,
            leading,
            trailing,
        } = self;
        let outline = move || look.get().outline.map(|(width, _)| width);
        let content = move || tint_color(Some(look.get().label));
        base.group()
            .flex_row()
            .items_center()
            .min_h(move || metrics.height)
            .ps(move || spaces.get().start)
            .pe(move || spaces.get().end)
            .child(
                // The container: its color, and its outline drawn inside its edge as a view's border is.
                drawn
                    .container(outline)
                    .bg(move || tint_color(look.get().container))
                    .stroke_color(move || {
                        tint_color(look.get().outline.map(|(_, outline)| outline))
                    }),
            )
            .child(
                drawn
                    .state_layer(outline)
                    .bg(content)
                    .opacity(0.0)
                    .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                    .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                    .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                    .style_with(move |s| {
                        // A disabled chip takes no press and shows no state.
                        if disabled.get() { s.hidden() } else { s }
                    }),
            )
            .child(dynamic(move || {
                leading.get().map(|leading| match leading {
                    Leading::Icon(icon) => icon_view(
                        icon,
                        move || metrics.leading_icon,
                        move || tint_color(Some(look.get().leading)),
                    ),
                    Leading::Avatar { slot, opacity } => slot
                        .element()
                        .size(move || metrics.avatar)
                        .flex_shrink_0()
                        .overflow_hidden()
                        .rounded_full()
                        .opacity(opacity)
                        .into_element(),
                })
            }))
            .child(
                text(move || label.get())
                    .text_color(content)
                    .text_size(move || metrics.label.size)
                    .line_height(move || LineHeight::Points(metrics.label.line_height))
                    .letter_spacing(move || metrics.label.tracking)
                    .style_with(move |s| s.font_weight(metrics.label.weight))
                    .ms(move || spaces.get().label_start)
                    .me(move || spaces.get().label_end),
            )
            .child(dynamic(move || {
                trailing.get().map(|icon| {
                    icon_view(
                        icon,
                        move || metrics.trailing_icon,
                        move || tint_color(Some(look.get().trailing)),
                    )
                })
            }))
    }
}

/// An icon centered at `side`, filled with `color`: a canvas that does not shrink when the label is long.
fn icon_view(
    icon: Icon,
    side: impl Fn() -> Points + Copy + 'static,
    color: impl Fn() -> AdaptiveColor + 'static,
) -> AnyElement {
    canvas(move || icon.drawing(side()))
        .size(side)
        .flex_shrink_0()
        .fill_color(color)
        .into_element()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nsh_core::Checked;
    use nsh_core::canvas::{CanvasOp, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::{MountItem, ViewStyle};

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, layout_items, spring_position, started_springs, worst_gap,
    };
    use crate::chip_probe::{
        Config, KINDS, Kind, Mounted, drawn, morph_in, mount, painted, press_items, start,
    };
    use crate::material_symbols::outlined;
    use crate::morph_probe::{
        assert_traces, parameters, paths, replay, rounded_rectangle, set_values, tolerance,
    };

    /// What a layout puts around the label of each chip, from `Chip.kt` (the module's documentation): the case, then
    /// the padding before the content, the leading element's side (an icon's 18 points, an avatar's 24), the label's
    /// margins before and after it, the trailing icon's side, and the padding after the content.
    #[derive(Debug, Clone, Copy)]
    struct Case {
        kind: Kind,
        leading: bool,
        trailing: bool,
        avatar: bool,
        spaces: [f32; 6],
    }

    const fn case(
        kind: Kind,
        (leading, trailing, avatar): (bool, bool, bool),
        spaces: [f32; 6],
    ) -> Case {
        Case {
            kind,
            leading,
            trailing,
            avatar,
            spaces,
        }
    }

    /// Chip.kt: `AssistChipDefaults` and `SuggestionChipDefaults`, `ContentPadding` 8 and `horizontalArrangement()`
    /// 8 and 8, whatever the icons; `FilterChipDefaults` with `ContentPadding` 8 and `horizontalArrangement(
    /// hasLeadingIcon, hasTrailingIcon)`; `InputChipDefaults` with `contentPadding(..)` and its arrangement.
    const CASES: [Case; 17] = [
        case(
            Kind::Assist,
            (false, false, false),
            [8.0, 0.0, 8.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Assist,
            (true, false, false),
            [8.0, 18.0, 8.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Assist,
            (false, true, false),
            [8.0, 0.0, 8.0, 8.0, 18.0, 8.0],
        ),
        case(
            Kind::Assist,
            (true, true, false),
            [8.0, 18.0, 8.0, 8.0, 18.0, 8.0],
        ),
        case(
            Kind::Suggestion,
            (false, false, false),
            [8.0, 0.0, 8.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Suggestion,
            (true, false, false),
            [8.0, 18.0, 8.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Filter,
            (false, false, false),
            [8.0, 0.0, 8.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Filter,
            (true, false, false),
            [8.0, 18.0, 4.0, 8.0, 0.0, 8.0],
        ),
        case(
            Kind::Filter,
            (false, true, false),
            [8.0, 0.0, 4.0, 8.0, 18.0, 8.0],
        ),
        case(
            Kind::Filter,
            (true, true, false),
            [8.0, 18.0, 4.0, 4.0, 18.0, 8.0],
        ),
        case(
            Kind::Input,
            (false, false, false),
            [4.0, 0.0, 8.0, 8.0, 0.0, 4.0],
        ),
        case(
            Kind::Input,
            (true, false, false),
            [8.0, 18.0, 4.0, 8.0, 0.0, 4.0],
        ),
        case(
            Kind::Input,
            (false, false, true),
            [4.0, 24.0, 4.0, 8.0, 0.0, 4.0],
        ),
        case(
            Kind::Input,
            (false, true, false),
            [4.0, 0.0, 4.0, 8.0, 18.0, 8.0],
        ),
        case(
            Kind::Input,
            (true, true, false),
            [8.0, 18.0, 4.0, 4.0, 18.0, 8.0],
        ),
        case(
            Kind::Input,
            (false, true, true),
            [4.0, 24.0, 4.0, 4.0, 18.0, 8.0],
        ),
        // An avatar takes precedence over a leading icon (`leadingContent`): the icon is not drawn, and the spaces
        // are those of the avatar alone.
        case(
            Kind::Input,
            (true, false, true),
            [4.0, 24.0, 4.0, 8.0, 0.0, 4.0],
        ),
    ];

    #[test]
    fn each_chip_shows_the_views_its_icons_and_avatar_ask_for() {
        for Case {
            kind,
            leading,
            trailing,
            avatar,
            ..
        } in CASES
        {
            let what = format!("{kind:?} leading: {leading} trailing: {trailing} avatar: {avatar}");
            let mounted = mount(
                Config::of(kind)
                    .leading(leading)
                    .trailing(trailing)
                    .avatar(avatar),
            );
            assert_eq!(
                mounted.leading.is_some(),
                leading && !avatar,
                "{what}: the leading icon, unless an avatar takes its place"
            );
            assert_eq!(mounted.avatar.is_some(), avatar, "{what}");
            assert_eq!(mounted.trailing.is_some(), trailing, "{what}");
        }
        // The suggestion chip has no trailing icon, the others no more avatar than the input chip.
        for kind in [Kind::Suggestion, Kind::Assist, Kind::Filter] {
            let mounted = mount(
                Config::of(kind)
                    .trailing(kind == Kind::Suggestion)
                    .avatar(true),
            );
            assert!(mounted.avatar.is_none(), "{kind:?}");
            assert!(mounted.trailing.is_none(), "{kind:?}");
        }
    }

    #[test]
    fn each_chip_is_laid_out_as_chip_kt_lays_it_out() {
        for Case {
            kind,
            leading,
            trailing,
            avatar,
            spaces:
                [
                    pad_start,
                    leading_side,
                    label_start,
                    label_end,
                    trailing_side,
                    pad_end,
                ],
        } in CASES
        {
            let what = format!("{kind:?} leading: {leading} trailing: {trailing} avatar: {avatar}");
            let mounted = mount(
                Config::of(kind)
                    .leading(leading)
                    .trailing(trailing)
                    .avatar(avatar),
            );
            let label = mounted.frames[&mounted.label];
            let label_x = pad_start + leading_side + label_start;
            assert_eq!(label.origin.x.0, label_x, "{what}: the label's start");
            let trailing_x = label_x + label.size.width.0 + label_end;
            assert_eq!(
                mounted.size().width.0,
                trailing_x + trailing_side + pad_end,
                "{what}: the chip's width"
            );
            // 32 points tall, the content centered in it.
            assert_eq!(mounted.size().height.0, 32.0, "{what}");
            let centre = |frame: Bounds<Points>| frame.origin.y.0 + frame.size.height.0 / 2.0;
            assert!((centre(label) - 16.0).abs() <= 0.5, "{what}: {label:?}");
            if let Some(id) = mounted.leading.or(mounted.avatar) {
                let frame = mounted.frames[&id];
                assert_eq!(frame.origin.x.0, pad_start, "{what}");
                assert_eq!(frame.size.width.0, leading_side, "{what}");
                assert_eq!(frame.size.height.0, leading_side, "{what}");
                assert_eq!(centre(frame), 16.0, "{what}");
            }
            if let Some(id) = mounted.trailing {
                let frame = mounted.frames[&id];
                assert_eq!(frame.origin.x.0, trailing_x, "{what}");
                assert_eq!(frame.size.width.0, 18.0, "{what}");
                assert_eq!(frame.size.height.0, 18.0, "{what}");
                assert_eq!(centre(frame), 16.0, "{what}");
            }
            // The container and the state layer cover the chip, whatever is in it.
            for id in [mounted.container, mounted.layer] {
                assert_eq!(
                    mounted.frames[&id],
                    Bounds::new(point(px(0.0), px(0.0)), mounted.size()),
                    "{what}"
                );
            }
        }
    }

    #[test]
    fn the_label_is_label_large_in_the_chips_label_color() {
        for kind in KINDS {
            let mounted = mount(Config::of(kind));
            let style = mounted.text.as_ref().expect("the label's style");
            // androidx `TypographyTokens.LabelLarge`, which every chip token file names: 14 points, 20 tall, medium
            // (500), 0.1 apart.
            assert_eq!(
                (
                    style.font_size.0,
                    style.line_height.0,
                    style.font_weight,
                    style.letter_spacing.0
                ),
                (14.0, 20.0, 500, 0.1),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn every_chip_has_a_48_point_touch_target_without_growing() {
        for kind in KINDS {
            // A label as long as a word, and one as short as a letter, which makes a chip narrower than 48 points.
            for label in ["Chip", "A"] {
                let what = format!("{kind:?} {label}");
                let mounted = mount(Config::of(kind).label(label));
                let size = mounted.size();
                let slop = mounted.hit_slop;
                assert_eq!(size.height.0, 32.0, "{what}: the layout keeps its height");
                // 32 points tall: 8 points above and below.
                assert_eq!((slop.top.0, slop.bottom.0), (8.0, 8.0), "{what}");
                // Each side reaches 48 points with the slop, and a side already 48 or more gets none.
                let reach = ((48.0 - size.width.0) / 2.0).max(0.0);
                assert_eq!((slop.left.0, slop.right.0), (reach, reach), "{what}");
                assert!(size.width.0 + slop.left.0 + slop.right.0 >= 48.0, "{what}");
            }
        }
        // A letter's chip is narrower than 48 points and gets slop at its sides; a word's does not.
        let narrow = mount(Config::of(Kind::Assist).label("A"));
        assert!(narrow.size().width.0 < 48.0 && narrow.hit_slop.left.0 > 0.0);
        let wide = mount(Config::of(Kind::Assist));
        assert!(wide.size().width.0 > 48.0 && wide.hit_slop.left.0 == 0.0);
    }

    #[test]
    fn the_touch_slop_counts_what_each_side_is_missing() {
        let slop = touch_slop(Size::new(px(40.0), px(32.0)));
        assert_eq!((slop.left.0, slop.top.0), (4.0, 8.0));
        let slop = touch_slop(Size::new(px(48.0), px(56.0)));
        assert_eq!((slop.left.0, slop.top.0), (0.0, 0.0));
        // A chip is never given a negative slop.
        let slop = touch_slop(Size::new(px(300.0), px(60.0)));
        assert_eq!((slop.right.0, slop.bottom.0), (0.0, 0.0));
    }

    #[test]
    fn screen_readers_get_a_button_that_says_what_the_chip_is_in() {
        for kind in KINDS {
            for disabled in [false, true] {
                let mounted = mount(Config::of(kind).disabled(disabled));
                assert_eq!(mounted.state.disabled, disabled, "{kind:?}");
                assert!(
                    !mounted.state.selected,
                    "{kind:?}: only the input chip says selected, and not while it is not"
                );
            }
        }
        // The filter chip is a toggle button: on or off, never selected as a tab is.
        let off = mount(Config::of(Kind::Filter));
        assert_eq!(off.state.pressed, Some(Checked::False));
        let on = mount(Config::of(Kind::Filter).selected(true));
        assert_eq!(on.state.pressed, Some(Checked::True));
        assert!(!on.state.selected);
        // The input chip is selected; the assist and suggestion chips are neither.
        let input = mount(Config::of(Kind::Input).selected(true));
        assert!(input.state.selected);
        assert_eq!(input.state.pressed, None);
        for kind in [Kind::Assist, Kind::Suggestion] {
            let mounted = mount(Config::of(kind));
            assert_eq!(mounted.state.pressed, None, "{kind:?}");
            assert!(!mounted.state.selected, "{kind:?}");
        }
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        for kind in KINDS {
            let (mut app, recording) = start(Config::of(kind));
            app.press("Chip")?;
            assert!(app.shows("presses: 1"), "{kind:?}");
            assert!(!app.shows("presses: 2"), "{kind:?}");
            if kind == Kind::Filter {
                // A filter chip asks for the opposite, and shows what the app passes back.
                assert_eq!(
                    Mounted::of(&recording.items()).state.pressed,
                    Some(Checked::True)
                );
                app.press("Chip")?;
                assert!(app.shows("presses: 2"));
                assert_eq!(
                    Mounted::of(&recording.items()).state.pressed,
                    Some(Checked::False)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn a_disabled_chip_takes_no_press_and_shows_no_state() {
        for kind in KINDS {
            let (mut app, recording) = start(Config::of(kind).disabled(true));
            let _ = app.press("Chip");
            assert!(app.shows("presses: 0"), "{kind:?}");
            let mounted = Mounted::of(&recording.items());
            // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
            assert_eq!(
                mounted.frames.get(&mounted.layer).map(|frame| frame.size),
                Some(Size::default()),
                "{kind:?}"
            );
            // The container keeps its size: only the container's canvas reports it, not the hidden state layer.
            assert_eq!(
                mounted
                    .frames
                    .get(&mounted.container)
                    .map(|frame| frame.size),
                Some(mounted.size()),
                "{kind:?}"
            );
            assert_eq!(mounted.size().height.0, 32.0, "{kind:?}");
            // Enabled, its state layer is there.
            let enabled = mount(Config::of(kind));
            assert_eq!(
                enabled.frames.get(&enabled.layer).map(|frame| frame.size),
                Some(enabled.size()),
                "{kind:?}"
            );
        }
    }

    /// The opacity the state layer gets in `items`, if its style was sent.
    fn layer_opacities(items: &[MountItem], mounted: &Mounted) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn hover_shows_the_state_layer_in_the_label_color_and_lays_out_nothing() -> Result<(), String> {
        for kind in KINDS {
            // A selected chip's label color is another one, and so is its state layer's.
            for selected in [false, true] {
                let what = format!("{kind:?} selected: {selected}");
                let (mut app, recording) = start(Config::of(kind).selected(selected));
                let mounted = Mounted::of(&recording.items());
                assert_eq!(mounted.style(mounted.layer).opacity, 0.0, "{what}");
                app.hover("Chip")?;
                let hover = recording.in_step("hover(Chip)");
                // The state layer at `StateTokens.HoverStateLayerOpacity`, and no other style at all.
                assert_eq!(layer_opacities(&hover, &mounted), [0.08], "{what}");
                let painted = painted(&mounted);
                assert_eq!(painted.layer, painted.label, "{what}");
                // Nothing else: no frame, and neither the chip, its container nor its icons are styled or drawn
                // again.
                assert_eq!(layout_items(&hover), 0, "{what}");
                assert!(
                    hover.iter().all(|item| match item {
                        MountItem::ViewStyle { id, .. } | MountItem::Canvas { id, .. } =>
                            *id == mounted.layer,
                        _ => true,
                    }),
                    "{what}: {hover:?}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn the_container_its_outline_and_the_state_layer_share_the_chips_corners() {
        // The corners at rest: 8 points for the assist and suggestion chips, 12 for the others, fully rounded (16)
        // when they are selected. The outline is 1 point wide and stroked inside the edge, the state layer inside it;
        // a selected chip has no outline, and its state layer reaches the edge.
        for (kind, selected, radius, outline) in [
            (Kind::Assist, false, 8.0, true),
            (Kind::Suggestion, false, 8.0, true),
            (Kind::Filter, false, 12.0, true),
            (Kind::Filter, true, 16.0, false),
            (Kind::Input, false, 12.0, true),
            (Kind::Input, true, 16.0, false),
        ] {
            let what = format!("{kind:?} selected: {selected}");
            let (_app, recording) = start(
                Config::of(kind)
                    .selected(selected)
                    .leading(true)
                    .trailing(kind != Kind::Suggestion),
            );
            let (container, layer, frame) = drawn(&recording);
            let mut expected = vec![rounded_rectangle(frame, 0.0, radius)];
            if outline {
                expected.push(rounded_rectangle(frame, 0.5, radius - 0.5));
            }
            assert_eq!(container.len(), expected.len(), "{what}: {container:?}");
            for (path, expected) in container.iter().zip(&expected) {
                assert_traces(path, expected, &what);
            }
            let inside = if outline { 1.0 } else { 0.0 };
            assert_eq!(layer.len(), 1, "{what}: {layer:?}");
            assert_traces(
                &layer[0],
                &rounded_rectangle(frame, inside, radius - inside),
                &what,
            );
            let painted = painted(&Mounted::of(&recording.items()));
            assert_eq!(
                painted.outline_width,
                if outline { 1.0 } else { 0.0 },
                "{what}"
            );
        }
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for kind in KINDS {
            for selected in [false, true] {
                let (_app, recording) = start(Config::of(kind).selected(selected));
                let items = recording.items();
                assert_eq!(
                    started_springs(&items),
                    Vec::new(),
                    "{kind:?} selected: {selected}"
                );
                // The corners are created where they rest: the layout sets nothing again.
                assert_eq!(set_values(&items), 0, "{kind:?} selected: {selected}");
                assert!(
                    !replay(&recording).is_animating(),
                    "{kind:?} selected: {selected}"
                );
            }
        }
    }

    /// The fill operations of `ops`.
    fn fills(ops: &[CanvasOp]) -> Vec<CanvasOp> {
        ops.iter()
            .filter(|op| matches!(op, CanvasOp::Fill(..)))
            .cloned()
            .collect()
    }

    #[test]
    fn an_icon_is_its_path_at_18_points_in_its_color() {
        let constant = |n: &Num| match n {
            Num::Const(v) => Some(*v),
            Num::Slot(_) => None,
        };
        for kind in KINDS {
            let mounted = mount(
                Config::of(kind)
                    .leading(true)
                    .trailing(kind != Kind::Suggestion),
            );
            let leading_icon = if matches!(kind, Kind::Filter | Kind::Input) {
                outlined::CHECK
            } else {
                outlined::ADD
            };
            let mut icons = vec![(mounted.leading, leading_icon)];
            if kind != Kind::Suggestion {
                icons.push((mounted.trailing, outlined::CLOSE));
            }
            for (id, icon) in icons {
                let ops = &mounted.drawing(id.expect("an icon")).ops;
                // The generated icons' 960-unit view box, scaled to the icon's 18 points.
                let scale = 18.0 / 960.0;
                let expected = icon.drawing(Points(18.0)).finish();
                assert_eq!(fills(ops), fills(&expected.ops), "{kind:?}");
                assert_eq!(fills(ops).len(), 1, "{kind:?}");
                assert!(
                    ops.iter().any(|op| matches!(op, CanvasOp::Scale(x, y)
                        if constant(x).is_some_and(|x| (x - scale).abs() < 1e-6)
                            && constant(y).is_some_and(|y| (y - scale).abs() < 1e-6))),
                    "{kind:?}: {ops:?}"
                );
            }
        }
    }

    #[test]
    fn an_icon_comes_and_goes_at_once_with_nothing_running() -> Result<(), String> {
        for kind in KINDS {
            let (mut app, recording) = start(Config::of(kind));
            let before = Mounted::of(&recording.items());
            assert!(before.leading.is_none(), "{kind:?}");
            app.press("Icon")?;
            let items = recording.in_step("press(Icon)");
            let mounted = Mounted::of(&recording.items());
            assert!(mounted.leading.is_some(), "{kind:?}");
            // 18 points of icon and the label's space after it: the chip grows by that, laid out once, no motion.
            let label = mounted.frames[&mounted.label].origin.x.0;
            assert!(label > before.frames[&before.label].origin.x.0, "{kind:?}");
            assert_eq!(started_springs(&items), Vec::new(), "{kind:?}");
            assert_eq!(set_values(&items), 0, "{kind:?}");
            assert!(!replay(&recording).is_animating(), "{kind:?}");
            // The container follows the chip's new width.
            assert_eq!(
                mounted.frames[&mounted.container].size,
                mounted.size(),
                "{kind:?}"
            );
            app.press("Icon")?;
            assert!(
                Mounted::of(&recording.items()).leading.is_none(),
                "{kind:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn the_state_layer_color_is_the_chips_label_color_in_every_state() {
        for kind in KINDS {
            for (selected, disabled, dark) in [
                (false, false, false),
                (true, false, false),
                (false, false, true),
                (true, false, true),
                (false, true, false),
                (true, true, true),
            ] {
                let mounted = mount(
                    Config::of(kind)
                        .selected(selected)
                        .disabled(disabled)
                        .dark(dark),
                );
                let painted = painted(&mounted);
                assert_eq!(
                    painted.layer, painted.label,
                    "{kind:?} selected: {selected} disabled: {disabled} dark: {dark}"
                );
            }
        }
    }

    #[test]
    fn a_style_of_the_chip_paints_no_background_or_border_under_its_canvases() {
        for kind in KINDS {
            let mounted = mount(
                Config::of(kind)
                    .selected(true)
                    .leading(true)
                    .trailing(kind != Kind::Suggestion),
            );
            for id in [mounted.button, mounted.container, mounted.layer] {
                let style: ViewStyle = mounted.style(id);
                assert_eq!(style.borders.widths.top, px(0.0), "{kind:?}");
                if id != mounted.container && id != mounted.layer {
                    assert_eq!(
                        style.background_color,
                        nsh_core::Color::TRANSPARENT,
                        "{kind:?}"
                    );
                }
            }
        }
    }

    /// The corners the filter and input chips rest on: `ChipsTokens.UnselectedShape` (12 points) and `SelectedShape`
    /// (fully rounded, half of the 32 points).
    const UNSELECTED_RADIUS: f32 = 12.0;
    const SELECTED_RADIUS: f32 = 16.0;

    /// The chips that morph when they are selected.
    const SELECTABLE: [Kind; 2] = [Kind::Filter, Kind::Input];

    fn rest_radius(selected: bool) -> f32 {
        if selected {
            SELECTED_RADIUS
        } else {
            UNSELECTED_RADIUS
        }
    }

    #[test]
    fn selecting_springs_the_corners_to_the_new_shape_and_the_drawing_follows() {
        for kind in SELECTABLE {
            for selected in [false, true] {
                let what = format!("{kind:?} selected: {selected}");
                let (mut app, recording) = start(Config::of(kind).selected(selected));
                let frame = Mounted::of(&recording.items()).size();
                let (from, to) = (rest_radius(selected), rest_radius(!selected));
                let items = press_items(&mut app, &recording, "Select");
                let spring = morph_in(&recording, &items, to);
                // While it plays, the engine sends nothing (ADR 0010).
                app.step("run");
                app.run_for(Duration::from_secs(1)).unwrap();
                assert!(recording.in_step("run").is_empty(), "{what}");

                let mut replay = replay(&recording);
                let mut samples = replay.run(spring.node, 3);
                // Early on, the corners are on their way, and the container and the state layer follow them. A chip
                // that gets selected loses its outline at once; one that gets unselected gains it.
                let now = replay.value(spring.node);
                assert!(
                    (now - from) * (to - from) > 0.0 && (now - from).abs() > 0.1,
                    "{what}: {now}"
                );
                let mounted = Mounted::of(&recording.items());
                let container = paths(mounted.drawing(mounted.container), &replay);
                assert_eq!(container.len(), if selected { 2 } else { 1 }, "{what}");
                assert_traces(&container[0], &rounded_rectangle(frame, 0.0, now), &what);
                if selected {
                    assert_traces(
                        &container[1],
                        &rounded_rectangle(frame, 0.5, now - 0.5),
                        &what,
                    );
                }
                let layer = paths(mounted.drawing(mounted.layer), &replay);
                let inside = if selected { 1.0 } else { 0.0 };
                assert_traces(
                    &layer[0],
                    &rounded_rectangle(frame, inside, now - inside),
                    &what,
                );
                // The whole curve is the closed-form spring from the corners at rest, within 0.1 % of its travel.
                samples.extend(replay.run(spring.node, 177));
                let gap = worst_gap(&samples, f64::from(from), &spring);
                assert!(
                    gap < tolerance(f64::from(from), f64::from(to)),
                    "{what}: {gap}"
                );
                // It settles on the new corners exactly, and the graph stops.
                let settled = replay.run(spring.node, 600);
                assert!(!replay.is_animating(), "{what}");
                assert_eq!(
                    settled.last().map(|v| v.to_bits()),
                    Some(to.to_bits()),
                    "{what}"
                );
            }
        }
    }

    #[test]
    fn a_spring_that_overshoots_the_full_corners_is_drawn_no_rounder_than_they_can_be() {
        // `FastSpatial` bounces (its damping ratio is 0.6): the radius goes past half of the height, and the path
        // keeps the corners within the container.
        for kind in SELECTABLE {
            let (mut app, recording) = start(Config::of(kind));
            let frame = Mounted::of(&recording.items()).size();
            let items = press_items(&mut app, &recording, "Select");
            let spring = morph_in(&recording, &items, SELECTED_RADIUS);
            let mounted = Mounted::of(&recording.items());
            let mut replay = replay(&recording);
            let mut overshot = 0;
            for frame_number in 0..120 {
                replay.run(spring.node, 1);
                let value = replay.value(spring.node);
                let container = paths(mounted.drawing(mounted.container), &replay);
                assert_traces(
                    &container[0],
                    &rounded_rectangle(frame, 0.0, value.min(SELECTED_RADIUS)),
                    &format!("{kind:?} frame {frame_number}: {value}"),
                );
                overshot += usize::from(value > SELECTED_RADIUS);
            }
            assert!(
                overshot > 0,
                "{kind:?}: the spring never went past the corners"
            );
        }
    }

    #[test]
    fn unselecting_springs_back_from_where_and_as_fast_as_the_corners_move() {
        for kind in SELECTABLE {
            let (mut app, recording) = start(Config::of(kind));
            let items = press_items(&mut app, &recording, "Select");
            let down = morph_in(&recording, &items, SELECTED_RADIUS);
            let mut replay = replay(&recording);
            // The selection is undone 50 ms in, while the corners still move.
            let before = replay.run(down.node, 6);
            let items = press_items(&mut app, &recording, "Select");
            let up = morph_in(&recording, &items, UNSELECTED_RADIUS);
            assert_eq!(up.node, down.node, "{kind:?}");
            replay.apply(&items);
            let after = replay.run(up.node, 180);

            // The graph last ticked 5 frames into the first spring: the second starts there, at that value and that
            // speed.
            let selecting = parameters(&down);
            let at = 5.0 / REPLAY_HZ;
            let position = |t: f64| {
                spring_position(
                    f64::from(UNSELECTED_RADIUS),
                    f64::from(SELECTED_RADIUS),
                    0.0,
                    selecting,
                    t,
                )
            };
            let from = position(at);
            let h = 1e-6;
            let speed = (position(at + h) - position(at - h)) / (2.0 * h);
            assert!(
                (f64::from(before[5]) - from).abs() < 1e-3,
                "{kind:?}: {before:?} {from}"
            );
            assert!(speed > 10.0, "{kind:?}: {speed}");
            let expected = |velocity: f64| {
                after
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| {
                        let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                        let closed = spring_position(
                            from,
                            f64::from(UNSELECTED_RADIUS),
                            velocity,
                            parameters(&up),
                            t,
                        );
                        (f64::from(value) - closed).abs()
                    })
                    .fold(0.0, f64::max)
            };
            let tolerance = tolerance(from, f64::from(UNSELECTED_RADIUS));
            assert!(expected(speed) < tolerance, "{kind:?}: {}", expected(speed));
            // Starting again from rest, as a new animation would, is another curve.
            assert!(
                expected(0.0) > 10.0 * tolerance,
                "{kind:?}: {}",
                expected(0.0)
            );
            let settled = replay.run(up.node, 600);
            assert!(!replay.is_animating(), "{kind:?}");
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(UNSELECTED_RADIUS.to_bits()),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn selecting_again_and_again_ends_on_the_last_corners_with_nothing_left_running() {
        for kind in SELECTABLE {
            let (mut app, recording) = start(Config::of(kind));
            let frame = Mounted::of(&recording.items()).size();
            let mut nodes = Vec::new();
            for _ in 0..7 {
                let items = press_items(&mut app, &recording, "Select");
                let springs = started_springs(&items);
                assert_eq!(springs.len(), 1, "{kind:?}: {springs:?}");
                nodes.extend(springs.iter().map(|spring| spring.node));
            }
            // One value holds the corners: every press moves it, none makes another.
            assert!(nodes.windows(2).all(|pair| pair[0] == pair[1]), "{kind:?}");
            // Seven presses leave the chip selected.
            let mut replay = replay(&recording);
            let settled = replay.run(nodes[0], 600);
            assert!(!replay.is_animating(), "{kind:?}");
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(SELECTED_RADIUS.to_bits()),
                "{kind:?}"
            );
            let mounted = Mounted::of(&recording.items());
            let container = paths(mounted.drawing(mounted.container), &replay);
            assert_traces(
                &container[0],
                &rounded_rectangle(frame, 0.0, SELECTED_RADIUS),
                &format!("{kind:?}"),
            );
        }
    }

    #[test]
    fn with_reduce_motion_selecting_changes_the_corners_at_once() {
        // The shape shows the state, so it cannot stay as it was (ADR 0011, "Reduce motion"): no spring, a jump.
        for kind in SELECTABLE {
            for selected in [false, true] {
                let what = format!("{kind:?} selected: {selected}");
                let (mut app, recording) =
                    start(Config::of(kind).selected(selected).reduce_motion(true));
                let frame = Mounted::of(&recording.items()).size();
                let items = press_items(&mut app, &recording, "Select");
                assert_eq!(started_springs(&items), Vec::new(), "{what}");
                assert_eq!(set_values(&items), 1, "{what}: {items:?}");
                let (container, layer, _) = drawn(&recording);
                let to = rest_radius(!selected);
                let inside = if selected { 1.0 } else { 0.0 };
                assert_traces(&container[0], &rounded_rectangle(frame, 0.0, to), &what);
                assert_traces(
                    &layer[0],
                    &rounded_rectangle(frame, inside, to - inside),
                    &what,
                );
                assert!(!replay(&recording).is_animating(), "{what}");
            }
        }
    }

    #[test]
    fn a_state_that_does_not_change_the_corners_does_not_move_them() {
        // Hovering, disabling and giving an icon change what the chip shows, not its shape: nothing is sprung or set.
        for kind in SELECTABLE {
            let (mut app, recording) = start(Config::of(kind));
            app.hover("Chip").unwrap();
            for selector in ["Icon", "Disable"] {
                let items = press_items(&mut app, &recording, selector);
                assert_eq!(started_springs(&items), Vec::new(), "{kind:?} {selector}");
                assert_eq!(set_values(&items), 0, "{kind:?} {selector}");
            }
            assert_eq!(started_springs(&recording.items()), Vec::new(), "{kind:?}");
        }
    }
}
