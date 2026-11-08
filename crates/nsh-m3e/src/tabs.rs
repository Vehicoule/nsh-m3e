//! The Material 3 Expressive tabs: a row of tabs, each a label, an icon or both, with the indicator that slides under
//! the selected one, over a divider. There are two rows: [`PrimaryTabRow`], whose indicator is a short bar under the
//! tab's content, and [`SecondaryTabRow`], whose indicator is as wide as the tab.
//!
//! The tabs follow androidx's `Tab`, `LeadingIconTab`, `PrimaryTabRow` and `SecondaryTabRow` in `Tab.kt` and
//! `TabRow.kt` at the pinned commit, as [ADR 0012] says, with the colors and sizes of `PrimaryNavigationTabTokens`,
//! `SecondaryNavigationTabTokens` and `DividerTokens`; every value they write themselves is cited where it is used.
//! The indicator is drawn by a canvas whose position and width are two animated values that the M3E spring moves, so
//! no layout runs while it slides ([ADR 0011], [ADR 0010]); androidx lays it out again on every frame.
//!
//! What this module does not have yet, and why, is in [ADR 0021]: the tab with its icon above its label, the scrollable
//! rows, and the fade of a tab's content color.
//!
//! [ADR 0010]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0010-measure-animations-in-ci.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0021]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0021-tabs-follow-the-tokens-and-leave-three-things-out.md

use std::f32::consts::{FRAC_PI_2, PI};

use nsh_core::LineHeight;
use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::icon::Icon;
use crate::motion::{SPRING_MASS, SpringToken};
use crate::navigation_item::{DISABLED_OPACITY, NavigationItemLabel};
use crate::tint::{Tint, role_color, tint_color};
use crate::tokens::androidx::{
    divider_tokens as divider, primary_navigation_tab_tokens as primary,
    secondary_navigation_tab_tokens as secondary, state_tokens,
};
use crate::tokens::{Shape, TypeStyle};

/// The space on each side of a tab's content: androidx `HorizontalTextPadding`, a literal 16 (`Tab.kt`, lines 422-424),
/// that no token holds.
const HORIZONTAL_PADDING: Points = Points(16.0);

/// The space between the icon and the label of a tab with both: androidx `TextDistanceFromLeadingIcon`, a literal 8
/// (`Tab.kt`, lines 440-442), that no token holds.
const ICON_TO_LABEL: Points = Points(8.0);

/// The narrowest the primary indicator gets: 24 points, "to enforce minimum touch target of 24.dp" (`TabRow.kt`, lines
/// 462-463), a literal.
const MIN_INDICATOR_WIDTH: Points = Points(24.0);

/// The label's type: `PrimaryNavigationTabTokens.LabelTextFont`, title small, for both rows (`Tab.kt`, lines 104-106
/// and 188-190: androidx reads the primary token for the tabs of both).
const LABEL: TypeStyle = primary::LABEL_TEXT_FONT;

/// Which of the two rows a tab is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// [`PrimaryTabRow`].
    Primary,
    /// [`SecondaryTabRow`].
    Secondary,
}

/// What a tab shows of the pointer and the keyboard: nothing, the pointer over it, the keyboard's focus on it, or a
/// press on it. The tokens name each its own colors for the label and the icon (ADR 0021).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interaction {
    /// None of the three.
    Rest,
    /// The pointer is over the tab.
    Hovered,
    /// The keyboard's focus is on the tab.
    Focused,
    /// The tab is being pressed.
    Pressed,
}

impl Interaction {
    /// What a tab shows of `self`: nothing when it is `disabled`, which takes no press and shows no state.
    fn shown(self, disabled: bool) -> Self {
        if disabled { Self::Rest } else { self }
    }
}

impl Kind {
    /// The row's color: `TabRowDefaults.primaryContainerColor` and `secondaryContainerColor` (`TabRow.kt`, lines
    /// 1000-1005).
    fn container(self) -> ColorRole {
        match self {
            Self::Primary => primary::CONTAINER_COLOR,
            Self::Secondary => secondary::CONTAINER_COLOR,
        }
    }

    /// The color a selected tab's label has, which is also the color of the state layer: androidx draws the state layer
    /// with `ripple(bounded = true, color = selectedContentColor)` (`Tab.kt`, lines 166 and 239), and
    /// `selectedContentColor` is the row's content color, `TabRowDefaults.primaryContentColor` and
    /// `secondaryContentColor` (`TabRow.kt`, lines 1016-1021), the tokens' active label colors.
    fn state_layer(self) -> ColorRole {
        match self {
            Self::Primary => primary::ACTIVE_LABEL_TEXT_COLOR,
            Self::Secondary => secondary::ACTIVE_LABEL_TEXT_COLOR,
        }
    }

    /// The colors of a tab's icon in `interaction`: the selected tab's, then the unselected one's. The primary tokens
    /// name both in each state; the secondary ones name one color for a state, and the same for both tabs.
    fn icon_roles(self, interaction: Interaction) -> (ColorRole, ColorRole) {
        match (self, interaction) {
            (Self::Primary, Interaction::Rest) => {
                (primary::ACTIVE_ICON_COLOR, primary::INACTIVE_ICON_COLOR)
            }
            (Self::Primary, Interaction::Hovered) => (
                primary::ACTIVE_HOVER_ICON_COLOR,
                primary::INACTIVE_HOVER_ICON_COLOR,
            ),
            (Self::Primary, Interaction::Focused) => (
                primary::ACTIVE_FOCUS_ICON_COLOR,
                primary::INACTIVE_FOCUS_ICON_COLOR,
            ),
            (Self::Primary, Interaction::Pressed) => (
                primary::ACTIVE_PRESSED_ICON_COLOR,
                primary::INACTIVE_PRESSED_ICON_COLOR,
            ),
            (Self::Secondary, Interaction::Rest) => {
                (secondary::ACTIVE_ICON_COLOR, secondary::INACTIVE_ICON_COLOR)
            }
            (Self::Secondary, Interaction::Hovered) => {
                (secondary::HOVER_ICON_COLOR, secondary::HOVER_ICON_COLOR)
            }
            (Self::Secondary, Interaction::Focused) => {
                (secondary::FOCUS_ICON_COLOR, secondary::FOCUS_ICON_COLOR)
            }
            (Self::Secondary, Interaction::Pressed) => {
                (secondary::PRESSED_ICON_COLOR, secondary::PRESSED_ICON_COLOR)
            }
        }
    }

    /// The colors of a tab's label in `interaction`, by the same rule as [`Self::icon_roles`].
    fn label_roles(self, interaction: Interaction) -> (ColorRole, ColorRole) {
        match (self, interaction) {
            (Self::Primary, Interaction::Rest) => (
                primary::ACTIVE_LABEL_TEXT_COLOR,
                primary::INACTIVE_LABEL_TEXT_COLOR,
            ),
            (Self::Primary, Interaction::Hovered) => (
                primary::ACTIVE_HOVER_LABEL_TEXT_COLOR,
                primary::INACTIVE_HOVER_LABEL_TEXT_COLOR,
            ),
            (Self::Primary, Interaction::Focused) => (
                primary::ACTIVE_FOCUS_LABEL_TEXT_COLOR,
                primary::INACTIVE_FOCUS_LABEL_TEXT_COLOR,
            ),
            (Self::Primary, Interaction::Pressed) => (
                primary::ACTIVE_PRESSED_LABEL_TEXT_COLOR,
                primary::INACTIVE_PRESSED_LABEL_TEXT_COLOR,
            ),
            (Self::Secondary, Interaction::Rest) => (
                secondary::ACTIVE_LABEL_TEXT_COLOR,
                secondary::INACTIVE_LABEL_TEXT_COLOR,
            ),
            (Self::Secondary, Interaction::Hovered) => (
                secondary::HOVER_LABEL_TEXT_COLOR,
                secondary::HOVER_LABEL_TEXT_COLOR,
            ),
            (Self::Secondary, Interaction::Focused) => (
                secondary::FOCUS_LABEL_TEXT_COLOR,
                secondary::FOCUS_LABEL_TEXT_COLOR,
            ),
            (Self::Secondary, Interaction::Pressed) => (
                secondary::PRESSED_LABEL_TEXT_COLOR,
                secondary::PRESSED_LABEL_TEXT_COLOR,
            ),
        }
    }

    /// The color of a tab's icon in `interaction`: the selected tab's when selected, the unselected one's when not,
    /// and when disabled the unselected one's at rest, faded: a disabled tab shows no state.
    fn icon(self, selected: bool, disabled: bool, interaction: Interaction) -> Tint {
        let (active, inactive) = self.icon_roles(interaction.shown(disabled));
        Self::tint(active, inactive, selected, disabled)
    }

    /// The color of a tab's label in `interaction`, by the same rule.
    fn label(self, selected: bool, disabled: bool, interaction: Interaction) -> Tint {
        let (active, inactive) = self.label_roles(interaction.shown(disabled));
        Self::tint(active, inactive, selected, disabled)
    }

    /// The tint of `active` or `inactive`; disabled, `inactive` at the opacity of a disabled component. The tab of
    /// androidx dims nothing, though its documentation says that a disabled tab "will appear visually disabled"
    /// (`Tab.kt`, lines 76-78): ADR 0021 says why this one follows the documentation.
    fn tint(active: ColorRole, inactive: ColorRole, selected: bool, disabled: bool) -> Tint {
        if disabled {
            Tint::faded(inactive, DISABLED_OPACITY)
        } else if selected {
            Tint::solid(active)
        } else {
            Tint::solid(inactive)
        }
    }

    /// Where the indicator goes for a selected tab at `place`, as its left edge and its width. The primary one is as
    /// wide as the tab's content, at least 24 points (androidx's `contentWidth`: the tab's intrinsic width less its
    /// padding on both sides, `TabRow.kt`, lines 458-463), centered under it; the secondary one is as wide as the tab
    /// (`matchContentSize = false`, line 211).
    ///
    /// androidx's fixed row puts the primary indicator at its tab's left edge (`TabRow.kt`, lines 721 and 751:
    /// the offset is `left`); its scrollable row centers it (lines 637-639), as Material's specification shows it, and
    /// so does this one (ADR 0021).
    fn indicator(self, place: Placement) -> (f32, f32) {
        match self {
            Self::Primary => {
                let content =
                    (place.content.0 - 2.0 * HORIZONTAL_PADDING.0).max(MIN_INDICATOR_WIDTH.0);
                (
                    place.left.0 + ((place.width.0 - content) / 2.0).max(0.0),
                    content,
                )
            }
            Self::Secondary => (place.left.0, place.width.0),
        }
    }
}

/// Where a tab is in its row, from its layout: its left edge, its width, and the width of its content with the padding
/// around it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Placement {
    left: Points,
    width: Points,
    content: Points,
}

impl Placement {
    /// Whether layout has given the tab and its content a size: until it has, the tab is nowhere.
    fn is_laid_out(self) -> bool {
        self.width.0 > 0.0 && self.content.0 > 0.0
    }
}

/// The spring of the indicator's slide: androidx `TabRowImpl` animates its offset and its width with
/// `MotionSchemeKeyTokens.DefaultSpatial` (`TabRow.kt`, line 408), converted as ADR 0004 says. Every field is written
/// out, none taken from NSH's defaults: it starts from the speed the value already has, at the screen's own rate (ADR
/// 0010).
fn indicator_spring() -> Spring {
    let spec = SpringToken::DefaultSpatial.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// The radius of the corners of the primary indicator, when it is `height` points tall: the token's corners, each cut
/// down so that the two of a side fit it, as Compose does (`RoundedCornerShape` scales radii that add up to more than a
/// side). The token is a corner of 3 on a bar of 3, so a capsule.
fn indicator_radius(height: f32) -> f32 {
    corner_radius(primary::ACTIVE_INDICATOR_SHAPE, height)
}

/// The radius `shape` gives the corners of a bar `height` points tall, cut down to half of it.
fn corner_radius(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::Rounded(corners) => corners.top_start.0.min(height / 2.0),
        Shape::Circle => height / 2.0,
        // A rectangle has none, and `Shape` is non-exhaustive: a shape added later is square.
        _ => 0.0,
    }
}

/// Whether a canvas of `size` has room to draw in: some width and some height.
fn has_room(size: Size<Points>) -> bool {
    size.width.0 > 0.0 && size.height.0 > 0.0
}

/// The indicator of a row: a bar under the selected tab, drawn by a canvas, whose left edge and width are animated
/// values. A `Copy` handle, like the signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Indicator {
    /// Where the bar's left edge is, in the row's points.
    left: AnimatedValue,
    /// How wide the bar is.
    width: AnimatedValue,
    /// The selected tab, by its number in the row, and where it is; none when no tab is selected.
    selected: Signal<Option<(usize, Placement)>>,
    /// Where the bar has been sent: its left edge and its width. None before it has been anywhere.
    going_to: Signal<Option<(f32, f32)>>,
    /// The size the canvas was laid out at.
    size: Signal<Size<Points>>,
}

impl Indicator {
    fn new() -> Self {
        Self {
            left: animated_value(0.0),
            width: animated_value(0.0),
            selected: signal(None),
            going_to: signal(None),
            size: signal(Size::default()),
        }
    }

    /// Tab number `id` says whether it is selected, and where it is. A tab that has not been laid out yet says
    /// nothing; the tab that was selected and is not anymore takes the indicator away, unless another one has it.
    fn report(self, id: usize, selected: bool, place: Placement) {
        untracked(|| {
            let current = self.selected.get();
            if selected {
                if place.is_laid_out() && current != Some((id, place)) {
                    self.selected.set(Some((id, place)));
                }
            } else if current.is_some_and(|(owner, _)| owner == id) {
                self.selected.set(None);
            }
        });
    }

    /// Tab number `id` is gone: if it had the indicator, the indicator has no tab. The row may be gone before its tabs.
    fn forget(self, id: usize) {
        let owns = self
            .selected
            .try_get()
            .flatten()
            .is_some_and(|(owner, _)| owner == id);
        if owns {
            let _ = self.selected.try_set(None);
        }
    }

    /// The selected tab, or none, changed or moved: the bar goes where it says. The first time it is set where it is,
    /// as androidx's `initialOffset` and `initialWidth` do (`TabRow.kt`, lines 690-735); after that it springs from
    /// where it is and as fast as it moves. With reduce motion on it goes there at once, as ADR 0011 says for the
    /// button's corners.
    fn follow(self, kind: Kind) {
        let selected = self.selected.get();
        untracked(|| {
            let Some((_, place)) = selected else {
                self.going_to.set(None);
                return;
            };
            let to = kind.indicator(place);
            let now = self.going_to.get();
            if now == Some(to) {
                return;
            }
            self.going_to.set(Some(to));
            if now.is_none() || reduce_motion() {
                self.left.set(to.0);
                self.width.set(to.1);
            } else {
                let _ = self.left.spring(to.0, indicator_spring());
                let _ = self.width.spring(to.1, indicator_spring());
            }
        });
    }

    /// The canvas of the bar: it fills the bottom of the row, as tall as the indicator, and takes no presses. The bar
    /// is the canvas view's background, over the divider (androidx draws it after, `TabRow.kt`, lines 500-504).
    fn canvas(self, kind: Kind) -> CanvasView {
        canvas(move || self.drawing(kind))
            .absolute()
            .left(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0))
            .h(primary::ACTIVE_INDICATOR_HEIGHT)
            .bg(tint_color(Some(Tint::solid(
                primary::ACTIVE_INDICATOR_COLOR,
            ))))
            .pointer_events(PointerEvents::None)
            .on_layout(move |bounds: &Bounds<Points>| {
                untracked(|| {
                    if self.size.get() != bounds.size {
                        self.size.set(bounds.size);
                    }
                });
            })
    }

    /// The bar, nothing when no tab is selected: a capsule as tall as the indicator for the primary row (androidx's
    /// `ActiveIndicatorShape` is a corner of 3 on a bar 3 points tall, which Compose cuts down to half the height), a
    /// rectangle for the secondary one. It goes through the animated left edge and width, so nothing is drawn again as
    /// they move.
    fn drawing(self, kind: Kind) -> Painter {
        let size = self.size.get();
        if self.selected.get().is_none() || !has_room(size) {
            return Painter::new();
        }
        let height = size.height.0;
        let area = Bounds::new(point(px(0.0), px(0.0)), size);
        let left = self.left.add(0.0);
        let right = self.left.add(&self.width);
        let path = Painter::new().begin_path();
        let path = match kind {
            Kind::Secondary => path
                .move_to(&left, 0.0)
                .line_to(&right, 0.0)
                .line_to(&right, height)
                .line_to(&left, height),
            Kind::Primary => {
                let radius = indicator_radius(height);
                let near_x = left.add(radius);
                let far_x = right.sub(radius);
                path.move_to(&near_x, 0.0)
                    .line_to(&far_x, 0.0)
                    .arc(&far_x, radius, radius, -FRAC_PI_2, 0.0)
                    .line_to(&right, height - radius)
                    .arc(&far_x, height - radius, radius, 0.0, FRAC_PI_2)
                    .line_to(&near_x, height)
                    .arc(&near_x, height - radius, radius, FRAC_PI_2, PI)
                    .line_to(&left, radius)
                    .arc(&near_x, radius, radius, PI, PI + FRAC_PI_2)
            }
        };
        path.close_path().fill_background_from_view(area)
    }
}

/// What a row tells its tabs: which row it is, where its indicator is, and how many tabs have asked for a number.
#[derive(Debug, Clone, Copy)]
struct RowContext {
    kind: Kind,
    indicator: Indicator,
    count: Signal<usize>,
}

/// The props of a [`PrimaryTabRow`] and of a [`SecondaryTabRow`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct TabRowProps {
    /// Its tabs: [`Tab`]s, each as wide as the others.
    pub children: Prop<Children>,
}

/// The row of a kind: the container, the tabs, the divider over the bottom of the tabs, and the indicator over it.
fn tab_row(kind: Kind, children: Prop<Children>) -> impl IntoElement {
    let indicator = Indicator::new();
    provide_context(RowContext {
        kind,
        indicator,
        count: signal(0),
    });
    effect(move || indicator.follow(kind));
    view()
        .relative()
        .bg(tint_color(Some(Tint::solid(kind.container()))))
        .child(nsh_base::tabs().flex_row().child(children))
        // androidx's `HorizontalDivider()` (`TabRow.kt`, lines 163 and 214): `DividerDefaults.color`, which is
        // `DividerTokens.Color`, and its thickness (`Divider.kt`, lines 111-119). The secondary tokens name another
        // color for it, `SurfaceVariant`; androidx's row never reads it (ADR 0012).
        .child(
            view()
                .absolute()
                .left(px(0.0))
                .right(px(0.0))
                .bottom(px(0.0))
                .h(divider::THICKNESS)
                .bg(role_color(divider::COLOR))
                .pointer_events(PointerEvents::None),
        )
        .child(indicator.canvas(kind))
}

/// A Material 3 Expressive primary tab row: the main destinations of a screen, under a top app bar. Its indicator is a
/// short bar under the content of the selected tab.
///
/// It is a tab list for screen readers and the keyboard, as `nsh-base`'s [`Tabs`](nsh_base::Tabs) is (arrow keys move
/// between the tabs and select them). Its tabs share its width equally.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::tabs::{primary_tab_row, tab};
///
/// fn tabs() -> impl IntoElement {
///     let selected = signal(0_usize);
///     primary_tab_row()
///         .child(
///             tab()
///                 .label("Songs")
///                 .selected(move || selected.get() == 0)
///                 .on_press(move || selected.set(0)),
///         )
///         .child(
///             tab()
///                 .label("Albums")
///                 .selected(move || selected.get() == 1)
///                 .on_press(move || selected.set(1)),
///         )
/// }
/// ```
#[component]
pub fn PrimaryTabRow(props: TabRowProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    tab_row(Kind::Primary, props.children)
}

/// A Material 3 Expressive secondary tab row: the content of an area, under the primary tabs. Its indicator is as wide
/// as the selected tab. Otherwise it is a [`PrimaryTabRow`]; its selected tabs are in the content color instead of the
/// primary one.
#[component]
pub fn SecondaryTabRow(props: TabRowProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    tab_row(Kind::Secondary, props.children)
}

/// The props of a [`Tab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct TabProps {
    /// What the tab says: its label, and its name for screen readers.
    pub label: Prop<SharedString>,
    /// Its icon, if it has one. With a label it goes before it (androidx `LeadingIconTab`).
    #[prop(default)]
    pub icon: Prop<Option<Icon>>,
    /// Whether it is the current tab: the indicator slides under it, and screen readers say it is selected. The app
    /// owns the value: a press reports [`TabEvent::Press`], and the app selects the tab.
    #[prop(default)]
    pub selected: Prop<bool>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
    /// Whether its label is shown. A tab with no icon shows it either way: it has nothing else to show.
    #[prop(default)]
    pub label_visibility: Prop<NavigationItemLabel>,
}

/// The events of a [`Tab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum TabEvent {
    /// The tab was pressed, or the keyboard focus moved to it: the app selects it. It is sent for the selected tab too.
    Press,
}

/// A tab of a [`PrimaryTabRow`] or a [`SecondaryTabRow`]: a label, an icon, or the icon before the label.
///
/// It has the behavior of `nsh-base`'s [`Tab`](nsh_base::Tab) (presses, focus, the disabled state, the tab role and the
/// selected state for screen readers) and the look of androidx's `Tab` and `LeadingIconTab`. Hover, focus and press lay
/// the row's content color over the whole tab at the state layer opacities of `StateTokens`, as androidx's ripple
/// does, and turn the label and the icon of an unselected tab from `OnSurfaceVariant` to `OnSurface`, as the tokens say
/// ([ADR 0021]); nothing else changes, so nothing is laid out again.
///
/// It is at least 48 points tall, the row's height and Material's touch target.
#[component]
pub fn Tab(props: TabProps, cx: Scope<TabEvent>) -> impl IntoElement {
    let context = use_context::<RowContext>();
    let kind = context.map_or(Kind::Primary, |context| context.kind);
    let place = signal(Placement::default());
    if let Some(context) = context {
        let id = untracked(|| {
            let id = context.count.get();
            context.count.set(id + 1);
            id
        });
        effect(move || {
            let selected = props.selected.get();
            let place = place.get();
            context.indicator.report(id, selected, place);
        });
        // A tab that goes away while selected takes the indicator with it, unless the row is gone first.
        cleanup(move || context.indicator.forget(id));
    }
    let has_icon = move || props.icon.get().is_some();
    let label_shown =
        move || !has_icon() || props.label_visibility.get() != NavigationItemLabel::Hidden;
    // The colors of the icon and of the label in each of the states the tab shows: the tokens name a color for each, and
    // an unselected tab's goes from gray to `OnSurface` under the pointer, at focus and on a press (ADR 0021).
    let icon_color = move |interaction| {
        move || {
            tint_color(Some(kind.icon(
                props.selected.get(),
                props.disabled.get(),
                interaction,
            )))
        }
    };
    let label_color = move |interaction| {
        move || {
            tint_color(Some(kind.label(
                props.selected.get(),
                props.disabled.get(),
                interaction,
            )))
        }
    };

    let icon = move || {
        canvas(move || {
            props
                .icon
                .get()
                .map_or_else(Painter::new, |icon| icon.drawing(primary::ICON_SIZE))
        })
        .w(primary::ICON_SIZE)
        .h(primary::ICON_SIZE)
        .fill_color(icon_color(Interaction::Rest))
        .group_hover(|s| s.fill_color(icon_color(Interaction::Hovered)))
        .group_focus_within(|s| s.fill_color(icon_color(Interaction::Focused)))
        .group_active(|s| s.fill_color(icon_color(Interaction::Pressed)))
    };
    let label = move || {
        text(move || props.label.get())
            .text_color(label_color(Interaction::Rest))
            .group_hover(|s| s.text_color(label_color(Interaction::Hovered)))
            .group_focus_within(|s| s.text_color(label_color(Interaction::Focused)))
            .group_active(|s| s.text_color(label_color(Interaction::Pressed)))
            .text_size(LABEL.size)
            .line_height(LineHeight::Points(LABEL.line_height))
            .letter_spacing(LABEL.tracking)
            .text_center()
            .style_with(|s| s.font_weight(LABEL.weight))
    };
    let state_layer = move || {
        view()
            .absolute()
            .inset(px(0.0))
            .bg(role_color(kind.state_layer()))
            .opacity(0.0)
            .pointer_events(PointerEvents::None)
            .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
            .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
            .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
            .style_with(move |s| {
                // A disabled tab takes no press and shows no state.
                if props.disabled.get() { s.hidden() } else { s }
            })
    };

    let tab = nsh_base::tab()
        .selected(move || props.selected.get())
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(TabEvent::Press))
        .group()
        .relative()
        .items_center()
        .justify_center()
        .min_h(primary::CONTAINER_HEIGHT)
        .child(
            // The content and the padding around it: what androidx measures for the indicator's width.
            view()
                .flex_row()
                .items_center()
                .justify_center()
                .px(HORIZONTAL_PADDING)
                .on_layout(move |bounds: &Bounds<Points>| {
                    untracked(|| {
                        let now = place.get();
                        if now.content != bounds.size.width {
                            place.set(Placement {
                                content: bounds.size.width,
                                ..now
                            });
                        }
                    });
                })
                .child(when(has_icon, icon))
                .child(when(label_shown, move || {
                    let label = label();
                    if has_icon() {
                        label.ms(ICON_TO_LABEL)
                    } else {
                        label
                    }
                })),
        )
        .child(state_layer());
    // The tab's share of the row, which says where it is: equal shares, whatever the content, as androidx gives each tab
    // `tabRowWidth / tabCount`. A view around the tab reports it, because the tab itself reports no layout.
    view()
        .flex_1()
        .on_layout(move |bounds: &Bounds<Points>| {
            untracked(|| {
                let now = place.get();
                let next = Placement {
                    left: bounds.origin.x,
                    width: bounds.size.width,
                    ..now
                };
                if next != now {
                    place.set(next);
                }
            });
        })
        .child(tab)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::canvas::{self, CanvasOp, Drawing};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{
        AccessibilityRole, AttributedText, Color, MountItem, NodeId, PointerEvents as Pointer,
        ViewKind, ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::ColorScheme;
    use crate::morph_probe::{
        Step, assert_traces, parameters, paths, replay, rounded_rectangle, set_values, tolerance,
    };

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ICON: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";
    const LABELS: [&str; 5] = ["Songs", "Albums", "Artists", "Genres", "Radio"];
    /// The width of the probe's window, which the root view and the row fill.
    const WINDOW: f32 = 390.0;

    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        secondary: Prop<bool>,
        count: Prop<usize>,
        selected: Prop<usize>,
        disabled: Prop<bool>,
        icon: Prop<bool>,
        /// Whether the app selects a tab when it is pressed, as an app does; when not, a press only counts.
        selecting: Prop<bool>,
        #[prop(default)]
        label_visibility: Prop<NavigationItemLabel>,
    }

    /// A tab of the probe's row: the app selects it when it is pressed, and counts the presses.
    fn one_tab(
        index: usize,
        selected: Signal<usize>,
        disabled: Signal<bool>,
        presses: Signal<u32>,
        icon: bool,
        selecting: bool,
        visibility: Prop<NavigationItemLabel>,
    ) -> impl IntoElement {
        tab()
            .label(LABELS[index])
            .icon(icon.then(|| Icon::material_symbol(ICON)))
            .selected(move || selected.get() == index)
            // The second tab is the one that gets disabled.
            .disabled(move || index == 1 && disabled.get())
            .label_visibility(visibility)
            .on_press(move || {
                presses.update(|n| *n += 1);
                if selecting {
                    selected.set(index);
                }
            })
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let selected = signal(props.selected.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        let presses = signal(0_u32);
        let last_shown = signal(true);
        let count = props.count.get_untracked();
        let icon = props.icon.get_untracked();
        let selecting = props.selecting.get_untracked();
        let visibility = props.label_visibility;
        let tabs: Vec<AnyElement> = (0..count)
            .map(|index| {
                if index + 1 == count {
                    // The last tab can go away, as an app's destinations can.
                    when(
                        move || last_shown.get(),
                        move || {
                            one_tab(
                                index, selected, disabled, presses, icon, selecting, visibility,
                            )
                        },
                    )
                    .into_element()
                } else {
                    one_tab(
                        index, selected, disabled, presses, icon, selecting, visibility,
                    )
                    .into_element()
                }
            })
            .collect();
        let row = if props.secondary.get_untracked() {
            secondary_tab_row().children(tabs).into_element()
        } else {
            primary_tab_row().children(tabs).into_element()
        };
        view()
            .child(row)
            .child(text(move || format!("selected: {}", selected.get())))
            .child(text(move || format!("presses: {}", presses.get())))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Hide last").on_press(move |(): &()| last_shown.set(false)))
    }

    /// Whether the system asks for reduced motion.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Motion {
        Full,
        Reduced,
    }

    /// What the probe's app does when a tab is pressed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum OnPress {
        /// It selects the tab, as an app does.
        Select,
        /// It only counts the press, so that the tab the keyboard reaches stays an unselected one.
        Count,
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        kind: Kind,
        count: usize,
        selected: usize,
        disabled: bool,
        icon: bool,
        on_press: OnPress,
        label_visibility: NavigationItemLabel,
        dark: bool,
        motion: Motion,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                kind: Kind::Primary,
                count: 3,
                selected: 0,
                disabled: false,
                icon: false,
                on_press: OnPress::Select,
                label_visibility: NavigationItemLabel::default(),
                dark: false,
                motion: Motion::Full,
            }
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            secondary: Prop::new(config.kind == Kind::Secondary),
            count: Prop::new(config.count),
            selected: Prop::new(config.selected),
            disabled: Prop::new(config.disabled),
            icon: Prop::new(config.icon),
            selecting: Prop::new(config.on_press == OnPress::Select),
            label_visibility: Prop::new(config.label_visibility),
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

    /// One tab's views.
    #[derive(Debug, Clone, Copy)]
    struct TabViews {
        /// The view that reports the tab's share of the row.
        share: NodeId,
        /// The press target, with the tab role: the whole tab.
        tab: NodeId,
        /// The view that holds the icon and the label, and their padding.
        content: NodeId,
        /// The canvas that draws the icon, when it has one.
        icon: Option<NodeId>,
        /// The label's text view, when it is shown.
        label: Option<NodeId>,
        /// The view that is the state layer.
        layer: NodeId,
    }

    impl TabViews {
        /// The views of the tab whose share view is `share`.
        fn of(
            share: NodeId,
            kinds: &HashMap<NodeId, ViewKind>,
            children: &HashMap<NodeId, Vec<NodeId>>,
            accessibility: &HashMap<NodeId, nsh_core::Accessibility>,
        ) -> Self {
            let [tab] = children[&share][..] else {
                panic!("one tab in a share");
            };
            assert_eq!(accessibility[&tab].role, AccessibilityRole::Tab);
            let [content, layer] = children[&tab][..] else {
                panic!("the content and the state layer: {:?}", children[&tab]);
            };
            let mut all = Vec::new();
            let mut todo = vec![content];
            while let Some(at) = todo.pop() {
                for child in children.get(&at).into_iter().flatten() {
                    all.push(*child);
                    todo.push(*child);
                }
            }
            let of_kind = |kind: ViewKind| -> Vec<NodeId> {
                all.iter()
                    .copied()
                    .filter(|id| kinds.get(id) == Some(&kind))
                    .collect()
            };
            let icons = of_kind(ViewKind::Canvas);
            let labels = of_kind(ViewKind::Text);
            assert!(
                icons.len() <= 1 && labels.len() <= 1,
                "{icons:?} {labels:?}"
            );
            Self {
                share,
                tab,
                content,
                icon: icons.first().copied(),
                label: labels.first().copied(),
                layer,
            }
        }
    }

    /// The row's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The container, under the list of tabs.
        container: NodeId,
        tabs: Vec<TabViews>,
        /// The divider, and the canvas of the indicator over it.
        divider: NodeId,
        indicator: NodeId,
        parents: HashMap<NodeId, NodeId>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        texts: HashMap<NodeId, Arc<AttributedText>>,
        accessibility: HashMap<NodeId, nsh_core::Accessibility>,
        pointer: HashMap<NodeId, Pointer>,
        children: HashMap<NodeId, Vec<NodeId>>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds: HashMap<NodeId, ViewKind> = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut parents = HashMap::new();
            let mut accessibility = HashMap::new();
            let mut pointer = HashMap::new();
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            let mut texts = HashMap::new();
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
                        let siblings = children.entry(*parent).or_default();
                        let at = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(at, *child);
                        parents.insert(*child, *parent);
                    }
                    MountItem::Remove { id } | MountItem::Delete { id } => {
                        if let Some(parent) = parents.remove(id)
                            && let Some(siblings) = children.get_mut(&parent)
                        {
                            siblings.retain(|child| child != id);
                        }
                    }
                    MountItem::ViewProps { id, props } => {
                        accessibility.insert(*id, props.accessibility.clone());
                        pointer.insert(*id, props.pointer_events);
                    }
                    MountItem::ViewStyle { id, style } => {
                        styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    MountItem::Canvas { id, drawing } => {
                        drawings.insert(*id, Arc::clone(drawing));
                    }
                    MountItem::Text { id, text, .. } => {
                        texts.insert(*id, Arc::clone(text));
                    }
                    _ => {}
                }
            }
            let role = |id: &NodeId| accessibility.get(id).map(|a| a.role);
            let list = *accessibility
                .keys()
                .find(|id| role(id) == Some(AccessibilityRole::Tablist))
                .expect("a view with the tab list role");
            let container = parents[&list];
            let [_, divider, indicator] = children[&container][..] else {
                panic!(
                    "the list, the divider and the indicator: {:?}",
                    children[&container]
                );
            };
            let tabs = children[&list]
                .iter()
                .map(|&share| TabViews::of(share, &kinds, &children, &accessibility))
                .collect();
            Self {
                container,
                tabs,
                divider,
                indicator,
                parents,
                styles,
                frames,
                drawings,
                texts,
                accessibility,
                pointer,
                children,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// The paths the indicator's canvas traces, none when it was never sent anything to draw.
        fn indicator_paths(&self, played: &Replay) -> Vec<Vec<Step>> {
            self.drawings
                .get(&self.indicator)
                .map_or_else(Vec::new, |drawing| paths(drawing, played))
        }

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }

        /// Where `id` is in the window and how big: left, top, width and height.
        fn rect(&self, id: NodeId) -> (f32, f32, f32, f32) {
            let (mut x, mut y) = (0.0, 0.0);
            let mut at = id;
            loop {
                if let Some(frame) = self.frames.get(&at) {
                    x += frame.origin.x.0;
                    y += frame.origin.y.0;
                }
                match self.parents.get(&at) {
                    Some(parent) => at = *parent,
                    None => break,
                }
            }
            let size = self.frame(id).size;
            (x, y, size.width.0, size.height.0)
        }

        fn label_text(&self, tab: &TabViews) -> &AttributedText {
            &self.texts[&tab.label.expect("a label")]
        }

        fn label_color(&self, tab: &TabViews) -> Color {
            self.label_text(tab).spans[0].style.color
        }

        fn icon_color(&self, tab: &TabViews) -> Color {
            fill(&self.drawing(tab.icon.expect("an icon")).ops).expect("the icon's fill")
        }

        /// How many of `items` lay out one of the row's views again.
        fn frames_in(&self, items: &[MountItem]) -> usize {
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Frame { id, .. } if self.parents.contains_key(id)))
                .count()
        }

        /// How wide the content of tab `index` is, padding included: what the indicator is as wide as, less the padding.
        fn content_width(&self, index: usize) -> f32 {
            self.frame(self.tabs[index].content).size.width.0
        }

        /// Where the indicator should be under tab `index` of a row of `kind`, from the layout: its left edge and its
        /// width, as androidx's `TabPosition` gives them (and the primary one centered, as ADR 0021 says).
        fn bar(&self, kind: Kind, index: usize) -> (f32, f32) {
            let share = self.frame(self.tabs[index].share);
            let (left, width) = (share.origin.x.0, share.size.width.0);
            match kind {
                Kind::Primary => {
                    let bar = (self.content_width(index) - 32.0).max(24.0);
                    (left + (width - bar) / 2.0, bar)
                }
                Kind::Secondary => (left, width),
            }
        }
    }

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// The row of the kind a test says in a boolean: secondary or not.
    fn kind(secondary: bool) -> Kind {
        if secondary {
            Kind::Secondary
        } else {
            Kind::Primary
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

    fn solid(role: ColorRole, dark: bool) -> Color {
        color(scheme_color(role, dark), 255)
    }

    /// The 8-bit alpha NSH rounds 0.38 of 255 to.
    const ALPHA_38: u32 = 97;

    /// The color `ops` fill with, if any.
    fn fill(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    /// `found` is `wanted`, give or take the rounding of layout: NSH lays out on whole points.
    fn assert_near(found: f32, wanted: f32, what: &str) {
        assert!(
            (found - wanted).abs() <= 1.0,
            "{what}: {found} is not {wanted}"
        );
    }

    const KINDS: [bool; 2] = [false, true];

    #[test]
    fn the_row_is_a_tab_list_of_tabs_named_by_their_labels() {
        for secondary in KINDS {
            let mounted = mount(Config {
                kind: kind(secondary),
                selected: 2,
                ..Config::default()
            });
            assert_eq!(mounted.tabs.len(), 3);
            for (index, tab) in mounted.tabs.iter().enumerate() {
                let access = &mounted.accessibility[&tab.tab];
                assert_eq!(access.role, AccessibilityRole::Tab);
                assert_eq!(access.label.as_deref(), Some(LABELS[index]));
                assert_eq!(access.state.selected, index == 2, "{index}");
                assert!(!access.state.disabled, "{index}");
                assert_eq!(mounted.label_text(tab).text, LABELS[index]);
            }
        }
    }

    #[test]
    fn a_disabled_tab_says_so_to_screen_readers() {
        let mounted = mount(Config {
            disabled: true,
            ..Config::default()
        });
        let disabled: Vec<bool> = mounted
            .tabs
            .iter()
            .map(|tab| mounted.accessibility[&tab.tab].state.disabled)
            .collect();
        assert_eq!(disabled, [false, true, false]);
    }

    #[test]
    fn the_container_the_divider_and_the_indicator_have_their_colors_light_and_dark() {
        for secondary in KINDS {
            for dark in [false, true] {
                let what = format!("secondary: {secondary} dark: {dark}");
                let mounted = mount(Config {
                    kind: kind(secondary),
                    dark,
                    ..Config::default()
                });
                // `Surface` for both rows (`TabRowDefaults.primaryContainerColor`, `secondaryContainerColor`), with no
                // shadow and square corners (`CornerNone`, level 0).
                let container = mounted.style(mounted.container);
                assert_eq!(
                    container.background_color,
                    solid(ColorRole::Surface, dark),
                    "{what}"
                );
                assert_eq!(container.elevation, px(0.0), "{what}");
                assert!(container.effects.is_none(), "{what}");
                assert_eq!(container.border_radius, px(0.0), "{what}");
                // `HorizontalDivider()`: `DividerTokens.Color`, 1 point.
                let divider = mounted.style(mounted.divider);
                assert_eq!(
                    divider.background_color,
                    solid(ColorRole::OutlineVariant, dark),
                    "{what}"
                );
                // The indicator is the primary one's color, in both rows (`SecondaryIndicator` reads the primary tokens).
                assert_eq!(
                    fill(&mounted.drawing(mounted.indicator).ops),
                    Some(solid(ColorRole::Primary, dark)),
                    "{what}"
                );
                // Neither takes a press.
                for id in [mounted.divider, mounted.indicator] {
                    assert_eq!(mounted.pointer[&id], Pointer::None, "{what}");
                }
                // The tabs paint nothing of their own around their content.
                for tab in &mounted.tabs {
                    assert_eq!(
                        mounted.style(tab.tab).background_color,
                        Color::TRANSPARENT,
                        "{what}"
                    );
                }
            }
        }
    }

    /// The icon's and the label's colors a tab should have, in the baseline schemes: the tokens' active and inactive
    /// colors (ADR 0021), the inactive ones faded when disabled.
    fn expected_colors(kind: Kind, selected: bool, disabled: bool, dark: bool) -> (Color, Color) {
        let faded = |role| color(scheme_color(role, dark), ALPHA_38);
        match (disabled, selected, kind == Kind::Secondary) {
            (true, ..) => (
                faded(ColorRole::OnSurfaceVariant),
                faded(ColorRole::OnSurfaceVariant),
            ),
            (false, true, false) => (
                solid(ColorRole::Primary, dark),
                solid(ColorRole::Primary, dark),
            ),
            (false, true, true) => (
                solid(ColorRole::OnSurface, dark),
                solid(ColorRole::OnSurface, dark),
            ),
            (false, false, _) => (
                solid(ColorRole::OnSurfaceVariant, dark),
                solid(ColorRole::OnSurfaceVariant, dark),
            ),
        }
    }

    #[test]
    fn every_tab_has_its_colors_in_each_state_light_and_dark() {
        for secondary in KINDS {
            for dark in [false, true] {
                for disabled in [false, true] {
                    for selected in [false, true] {
                        let what = format!(
                            "secondary: {secondary} dark: {dark} disabled: {disabled} selected: {selected}"
                        );
                        // The second tab is the one that is disabled, and the one selected here.
                        let mounted = mount(Config {
                            kind: kind(secondary),
                            selected: usize::from(selected),
                            disabled,
                            icon: true,
                            dark,
                            ..Config::default()
                        });
                        let tab = mounted.tabs[1];
                        let (icon, label) =
                            expected_colors(kind(secondary), selected, disabled, dark);
                        assert_eq!(mounted.icon_color(&tab), icon, "{what}: icon");
                        assert_eq!(mounted.label_color(&tab), label, "{what}: label");
                        // The state layer is the row's content color, whatever the tab's state: androidx's ripple is
                        // `selectedContentColor`.
                        let layer = if secondary {
                            ColorRole::OnSurface
                        } else {
                            ColorRole::Primary
                        };
                        assert_eq!(
                            mounted.style(tab.layer).background_color,
                            solid(layer, dark),
                            "{what}: state layer"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_label_is_title_small_centered() {
        let mounted = mount(Config::default());
        let text = mounted.label_text(&mounted.tabs[0]);
        let span = &text.spans[0].style;
        // `PrimaryNavigationTabTokens.LabelTextFont` is `TitleSmall`: 14 points, a 20 point line, medium weight, 0.1 of
        // tracking.
        assert_eq!(span.font_size, px(14.0));
        assert_eq!(span.line_height, px(20.0));
        assert_eq!(span.font_weight, 500);
        assert_eq!(span.letter_spacing, px(0.1));
        assert_eq!(text.align, nsh_core::TextAlign::Center);
        // Both rows read the primary token, and the secondary one says the same.
        assert_eq!(secondary::LABEL_TEXT_FONT, primary::LABEL_TEXT_FONT);
    }

    #[test]
    fn a_tab_is_nowhere_until_it_and_its_content_have_a_width() {
        let at = |width: f32, content: f32| Placement {
            left: Points(10.0),
            width: Points(width),
            content: Points(content),
        };
        assert!(at(100.0, 40.0).is_laid_out());
        assert!(at(0.5, 0.5).is_laid_out());
        assert!(!at(0.0, 40.0).is_laid_out());
        assert!(!at(100.0, 0.0).is_laid_out());
        assert!(!at(0.0, 0.0).is_laid_out());
        assert!(!Placement::default().is_laid_out());
    }

    #[test]
    fn a_bar_has_room_to_draw_only_with_some_width_and_some_height() {
        let size = |width: f32, height: f32| Size::new(px(width), px(height));
        assert!(has_room(size(24.0, 3.0)));
        assert!(has_room(size(0.5, 0.5)));
        assert!(!has_room(size(0.0, 3.0)));
        assert!(!has_room(size(24.0, 0.0)));
        assert!(!has_room(size(0.0, 0.0)));
        assert!(!has_room(Size::default()));
    }

    #[test]
    fn a_corner_is_cut_down_to_half_the_bar_whatever_the_shape() {
        let rounded = |radius: f32| Shape::rounded_all(Points(radius));
        // A radius that fits is the radius; one that does not is half the height (a capsule).
        assert_eq!(corner_radius(rounded(1.0), 3.0), 1.0);
        assert_eq!(corner_radius(rounded(3.0), 3.0), 1.5);
        assert_eq!(corner_radius(rounded(9.0), 8.0), 4.0);
        // `CircleShape` is half the height, a rectangle has no corner.
        assert_eq!(corner_radius(Shape::Circle, 3.0), 1.5);
        assert_eq!(corner_radius(Shape::Circle, 8.0), 4.0);
        assert_eq!(corner_radius(Shape::Rectangle, 3.0), 0.0);
        // The token is a corner of 3 on a bar of 3: a capsule, at the height the token gives.
        assert_eq!(indicator_radius(3.0), 1.5);
        assert_eq!(primary::ACTIVE_INDICATOR_HEIGHT, px(3.0));
    }

    #[test]
    fn the_icon_is_its_path_at_the_token_size() {
        for secondary in KINDS {
            let mounted = mount(Config {
                kind: kind(secondary),
                icon: true,
                ..Config::default()
            });
            let tab = mounted.tabs[0];
            let icon = tab.icon.expect("an icon");
            assert_eq!(mounted.frame(icon).size, Size::new(px(24.0), px(24.0)));
            assert_eq!(primary::ICON_SIZE, secondary::ICON_SIZE);
            assert_eq!(primary::ICON_SIZE, px(24.0));
        }
    }

    #[test]
    fn the_tabs_share_the_row_equally_and_are_48_points_tall() {
        for icon in [false, true] {
            let mounted = mount(Config {
                icon,
                ..Config::default()
            });
            for (index, tab) in mounted.tabs.iter().enumerate() {
                let (x, _, width, height) = mounted.rect(tab.tab);
                #[expect(clippy::cast_precision_loss, reason = "a handful of tabs")]
                let left = WINDOW / 3.0 * index as f32;
                assert_near(x, left, "tab");
                assert_near(width, WINDOW / 3.0, "tab");
                assert_eq!(height, 48.0, "icon: {icon}");
            }
            // The row is as wide as the window and as tall as its tabs: the divider and the indicator are over its bottom.
            assert_eq!(mounted.rect(mounted.container), (0.0, 0.0, WINDOW, 48.0));
        }
    }

    #[test]
    fn a_tab_with_an_icon_and_a_label_puts_the_icon_first_and_8_points_before_the_label() {
        let mounted = mount(Config {
            icon: true,
            ..Config::default()
        });
        let tab = mounted.tabs[0];
        let (icon_x, icon_y, icon_w, icon_h) = mounted.rect(tab.icon.expect("an icon"));
        let (label_x, ..) = mounted.rect(tab.label.expect("a label"));
        assert_eq!((icon_w, icon_h), (24.0, 24.0));
        assert_near(label_x, icon_x + icon_w + 8.0, "label");
        // Both are centered in the 48 points of the tab, and so is the pair in the tab's width.
        assert_near(icon_y, 12.0, "icon");
        let (tab_x, _, tab_w, _) = mounted.rect(tab.tab);
        let (content_x, _, content_w, _) = mounted.rect(tab.content);
        assert_near(content_x + content_w / 2.0, tab_x + tab_w / 2.0, "content");
        // 16 points of padding on each side of the icon and the label.
        assert_near(icon_x, content_x + 16.0, "icon");
    }

    #[test]
    fn a_hidden_label_leaves_the_icon_alone_and_a_tab_with_no_icon_shows_its_label_anyway() {
        let mounted = mount(Config {
            icon: true,
            label_visibility: NavigationItemLabel::Hidden,
            ..Config::default()
        });
        for tab in &mounted.tabs {
            assert!(tab.icon.is_some());
            assert!(tab.label.is_none());
        }
        // Its name for screen readers stays.
        let access = &mounted.accessibility[&mounted.tabs[0].tab];
        assert_eq!(access.label.as_deref(), Some("Songs"));
        // The same tabs with no icon have nothing else to show.
        let mounted = mount(Config {
            label_visibility: NavigationItemLabel::Hidden,
            ..Config::default()
        });
        for tab in &mounted.tabs {
            assert!(tab.icon.is_none());
            assert!(tab.label.is_some());
        }
    }

    #[test]
    fn the_divider_and_the_indicator_lie_over_the_bottom_of_the_row_the_indicator_over_the_divider()
    {
        for secondary in KINDS {
            let mounted = mount(Config {
                kind: kind(secondary),
                ..Config::default()
            });
            let (_, top, width, height) = mounted.rect(mounted.divider);
            assert_eq!((top, width, height), (47.0, WINDOW, 1.0));
            let (_, top, width, height) = mounted.rect(mounted.indicator);
            assert_eq!((top, width, height), (45.0, WINDOW, 3.0));
            // The indicator is the last child, drawn over the divider.
            assert_eq!(
                mounted.children[&mounted.container].last(),
                Some(&mounted.indicator)
            );
        }
    }

    #[test]
    fn a_press_reports_and_the_app_selects_the_tab() {
        let (mut app, recording) = start(Config::default());
        app.press("label:Artists").unwrap();
        assert!(app.shows("presses: 1"));
        assert!(app.shows("selected: 2"));
        let mounted = Mounted::of(&recording.items());
        let selected: Vec<bool> = mounted
            .tabs
            .iter()
            .map(|tab| mounted.accessibility[&tab.tab].state.selected)
            .collect();
        assert_eq!(selected, [false, false, true]);
        // The press of the selected tab is reported too.
        app.press("label:Artists").unwrap();
        assert!(app.shows("presses: 2"));
        assert!(app.shows("selected: 2"));
    }

    #[test]
    fn a_disabled_tab_takes_no_press_also_when_it_is_disabled_later() {
        let (mut app, _recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Albums");
        assert!(app.shows("presses: 0"));
        assert!(app.shows("selected: 0"));
        app.press("label:Artists").unwrap();
        assert!(app.shows("selected: 2"));

        let (mut app, recording) = start(Config::default());
        app.press("Disable").unwrap();
        let _ = app.press("label:Albums");
        assert!(app.shows("presses: 0"));
        let mounted = Mounted::of(&recording.items());
        assert!(mounted.accessibility[&mounted.tabs[1].tab].state.disabled);
    }

    #[test]
    fn the_arrow_keys_move_between_the_tabs_and_select_them() {
        let (mut app, _recording) = start(Config::default());
        app.press_key("tab").unwrap();
        app.press_key("right").unwrap();
        assert!(app.shows("selected: 1"));
        assert!(app.shows("presses: 1"));
    }

    /// The opacity the state layer of tab `index` was last given.
    fn layer_opacity(recording: &Recording, index: usize) -> f32 {
        let mounted = Mounted::of(&recording.items());
        mounted.style(mounted.tabs[index].layer).opacity
    }

    /// The platform reports `states` of view `id` (the headless driver reports hover and focus itself, not a press).
    fn report(app: &mut Headless, id: NodeId, states: u16) {
        app.step("report");
        app.engine().control().set_states(id, states);
        app.settle();
    }

    #[test]
    fn hover_shows_the_state_layer_and_the_hover_colors_and_lays_nothing_out() -> Result<(), String>
    {
        let (mut app, recording) = start(Config {
            icon: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        for index in 0..3 {
            assert_eq!(layer_opacity(&recording, index), 0.0, "{index}");
        }
        app.hover("label:Albums")?;
        let hover = recording.in_step("hover(label:Albums)");
        // `StateTokens.HoverStateLayerOpacity`, over the whole tab.
        assert_eq!(layer_opacity(&recording, 1), 0.08);
        assert_eq!(layer_opacity(&recording, 0), 0.0);
        assert_eq!(layer_opacity(&recording, 2), 0.0);
        // The tab's state layer, its icon (drawn again in its new color) and its label (in its new color) are what is
        // sent, and nothing else: no frame, so nothing is laid out again, and nothing of any other tab.
        let tab = mounted.tabs[1];
        assert!(
            hover.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } => *id == tab.layer,
                MountItem::Canvas { id, .. } => Some(*id) == tab.icon,
                MountItem::Text { id, .. } => Some(*id) == tab.label,
                _ => false,
            }),
            "{hover:?}"
        );
        assert_eq!(mounted.frames_in(&hover), 0);
        // The layer covers the tab, from edge to edge.
        assert_eq!(
            mounted.frame(mounted.tabs[1].layer),
            Bounds::new(
                point(px(0.0), px(0.0)),
                mounted.frame(mounted.tabs[1].tab).size
            )
        );
        Ok(())
    }

    #[test]
    fn a_press_shows_the_pressed_state_layer_over_the_hover_one_until_it_ends() {
        use nsh_core::tailwind::state::{HOVERED, PRESSED};
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let tab = mounted.tabs[2].tab;
        let tracked = recording.items().iter().find_map(|item| match item {
            MountItem::TrackStates { id, states } if *id == tab => Some(*states),
            _ => None,
        });
        assert_eq!(tracked, Some(HOVERED | PRESSED));
        report(&mut app, tab, HOVERED);
        assert_eq!(layer_opacity(&recording, 2), 0.08);
        report(&mut app, tab, HOVERED | PRESSED);
        assert_eq!(layer_opacity(&recording, 2), 0.1);
        report(&mut app, tab, HOVERED);
        assert_eq!(layer_opacity(&recording, 2), 0.08);
        report(&mut app, tab, 0);
        assert_eq!(layer_opacity(&recording, 2), 0.0);
        assert_eq!(layer_opacity(&recording, 0), 0.0);
        assert_eq!(layer_opacity(&recording, 1), 0.0);
    }

    /// The colors of tab `index`'s icon and label, as last sent.
    fn tab_colors(recording: &Recording, index: usize) -> (Color, Color) {
        let mounted = Mounted::of(&recording.items());
        let tab = mounted.tabs[index];
        (mounted.icon_color(&tab), mounted.label_color(&tab))
    }

    #[test]
    fn an_unselected_tab_goes_on_surface_when_hovered_or_pressed_and_comes_back() {
        use nsh_core::tailwind::state::{HOVERED, PRESSED};
        for secondary in KINDS {
            for dark in [false, true] {
                let (mut app, recording) = start(Config {
                    kind: kind(secondary),
                    icon: true,
                    dark,
                    ..Config::default()
                });
                let mounted = Mounted::of(&recording.items());
                // The third tab is neither selected (the first) nor disabled.
                let id = mounted.tabs[2].tab;
                let rest = solid(ColorRole::OnSurfaceVariant, dark);
                let on_surface = solid(ColorRole::OnSurface, dark);
                for (states, what) in [(HOVERED, "hovered"), (HOVERED | PRESSED, "pressed")] {
                    let what = format!("secondary: {secondary} dark: {dark} {what}");
                    assert_eq!(tab_colors(&recording, 2), (rest, rest), "{what}: at rest");
                    report(&mut app, id, states);
                    // The tokens' `Inactive{Hover,Pressed}{Icon,LabelText}Color`, and the secondary row's
                    // `{Hover,Pressed}{Icon,LabelText}Color`: `OnSurface` in each.
                    assert_eq!(
                        tab_colors(&recording, 2),
                        (on_surface, on_surface),
                        "{what}"
                    );
                    // The other tabs show nothing of it.
                    assert_eq!(
                        tab_colors(&recording, 1),
                        (rest, rest),
                        "{what}: the second tab"
                    );
                    report(&mut app, id, 0);
                    assert_eq!(tab_colors(&recording, 2), (rest, rest), "{what}: after");
                }
            }
        }
    }

    #[test]
    fn an_unselected_tab_the_keyboard_reaches_goes_on_surface_until_the_focus_leaves() {
        for secondary in KINDS {
            for dark in [false, true] {
                // The app leaves the selection where it is when a tab is pressed, so the tab that the arrow key reaches is
                // an unselected one with the focus.
                let (mut app, recording) = start(Config {
                    kind: kind(secondary),
                    icon: true,
                    on_press: OnPress::Count,
                    dark,
                    ..Config::default()
                });
                let what = format!("secondary: {secondary} dark: {dark}");
                let rest = solid(ColorRole::OnSurfaceVariant, dark);
                let on_surface = solid(ColorRole::OnSurface, dark);
                app.press_key("tab").unwrap();
                app.press_key("right").unwrap();
                assert!(app.shows("selected: 0"), "{what}");
                // `Inactive{Focus}{Icon,LabelText}Color`, and the secondary row's `Focus...Color`.
                assert_eq!(layer_opacity(&recording, 1), 0.1, "{what}");
                assert_eq!(
                    tab_colors(&recording, 1),
                    (on_surface, on_surface),
                    "{what}"
                );
                assert_eq!(tab_colors(&recording, 2), (rest, rest), "{what}: the third");
                app.press_key("right").unwrap();
                assert_eq!(layer_opacity(&recording, 1), 0.0, "{what}");
                assert_eq!(tab_colors(&recording, 1), (rest, rest), "{what}: after");
                assert_eq!(
                    tab_colors(&recording, 2),
                    (on_surface, on_surface),
                    "{what}: the third, focused"
                );
            }
        }
    }

    #[test]
    fn a_selected_tab_keeps_its_color_when_hovered_focused_or_pressed() {
        use nsh_core::tailwind::state::{HOVERED, PRESSED};
        for secondary in KINDS {
            for dark in [false, true] {
                let (mut app, recording) = start(Config {
                    kind: kind(secondary),
                    icon: true,
                    dark,
                    ..Config::default()
                });
                let mounted = Mounted::of(&recording.items());
                let id = mounted.tabs[0].tab;
                // `Active{Hover,Focus,Pressed}...Color` of the primary row are `Primary` like the rest ones, and the
                // secondary row's `{Hover,Focus,Pressed}...Color` are `OnSurface` like its active ones.
                let (icon, label) = expected_colors(kind(secondary), true, false, dark);
                let what = format!("secondary: {secondary} dark: {dark}");
                for states in [HOVERED, HOVERED | PRESSED] {
                    report(&mut app, id, states);
                    assert_eq!(
                        tab_colors(&recording, 0),
                        (icon, label),
                        "{what} states: {states}"
                    );
                }
                report(&mut app, id, 0);
                app.press_key("tab").unwrap();
                assert_eq!(layer_opacity(&recording, 0), 0.1, "{what}: focused");
                assert_eq!(tab_colors(&recording, 0), (icon, label), "{what}: focused");
            }
        }
    }

    #[test]
    fn a_disabled_tab_keeps_its_faded_color_whatever_the_pointer_does() {
        use nsh_core::tailwind::state::{HOVERED, PRESSED};
        for secondary in KINDS {
            let (mut app, recording) = start(Config {
                kind: kind(secondary),
                icon: true,
                disabled: true,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let id = mounted.tabs[1].tab;
            let rest = tab_colors(&recording, 1);
            let faded = color(scheme_color(ColorRole::OnSurfaceVariant, false), ALPHA_38);
            assert_eq!(rest, (faded, faded), "secondary: {secondary}");
            for states in [HOVERED, HOVERED | PRESSED] {
                report(&mut app, id, states);
                assert_eq!(
                    tab_colors(&recording, 1),
                    rest,
                    "secondary: {secondary} states: {states}"
                );
            }
        }
    }

    #[test]
    fn keyboard_focus_shows_the_focused_state_layer() {
        let (mut app, recording) = start(Config::default());
        app.press_key("tab").unwrap();
        // `StateTokens.FocusStateLayerOpacity`, on the selected tab, which is the one the keyboard reaches first.
        assert_eq!(layer_opacity(&recording, 0), 0.1);
        assert_eq!(layer_opacity(&recording, 1), 0.0);
    }

    #[test]
    fn a_disabled_tab_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        // Its state layer is not laid out (`hidden`): it has no size to show a state in.
        assert_eq!(mounted.frame(mounted.tabs[1].layer).size, Size::default());
        assert_ne!(mounted.frame(mounted.tabs[0].layer).size, Size::default());
        let _ = app.hover("label:Albums");
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.frame(mounted.tabs[1].layer).size, Size::default());
    }

    fn press_step(selector: &str) -> String {
        format!("press({selector})")
    }

    /// `spring` is the indicator's: `DefaultSpatial`, converted as ADR 0004 says, to `to`, from where the value is and
    /// as fast as it moves, at the screen's rate.
    fn assert_indicator_spring(spring: &StartedSpring, to: f32, what: &str) {
        let spec = SpringToken::DefaultSpatial.spec();
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

    /// The two springs the indicator starts, the left edge's and the width's: the left edge's value is the first the row
    /// makes, so its node comes first, and each goes where `to` says (the two can go to the same number).
    fn springs_of(springs: &[StartedSpring], to: (f32, f32)) -> (StartedSpring, StartedSpring) {
        let mut ours = springs.to_vec();
        ours.sort_by_key(|spring| spring.node);
        let [left, width] = ours[..] else {
            panic!("two springs, not {springs:?}");
        };
        assert_eq!(left.to.to_bits(), to.0.to_bits(), "left: {springs:?}");
        assert_eq!(width.to.to_bits(), to.1.to_bits(), "width: {springs:?}");
        (left, width)
    }

    /// `steps` moved `dx` to the right.
    fn shifted(steps: Vec<Step>, dx: f32) -> Vec<Step> {
        steps
            .into_iter()
            .map(|step| match step {
                Step::Move(x, y) => Step::Move(x + dx, y),
                Step::Line(x, y) => Step::Line(x + dx, y),
                Step::Arc {
                    x,
                    y,
                    radius,
                    start,
                    end,
                    ccw,
                } => Step::Arc {
                    x: x + dx,
                    y,
                    radius,
                    start,
                    end,
                    ccw,
                },
                Step::Close => Step::Close,
            })
            .collect()
    }

    /// The indicator of a row of `kind`, `width` wide with its left edge at `left`, as the canvas traces it: a capsule 3
    /// points tall for the primary row (ADR 0011's rounded rectangle), a rectangle for the secondary one.
    fn bar_at(kind: Kind, left: f32, width: f32) -> Vec<Step> {
        match kind {
            Kind::Primary => shifted(
                rounded_rectangle(Size::new(Points(width), Points(3.0)), 0.0, 1.5),
                left,
            ),
            Kind::Secondary => vec![
                Step::Move(left, 0.0),
                Step::Line(left + width, 0.0),
                Step::Line(left + width, 3.0),
                Step::Line(left, 3.0),
                Step::Close,
            ],
        }
    }

    #[test]
    fn the_indicator_at_rest_is_under_the_selected_tab_and_nowhere_else() {
        for secondary in KINDS {
            for icon in [false, true] {
                for selected in 0..3 {
                    let config = Config {
                        kind: kind(secondary),
                        selected,
                        icon,
                        ..Config::default()
                    };
                    let what = format!("secondary: {secondary} icon: {icon} selected: {selected}");
                    let (_app, recording) = start(config);
                    let mounted = Mounted::of(&recording.items());
                    let rest = replay(&recording);
                    let drawn = mounted.indicator_paths(&rest);
                    assert_eq!(drawn.len(), 1, "{what}");
                    let (left, width) = mounted.bar(config.kind, selected);
                    assert_traces(&drawn[0], &bar_at(config.kind, left, width), &what);
                }
            }
        }
    }

    #[test]
    fn the_primary_indicator_is_as_wide_as_the_content_and_at_least_24_points_centered_under_it() {
        let mounted = mount(Config::default());
        for index in 0..3 {
            let (left, width) = mounted.bar(Kind::Primary, index);
            let share = mounted.frame(mounted.tabs[index].share);
            // The content is the label and 16 points of padding on each side: the bar is the label's width.
            assert_eq!(width, mounted.content_width(index) - 32.0, "{index}");
            assert!(width >= 24.0, "{index}");
            // Centered in the tab (the label is: the content is centered, the padding is equal).
            assert_near(
                left + width / 2.0,
                share.origin.x.0 + share.size.width.0 / 2.0,
                "centered",
            );
        }
        // An icon alone is 24 points wide, which the minimum leaves as it is.
        let mounted = mount(Config {
            icon: true,
            label_visibility: NavigationItemLabel::Hidden,
            ..Config::default()
        });
        for index in 0..3 {
            assert_eq!(mounted.bar(Kind::Primary, index).1, 24.0, "{index}");
        }
    }

    #[test]
    fn the_secondary_indicator_is_as_wide_as_the_tab() {
        let mounted = mount(Config {
            kind: Kind::Secondary,
            ..Config::default()
        });
        for index in 0..3 {
            let share = mounted.frame(mounted.tabs[index].share);
            assert_eq!(
                mounted.bar(Kind::Secondary, index),
                (share.origin.x.0, share.size.width.0)
            );
        }
    }

    #[test]
    fn a_row_with_no_selected_tab_draws_no_indicator() {
        let (_app, recording) = start(Config {
            selected: 9,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let rest = replay(&recording);
        assert_eq!(mounted.indicator_paths(&rest).len(), 0);
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for secondary in KINDS {
            for selected in [0, 1] {
                let (_app, recording) = start(Config {
                    kind: kind(secondary),
                    selected,
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{secondary}");
                // The indicator is created where it rests: layout sets nothing again.
                assert!(
                    set_values(&items) <= 2,
                    "{secondary}: {}",
                    set_values(&items)
                );
                assert!(!replay(&recording).is_animating(), "{secondary}");
            }
        }
    }

    #[test]
    fn selecting_a_tab_springs_the_indicator_from_one_tab_to_the_other_and_the_drawing_follows() {
        for secondary in KINDS {
            let config = Config {
                kind: kind(secondary),
                ..Config::default()
            };
            let kind = config.kind;
            let (mut app, recording) = start(config);
            let mounted = Mounted::of(&recording.items());
            let from = mounted.bar(kind, 0);
            let to = mounted.bar(kind, 1);
            app.press("label:Albums").unwrap();
            let items = recording.in_step(&press_step("label:Albums"));
            // Two springs and no more: the left edge and the width.
            let springs = started_springs(&items);
            assert_eq!(springs.len(), 2, "{springs:?}");
            let (left, width) = springs_of(&springs, to);
            assert_indicator_spring(&left, to.0, "left");
            assert_indicator_spring(&width, to.1, "width");
            // Nothing is laid out or drawn again: the canvas draws through the animated values.
            assert_eq!(mounted.frames_in(&items), 0, "{items:?}");
            assert!(!items.iter().any(
                |item| matches!(item, MountItem::Canvas { id, .. } if *id == mounted.indicator)
            ));
            // While they play, the engine sends nothing (ADR 0010).
            app.step("run");
            app.run_for(Duration::from_secs(1)).unwrap();
            assert!(recording.in_step("run").is_empty());

            // Each curve is the closed-form spring, within 0.1 % of its travel.
            for (spring, begin) in [(&left, from.0), (&width, from.1)] {
                let mut played = replay(&recording);
                let samples = played.run(spring.node, 180);
                let gap = worst_gap(&samples, f64::from(begin), spring);
                // A width that does not change (a secondary row's tabs are as wide as each other) has no travel: no gap.
                assert!(
                    gap <= tolerance(f64::from(begin), f64::from(spring.to)),
                    "{gap}"
                );
            }
            // Along the way the canvas traces the bar where the values are.
            for frame in [2_usize, 5, 9, 14, 30] {
                let mut played = replay(&recording);
                let _ = played.run(left.node, frame + 1);
                let (now_left, now_width) = (played.value(left.node), played.value(width.node));
                let drawn = mounted.indicator_paths(&played);
                assert_traces(
                    &drawn[0],
                    &bar_at(kind, now_left, now_width),
                    &format!("secondary: {secondary} frame {frame}"),
                );
            }
            // They settle exactly where they go, and the graph stops.
            let mut played = replay(&recording);
            let _ = played.run(left.node, 600);
            assert_eq!(played.value(left.node).to_bits(), to.0.to_bits());
            assert_eq!(played.value(width.node).to_bits(), to.1.to_bits());
            assert!(!played.is_animating());
        }
    }

    #[test]
    fn selecting_another_tab_while_the_indicator_moves_goes_from_where_and_as_fast_as_it_moves() {
        let config = Config::default();
        let (mut app, recording) = start(config);
        let mounted = Mounted::of(&recording.items());
        let (first_to, second_to) = (mounted.bar(Kind::Primary, 1), mounted.bar(Kind::Primary, 2));
        app.press("label:Albums").unwrap();
        let (first, _) = springs_of(
            &started_springs(&recording.in_step(&press_step("label:Albums"))),
            first_to,
        );
        let mut played = replay(&recording);
        // The finger moves on 50 ms in, while the indicator still slides.
        let before = played.run(first.node, 6);
        app.press("label:Artists").unwrap();
        let items = recording.in_step(&press_step("label:Artists"));
        let springs = started_springs(&items);
        assert_eq!(springs.len(), 2, "{springs:?}");
        let (back, width) = springs_of(&springs, second_to);
        assert_indicator_spring(&back, second_to.0, "left");
        assert_indicator_spring(&width, second_to.1, "width");
        assert_eq!(set_values(&items), 0);
        played.apply(&items);
        let after = played.run(back.node, 180);
        // The graph last ticked 5 frames into the first spring: the second starts there, at that value and that speed.
        let press = parameters(&first);
        let at = 5.0 / REPLAY_HZ;
        let begin = f64::from(0.0_f32);
        let _ = begin;
        let from_value = f64::from(mounted.bar(Kind::Primary, 0).0);
        let position = |t: f64| spring_position(from_value, f64::from(first_to.0), 0.0, press, t);
        let now = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - now).abs() < 1e-3,
            "{before:?} {now}"
        );
        assert!(speed.abs() > 1.0, "{speed}");
        let gap = |velocity: f64| {
            after
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                    let closed = spring_position(
                        now,
                        f64::from(second_to.0),
                        velocity,
                        parameters(&back),
                        t,
                    );
                    (f64::from(value) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        let tolerance = tolerance(now, f64::from(second_to.0));
        assert!(gap(speed) < tolerance, "{}", gap(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(gap(0.0) > 10.0 * tolerance, "{}", gap(0.0));
        let settled = played.run(back.node, 600);
        assert!(!played.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(second_to.0.to_bits())
        );
    }

    #[test]
    fn with_reduce_motion_the_indicator_goes_where_it_goes_at_once() {
        // ADR 0011, "Reduce motion": no spring.
        for secondary in KINDS {
            let config = Config {
                kind: kind(secondary),
                motion: Motion::Reduced,
                ..Config::default()
            };
            let kind = config.kind;
            let (mut app, recording) = start(config);
            let mounted = Mounted::of(&recording.items());
            app.press("label:Albums").unwrap();
            let items = recording.in_step(&press_step("label:Albums"));
            assert_eq!(started_springs(&items), Vec::new());
            assert_eq!(set_values(&items), 2);
            let played = replay(&recording);
            assert!(!played.is_animating());
            let drawn = mounted.indicator_paths(&played);
            let (left, width) = mounted.bar(kind, 1);
            assert_traces(&drawn[0], &bar_at(kind, left, width), "at once");
        }
    }

    #[test]
    fn pressing_the_selected_tab_moves_nothing() {
        let (mut app, recording) = start(Config::default());
        app.press("label:Songs").unwrap();
        let items = recording.in_step(&press_step("label:Songs"));
        assert!(app.shows("presses: 1"));
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 0);
    }

    #[test]
    fn a_tab_that_goes_away_while_selected_takes_the_indicator_with_it() {
        let (mut app, recording) = start(Config {
            selected: 2,
            ..Config::default()
        });
        app.press("Hide last").unwrap();
        let mounted = Mounted::of(&recording.items());
        let played = replay(&recording);
        assert_eq!(mounted.indicator_paths(&played).len(), 0);
        // The tabs that remain share the row between two.
        assert_eq!(mounted.tabs.len(), 2);
    }

    #[test]
    fn a_tab_outside_a_row_is_still_a_tab() {
        #[component]
        fn Lone() -> impl IntoElement {
            view()
                .child(view().flex_row().child(tab().label("Lone").selected(true)))
                .child(text("presses: 0"))
        }
        let mut app = Headless::start(EngineConfig::new(Root::of::<Lone>())).unwrap();
        app.wait_for("text:presses: 0").unwrap();
        assert!(app.shows("Lone"));
    }

    /// The vendored androidx files the values come from.
    const TAB_KT: &str = include_str!("../../../reference/androidx-material3-components/Tab.kt");
    const TAB_ROW_KT: &str =
        include_str!("../../../reference/androidx-material3-components/TabRow.kt");
    const DIVIDER_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Divider.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_spaces_and_the_height_the_tab_writes_are_androidxs() {
        assert!(cites(
            TAB_KT,
            &["internal val HorizontalTextPadding", "get() = 16.dp"]
        ));
        assert!(cites(
            TAB_KT,
            &["private val TextDistanceFromLeadingIcon", "get() = 8.dp"]
        ));
        assert!(cites(
            TAB_KT,
            &[
                "private val SmallTabHeight",
                "get() = PrimaryNavigationTabTokens.ContainerHeight"
            ]
        ));
        assert_eq!(HORIZONTAL_PADDING, Points(16.0));
        assert_eq!(ICON_TO_LABEL, Points(8.0));
        assert_eq!(primary::CONTAINER_HEIGHT, Points(48.0));
        assert_eq!(secondary::CONTAINER_HEIGHT, Points(48.0));
        // The label's type is the primary token's, for both rows.
        assert!(cites(
            TAB_KT,
            &["PrimaryNavigationTabTokens.LabelTextFont.value.copy(textAlign = TextAlign.Center)"]
        ));
        assert!(!TAB_KT.contains("SecondaryNavigationTabTokens"));
    }

    #[test]
    fn the_state_layer_and_the_colors_are_androidxs() {
        // The ripple is the selected content color, over the whole tab (`bounded`).
        assert!(cites(
            TAB_KT,
            &["val ripple = ripple(bounded = true, color = selectedContentColor)"]
        ));
        // `selectedContentColor` is the row's content color, the tokens' active label color.
        assert!(cites(
            TAB_KT,
            &["selectedContentColor: Color = LocalContentColor.current,"]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &[
                "public val primaryContentColor: Color",
                "@Composable get() = PrimaryNavigationTabTokens.ActiveLabelTextColor.value",
            ]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &[
                "public val secondaryContentColor: Color",
                "@Composable get() = SecondaryNavigationTabTokens.ActiveLabelTextColor.value",
            ]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &[
                "public val primaryContainerColor: Color",
                "@Composable get() = PrimaryNavigationTabTokens.ContainerColor.value",
            ]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &[
                "public val secondaryContainerColor: Color",
                "@Composable get() = SecondaryNavigationTabTokens.ContainerColor.value",
            ]
        ));
        // What androidx does not do, and ADR 0021 says this one does: it gives an unselected tab the selected color
        // and never reads the inactive tokens, and it dims nothing for a disabled tab.
        assert!(cites(
            TAB_KT,
            &["unselectedContentColor: Color = selectedContentColor,"]
        ));
        assert!(!TAB_KT.contains("Inactive"));
        assert!(!TAB_ROW_KT.contains("Inactive"));
        assert!(!TAB_KT.contains("DisabledAlpha"));
        assert!(cites(
            TAB_KT,
            &[
                "*   respond to user input, and it will appear visually disabled and disabled to accessibility"
            ]
        ));
    }

    #[test]
    fn the_indicator_the_divider_and_the_spring_are_androidxs() {
        assert!(cites(
            TAB_ROW_KT,
            &[
                "// Enforce minimum touch target of 24.dp",
                "val indicatorWidth = maxOf(contentWidth, 24.dp)",
            ]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &["contentWidth -= HorizontalTextPadding * 2"]
        ));
        assert_eq!(MIN_INDICATOR_WIDTH, Points(24.0));
        // The primary row's indicator follows the content, the secondary one's the tab.
        assert!(cites(
            TAB_ROW_KT,
            &["modifier = Modifier.tabIndicatorOffset(selectedTabIndex, matchContentSize = true),"]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &["Modifier.tabIndicatorOffset(selectedTabIndex, matchContentSize = false)"]
        ));
        // Both read the primary token's height and color; the secondary one is a plain box.
        assert!(cites(
            TAB_ROW_KT,
            &[
                "height: Dp = PrimaryNavigationTabTokens.ActiveIndicatorHeight,",
                "color: Color = PrimaryNavigationTabTokens.ActiveIndicatorColor.value,",
            ]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &["Box(modifier.fillMaxWidth().height(height).background(color = color))"]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &["shape: Shape = PrimaryNavigationTabTokens.ActiveIndicatorShape,"]
        ));
        assert_eq!(primary::ACTIVE_INDICATOR_HEIGHT, Points(3.0));
        assert_eq!(indicator_radius(3.0), 1.5);
        // The divider, over the tabs and under the indicator.
        assert!(cites(
            TAB_ROW_KT,
            &["divider: @Composable () -> Unit = @Composable { HorizontalDivider() },"]
        ));
        assert!(cites(
            DIVIDER_KT,
            &[
                "public val color: Color",
                "@Composable get() = DividerTokens.Color.value"
            ]
        ));
        assert!(cites(
            DIVIDER_KT,
            &["public val Thickness: Dp = DividerTokens.Thickness"]
        ));
        assert_eq!(divider::THICKNESS, Points(1.0));
        // The spring.
        assert!(cites(
            TAB_ROW_KT,
            &["val tabIndicatorAnimationSpec = MotionSchemeKeyTokens.DefaultSpatial.value<Dp>()"]
        ));
        // The fixed row's indicator sits at the tab's left edge, and the scrollable row's is centered: ADR 0021.
        assert!(cites(
            TAB_ROW_KT,
            &["val indicatorOffset = tabPositionsState.value[selectedTabIndex].left"]
        ));
        assert!(cites(
            TAB_ROW_KT,
            &[
                "val relativeOffset =",
                "max(0, (positions[selectedTabIndex].width.roundToPx() - it.width) / 2)",
            ]
        ));
    }
}
