//! The Material 3 Expressive list item: a headline with an optional overline and supporting text, leading and
//! trailing content, a container that takes the shape of its state, and, for [`ClickableListItem`], the state layer of
//! hover, focus and press.
//!
//! The list item follows androidx's Expressive `ListItem` in `ListItem.kt` and `ListItemDefaults.kt`, at the pinned
//! commit, as [ADR 0012] says: `ListItem(modifier, …)` is [`ListItem`], and `ListItem(onClick, …)` and
//! `ListItem(selected, onClick, …)` are [`ClickableListItem`], whose `selected` is `None` for the first and `Some` for
//! the second. Its values are the ones of `ListTokens.kt`. Its container is drawn by a canvas whose corner radius the
//! M3E spring moves, as the button's is ([ADR 0011]); its icons are drawn from path data ([ADR 0017]).
//!
//! # What androidx does, and what this list item does in its place
//!
//! * The container is one of five shapes, and takes the first that holds, as `shapeForInteraction` says
//!   (`ListItemDefaults.kt`, line 1136): the pressed shape, the selected one, the focused one, the hovered one, the
//!   shape at rest. A disabled item has no press, focus or hover, so it is selected or at rest. The corners move to
//!   the next shape with `MotionSchemeKeyTokens.FastSpatial` (`ListItem.kt`, line 1504), which overshoots, and a
//!   change of shape in the middle of a move goes on from where the corners are and as fast as they move.
//! * The colors depend on whether the item is enabled and selected, never on hover or press, and a disabled item has no
//!   container of its own: `ListItemDefaults.kt` gives it `ItemContainerColor` (line 215), not the disabled tokens
//!   `ListTokens.kt` also holds. They are the item's parts' colors: headline, overline, supporting text, leading and
//!   trailing content (`defaultListItemColors`, lines 192 to 243). androidx changes them with `DefaultEffects`; here
//!   they change at once.
//! * The item is 56, 72 or 88 points tall at least, as it has one, two or three lines (`ListItem.kt`, line 1952): an
//!   overline and supporting text, or supporting text that wraps, make three. androidx tells that the supporting text
//!   wraps by its baselines; NSH gives no baselines, so this item takes the 30 sp that androidx's own estimate
//!   compares the supporting text's height with (`ListItem.kt`, line 1294), scaled by the user's text size.
//! * Its content is aligned to the center when the space inside its padding is less than a breakpoint, and to the top
//!   from there (`ListItemDefaults.kt`, line 551; `ListItem.kt`, line 1999).
//! * Its role for screen readers: androidx gives a plain list item none (`ListItem.kt`, line 211) and a selectable one
//!   `Role.RadioButton` (lines 452 and 777). NSH has no list item role at the pinned revision: its roles for lists are
//!   `List`, for the list itself, `Listbox` and `Option`, for the choices of a popup, and `Row` and `Cell`, for the
//!   rows and cells of a table. This item takes androidx's, in NSH's names: none for [`ListItem`], the button role of
//!   `nsh-base`'s button for [`ClickableListItem`], and `Radio` once it has a `selected`. `Option` would say that the
//!   item is a choice of a listbox, and `Row` and `Cell` that it is in a table, which an app's list of list items is
//!   neither.
//!
//! # Not ported
//!
//! * `SegmentedListItem` and `ListItemDefaults.segmentedShapes`: their shapes have uneven corners, which the drawn
//!   container does not draw (ADR 0011). ADR 0021 proposes how it could.
//! * The dragged state, its shape, colors and elevation: `nsh-base` has no dragging that reports it. The
//!   `ListItem(checked, onCheckedChange, …)` overload, whose role is a checkbox, and `onLongClick`: `nsh-base`'s
//!   button reports presses only.
//! * Elevation: the list item's tokens give `ElevationTokens.Level0` at rest, which draws nothing, and `Level4` dragged,
//!   a state `nsh-base` does not report. The item draws no shadow, so it does not use [`crate::elevation`] (ADR 0018).
//! * The focus ring of androidx's ripple (`ripple(focusRingShape = shape)`, `ListItem.kt`, line 1606): the focused
//!   item takes its focused shape and the state layer.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::prelude::*;
use nsh_core::{AccessibilityRole, LineHeight};

use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::motion::SpringToken;
use crate::tint::{Tint, tint_color};
use crate::tokens::androidx::{list_tokens, state_tokens};
use crate::tokens::{Shape, TypeStyle};

/// How tall a list item is at least, in the unit of text sizes, for its supporting text to count as wrapping: androidx
/// `ListItem.kt`, line 1294, `estimatedSupportingHeight > 30.sp.roundToPx()`. No token holds it.
const SUPPORTING_WRAP_HEIGHT: f32 = 30.0;

/// The space above and below the content: `ListItemDefaults.ContentPadding`'s top and bottom
/// (`ListItemDefaults.kt`, lines 51 to 66, with the precision pointer sizing that androidx has off).
const VERTICAL_PADDING: Points =
    Points(list_tokens::ITEM_TOP_SPACE.0 + list_tokens::ITEM_BOTTOM_SPACE.0);

/// How tall the space inside the padding has to be for the content to be aligned to the top, not the center:
/// androidx `InteractiveListVerticalAlignmentBreakpoint` (`ListItem.kt`, line 1999), halfway between the heights of
/// the two-line and the three-line item, less the padding.
const ALIGNMENT_BREAKPOINT: Points = Points(
    f32::midpoint(
        list_tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT.0,
        list_tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT.0,
    ) - VERTICAL_PADDING.0,
);

// The tallest content an item holds before its own height is what it needs, and no item is less than 48 points tall,
// the smallest touch target (`tint::MIN_TOUCH_TARGET`): the one-line item is the shortest.
const _: () =
    assert!(list_tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT.0 >= crate::tint::MIN_TOUCH_TARGET.0);

/// Whether, and how far, the supporting text goes: androidx's `hasSupporting` and `isSupportingMultiline`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Supporting {
    /// There is none.
    Absent,
    /// It fits one line.
    OneLine,
    /// It wraps.
    Wraps,
}

/// How many lines a list item has: androidx's `ListItemType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lines {
    One,
    Two,
    Three,
}

/// `ListItem.kt`, line 1263: three lines for an overline with supporting text, or supporting text that wraps; two for
/// either alone; one for neither.
fn lines(has_overline: bool, supporting: Supporting) -> Lines {
    match (has_overline, supporting) {
        (_, Supporting::Wraps) | (true, Supporting::OneLine) => Lines::Three,
        (true, Supporting::Absent) | (false, Supporting::OneLine) => Lines::Two,
        (false, Supporting::Absent) => Lines::One,
    }
}

/// How tall an item of `lines` is at least, padding included: `ListItem.kt`, line 1952.
fn min_height(lines: Lines) -> Points {
    match lines {
        Lines::One => list_tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT,
        Lines::Two => list_tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT,
        Lines::Three => list_tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT,
    }
}

/// Where the content of an item sits between its padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
    Center,
    Top,
}

/// `ListItemDefaults.verticalAlignment` (line 551): the center while the space `inside` the padding is shorter than
/// the breakpoint, the top from it.
fn align(inside: Points) -> Align {
    if inside.0 < ALIGNMENT_BREAKPOINT.0 {
        Align::Center
    } else {
        Align::Top
    }
}

/// Whether supporting text `height` tall wraps, at the user's text size `scale`: more than the 30 sp androidx compares
/// it with.
fn supporting_of(height: Points, scale: f32) -> Supporting {
    if height.0 > SUPPORTING_WRAP_HEIGHT * scale {
        Supporting::Wraps
    } else {
        Supporting::OneLine
    }
}

/// The shape an item takes, by what it is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapeState {
    Rest,
    Hovered,
    Focused,
    Selected,
    Pressed,
}

/// What the user does to an item at the moment, the strongest first: androidx reads the interaction source for a press
/// before a focus and a focus before a hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interaction {
    Idle,
    Hovered,
    Focused,
    Pressed,
}

/// The strongest of the interactions that hold.
fn interaction(pressed: bool, focused: bool, hovered: bool) -> Interaction {
    if pressed {
        Interaction::Pressed
    } else if focused {
        Interaction::Focused
    } else if hovered {
        Interaction::Hovered
    } else {
        Interaction::Idle
    }
}

/// `shapeForInteraction` (`ListItemDefaults.kt`, line 1136), without the dragged state: the first that holds, in the
/// order pressed, selected, focused, hovered.
fn shape_state(interaction: Interaction, selected: bool) -> ShapeState {
    match interaction {
        Interaction::Pressed => ShapeState::Pressed,
        _ if selected => ShapeState::Selected,
        Interaction::Focused => ShapeState::Focused,
        Interaction::Hovered => ShapeState::Hovered,
        Interaction::Idle => ShapeState::Rest,
    }
}

/// The container's shape in `state`: `ListItemDefaults.defaultListItemShapes` (line 510), the Expressive ones.
fn shape(state: ShapeState) -> Shape {
    match state {
        ShapeState::Rest => list_tokens::ITEM_CONTAINER_EXPRESSIVE_SHAPE,
        ShapeState::Hovered => list_tokens::ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE,
        ShapeState::Focused => list_tokens::ITEM_FOCUSED_CONTAINER_EXPRESSIVE_SHAPE,
        ShapeState::Selected => list_tokens::ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE,
        ShapeState::Pressed => list_tokens::ITEM_PRESSED_CONTAINER_EXPRESSIVE_SHAPE,
    }
}

/// What decides an item's colors: `ListItemColors.containerColor` and its siblings (`ListItemDefaults.kt`, line 751)
/// ask whether it is enabled first, then whether it is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Enabled,
    Selected,
    Disabled,
}

/// The look of an item, disabled or not, selected or not: a disabled item is disabled whether it is selected.
fn look(disabled: bool, selected: bool) -> Look {
    if disabled {
        Look::Disabled
    } else if selected {
        Look::Selected
    } else {
        Look::Enabled
    }
}

/// The colors of the parts of an item in one look.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Palette {
    container: Tint,
    headline: Tint,
    overline: Tint,
    supporting: Tint,
    leading: Tint,
    trailing: Tint,
}

/// `ListItemDefaults.defaultListItemColors` (`ListItemDefaults.kt`, line 192). The disabled container is the enabled
/// one, as line 215 says, and not `ItemSelectedDisabledContainerColor`, which no androidx component reads.
fn palette(look: Look) -> Palette {
    match look {
        Look::Enabled => Palette {
            container: Tint::solid(list_tokens::ITEM_CONTAINER_COLOR),
            headline: Tint::solid(list_tokens::ITEM_LABEL_TEXT_COLOR),
            overline: Tint::solid(list_tokens::ITEM_OVERLINE_COLOR),
            supporting: Tint::solid(list_tokens::ITEM_SUPPORTING_TEXT_COLOR),
            leading: Tint::solid(list_tokens::ITEM_LEADING_ICON_COLOR),
            trailing: Tint::solid(list_tokens::ITEM_TRAILING_ICON_COLOR),
        },
        Look::Selected => Palette {
            container: Tint::solid(list_tokens::ITEM_SELECTED_CONTAINER_COLOR),
            headline: Tint::solid(list_tokens::ITEM_SELECTED_LABEL_TEXT_COLOR),
            overline: Tint::solid(list_tokens::ITEM_SELECTED_OVERLINE_COLOR),
            supporting: Tint::solid(list_tokens::ITEM_SELECTED_SUPPORTING_TEXT_COLOR),
            leading: Tint::solid(list_tokens::ITEM_SELECTED_LEADING_ICON_COLOR),
            trailing: Tint::solid(list_tokens::ITEM_SELECTED_TRAILING_ICON_COLOR),
        },
        Look::Disabled => Palette {
            container: Tint::solid(list_tokens::ITEM_CONTAINER_COLOR),
            headline: Tint::faded(
                list_tokens::ITEM_DISABLED_LABEL_TEXT_COLOR,
                list_tokens::ITEM_DISABLED_LABEL_TEXT_OPACITY,
            ),
            overline: Tint::faded(
                list_tokens::ITEM_DISABLED_OVERLINE_COLOR,
                list_tokens::ITEM_DISABLED_OVERLINE_OPACITY,
            ),
            supporting: Tint::faded(
                list_tokens::ITEM_DISABLED_SUPPORTING_TEXT_COLOR,
                list_tokens::ITEM_DISABLED_SUPPORTING_TEXT_OPACITY,
            ),
            leading: Tint::faded(
                list_tokens::ITEM_DISABLED_LEADING_ICON_COLOR,
                list_tokens::ITEM_DISABLED_LEADING_ICON_OPACITY,
            ),
            trailing: Tint::faded(
                list_tokens::ITEM_DISABLED_TRAILING_ICON_COLOR,
                list_tokens::ITEM_DISABLED_TRAILING_ICON_OPACITY,
            ),
        },
    }
}

/// The props of a [`ListItem`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ListItemProps {
    /// The headline: androidx's `content`, the main line.
    pub headline: Prop<SharedString>,
    /// The text above the headline.
    pub overline: Prop<Option<SharedString>>,
    /// The text below the headline. It may wrap; the item is three lines tall when it does.
    pub supporting: Prop<Option<SharedString>>,
    /// An icon before the text, 24 points square. It is not shown while the app fills `leading`.
    pub leading_icon: Prop<Option<Icon>>,
    /// Content before the text, in place of `leading_icon`: an image, an avatar. It takes the color and the text
    /// style of leading content.
    pub leading: Prop<Option<Slot>>,
    /// An icon after the text, 24 points square. It is not shown while the app fills `trailing`.
    pub trailing_icon: Prop<Option<Icon>>,
    /// Content after the text, in place of `trailing_icon`: a short text, a switch. It takes the color and the text
    /// style of trailing content.
    pub trailing: Prop<Option<Slot>>,
    /// The disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The props of a [`ClickableListItem`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ClickableListItemProps {
    /// The headline: androidx's `content`, the main line.
    pub headline: Prop<SharedString>,
    /// The text above the headline.
    pub overline: Prop<Option<SharedString>>,
    /// The text below the headline. It may wrap; the item is three lines tall when it does.
    pub supporting: Prop<Option<SharedString>>,
    /// An icon before the text, 24 points square. It is not shown while the app fills `leading`.
    pub leading_icon: Prop<Option<Icon>>,
    /// Content before the text, in place of `leading_icon`: an image, an avatar. It takes the color and the text
    /// style of leading content.
    pub leading: Prop<Option<Slot>>,
    /// An icon after the text, 24 points square. It is not shown while the app fills `trailing`.
    pub trailing_icon: Prop<Option<Icon>>,
    /// Content after the text, in place of `trailing_icon`: a short text, a switch. It takes the color and the text
    /// style of trailing content.
    pub trailing: Prop<Option<Slot>>,
    /// Whether it is the selected one of its list: androidx's `ListItem(selected, onClick)`, which screen readers
    /// know as a radio button. `None`, the default, is androidx's `ListItem(onClick)`: a button that is not about
    /// being selected.
    pub selected: Prop<Option<bool>>,
    /// No presses, no hover, focus or press shape: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`ClickableListItem`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ClickableListItemEvent {
    /// The list item was pressed.
    Press,
}

/// What both list items show, and what their colors depend on: the props they share.
#[derive(Debug, Clone, Copy)]
struct Content {
    headline: Prop<SharedString>,
    overline: Prop<Option<SharedString>>,
    supporting: Prop<Option<SharedString>>,
    leading_icon: Prop<Option<Icon>>,
    leading: Prop<Option<Slot>>,
    trailing_icon: Prop<Option<Icon>>,
    trailing: Prop<Option<Slot>>,
    selected: Prop<Option<bool>>,
    disabled: Prop<bool>,
}

/// A Material 3 Expressive list item that takes no press: androidx's `ListItem(modifier, …)`. It is as wide as the
/// view it is in and as tall as its text needs, 56, 72 or 88 points at least.
///
/// Its container is at rest, so a list of them is a column of rows with square-ish corners that show nothing of
/// hover, focus or press. For a row that is pressed, use [`ClickableListItem`].
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::list_item::list_item;
/// use nsh_m3e::material_symbols::outlined;
///
/// fn track() -> impl IntoElement {
///     list_item()
///         .headline("Song")
///         .supporting("Artist")
///         .leading_icon(outlined::MUSIC_NOTE)
/// }
/// ```
#[component]
pub fn ListItem(props: ListItemProps) -> impl IntoElement {
    item(
        Content {
            headline: props.headline,
            overline: props.overline,
            supporting: props.supporting,
            leading_icon: props.leading_icon,
            leading: props.leading,
            trailing_icon: props.trailing_icon,
            trailing: props.trailing,
            selected: Prop::new(None),
            disabled: props.disabled,
        },
        None,
    )
}

/// A Material 3 Expressive list item that is pressed: androidx's `ListItem(onClick, …)` and
/// `ListItem(selected, onClick, …)`.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, its role for screen readers: a
/// button, or a radio button once it has a `selected`) and the look of androidx's list item. Hover, focus and press lay
/// the headline's color over it at the state layer opacities of `StateTokens`, as androidx's ripple does, and the
/// container goes to the shape of the state it is in, with the corners springing there (see the module's
/// documentation). It is at least 56 points tall, so its touch target is never smaller than 48 points.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::list_item::clickable_list_item;
///
/// fn row(playing: bool) -> impl IntoElement {
///     clickable_list_item()
///         .headline("Song")
///         .supporting("Artist")
///         .selected(playing)
///         .on_press(|| println!("play"))
/// }
/// ```
#[component]
pub fn ClickableListItem(
    props: ClickableListItemProps,
    cx: Scope<ClickableListItemEvent>,
) -> impl IntoElement {
    item(
        Content {
            headline: props.headline,
            overline: props.overline,
            supporting: props.supporting,
            leading_icon: props.leading_icon,
            leading: props.leading,
            trailing_icon: props.trailing_icon,
            trailing: props.trailing,
            selected: props.selected,
            disabled: props.disabled,
        },
        Some(cx),
    )
}

/// `style`'s font, size, line height and letter spacing on `element`, and its color `color`.
fn typed<S: Styled>(
    element: S,
    style: TypeStyle,
    color: impl Fn() -> AdaptiveColor + 'static,
) -> S {
    element
        .text_color(color)
        .text_size(move || style.size)
        .line_height(move || LineHeight::Points(style.line_height))
        .letter_spacing(move || style.tracking)
        .style_with(move |s| s.font_weight(style.weight))
}

/// No outline: a list item has none.
fn no_outline() -> Option<Points> {
    None
}

/// The color of the tint `tint` gives, as the closure the bindings take.
fn color_of(
    tint: impl Fn() -> Tint + Copy + 'static,
) -> impl Fn() -> AdaptiveColor + Copy + 'static {
    move || tint_color(Some(tint()))
}

/// An icon, drawn at `side` points in `color`.
fn icon(
    icon: Icon,
    side: Points,
    color: impl Fn() -> AdaptiveColor + Copy + 'static,
) -> CanvasView {
    canvas(move || icon.drawing(side))
        .w(move || side)
        .h(move || side)
        .fill_color(color)
}

/// Whether an item is the selected one of its list: only a clickable item whose `selected` is true is.
fn selected_of(content: Content) -> impl Fn() -> bool + Copy + 'static {
    move || content.selected.get() == Some(true)
}

/// The colors of an item as they are now.
fn palette_of(content: Content) -> impl Fn() -> Palette + Copy + 'static {
    move || palette(look(content.disabled.get(), selected_of(content)()))
}

/// How many lines an item has, given what its supporting text measured at its last layout (`measured`, which is only
/// looked at while there is such a text).
fn lines_of(content: Content, measured: Signal<Supporting>) -> impl Fn() -> Lines + Copy + 'static {
    move || {
        let supporting = if content.supporting.get().is_some() {
            measured.get()
        } else {
            Supporting::Absent
        };
        lines(content.overline.get().is_some(), supporting)
    }
}

/// The state layer: the headline's color over the container, in its shape, shown by the item's own states with nothing
/// running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
fn state_layer(drawn: DrawnShape, content: Content) -> AnyElement {
    let palette = palette_of(content);
    drawn
        .state_layer(no_outline)
        .bg(color_of(move || palette().headline))
        // Nothing to show until a state says so.
        .opacity(0.0)
        .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
        .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
        .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
        .style_with(move |s| {
            // A disabled item takes no press and shows no state.
            if content.disabled.get() { s.hidden() } else { s }
        })
        .into_element()
}

/// The content before the text, and the space between it and the text.
fn leading_part(content: Content) -> AnyElement {
    let palette = palette_of(content);
    when(
        move || content.leading_icon.get().is_some() || content.leading.get().is_some(),
        move || {
            let color = color_of(move || palette().leading);
            typed(
                view()
                    .flex_row()
                    .items_center()
                    .pe(list_tokens::ITEM_BETWEEN_SPACE)
                    .fill_color(color),
                list_tokens::ITEM_LEADING_AVATAR_LABEL_FONT,
                color,
            )
            .child(dynamic(move || {
                if content.leading.get().is_some() {
                    None
                } else {
                    content
                        .leading_icon
                        .get()
                        .map(|leading| icon(leading, list_tokens::ITEM_LEADING_ICON_SIZE, color))
                }
            }))
            .child(content.leading.place(|slot| slot))
        },
    )
    .into_element()
}

/// The content after the text, and the space between the text and it.
fn trailing_part(content: Content) -> AnyElement {
    let palette = palette_of(content);
    when(
        move || content.trailing_icon.get().is_some() || content.trailing.get().is_some(),
        move || {
            let color = color_of(move || palette().trailing);
            typed(
                view()
                    .flex_row()
                    .items_center()
                    .ps(list_tokens::ITEM_BETWEEN_SPACE)
                    .fill_color(color),
                list_tokens::ITEM_TRAILING_SUPPORTING_TEXT_FONT,
                color,
            )
            .child(dynamic(move || {
                if content.trailing.get().is_some() {
                    None
                } else {
                    content
                        .trailing_icon
                        .get()
                        .map(|trailing| icon(trailing, list_tokens::ITEM_TRAILING_ICON_SIZE, color))
                }
            }))
            .child(content.trailing.place(|slot| slot))
        },
    )
    .into_element()
}

/// The column of text: the overline, the headline and the supporting text, which reports how tall it is laid out
/// (`measured`) to tell whether it wraps.
fn text_column(content: Content, measured: Signal<Supporting>) -> AnyElement {
    let palette = palette_of(content);
    view()
        .flex_1()
        .flex_col()
        .child(when(
            move || content.overline.get().is_some(),
            move || {
                typed(
                    text(move || content.overline.get().unwrap_or_default()),
                    list_tokens::ITEM_OVERLINE_FONT,
                    color_of(move || palette().overline),
                )
            },
        ))
        .child(typed(
            text(move || content.headline.get()),
            list_tokens::ITEM_LABEL_TEXT_FONT,
            color_of(move || palette().headline),
        ))
        .child(when(
            move || content.supporting.get().is_some(),
            move || {
                typed(
                    text(move || content.supporting.get().unwrap_or_default()),
                    list_tokens::ITEM_SUPPORTING_TEXT_FONT,
                    color_of(move || palette().supporting),
                )
                .on_layout(move |bounds: &Bounds<Points>| {
                    measured.set(supporting_of(bounds.size.height, font_scale()));
                })
            },
        ))
        .into_element()
}

/// The list item both components make: `press` is the scope of a [`ClickableListItem`], `None` for a [`ListItem`].
fn item(content: Content, press: Option<Scope<ClickableListItemEvent>>) -> AnyElement {
    let pressed = signal(false);
    let hovered = signal(false);
    let focused = signal(false);
    let disabled = move || content.disabled.get();
    // A disabled item has no press, focus or hover: it is selected or at rest.
    let state = move || {
        let live = !disabled();
        shape_state(
            interaction(
                live && pressed.get(),
                live && focused.get(),
                live && hovered.get(),
            ),
            selected_of(content)(),
        )
    };
    let palette = palette_of(content);
    let measured = signal(Supporting::OneLine);
    let lines = lines_of(content, measured);
    // Before its first layout the container is taken to be as wide as it is tall: its corners are a radius in points
    // and do not depend on it.
    let drawn = untracked(|| {
        let height = min_height(lines());
        let rest = corners(shape(state()));
        DrawnShape::toggle(
            [rest; 3],
            false,
            Size::new(height, height),
            SpringToken::FastSpatial,
        )
    });
    effect(move || drawn.morph_to(corners(shape(state()))));
    effect(move || {
        // An item disabled while pressed may get no end of its press.
        if disabled() {
            pressed.set(false);
        }
    });
    let inside = move || Points(drawn.size().height.0 - VERTICAL_PADDING.0);

    let mut parts = vec![
        drawn
            .container(no_outline)
            .bg(color_of(move || palette().container))
            .into_element(),
    ];
    if press.is_some() {
        parts.push(state_layer(drawn, content));
    }
    parts.push(leading_part(content));
    parts.push(text_column(content, measured));
    parts.push(trailing_part(content));

    let Some(press) = press else {
        return frame(
            view().disabled(disabled).accessible(true),
            min_height_of(lines),
            inside,
        )
        .children(parts)
        .into_element();
    };
    let focus = element_ref();
    focus.on_focus_in(move || focused.set(true));
    focus.on_focus_out(move || focused.set(false));
    let button = nsh_base::button()
        .disabled(disabled)
        .selected(selected_of(content))
        .role(move || content.selected.get().map(|_| AccessibilityRole::Radio))
        .on_press(move || press.emit(ClickableListItemEvent::Press))
        .on_press_down(move || pressed.set(true))
        .on_press_up(move || pressed.set(false))
        .element_ref(&focus)
        .group();
    // Hover is the pointer over the item or what is in it: only a view hears it.
    view()
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(true))
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(false))
        .child(frame(button, min_height_of(lines), inside).children(parts))
        .into_element()
}

/// The height an item of `lines` is at least, as the closure the style takes.
fn min_height_of(lines: impl Fn() -> Lines + 'static) -> impl Fn() -> Points + 'static {
    move || min_height(lines())
}

/// The row of an item: its padding, at least as tall as `min_height`, its content aligned by the space `inside` its
/// padding.
fn frame<S: Styled>(
    root: S,
    min_height: impl Fn() -> Points + 'static,
    inside: impl Fn() -> Points + 'static,
) -> S {
    root.flex_row()
        .min_h(min_height)
        .ps(list_tokens::ITEM_LEADING_SPACE)
        .pe(list_tokens::ITEM_TRAILING_SPACE)
        .pt(list_tokens::ITEM_TOP_SPACE)
        .pb(list_tokens::ITEM_BOTTOM_SPACE)
        .style_with(move |s| match align(inside()) {
            Align::Center => s.items_center(),
            Align::Top => s.items_start(),
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing};
    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{Color, MountItem, NodeId, ViewKind, ViewProps, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;
    use proptest::prelude::*;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph::Corners;
    use crate::morph_probe::{
        assert_traces, parameters, paths, replay, rounded_rectangle, set_values, spring_with,
        tolerance,
    };

    /// The width the probe gives its list item.
    const WIDTH: f32 = 360.0;

    /// What the probe places: one list item with these props, a line counting its presses, and two lines that select
    /// it and disable it when pressed.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        clickable: Prop<bool>,
        selected: Prop<Option<bool>>,
        headline: Prop<SharedString>,
        overline: Prop<Option<SharedString>>,
        supporting: Prop<Option<SharedString>>,
        leading_icon: Prop<bool>,
        trailing_icon: Prop<bool>,
        leading_slot: Prop<bool>,
        trailing_slot: Prop<bool>,
        disabled: Prop<bool>,
    }

    /// What the probe puts before or after the text of its item.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Part {
        /// Nothing.
        Nothing,
        /// The icon.
        Icon,
        /// A slot.
        Slot,
        /// Both: the slot is the one shown.
        Both,
    }

    impl Part {
        fn icon(self) -> bool {
            matches!(self, Self::Icon | Self::Both)
        }

        fn slot(self) -> bool {
            matches!(self, Self::Slot | Self::Both)
        }
    }

    /// Whether the user asked the system for less motion.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Motion {
        /// Not asked: the corners spring.
        Full,
        /// Asked: they do not move.
        Reduced,
    }

    /// A 40-point square the app fills a slot with.
    fn slot(test_id: &str) -> View {
        view().w(px(40.0)).h(px(40.0)).test_id(test_id)
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        let selected = signal(props.selected.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        let (leading_icon, trailing_icon) = (
            props.leading_icon.get_untracked(),
            props.trailing_icon.get_untracked(),
        );
        let (leading_slot, trailing_slot) = (
            props.leading_slot.get_untracked(),
            props.trailing_slot.get_untracked(),
        );
        let item = if props.clickable.get_untracked() {
            let mut item = clickable_list_item()
                .headline(props.headline)
                .overline(props.overline)
                .supporting(props.supporting)
                .selected(move || selected.get())
                .disabled(move || disabled.get())
                .on_press(move || presses.update(|n| *n += 1));
            if leading_icon {
                item = item.leading_icon(ICON);
            }
            if trailing_icon {
                item = item.trailing_icon(ICON);
            }
            if leading_slot {
                item = item.leading(slot("leading-slot"));
            }
            if trailing_slot {
                item = item.trailing(slot("trailing-slot"));
            }
            item.into_element()
        } else {
            let mut item = list_item()
                .headline(props.headline)
                .overline(props.overline)
                .supporting(props.supporting)
                .disabled(move || disabled.get());
            if leading_icon {
                item = item.leading_icon(ICON);
            }
            if trailing_icon {
                item = item.trailing_icon(ICON);
            }
            if leading_slot {
                item = item.leading(slot("leading-slot"));
            }
            if trailing_slot {
                item = item.trailing(slot("trailing-slot"));
            }
            item.into_element()
        };
        view()
            .w(px(WIDTH))
            .child(view().test_id("item").child(item))
            .child(text(move || format!("presses: {}", presses.get())))
            .child(text("Select").on_press(move |(): &()| {
                selected.update(|selected| *selected = selected.map(|selected| !selected));
            }))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
    }

    /// The icon the probe's items show: a 960-point square, whole, so its drawing is easy to read.
    const ICON: Icon = crate::material_symbols::outlined::HOME;

    #[derive(Debug, Clone)]
    struct Config {
        clickable: bool,
        selected: Option<bool>,
        headline: &'static str,
        overline: Option<&'static str>,
        supporting: Option<&'static str>,
        leading: Part,
        trailing: Part,
        disabled: bool,
        dark: bool,
        motion: Motion,
        font_scale: f32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                clickable: true,
                selected: None,
                headline: "Headline",
                overline: None,
                supporting: None,
                leading: Part::Nothing,
                trailing: Part::Nothing,
                disabled: false,
                dark: false,
                motion: Motion::Full,
                font_scale: 1.0,
            }
        }
    }

    fn start(config: &Config) -> (Headless, Recording) {
        let config = config.clone();
        let root = Root::new::<Probe>(move || ProbeProps {
            clickable: Prop::new(config.clickable),
            selected: Prop::new(config.selected),
            headline: Prop::new(SharedString::from(config.headline)),
            overline: Prop::new(config.overline.map(SharedString::from)),
            supporting: Prop::new(config.supporting.map(SharedString::from)),
            leading_icon: Prop::new(config.leading.icon()),
            trailing_icon: Prop::new(config.trailing.icon()),
            leading_slot: Prop::new(config.leading.slot()),
            trailing_slot: Prop::new(config.trailing.slot()),
            disabled: Prop::new(config.disabled),
        });
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            reduce_motion: config.motion == Motion::Reduced,
            font_scale: config.font_scale,
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:presses: 0").unwrap();
        (app, recording)
    }

    /// The views of the list item, and the last style, frame, drawing and text each was sent.
    struct Mounted {
        /// The row: the press target of a clickable item, the root of a plain one.
        root: NodeId,
        container: NodeId,
        layer: Option<NodeId>,
        parent: HashMap<NodeId, NodeId>,
        props: HashMap<NodeId, ViewProps>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        texts: HashMap<String, (NodeId, nsh_core::ResolvedTextStyle)>,
        test_ids: HashMap<String, NodeId>,
        children: HashMap<NodeId, Vec<NodeId>>,
        kinds: HashMap<NodeId, ViewKind>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut parent = HashMap::new();
            let mut props = HashMap::new();
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            let mut texts = HashMap::new();
            let mut test_ids = HashMap::new();
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        kinds.insert(*id, *kind);
                    }
                    MountItem::Insert {
                        parent: at,
                        child,
                        before,
                    } => {
                        let siblings: &mut Vec<NodeId> = children.entry(*at).or_default();
                        let place = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(place, *child);
                        parent.insert(*child, *at);
                    }
                    MountItem::ViewProps { id, props: view } => {
                        if let Some(test_id) = &view.test_id {
                            test_ids.insert(test_id.to_string(), *id);
                        }
                        props.insert(*id, (**view).clone());
                    }
                    MountItem::ViewStyle { id, style } => {
                        styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    MountItem::Text { id, text, .. } => {
                        if let Some(span) = text.spans.first() {
                            texts.insert(text.text.clone(), (*id, span.style));
                        }
                    }
                    MountItem::Canvas { id, drawing } => {
                        drawings.insert(*id, Arc::clone(drawing));
                    }
                    _ => {}
                }
            }
            let item = test_ids["item"];
            let canvases = |id: &NodeId| -> Vec<NodeId> {
                children
                    .get(id)
                    .into_iter()
                    .flatten()
                    .filter(|child| kinds.get(child) == Some(&ViewKind::Canvas))
                    .copied()
                    .collect()
            };
            // The row is the first view under the item with a canvas among its children.
            let mut root = item;
            while canvases(&root).is_empty() {
                root = *children
                    .get(&root)
                    .and_then(|below| below.first())
                    .expect("a view with the container's canvas under the item");
            }
            let layers = canvases(&root);
            let (container, layer) = match layers[..] {
                [container] => (container, None),
                [container, layer] => (container, Some(layer)),
                _ => panic!("one or two canvases in the row: {layers:?}"),
            };
            Self {
                root,
                container,
                layer,
                parent,
                props,
                styles,
                frames,
                drawings,
                texts,
                test_ids,
                children,
                kinds,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// Where `id` is in the root view: its frame, moved by those of the views it is in.
        fn bounds(&self, id: NodeId) -> Bounds<Points> {
            let mut bounds = self.frames[&id];
            let mut at = id;
            while let Some(above) = self.parent.get(&at) {
                at = *above;
                if let Some(frame) = self.frames.get(&at) {
                    bounds.origin.x += frame.origin.x;
                    bounds.origin.y += frame.origin.y;
                }
            }
            bounds
        }

        /// The row's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.root].size
        }

        /// Where the text `content` is, relative to the row.
        fn text_in_row(&self, content: &str) -> Bounds<Points> {
            let mut bounds = self.bounds(self.texts[content].0);
            let row = self.bounds(self.root);
            bounds.origin.x -= row.origin.x;
            bounds.origin.y -= row.origin.y;
            bounds
        }
    }

    fn mount(config: &Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    impl Mounted {
        /// Where `id` is, relative to the row.
        fn in_row(&self, id: NodeId) -> Bounds<Points> {
            let mut bounds = self.bounds(id);
            let row = self.bounds(self.root);
            bounds.origin.x -= row.origin.x;
            bounds.origin.y -= row.origin.y;
            bounds
        }

        /// The canvases in the row that draw neither its container nor its state layer: the icons, leading first.
        fn icons(&self) -> Vec<NodeId> {
            let mut icons = Vec::new();
            let mut stack = vec![self.root];
            while let Some(id) = stack.pop() {
                if self.kinds.get(&id) == Some(&ViewKind::Canvas)
                    && id != self.container
                    && Some(id) != self.layer
                {
                    icons.push(id);
                }
                stack.extend(self.children.get(&id).into_iter().flatten().rev());
            }
            icons
        }

        /// The style the text `content` was last sent with.
        fn text_style(&self, content: &str) -> &nsh_core::ResolvedTextStyle {
            &self.texts[content].1
        }

        /// The accessibility the row was last sent with.
        fn accessibility(&self) -> &nsh_core::Accessibility {
            &self.props[&self.root].accessibility
        }

        /// Whether `id` is the item or a view in it: the probe's own lines are not.
        fn in_item(&self, id: NodeId) -> bool {
            let mut at = id;
            loop {
                if at == self.test_ids["item"] {
                    return true;
                }
                match self.parent.get(&at) {
                    Some(above) => at = *above,
                    None => return false,
                }
            }
        }

        /// How many of `items` lay the item out again: a frame of one of its views, or one of its texts.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            items
                .iter()
                .filter(|item| match item {
                    MountItem::Frame { id, .. } | MountItem::Text { id, .. } => self.in_item(*id),
                    _ => false,
                })
                .count()
        }

        /// The size of the state layer's frame: none for a view that is not laid out.
        fn layer_size(&self) -> Option<Size<Points>> {
            self.frames
                .get(&self.layer.expect("a state layer"))
                .map(|frame| frame.size)
        }
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

    /// The color `ops` fill with, if any.
    fn paint(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    /// The color of the container, as its canvas paints it.
    fn container_color(mounted: &Mounted) -> Color {
        paint(&mounted.drawing(mounted.container).ops).expect("the container's fill")
    }

    /// The color of the icon `index` (leading first).
    fn icon_color(mounted: &Mounted, index: usize) -> Color {
        paint(&mounted.drawing(mounted.icons()[index]).ops).expect("the icon's fill")
    }

    /// Everything a list item shows, so that every part has a color.
    fn full() -> Config {
        Config {
            overline: Some("Overline"),
            supporting: Some("Supporting"),
            leading: Part::Icon,
            trailing: Part::Icon,
            ..Config::default()
        }
    }

    // The pure decisions.

    #[test]
    fn lines_follow_the_overline_and_the_supporting_text() {
        for (overline, supporting, expected) in [
            (false, Supporting::Absent, Lines::One),
            (true, Supporting::Absent, Lines::Two),
            (false, Supporting::OneLine, Lines::Two),
            (true, Supporting::OneLine, Lines::Three),
            // Supporting text that wraps is three lines with an overline or without, and none is wrapping text.
            (false, Supporting::Wraps, Lines::Three),
            (true, Supporting::Wraps, Lines::Three),
        ] {
            assert_eq!(
                lines(overline, supporting),
                expected,
                "overline {overline}, supporting {supporting:?}"
            );
        }
    }

    #[test]
    fn each_count_of_lines_has_the_height_of_its_token() {
        // `ListTokens.ItemOneLineContainerHeight`, `ItemTwoLineContainerHeight` and `ItemThreeLineContainerHeight`.
        assert_eq!(min_height(Lines::One), Points(56.0));
        assert_eq!(min_height(Lines::Two), Points(72.0));
        assert_eq!(min_height(Lines::Three), Points(88.0));
        assert_eq!(
            min_height(Lines::One),
            list_tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT
        );
        assert_eq!(
            min_height(Lines::Two),
            list_tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT
        );
        assert_eq!(
            min_height(Lines::Three),
            list_tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT
        );
    }

    #[test]
    fn the_content_is_aligned_to_the_center_below_60_points_inside_the_padding_and_to_the_top_from_there()
     {
        // (88 + 72) / 2 - 10 - 10, as androidx computes it.
        assert_eq!(VERTICAL_PADDING, Points(20.0));
        assert_eq!(ALIGNMENT_BREAKPOINT, Points(60.0));
        assert_eq!(align(Points(59.9)), Align::Center);
        assert_eq!(align(Points(60.0)), Align::Top);
        assert_eq!(align(Points(60.1)), Align::Top);
        // The space inside the padding of an item with no more than its minimum height: the one-line and two-line
        // items are centered, the three-line one is at the top.
        for (lines, expected) in [
            (Lines::One, Align::Center),
            (Lines::Two, Align::Center),
            (Lines::Three, Align::Top),
        ] {
            let inside = Points(min_height(lines).0 - VERTICAL_PADDING.0);
            assert_eq!(align(inside), expected, "{lines:?}");
        }
    }

    #[test]
    fn supporting_text_wraps_above_30_sp_of_the_users_text_size() {
        // `30.sp`, at the user's text size.
        assert_eq!(supporting_of(Points(30.0), 1.0), Supporting::OneLine);
        assert_eq!(supporting_of(Points(30.1), 1.0), Supporting::Wraps);
        // One line at 200 % is as tall as two at 100 %: it is not wrapping.
        assert_eq!(supporting_of(Points(34.0), 2.0), Supporting::OneLine);
        assert_eq!(supporting_of(Points(34.0), 1.0), Supporting::Wraps);
        assert_eq!(supporting_of(Points(60.0), 2.0), Supporting::OneLine);
        assert_eq!(supporting_of(Points(60.1), 2.0), Supporting::Wraps);
    }

    proptest! {
        #[test]
        fn a_taller_space_never_moves_the_content_from_the_top_to_the_center(
            inside in 0.0_f32..400.0,
            more in 0.0_f32..400.0,
        ) {
            if align(Points(inside)) == Align::Top {
                prop_assert_eq!(align(Points(inside + more)), Align::Top);
            }
            prop_assert_eq!(
                align(Points(inside)) == Align::Center,
                inside < ALIGNMENT_BREAKPOINT.0
            );
        }

        #[test]
        fn taller_supporting_text_never_stops_wrapping_and_a_larger_text_size_never_starts_it(
            height in 0.0_f32..600.0,
            more in 0.0_f32..600.0,
            scale in 0.5_f32..4.0,
            larger in 0.0_f32..4.0,
        ) {
            if supporting_of(Points(height), scale) == Supporting::Wraps {
                prop_assert_eq!(supporting_of(Points(height + more), scale), Supporting::Wraps);
                // The same height is less lines at a larger text size.
                if supporting_of(Points(height), scale + larger) == Supporting::Wraps {
                    prop_assert_eq!(supporting_of(Points(height), scale), Supporting::Wraps);
                }
            }
        }
    }

    #[test]
    fn the_container_takes_the_first_shape_that_holds() {
        // (pressed, selected, focused, hovered): pressed beats selected beats focused beats hovered.
        let table = [
            ((false, false, false, false), ShapeState::Rest),
            ((false, false, false, true), ShapeState::Hovered),
            ((false, false, true, false), ShapeState::Focused),
            ((false, false, true, true), ShapeState::Focused),
            ((false, true, false, false), ShapeState::Selected),
            ((false, true, false, true), ShapeState::Selected),
            ((false, true, true, false), ShapeState::Selected),
            ((false, true, true, true), ShapeState::Selected),
            ((true, false, false, false), ShapeState::Pressed),
            ((true, false, false, true), ShapeState::Pressed),
            ((true, false, true, false), ShapeState::Pressed),
            ((true, false, true, true), ShapeState::Pressed),
            ((true, true, false, false), ShapeState::Pressed),
            ((true, true, false, true), ShapeState::Pressed),
            ((true, true, true, false), ShapeState::Pressed),
            ((true, true, true, true), ShapeState::Pressed),
        ];
        for ((pressed, selected, focused, hovered), expected) in table {
            assert_eq!(
                shape_state(interaction(pressed, focused, hovered), selected),
                expected,
                "pressed {pressed}, selected {selected}, focused {focused}, hovered {hovered}"
            );
        }
    }

    #[test]
    fn each_shape_has_the_corners_of_its_token_and_can_morph() {
        // `CornerExtraSmall` (4), `CornerMedium` (12) and `CornerLarge` (16): `ShapeKeyTokens`.
        for (state, radius) in [
            (ShapeState::Rest, 4.0),
            (ShapeState::Hovered, 12.0),
            (ShapeState::Focused, 16.0),
            (ShapeState::Selected, 16.0),
            (ShapeState::Pressed, 16.0),
        ] {
            // A shape with uneven corners would fall back to full corners: none of the list item's does.
            assert!(Corners::of(shape(state)).is_some(), "{state:?}");
            assert_eq!(
                corners(shape(state)),
                Corners::Radius(Points(radius)),
                "{state:?}"
            );
        }
    }

    #[test]
    fn a_disabled_item_is_disabled_whether_it_is_selected() {
        assert_eq!(look(false, false), Look::Enabled);
        assert_eq!(look(false, true), Look::Selected);
        assert_eq!(look(true, false), Look::Disabled);
        assert_eq!(look(true, true), Look::Disabled);
    }

    #[test]
    fn the_palettes_come_from_the_list_tokens() {
        let enabled = palette(Look::Enabled);
        assert_eq!(enabled.container, Tint::solid(ColorRole::Surface));
        assert_eq!(enabled.headline, Tint::solid(ColorRole::OnSurface));
        for tint in [
            enabled.overline,
            enabled.supporting,
            enabled.leading,
            enabled.trailing,
        ] {
            assert_eq!(tint, Tint::solid(ColorRole::OnSurfaceVariant));
        }
        let selected = palette(Look::Selected);
        assert_eq!(
            selected.container,
            Tint::solid(ColorRole::SecondaryContainer)
        );
        for tint in [
            selected.headline,
            selected.overline,
            selected.supporting,
            selected.leading,
            selected.trailing,
        ] {
            assert_eq!(tint, Tint::solid(ColorRole::OnSecondaryContainer));
        }
        let disabled = palette(Look::Disabled);
        // `ListItemDefaults.kt`, line 215: the disabled container is the enabled one.
        assert_eq!(disabled.container, enabled.container);
        for tint in [
            disabled.headline,
            disabled.overline,
            disabled.supporting,
            disabled.leading,
            disabled.trailing,
        ] {
            assert_eq!(tint, Tint::faded(ColorRole::OnSurface, 0.38));
        }
    }

    // What a mounted item looks like.

    #[test]
    fn each_look_has_its_colors_in_both_appearances() {
        for dark in [false, true] {
            let role = |role| scheme_color(role, dark);
            for (selected, disabled, container, headline, others) in [
                (
                    None,
                    false,
                    (ColorRole::Surface, 255),
                    (ColorRole::OnSurface, 255),
                    (ColorRole::OnSurfaceVariant, 255),
                ),
                (
                    Some(true),
                    false,
                    (ColorRole::SecondaryContainer, 255),
                    (ColorRole::OnSecondaryContainer, 255),
                    (ColorRole::OnSecondaryContainer, 255),
                ),
                // Selected or not, a disabled item is the same: 0.38 of 255 is 97.
                (
                    Some(false),
                    true,
                    (ColorRole::Surface, 255),
                    (ColorRole::OnSurface, 97),
                    (ColorRole::OnSurface, 97),
                ),
                (
                    Some(true),
                    true,
                    (ColorRole::Surface, 255),
                    (ColorRole::OnSurface, 97),
                    (ColorRole::OnSurface, 97),
                ),
            ] {
                let what = format!("selected {selected:?}, disabled {disabled}, dark {dark}");
                let mounted = mount(&Config {
                    selected,
                    disabled,
                    dark,
                    ..full()
                });
                assert_eq!(
                    container_color(&mounted),
                    color(role(container.0), container.1),
                    "container, {what}"
                );
                assert_eq!(
                    mounted.text_style("Headline").color,
                    color(role(headline.0), headline.1),
                    "headline, {what}"
                );
                let others = color(role(others.0), others.1);
                assert_eq!(
                    mounted.text_style("Overline").color,
                    others,
                    "overline, {what}"
                );
                assert_eq!(
                    mounted.text_style("Supporting").color,
                    others,
                    "supporting, {what}"
                );
                assert_eq!(icon_color(&mounted, 0), others, "leading icon, {what}");
                assert_eq!(icon_color(&mounted, 1), others, "trailing icon, {what}");
            }
        }
    }

    #[test]
    fn a_plain_item_is_never_selected_and_has_the_colors_at_rest() {
        let mounted = mount(&Config {
            clickable: false,
            ..full()
        });
        assert_eq!(
            container_color(&mounted),
            color(scheme_color(ColorRole::Surface, false), 255)
        );
        assert_eq!(
            mounted.text_style("Headline").color,
            color(scheme_color(ColorRole::OnSurface, false), 255)
        );
        assert_eq!(
            icon_color(&mounted, 0),
            color(scheme_color(ColorRole::OnSurfaceVariant, false), 255)
        );
    }

    #[test]
    fn the_text_has_the_styles_of_its_tokens() {
        let mounted = mount(&full());
        // `BodyLarge` for the headline, `LabelSmall` for the overline and `BodyMedium` for the supporting text.
        for (content, token) in [
            ("Headline", list_tokens::ITEM_LABEL_TEXT_FONT),
            ("Overline", list_tokens::ITEM_OVERLINE_FONT),
            ("Supporting", list_tokens::ITEM_SUPPORTING_TEXT_FONT),
        ] {
            let style = mounted.text_style(content);
            assert_eq!(style.font_size, token.size, "{content}");
            assert_eq!(style.line_height, token.line_height, "{content}");
            assert_eq!(style.letter_spacing, token.tracking, "{content}");
            assert_eq!(style.font_weight, token.weight.0, "{content}");
        }
        let sizes: Vec<f32> = ["Headline", "Overline", "Supporting"]
            .iter()
            .map(|content| mounted.text_style(content).font_size.0)
            .collect();
        assert_eq!(sizes, [16.0, 11.0, 14.0]);
    }

    #[test]
    fn each_count_of_lines_makes_the_item_as_tall_as_its_token_at_least() {
        for clickable in [false, true] {
            for (overline, supporting, height) in [
                (None, None, 56.0),
                (None, Some("Supporting"), 72.0),
                (Some("Overline"), None, 72.0),
                (Some("Overline"), Some("Supporting"), 88.0),
            ] {
                let mounted = mount(&Config {
                    clickable,
                    overline,
                    supporting,
                    ..Config::default()
                });
                let what = format!("clickable {clickable}, {overline:?}, {supporting:?}");
                assert_eq!(mounted.size(), Size::new(px(WIDTH), px(height)), "{what}");
                // No item is smaller than the touch target, pressed or not.
                assert!(
                    mounted.size().height.0 >= crate::tint::MIN_TOUCH_TARGET.0,
                    "{what}"
                );
            }
        }
    }

    /// A supporting text that wraps onto a second line at the probe's width.
    const WRAPPING: &str = "Supporting text that is long enough to wrap onto a second line";

    #[test]
    fn supporting_text_that_wraps_makes_three_lines() {
        let one_line = mount(&Config {
            supporting: Some("Supporting"),
            ..Config::default()
        });
        let line = one_line
            .bounds(one_line.texts["Supporting"].0)
            .size
            .height
            .0;
        let wrapped = mount(&Config {
            supporting: Some(WRAPPING),
            ..Config::default()
        });
        let height = wrapped.bounds(wrapped.texts[WRAPPING].0).size.height.0;
        // Two lines of text, and still less than the three-line item holds: the height is the token's.
        assert_eq!(height, 2.0 * line);
        assert_eq!(wrapped.size().height, Points(88.0));
        assert_eq!(one_line.size().height, Points(72.0));
    }

    #[test]
    fn content_taller_than_the_token_height_makes_the_item_grow_with_it() {
        // Two slots of 40 points and an icon: 40 and the padding is 60, over the one-line 56. Nothing is cut.
        let mounted = mount(&Config {
            leading: Part::Slot,
            ..Config::default()
        });
        assert_eq!(mounted.size().height, Points(60.0));
        // At 200 % of the text size, the headline and the supporting text are twice as large and the item holds both
        // lines, its padding around them.
        let mounted = mount(&Config {
            supporting: Some("Supporting"),
            font_scale: 2.0,
            ..Config::default()
        });
        assert_eq!(mounted.text_style("Headline").font_size, Points(32.0));
        assert_eq!(mounted.text_style("Supporting").font_size, Points(28.0));
        let headline = mounted.text_in_row("Headline");
        let supporting = mounted.text_in_row("Supporting");
        assert!(mounted.size().height.0 > 72.0, "{:?}", mounted.size());
        assert_eq!(
            mounted.size().height.0,
            VERTICAL_PADDING.0 + headline.size.height.0 + supporting.size.height.0
        );
        assert_eq!(headline.origin.y, list_tokens::ITEM_TOP_SPACE);
        assert_eq!(
            supporting.origin.y.0 + supporting.size.height.0,
            mounted.size().height.0 - list_tokens::ITEM_BOTTOM_SPACE.0
        );
    }

    #[test]
    fn long_text_wraps_inside_the_row_and_never_past_its_spaces() {
        let long = "Supporting text that is long enough to wrap onto a second line, and then onto a third one besides";
        let mounted = mount(&Config {
            supporting: Some(long),
            leading: Part::Icon,
            trailing: Part::Icon,
            ..Config::default()
        });
        let supporting = mounted.text_in_row(long);
        // The row, less its side spaces, the icons and the 12 points around each.
        assert_eq!(supporting.origin.x, Points(16.0 + 24.0 + 12.0));
        assert_eq!(
            supporting.size.width,
            Points(WIDTH - 2.0 * (16.0 + 24.0 + 12.0))
        );
        assert!(supporting.size.height.0 > 2.0 * 17.0, "{supporting:?}");
        assert_eq!(
            supporting.origin.y.0 + supporting.size.height.0 + 10.0,
            mounted.size().height.0
        );
    }

    #[test]
    fn the_spaces_around_the_content_are_those_of_the_tokens() {
        let mounted = mount(&Config {
            supporting: Some("Supporting"),
            leading: Part::Icon,
            trailing: Part::Icon,
            ..Config::default()
        });
        let [leading, trailing] = mounted.icons()[..] else {
            panic!("two icons: {:?}", mounted.icons());
        };
        let (leading, trailing) = (mounted.in_row(leading), mounted.in_row(trailing));
        // 16 before the leading icon, 24 of icon and 12 before the text; the same after it.
        assert_eq!(leading.origin.x, Points(16.0));
        assert_eq!(leading.size, Size::new(px(24.0), px(24.0)));
        assert_eq!(mounted.text_in_row("Headline").origin.x, Points(52.0));
        assert_eq!(trailing.size, Size::new(px(24.0), px(24.0)));
        assert_eq!(trailing.origin.x, Points(WIDTH - 16.0 - 24.0));
        assert_eq!(
            mounted.text_in_row("Headline").size.width,
            Points(WIDTH - 52.0 - 24.0 - 12.0 - 16.0)
        );
        // The row is 72 tall and its content 37 plus the padding: the icons are centered.
        assert_eq!(mounted.size().height, Points(72.0));
        assert_eq!(leading.origin.y, Points(24.0));
        assert_eq!(trailing.origin.y, Points(24.0));
    }

    #[test]
    fn no_leading_content_and_no_trailing_content_leave_no_space() {
        let mounted = mount(&Config::default());
        assert_eq!(mounted.icons(), Vec::<NodeId>::new());
        let headline = mounted.text_in_row("Headline");
        // Only the 16-point spaces at the sides, none of the 12 between the text and what is not there.
        assert_eq!(headline.origin.x, Points(16.0));
        assert_eq!(headline.size.width, Points(WIDTH - 32.0));
    }

    #[test]
    fn content_is_at_the_top_from_the_breakpoint_and_in_the_middle_below_it() {
        // One line: 36 inside the padding, the icon is centered in it.
        let one = mount(&Config {
            leading: Part::Icon,
            trailing: Part::Icon,
            ..Config::default()
        });
        for icon in one.icons() {
            assert_eq!(
                one.in_row(icon).origin.y,
                Points(10.0 + (36.0 - 24.0) / 2.0)
            );
        }
        // Three lines: 68 inside the padding, over the breakpoint: the icons are at the top, under the 10 points.
        let three = mount(&full());
        assert_eq!(three.size().height, Points(88.0));
        for icon in three.icons() {
            assert_eq!(three.in_row(icon).origin.y, Points(10.0));
        }
        // An item that is taller than its tokens say is aligned by what it measures: 70 points of slots make it
        // 90 tall, 70 inside the padding.
        let tall = mount(&Config {
            leading: Part::Slot,
            overline: Some("Overline"),
            supporting: Some("Supporting"),
            trailing: Part::Icon,
            ..Config::default()
        });
        assert_eq!(tall.in_row(tall.icons()[0]).origin.y, Points(10.0));
    }

    #[test]
    fn a_slot_takes_the_place_of_the_icon_and_has_the_same_spaces() {
        let mounted = mount(&Config {
            leading: Part::Both,
            trailing: Part::Both,
            ..Config::default()
        });
        // The icons are not drawn while the app fills the slots.
        assert_eq!(mounted.icons(), Vec::<NodeId>::new());
        let leading = mounted.in_row(mounted.test_ids["leading-slot"]);
        let trailing = mounted.in_row(mounted.test_ids["trailing-slot"]);
        assert_eq!(leading.origin.x, Points(16.0));
        assert_eq!(leading.size, Size::new(px(40.0), px(40.0)));
        assert_eq!(
            mounted.text_in_row("Headline").origin.x,
            Points(16.0 + 40.0 + 12.0)
        );
        assert_eq!(trailing.origin.x, Points(WIDTH - 16.0 - 40.0));
        // 40 points and the padding is 60, over the one-line 56.
        assert_eq!(mounted.size().height, Points(60.0));
        // A slot alone is placed with no icon: one on each side, and the other not there.
        let mounted = mount(&Config {
            trailing: Part::Slot,
            leading: Part::Icon,
            ..Config::default()
        });
        assert_eq!(mounted.icons().len(), 1);
        assert_eq!(mounted.in_row(mounted.icons()[0]).origin.x, Points(16.0));
        assert_eq!(
            mounted.in_row(mounted.test_ids["trailing-slot"]).origin.x,
            Points(WIDTH - 16.0 - 40.0)
        );
    }

    #[test]
    fn each_shape_is_drawn_on_the_container_and_the_state_layer() {
        for (config, radius) in [
            // At rest, selected, disabled and selected, and the plain item that cannot be selected.
            (Config::default(), 4.0),
            (
                Config {
                    selected: Some(true),
                    ..Config::default()
                },
                16.0,
            ),
            (
                Config {
                    selected: Some(true),
                    disabled: true,
                    ..Config::default()
                },
                16.0,
            ),
            (
                Config {
                    clickable: false,
                    ..Config::default()
                },
                4.0,
            ),
        ] {
            let what = format!("{config:?}");
            let (_app, recording) = start(&config);
            let mounted = Mounted::of(&recording.items());
            let replay = replay(&recording);
            let expected = rounded_rectangle(mounted.size(), 0.0, radius);
            let container = paths(mounted.drawing(mounted.container), &replay);
            assert_eq!(container.len(), 1, "{what}");
            assert_traces(&container[0], &expected, &what);
            // The state layer is the container's shape, and only a list item that is pressed has one.
            assert_eq!(mounted.layer.is_some(), config.clickable, "{what}");
            if let Some(layer) = mounted.layer {
                let layer = paths(mounted.drawing(layer), &replay);
                assert_eq!(layer.len(), 1, "{what}");
                assert_traces(&layer[0], &expected, &what);
            }
            // No view rounds itself: the drawing gives the corners.
            for id in [mounted.root, mounted.container] {
                let corners = mounted.style(id).corner_radius;
                assert_eq!(mounted.style(id).border_radius, Points(0.0), "{what}");
                assert!(
                    [
                        corners.top_left,
                        corners.top_right,
                        corners.bottom_right,
                        corners.bottom_left
                    ]
                    .iter()
                    .all(Option::is_none),
                    "{what}"
                );
            }
        }
    }

    // Accessibility and presses.

    #[test]
    fn a_clickable_item_is_a_button_a_selectable_one_a_radio_button_and_a_plain_one_neither() {
        let button = mount(&Config::default());
        assert_eq!(button.accessibility().role, AccessibilityRole::Button);
        assert!(!button.accessibility().state.selected);
        for selected in [false, true] {
            let radio = mount(&Config {
                selected: Some(selected),
                ..Config::default()
            });
            assert_eq!(
                radio.accessibility().role,
                AccessibilityRole::Radio,
                "{selected}"
            );
            assert_eq!(radio.accessibility().state.selected, selected);
        }
        let plain = mount(&Config {
            clickable: false,
            ..Config::default()
        });
        // One element for screen readers, as androidx's `mergeDescendants`, and not a control.
        assert_eq!(plain.accessibility().accessible, Some(true));
        assert_eq!(plain.accessibility().role, AccessibilityRole::default());
        assert_ne!(plain.accessibility().role, AccessibilityRole::Button);
    }

    #[test]
    fn selecting_updates_the_state_for_screen_readers() {
        let (mut app, recording) = start(&Config {
            selected: Some(false),
            ..Config::default()
        });
        app.press("Select").unwrap();
        let mounted = Mounted::of(&recording.items());
        assert!(mounted.accessibility().state.selected);
        assert_eq!(mounted.accessibility().role, AccessibilityRole::Radio);
        app.press("Select").unwrap();
        assert!(
            !Mounted::of(&recording.items())
                .accessibility()
                .state
                .selected
        );
    }

    #[test]
    fn a_disabled_item_is_dimmed_for_screen_readers_and_takes_no_press() {
        for clickable in [false, true] {
            let (mut app, recording) = start(&Config {
                clickable,
                disabled: true,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            assert!(
                mounted.accessibility().state.disabled,
                "clickable {clickable}"
            );
            let _ = app.press("Headline");
            assert!(app.shows("presses: 0"), "clickable {clickable}");
        }
        // Enabled, it is not dimmed.
        let mounted = mount(&Config::default());
        assert!(!mounted.accessibility().state.disabled);
        let mounted = mount(&Config {
            clickable: false,
            ..Config::default()
        });
        assert!(!mounted.accessibility().state.disabled);
    }

    #[test]
    fn a_press_is_reported_once_and_a_plain_item_reports_none() -> Result<(), String> {
        let (mut app, _recording) = start(&Config::default());
        app.press("Headline")?;
        app.wait_for("text:presses: 1")?;
        app.press("role:button")?;
        app.wait_for("text:presses: 2")?;
        // A plain item has no press target: the row is no button.
        let (mut app, recording) = start(&Config {
            clickable: false,
            ..Config::default()
        });
        let _ = app.press("Headline");
        assert!(app.shows("presses: 0"));
        assert!(
            Mounted::of(&recording.items())
                .props
                .values()
                .all(|props| props.accessibility.role != AccessibilityRole::Button)
        );
        Ok(())
    }

    #[test]
    fn the_touch_target_is_the_row_and_needs_no_slop() {
        for lines in [Lines::One, Lines::Two, Lines::Three] {
            let (overline, supporting) = match lines {
                Lines::One => (None, None),
                Lines::Two => (None, Some("Supporting")),
                Lines::Three => (Some("Overline"), Some("Supporting")),
            };
            let mounted = mount(&Config {
                overline,
                supporting,
                ..Config::default()
            });
            let size = mounted.size();
            // 48 points on each side: `minimumInteractiveComponentSize`, which the row's own size meets.
            assert!(
                size.height.0 >= crate::tint::MIN_TOUCH_TARGET.0,
                "{lines:?}"
            );
            assert!(size.width.0 >= crate::tint::MIN_TOUCH_TARGET.0, "{lines:?}");
            assert_eq!(
                mounted.props[&mounted.root].hit_slop,
                EdgeInsets::default(),
                "{lines:?}"
            );
        }
    }

    // The state layer and the morph.

    /// The step of a press that goes down on the headline and stays down: a finger that lands and does not move.
    fn press_down(app: &mut Headless) -> String {
        let still = point(px(0.0), px(0.0));
        app.drag_and_hold("Headline", still).unwrap();
        format!("dragAndHold(Headline, {}, {})", still.x, still.y)
    }

    /// The step `step` springs the corners as [`spring_with`] says, to `to`, and changes nothing but the state layer:
    /// nothing in the item is laid out, and the container is not drawn again.
    fn morph_in(recording: &Recording, step: &str, to: f32) -> StartedSpring {
        let items = recording.in_step(step);
        let spring = spring_with(&items, step, to, SpringToken::FastSpatial);
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.layout_items(&items), 0, "{step}: {items:?}");
        assert!(
            !items.iter().any(
                |item| matches!(item, MountItem::Canvas { id, .. } if *id == mounted.container)
            ),
            "{step}: {items:?}"
        );
        spring
    }

    /// The operations of `drawing` that are not a color: its path.
    fn path_of(drawing: &Drawing) -> Vec<CanvasOp> {
        drawing
            .ops
            .iter()
            .filter(|op| !matches!(op, CanvasOp::FillStyle(_)))
            .cloned()
            .collect()
    }

    /// [`morph_in`] for a step that also changes the colors of the item, as `before` was mounted: its texts are sent
    /// again and so are its canvases, with the path they had and the animated corners they read, and a new fill.
    fn morph_and_recolor_in(
        recording: &Recording,
        step: &str,
        to: f32,
        before: &Mounted,
    ) -> StartedSpring {
        let items = recording.in_step(step);
        let spring = spring_with(&items, step, to, SpringToken::FastSpatial);
        let mounted = Mounted::of(&recording.items());
        assert!(
            !items
                .iter()
                .any(|item| matches!(item, MountItem::Frame { id, .. } if mounted.in_item(*id))),
            "{step}: {items:?}"
        );
        let mut redrawn = Vec::new();
        for item in &items {
            if let MountItem::Canvas { id, drawing } = item {
                let old = before.drawing(*id);
                assert_eq!(path_of(drawing), path_of(old), "{step}: canvas {id:?}");
                assert_eq!(drawing.slots, old.slots, "{step}: canvas {id:?}");
                assert_ne!(
                    paint(&drawing.ops),
                    paint(&old.ops),
                    "{step}: canvas {id:?}"
                );
                redrawn.push(*id);
            }
        }
        // The container and the state layer, in their new color.
        assert!(redrawn.contains(&mounted.container), "{step}: {redrawn:?}");
        spring
    }

    /// What the platform reports of the row's states while a finger is on it and the mouse is over it (`pressed`), and
    /// once the finger lifts. The headless driver reports hovers and focus and never a press, so the test does.
    fn platform_reports(app: &mut Headless, recording: &Recording, pressed: bool) {
        use nsh_core::tailwind::state;
        let root = Mounted::of(&recording.items()).root;
        app.step(if pressed {
            "platform: pressed"
        } else {
            "platform: released"
        });
        let states = if pressed {
            state::PRESSED | state::HOVERED
        } else {
            state::HOVERED
        };
        app.engine().control().set_states(root, states);
        assert!(app.settle());
    }

    /// The opacity the state layer was last sent with.
    fn layer_opacity(recording: &Recording) -> f32 {
        let mounted = Mounted::of(&recording.items());
        mounted.style(mounted.layer.expect("a state layer")).opacity
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for config in [
            Config::default(),
            Config {
                selected: Some(true),
                ..Config::default()
            },
            Config {
                disabled: true,
                ..Config::default()
            },
            Config {
                clickable: false,
                ..Config::default()
            },
        ] {
            let (_app, recording) = start(&config);
            let items = recording.items();
            assert_eq!(started_springs(&items), Vec::new(), "{config:?}");
            // The corners are created where they rest: the layout sets nothing again.
            assert_eq!(set_values(&items), 0, "{config:?}");
            assert!(!replay(&recording).is_animating(), "{config:?}");
        }
    }

    #[test]
    fn hover_springs_the_corners_with_the_fast_spatial_spring_and_shows_the_state_layer() {
        let (mut app, recording) = start(&Config::default());
        assert_eq!(layer_opacity(&recording), 0.0);
        app.hover("Headline").unwrap();
        let spring = morph_in(&recording, "hover(Headline)", 12.0);
        // `StateTokens.HoverStateLayerOpacity`, in the headline's color, over the container.
        assert_eq!(layer_opacity(&recording), 0.08);
        let mounted = Mounted::of(&recording.items());
        assert_eq!(
            paint(&mounted.drawing(mounted.layer.expect("a layer")).ops),
            Some(color(scheme_color(ColorRole::OnSurface, false), 255))
        );
        // While it plays, the engine sends nothing (ADR 0010).
        app.step("run");
        app.run_for(std::time::Duration::from_secs(1)).unwrap();
        assert!(recording.in_step("run").is_empty());

        // Mid-way, the container and the state layer are drawn with the corners where the spring has them.
        let frame = mounted.size();
        let mut replay = replay(&recording);
        let mut samples = replay.run(spring.node, 5);
        let now = replay.value(spring.node);
        assert!(now > 4.0 && now < 12.0, "{now}");
        let container = paths(mounted.drawing(mounted.container), &replay);
        assert_traces(
            &container[0],
            &rounded_rectangle(frame, 0.0, now),
            "container",
        );
        let layer = paths(mounted.drawing(mounted.layer.expect("a layer")), &replay);
        assert_traces(&layer[0], &rounded_rectangle(frame, 0.0, now), "layer");
        // The whole curve is the closed-form spring from the corners at rest, within 0.1 % of its travel.
        samples.extend(replay.run(spring.node, 175));
        let gap = worst_gap(&samples, 4.0, &spring);
        assert!(gap < tolerance(4.0, 12.0), "{gap}");
        // FastSpatial overshoots (its damping ratio is 0.6): the corners go past 12 before they settle on it.
        let peak = samples.iter().copied().fold(f32::MIN, f32::max);
        assert!(peak > 12.0 + 0.5, "{peak}");
        let settled = replay.run(spring.node, 600);
        assert!(!replay.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(12.0_f32.to_bits())
        );
    }

    #[test]
    fn leaving_the_item_springs_the_corners_back_to_rest_and_hides_the_state_layer() {
        let (mut app, recording) = start(&Config::default());
        app.hover("Headline").unwrap();
        let _ = morph_in(&recording, "hover(Headline)", 12.0);
        // The pointer goes over another line of the probe.
        app.hover("text:presses: 0").unwrap();
        let _ = morph_in(&recording, "hover(text:presses: 0)", 4.0);
        assert_eq!(layer_opacity(&recording), 0.0);
    }

    #[test]
    fn a_press_springs_the_corners_to_the_pressed_shape_from_where_and_as_fast_as_they_move() {
        let (mut app, recording) = start(&Config::default());
        app.hover("Headline").unwrap();
        let hover = morph_in(&recording, "hover(Headline)", 12.0);
        let mut replay = replay(&recording);
        // The finger lands 50 ms into the hover, while the corners still move.
        let before = replay.run(hover.node, 6);
        let step = press_down(&mut app);
        let down = morph_in(&recording, &step, 16.0);
        assert_eq!(down.node, hover.node);
        // The hovered state layer stays until the platform says the row is pressed; then it is the pressed one,
        // `StateTokens.PressedStateLayerOpacity`, though the mouse is still over the row.
        assert_eq!(layer_opacity(&recording), 0.08);
        platform_reports(&mut app, &recording, true);
        assert_eq!(layer_opacity(&recording), 0.1);
        replay.apply(&recording.in_step(&step));
        let after = replay.run(down.node, 180);

        // The graph last ticked 5 frames into the hover: the press starts there, at that value and that speed.
        let first = parameters(&hover);
        let at = 5.0 / REPLAY_HZ;
        let position = |t: f64| spring_position(4.0, 12.0, 0.0, first, t);
        let from = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed > 10.0, "{speed}");
        let gap = |velocity: f64| {
            after
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                    let closed = spring_position(from, 16.0, velocity, parameters(&down), t);
                    (f64::from(value) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        let tolerance = tolerance(from, 16.0);
        assert!(gap(speed) < tolerance, "{}", gap(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(gap(0.0) > 10.0 * tolerance, "{}", gap(0.0));
        let settled = replay.run(down.node, 600);
        assert!(!replay.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(16.0_f32.to_bits())
        );

        // Letting go, the pointer is still over the item: back to the hovered corners, and the press is reported.
        app.drop_held().unwrap();
        let _ = morph_in(&recording, "drop()", 12.0);
        assert!(app.shows("presses: 1"));
        platform_reports(&mut app, &recording, false);
        assert_eq!(layer_opacity(&recording), 0.08);
    }

    #[test]
    fn keyboard_focus_springs_the_corners_to_the_focused_shape_and_shows_the_state_layer() {
        let (mut app, recording) = start(&Config::default());
        app.press_key("tab").unwrap();
        let _ = morph_in(&recording, "keyboard.press(tab)", 16.0);
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(layer_opacity(&recording), 0.1);
        app.press_key("shift+tab").unwrap();
        let _ = morph_in(&recording, "keyboard.press(shift+tab)", 4.0);
        assert_eq!(layer_opacity(&recording), 0.0);
    }

    #[test]
    fn a_new_state_with_the_same_corners_springs_nothing() {
        // Focused (16), then pressed (16): the corners are where they are going.
        let (mut app, recording) = start(&Config::default());
        app.press_key("tab").unwrap();
        let _ = morph_in(&recording, "keyboard.press(tab)", 16.0);
        let step = press_down(&mut app);
        assert_eq!(
            started_springs(&recording.in_step(&step)),
            Vec::new(),
            "{step}"
        );
        assert_eq!(set_values(&recording.in_step(&step)), 0, "{step}");
    }

    #[test]
    fn selecting_springs_the_corners_and_changes_the_container_color() {
        let (mut app, recording) = start(&Config {
            selected: Some(false),
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        assert_eq!(
            container_color(&mounted),
            color(scheme_color(ColorRole::Surface, false), 255)
        );
        app.press("Select").unwrap();
        // The texts take the selected colors, so they are sent again; nothing is laid out again.
        let _ = morph_and_recolor_in(&recording, "press(Select)", 16.0, &mounted);
        let mounted = Mounted::of(&recording.items());
        assert_eq!(
            container_color(&mounted),
            color(scheme_color(ColorRole::SecondaryContainer, false), 255)
        );
        assert_eq!(
            mounted.text_style("Headline").color,
            color(scheme_color(ColorRole::OnSecondaryContainer, false), 255)
        );
        // A selected item that is hovered or pressed is still selected, and its corners stay where they are.
        app.hover("Headline").unwrap();
        assert_eq!(
            started_springs(&recording.in_step("hover(Headline)")),
            Vec::new()
        );
        let step = press_down(&mut app);
        assert_eq!(
            started_springs(&recording.in_step(&step)),
            Vec::new(),
            "{step}"
        );
    }

    #[test]
    fn an_item_selected_no_more_goes_to_the_hovered_corners_while_the_pointer_is_over_it() {
        let (mut app, recording) = start(&Config {
            selected: Some(true),
            ..Config::default()
        });
        app.hover("Headline").unwrap();
        let before = Mounted::of(&recording.items());
        app.press("Select").unwrap();
        let _ = morph_and_recolor_in(&recording, "press(Select)", 12.0, &before);
        let mounted = Mounted::of(&recording.items());
        assert_eq!(
            container_color(&mounted),
            color(scheme_color(ColorRole::Surface, false), 255)
        );
    }

    #[test]
    fn a_disabled_item_shows_no_state_and_springs_nothing() {
        let (mut app, recording) = start(&Config {
            disabled: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
        assert_eq!(
            mounted
                .frames
                .get(&mounted.layer.expect("a layer"))
                .map(|frame| frame.size),
            Some(Size::default())
        );
        app.hover("Headline").unwrap();
        assert_eq!(
            started_springs(&recording.in_step("hover(Headline)")),
            Vec::new()
        );
        let _ = app.press_key("tab");
        assert_eq!(
            started_springs(&recording.in_step("keyboard.press(tab)")),
            Vec::new()
        );
        let step = press_down(&mut app);
        assert_eq!(
            started_springs(&recording.in_step(&step)),
            Vec::new(),
            "{step}"
        );
        // The platform may still report the hover of a disabled row, and the state layer, hidden, shows none of it.
        assert_eq!(
            Mounted::of(&recording.items()).layer_size(),
            Some(Size::default())
        );
        // Enabled, the state layer is laid out over the whole container.
        let mounted = mount(&Config::default());
        assert_eq!(
            mounted
                .frames
                .get(&mounted.layer.expect("a layer"))
                .map(|frame| frame.size),
            Some(mounted.size())
        );
    }

    #[test]
    fn an_item_disabled_while_pressed_goes_back_to_its_shape_at_rest() {
        let (mut app, recording) = start(&Config::default());
        let step = press_down(&mut app);
        let _ = morph_in(&recording, &step, 16.0);
        app.press("Disable").unwrap();
        let _ = spring_with(
            &recording.in_step("press(Disable)"),
            "press(Disable)",
            4.0,
            SpringToken::FastSpatial,
        );
    }

    #[test]
    fn a_plain_item_has_no_state_to_show_and_springs_nothing() {
        let (mut app, recording) = start(&Config {
            clickable: false,
            ..Config::default()
        });
        app.hover("Headline").unwrap();
        assert_eq!(
            started_springs(&recording.in_step("hover(Headline)")),
            Vec::new()
        );
        assert!(
            recording
                .in_step("hover(Headline)")
                .iter()
                .all(|item| !matches!(
                    item,
                    MountItem::Canvas { .. } | MountItem::Frame { .. } | MountItem::Text { .. }
                ))
        );
        let _ = app.press_key("tab");
        assert_eq!(
            started_springs(&recording.in_step("keyboard.press(tab)")),
            Vec::new()
        );
    }

    #[test]
    fn with_reduce_motion_nothing_springs_and_the_shape_stays_as_it_is() {
        // ADR 0011, "Reduce motion": no spring, and no jump either.
        let (mut app, recording) = start(&Config {
            motion: Motion::Reduced,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let at_rest = rounded_rectangle(mounted.size(), 0.0, 4.0);
        app.hover("Headline").unwrap();
        let step = press_down(&mut app);
        for step in ["hover(Headline)", step.as_str()] {
            let items = recording.in_step(step);
            assert_eq!(started_springs(&items), Vec::new(), "{step}");
            assert_eq!(set_values(&items), 0, "{step}");
        }
        let container = paths(mounted.drawing(mounted.container), &replay(&recording));
        assert_traces(&container[0], &at_rest, "pressed");
        app.drop_held().unwrap();
        assert_eq!(started_springs(&recording.in_step("drop()")), Vec::new());
        assert!(app.shows("presses: 1"));
    }

    // Where the values come from.

    /// The vendored `ListItem.kt` and `ListItemDefaults.kt`, at the pinned commit (ADR 0012).
    const LIST_ITEM_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ListItem.kt");
    const LIST_ITEM_DEFAULTS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ListItemDefaults.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_choice_taken_from_androidx_is_still_there() {
        // When androidx changes one of these, the line goes, and this fails in the bump's pull request.
        // The supporting text wraps above 30 sp, as the intrinsic estimate says (ListItem.kt, line 1294).
        assert!(cites(
            LIST_ITEM_KT,
            &["estimatedSupportingHeight > 30.sp.roundToPx()"]
        ));
        assert_eq!(SUPPORTING_WRAP_HEIGHT.to_bits(), 30.0_f32.to_bits());
        // The shapes and their order (ListItemDefaults.kt, lines 513 to 517 and 1146 to 1151).
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &[
                "shape = fromToken(ListTokens.ItemContainerExpressiveShape),",
                "selectedShape = fromToken(ListTokens.ItemSelectedContainerExpressiveShape),",
                "pressedShape = fromToken(ListTokens.ItemPressedContainerExpressiveShape),",
                "focusedShape = fromToken(ListTokens.ItemFocusedContainerExpressiveShape),",
                "hoveredShape = fromToken(ListTokens.ItemHoveredContainerExpressiveShape),",
            ]
        ));
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &[
                "pressed -> pressedShape",
                "dragged -> draggedShape",
                "selected -> selectedShape",
                "focused -> focusedShape",
                "hovered -> hoveredShape",
                "else -> shape",
            ]
        ));
        // The spring of the shape (ListItem.kt, line 1504).
        assert!(cites(
            LIST_ITEM_KT,
            &["val shapeAnimationSpec = MotionSchemeKeyTokens.FastSpatial.value<Float>()"]
        ));
        // The disabled container is the enabled one, not a disabled token (ListItemDefaults.kt, line 215), and the
        // colors of each part (lines 198 to 213).
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &["disabledContainerColor = fromToken(ListTokens.ItemContainerColor),"]
        ));
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &[
                "leadingContentColor = fromToken(ListTokens.ItemLeadingIconColor),",
                "trailingContentColor = fromToken(ListTokens.ItemTrailingIconColor),",
                "overlineContentColor = fromToken(ListTokens.ItemOverlineColor),",
                "supportingContentColor = fromToken(ListTokens.ItemSupportingTextColor),",
            ]
        ));
        // Disabled, then selected, decide the colors (ListItemDefaults.kt, lines 753 to 756).
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &[
                "!enabled -> disabledContainerColor",
                "dragged -> draggedContainerColor",
                "selected -> selectedContainerColor",
                "else -> containerColor",
            ]
        ));
        // The heights by count of lines (ListItem.kt, lines 1954 to 1956), the breakpoint of the alignment (line 1999)
        // and the alignment itself (ListItemDefaults.kt, lines 551 to 554).
        assert!(cites(
            LIST_ITEM_KT,
            &[
                "ListItemType.OneLine -> ListTokens.ItemOneLineContainerHeight",
                "ListItemType.TwoLine -> ListTokens.ItemTwoLineContainerHeight",
                "else /*ListItemType.ThreeLine*/ -> ListTokens.ItemThreeLineContainerHeight",
            ]
        ));
        assert!(cites(
            LIST_ITEM_KT,
            &[
                "(ListTokens.ItemThreeLineContainerHeight + ListTokens.ItemTwoLineContainerHeight) / 2 -",
                "ListItemDefaults.InteractiveListTopPadding -",
                "ListItemDefaults.InteractiveListBottomPadding",
            ]
        ));
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &[
                "if (space < breakpoint) {",
                "Alignment.CenterVertically",
                "} else {",
                "Alignment.Top",
            ]
        ));
        // The side spaces and the padding, as tokens, with the 12 points between the content and the text on the
        // leading and the trailing side (ListItem.kt, lines 1310 and 1335).
        assert!(cites(
            LIST_ITEM_KT,
            &["Box(Modifier.padding(end = InteractiveListInternalSpacing)) {"]
        ));
        assert!(cites(
            LIST_ITEM_KT,
            &["Box(Modifier.padding(start = InteractiveListInternalSpacing)) {"]
        ));
        assert!(cites(
            LIST_ITEM_DEFAULTS_KT,
            &["ListTokens.ItemTopSpace", "}"]
        ));
        // A selectable item is a radio button that is selected or not (ListItem.kt, lines 451 and 452).
        assert!(cites(
            LIST_ITEM_KT,
            &["this.selected = selected", "role = Role.RadioButton"]
        ));
        // A line that is not there is not found.
        assert!(!cites(
            LIST_ITEM_KT,
            &["estimatedSupportingHeight > 40.sp.roundToPx()"]
        ));
    }
}
