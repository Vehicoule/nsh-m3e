//! The Material 3 Expressive navigation bar: a bar of items, each an icon and a label, one of them selected behind a
//! pill that grows from nothing, the state layer of hover, focus and press on that pill, and the two arrangements and
//! two icon positions of the Expressive bar.
//!
//! The bar follows androidx's `ShortNavigationBar` and `ShortNavigationBarItem` in `ShortNavigationBar.kt`, and the
//! item they share with the rails in `NavigationItem.kt`, at the pinned commit, as [ADR 0012] says. Its colors and
//! sizes are the tokens they read (`NavigationBarTokens` and the two item token files); the few values they write
//! themselves are each cited where they are used. The indicator is drawn by a canvas whose width the M3E spring moves
//! ([ADR 0011], [`crate::navigation_item`]).
//!
//! androidx's bar has no shadow (its `Surface` takes no elevation), so this one has none either, whatever
//! `NavigationBarTokens.ContainerElevation` says; a badge on an item is the badge component's, which comes with it.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::geometry::{Points, Size, relative};
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::icon::Icon;
use crate::navigation_item::{
    DISABLED_OPACITY, Indicator, NavigationItemIconPosition, NavigationItemLabel,
};
use crate::tint::{MIN_TOUCH_TARGET, Tint, role_color, tint_color};
use crate::tokens::TypeStyle;
use crate::tokens::androidx::{
    navigation_bar_horizontal_item_tokens as horizontal, navigation_bar_tokens as bar,
    navigation_bar_vertical_item_tokens as vertical, state_tokens,
};

/// How a [`NavigationBar`] arranges its items (androidx `ShortNavigationBarArrangement`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum NavigationBarArrangement {
    /// The items share the bar's width equally: androidx's default, recommended for a small window, with three to five
    /// items.
    #[default]
    EqualWeight,
    /// The items sit together in the middle of the bar, each as wide as its share of the group, and wider (up to its
    /// share of the whole bar) when its content needs it: recommended for a medium window, with three to six items.
    Centered,
}

/// The color the state layer of an item is drawn in: the bar's content color. androidx draws it with `ripple()`, whose
/// color is "`LocalContentColor`" when none is given (`Ripple.kt`, lines 82-83), and the bar's `Surface` gives that
/// `contentColorFor(containerColor)` (`ShortNavigationBar.kt`, `ShortNavigationBarDefaults.contentColor`), which for
/// `SurfaceContainer` is `onSurface` (`ColorScheme.kt`, `contentColorFor`). It is not the item's own icon color.
const STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;

/// The space between the indicator and the label of an item whose icon is on top: androidx
/// `TopIconIndicatorToLabelPadding`, a literal 4 (`ShortNavigationBar.kt`, lines 637-638), where the token
/// `NavigationBarTokens.ItemActiveIndicatorIconLabelSpace` is the space of the item with its icon at the start.
const TOP_INDICATOR_TO_LABEL: Points = Points(4.0);

/// The space between the icon and the label of an item whose icon is at the start: androidx `StartIconToLabelPadding`
/// (`ShortNavigationBar.kt`, lines 643-644), the token `NavigationBarTokens.ItemActiveIndicatorIconLabelSpace`.
const START_ICON_TO_LABEL: Points = bar::ITEM_ACTIVE_INDICATOR_ICON_LABEL_SPACE;

/// The color of an item's icon (androidx `NavigationItemColors.iconColor`): the selected icon color when selected, the
/// unselected one when not, and when disabled the unselected one, faded.
fn icon_tint(selected: bool, disabled: bool) -> Tint {
    if disabled {
        Tint::faded(bar::ITEM_INACTIVE_ICON_COLOR, DISABLED_OPACITY)
    } else if selected {
        Tint::solid(bar::ITEM_ACTIVE_ICON_COLOR)
    } else {
        Tint::solid(bar::ITEM_INACTIVE_ICON_COLOR)
    }
}

/// The color of an item's label (androidx `NavigationItemColors.textColor`). Selected, it is the label color of the
/// tokens when the icon is on top, and the icon's color when the icon is at the start: androidx has no token for that
/// one yet (`ShortNavigationBar.kt`, line 471: "TODO: Replace with the correct token once it is available in
/// `NavigationBarTokens`").
fn label_tint(selected: bool, disabled: bool, position: NavigationItemIconPosition) -> Tint {
    if disabled {
        Tint::faded(bar::ITEM_INACTIVE_LABEL_TEXT_COLOR, DISABLED_OPACITY)
    } else if selected {
        match position {
            NavigationItemIconPosition::Start => Tint::solid(bar::ITEM_ACTIVE_ICON_COLOR),
            // `NavigationItemIconPosition` is non-exhaustive: a position added later is labelled as the top one.
            _ => Tint::solid(bar::ITEM_ACTIVE_LABEL_TEXT_COLOR),
        }
    } else {
        Tint::solid(bar::ITEM_INACTIVE_LABEL_TEXT_COLOR)
    }
}

/// The area of the indicator of an item whose icon is on top: the icon and the padding around it, androidx
/// `TopIconIndicatorHorizontalPadding` and `TopIconIndicatorVerticalPadding` (`ShortNavigationBar.kt`, lines 625-631),
/// which are what is left of the token's indicator width and height around the icon. That makes the tokens' 56 × 32.
fn top_indicator() -> Size<Points> {
    let icon = vertical::ICON_SIZE.0;
    let horizontal_padding = (vertical::ACTIVE_INDICATOR_WIDTH.0 - icon) / 2.0;
    let vertical_padding = (vertical::ACTIVE_INDICATOR_HEIGHT.0 - icon) / 2.0;
    Size::new(
        Points(icon + 2.0 * horizontal_padding),
        Points(icon + 2.0 * vertical_padding),
    )
}

/// The padding of the indicator of an item whose icon is at the start, on each side: androidx
/// `StartIconIndicatorHorizontalPadding` (the token's leading space) and `StartIconIndicatorVerticalPadding` (what is
/// left of the token's indicator height around the icon) (`ShortNavigationBar.kt`, lines 633-641).
fn start_indicator_padding() -> (Points, Points) {
    (
        horizontal::ACTIVE_INDICATOR_LEADING_SPACE,
        Points((horizontal::ACTIVE_INDICATOR_HEIGHT.0 - horizontal::ICON_SIZE.0) / 2.0),
    )
}

/// The label's type: `NavigationBarTokens.LabelTextFont`, label medium, for both icon positions.
const LABEL: TypeStyle = bar::LABEL_TEXT_FONT;

/// The share of the bar's width one item takes in the centered arrangement, at least and at most (androidx
/// `CenteredContentMeasurePolicy`, `ShortNavigationBar.kt`, lines 539-619).
///
/// With `count` items, the group takes `10 · (count + 3)` percent of the bar when there are six items or fewer, and the
/// whole bar when there are more, and each item takes its part of it; an item whose content is wider grows, to at most
/// `1 / count` of the bar. The numbers 10, 3 and 6 are in androidx's code and comment, not in a token.
fn centered_share(count: usize) -> (f32, f32) {
    if count == 0 {
        return (1.0, 1.0);
    }
    // Lossless: a bar has a handful of items; a count beyond `u16` is the same bar as the 7 items the formula stops at.
    let count_f = f32::from(u16::try_from(count).unwrap_or(u16::MAX));
    let group = if count > 6 {
        1.0
    } else {
        // androidx: `paddingPercentage = ((100 - 10 * (itemsCount + 3)) / 2f) / 100`, on each side.
        let padding = ((100.0 - 10.0 * (count_f + 3.0)) / 2.0) / 100.0;
        1.0 - 2.0 * padding
    };
    (group / count_f, 1.0 / count_f)
}

/// What a [`NavigationBar`] tells its items: how to arrange themselves, and how many of them there are.
#[derive(Debug, Clone, Copy)]
struct BarContext {
    arrangement: Prop<NavigationBarArrangement>,
    items: Signal<usize>,
}

/// The props of a [`NavigationBar`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct NavigationBarProps {
    /// How its items share the bar.
    #[prop(default)]
    pub arrangement: Prop<NavigationBarArrangement>,
    /// Its items: [`NavigationBarItem`]s.
    pub children: Prop<Children>,
}

/// A Material 3 Expressive navigation bar: the primary destinations of an app, one [`NavigationBarItem`] each.
///
/// It is a tab list for screen readers and the keyboard, as `nsh-base`'s [`Tabs`](nsh_base::Tabs) is (arrow keys move
/// between the items and select them), and is at least 64 points tall, as tall as its tallest item.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::icon::Icon;
/// use nsh_m3e::navigation_bar::{navigation_bar, navigation_bar_item};
///
/// fn bar() -> impl IntoElement {
///     let selected = signal(0_usize);
///     navigation_bar()
///         .child(
///             navigation_bar_item()
///                 // Material Symbols' `home`.
///                 .icon(Icon::material_symbol("M240-200h120v-240h240v240h120v-360L480-740 240-560v360Z"))
///                 .label("Home")
///                 .selected(move || selected.get() == 0)
///                 .on_press(move || selected.set(0)),
///         )
///         .child(
///             navigation_bar_item()
///                 .icon(Icon::material_symbol("M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z"))
///                 .label("Add")
///                 .selected(move || selected.get() == 1)
///                 .on_press(move || selected.set(1)),
///         )
/// }
/// ```
#[component]
pub fn NavigationBar(props: NavigationBarProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    provide_context(BarContext {
        arrangement: props.arrangement,
        items: signal(0),
    });
    // The container, and inside it the row of items: androidx's `Surface` and the `Layout` in it, which keeps the
    // container's height below the system bars (`ShortNavigationBarDefaults.windowInsets`: the horizontal and bottom
    // ones) around the row, whose own height is `ContainerHeight` (`ShortNavigationBar.kt`, lines 159-171).
    view()
        .bg(tint_color(Some(Tint::solid(bar::CONTAINER_COLOR))))
        .px_safe_area()
        .pb_safe_area()
        .child(
            nsh_base::tabs()
                .flex_row()
                .min_h(bar::CONTAINER_HEIGHT)
                .style_with(move |s| match props.arrangement.get() {
                    NavigationBarArrangement::Centered => s.justify_center(),
                    _ => s,
                })
                .child(props.children),
        )
}

/// The props of a [`NavigationBarItem`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct NavigationBarItemProps {
    /// What the item says: its label, and its name for screen readers.
    pub label: Prop<SharedString>,
    /// Its icon. Material shows the filled icon when the item is selected and the outlined one when it is not: give the
    /// one that fits `selected`.
    pub icon: Prop<Icon>,
    /// Whether it is the current destination: the indicator grows behind its icon, and screen readers say it is
    /// selected. The app owns the value: a press reports [`NavigationBarItemEvent::Press`], and the app selects the item.
    #[prop(default)]
    pub selected: Prop<bool>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
    /// Where its icon sits.
    #[prop(default)]
    pub icon_position: Prop<NavigationItemIconPosition>,
    /// Whether its label is shown.
    #[prop(default)]
    pub label_visibility: Prop<NavigationItemLabel>,
}

/// The events of a [`NavigationBarItem`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum NavigationBarItemEvent {
    /// The item was pressed, or the keyboard focus moved to it: the app selects it. It is sent for the selected item too.
    Press,
}

/// An item of a [`NavigationBar`]: an icon and a label, behind a pill when it is selected.
///
/// It has the behavior of `nsh-base`'s [`Tab`](nsh_base::Tab) (presses, focus, the disabled state, the tab role and the
/// selected state for screen readers) and the look of androidx's `ShortNavigationBarItem`. Hover, focus and press lay
/// the bar's content color over the pill at the state layer opacities of `StateTokens`, as androidx's ripple does, and
/// change nothing else, so nothing is laid out again.
///
/// It is at least 48 points on each side, Material's touch target.
#[component]
pub fn NavigationBarItem(
    props: NavigationBarItemProps,
    cx: Scope<NavigationBarItemEvent>,
) -> impl IntoElement {
    let context = use_context::<BarContext>();
    if let Some(context) = context {
        context.items.update(|count| *count += 1);
        // The bar may be gone before its items: the count is then no one's.
        cleanup(move || {
            if let Some(count) = context.items.try_get() {
                let _ = context.items.try_set(count.saturating_sub(1));
            }
        });
    }
    let position = props.icon_position;
    let is_top = move || position.get() != NavigationItemIconPosition::Start;
    let label_shown = move || props.label_visibility.get() != NavigationItemLabel::Hidden;
    let icon_color =
        move || tint_color(Some(icon_tint(props.selected.get(), props.disabled.get())));
    let label_color = move || {
        tint_color(Some(label_tint(
            props.selected.get(),
            props.disabled.get(),
            position.get(),
        )))
    };
    let indicator = untracked(|| {
        let area = if is_top() {
            top_indicator()
        } else {
            let (horizontal_padding, vertical_padding) = start_indicator_padding();
            Size::new(
                Points(horizontal::ICON_SIZE.0 + 2.0 * horizontal_padding.0),
                Points(horizontal::ICON_SIZE.0 + 2.0 * vertical_padding.0),
            )
        };
        Indicator::new(props.selected.get(), area)
    });
    effect(move || indicator.select(props.selected.get()));

    let icon = move || {
        canvas(move || props.icon.get().drawing(vertical::ICON_SIZE))
            .w(vertical::ICON_SIZE)
            .h(vertical::ICON_SIZE)
            .fill_color(icon_color)
    };
    let label = move || {
        text(move || props.label.get())
            .text_color(label_color)
            .text_size(LABEL.size)
            .line_height(LineHeight::Points(LABEL.line_height))
            .letter_spacing(LABEL.tracking)
            .text_center()
            .style_with(|s| s.font_weight(LABEL.weight))
    };
    // The pill's color, and the state layer over it: the bar's content color, shown by the item's own states.
    let pill = move || {
        indicator.canvas().bg(tint_color(Some(Tint::solid(
            bar::ITEM_ACTIVE_INDICATOR_COLOR,
        ))))
    };
    let state_layer = move || {
        view()
            .absolute()
            .inset(px(0.0))
            .rounded_full()
            .bg(role_color(STATE_LAYER_COLOR))
            .opacity(0.0)
            .pointer_events(PointerEvents::None)
            .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
            .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
            .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
            .style_with(move |s| {
                // A disabled item takes no press and shows no state.
                if props.disabled.get() { s.hidden() } else { s }
            })
    };

    nsh_base::tab()
        .selected(move || props.selected.get())
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(NavigationBarItemEvent::Press))
        .group()
        .items_center()
        .min_w(MIN_TOUCH_TARGET)
        .min_h(MIN_TOUCH_TARGET)
        .style_with(move |s| {
            let arrangement = context.map_or(NavigationBarArrangement::EqualWeight, |context| {
                context.arrangement.get()
            });
            let s = match arrangement {
                NavigationBarArrangement::Centered => {
                    let (least, most) =
                        context.map_or((1.0, 1.0), |context| centered_share(context.items.get()));
                    // The share replaces the touch target's minimum: androidx gives each item exactly this width
                    // (`Constraints.fixed`), which is at least 48 points wherever this arrangement is meant to be
                    // used, a window of the medium size or more.
                    s.flex_none().min_w(relative(least)).max_w(relative(most))
                }
                _ => s.flex_1(),
            };
            // With a label, the content hangs from the top with the space `ContainerBetweenSpace` above and below it;
            // without one, or with the icon at the start, it is centered (androidx `placeLabelAndTopIcon`, `placeIcon`
            // and `placeLabelAndStartIcon`).
            if is_top() && label_shown() {
                s.justify_start()
                    .pt(vertical::CONTAINER_BETWEEN_SPACE)
                    .pb(vertical::CONTAINER_BETWEEN_SPACE)
            } else {
                s.justify_center()
            }
        })
        .child(when(is_top, move || {
            // The icon on top: the pill is 56 × 32 around the icon, and the label is below it.
            let area = top_indicator();
            view()
                .items_center()
                .child(
                    view()
                        .relative()
                        .w(area.width)
                        .h(area.height)
                        .items_center()
                        .justify_center()
                        .child(pill())
                        .child(icon())
                        .child(state_layer()),
                )
                .child(when(label_shown, move || {
                    label().mt(TOP_INDICATOR_TO_LABEL)
                }))
        }))
        .child(when(
            move || !is_top(),
            move || {
                // The icon at the start: the pill is around the icon and the label.
                let (horizontal_padding, vertical_padding) = start_indicator_padding();
                view()
                    .relative()
                    .flex_row()
                    .items_center()
                    .px(horizontal_padding)
                    .py(vertical_padding)
                    .child(pill())
                    .child(icon())
                    .child(when(label_shown, move || label().ms(START_ICON_TO_LABEL)))
                    .child(state_layer())
            },
        ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{self, AnimationOp, AnimationProp};
    use nsh_core::canvas::{self, CanvasOp, Drawing};
    use nsh_core::geometry::{Bounds, Edges};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{
        AccessibilityRole, AttributedText, Color, MountItem, NodeId, TextAlign, ViewKind, ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::ColorScheme;
    use crate::morph_probe::{
        Step, assert_traces, parameters, paths, replay, rounded_rectangle, set_values, tolerance,
    };
    use crate::motion::{SPRING_MASS, SpringToken};

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ICON: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";
    const LABELS: [&str; 7] = [
        "Home", "Search", "Library", "Radio", "Profile", "Settings", "Help",
    ];
    /// The width of the probe's window, which the root view and the bar fill.
    const WINDOW: f32 = 390.0;

    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        arrangement: Prop<NavigationBarArrangement>,
        #[prop(default)]
        icon_position: Prop<NavigationItemIconPosition>,
        #[prop(default)]
        label_visibility: Prop<NavigationItemLabel>,
        count: Prop<usize>,
        selected: Prop<usize>,
        disabled: Prop<bool>,
    }

    /// An item of the probe's bar: the app selects it when it is pressed, and counts the presses.
    fn item(
        index: usize,
        selected: Signal<usize>,
        disabled: Signal<bool>,
        presses: Signal<u32>,
        position: Prop<NavigationItemIconPosition>,
        visibility: Prop<NavigationItemLabel>,
    ) -> impl IntoElement {
        navigation_bar_item()
            .icon(Icon::material_symbol(ICON))
            .label(LABELS[index])
            .selected(move || selected.get() == index)
            // The second item is the one that gets disabled.
            .disabled(move || index == 1 && disabled.get())
            .icon_position(position)
            .label_visibility(visibility)
            .on_press(move || {
                presses.update(|n| *n += 1);
                selected.set(index);
            })
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let selected = signal(props.selected.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        let presses = signal(0_u32);
        let last_shown = signal(true);
        let count = props.count.get_untracked();
        let mut bar = navigation_bar().arrangement(props.arrangement);
        for index in 0..count {
            let (position, visibility) = (props.icon_position, props.label_visibility);
            if index + 1 == count {
                // The last item can go away, as an app's destinations can.
                bar = bar.child(when(
                    move || last_shown.get(),
                    move || item(index, selected, disabled, presses, position, visibility),
                ));
            } else {
                bar = bar.child(item(
                    index, selected, disabled, presses, position, visibility,
                ));
            }
        }
        view()
            .child(bar)
            .child(text(move || format!("selected: {}", selected.get())))
            .child(text(move || format!("presses: {}", presses.get())))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Hide last").on_press(move |(): &()| last_shown.set(false)))
    }

    /// An item with no bar around it.
    #[component]
    fn Lone() -> impl IntoElement {
        view()
            .child(
                view().flex_row().child(
                    navigation_bar_item()
                        .icon(Icon::material_symbol(ICON))
                        .label("Lone")
                        .selected(true),
                ),
            )
            .child(text("presses: 0"))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        arrangement: NavigationBarArrangement,
        icon_position: NavigationItemIconPosition,
        label_visibility: NavigationItemLabel,
        count: usize,
        selected: usize,
        disabled: bool,
        dark: bool,
        reduce_motion: bool,
        safe_area: Edges<Points>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                arrangement: NavigationBarArrangement::default(),
                icon_position: NavigationItemIconPosition::default(),
                label_visibility: NavigationItemLabel::default(),
                count: 3,
                selected: 0,
                disabled: false,
                dark: false,
                reduce_motion: false,
                safe_area: Edges::default(),
            }
        }
    }

    fn headless(root: Root, config: Config) -> (Headless, Recording) {
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            reduce_motion: config.reduce_motion,
            safe_area: config.safe_area,
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

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            arrangement: Prop::new(config.arrangement),
            icon_position: Prop::new(config.icon_position),
            label_visibility: Prop::new(config.label_visibility),
            count: Prop::new(config.count),
            selected: Prop::new(config.selected),
            disabled: Prop::new(config.disabled),
        });
        headless(root, config)
    }

    /// One item's views.
    #[derive(Debug, Clone, Copy)]
    struct Item {
        /// The press target, with the tab role: the whole item.
        tab: NodeId,
        /// The view that holds the pill, the icon and the state layer: the pill's area.
        area: NodeId,
        /// The canvas that draws the pill.
        pill: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        /// The view that is the state layer.
        layer: NodeId,
        /// The label's text view, when it is shown.
        label: Option<NodeId>,
    }

    /// The navigation bar's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The container, under the row of items.
        container: NodeId,
        /// The row of items, with the tab list role.
        bar: NodeId,
        items: Vec<Item>,
        parents: HashMap<NodeId, NodeId>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        texts: HashMap<NodeId, Arc<AttributedText>>,
        accessibility: HashMap<NodeId, nsh_core::Accessibility>,
        /// What each animated view is bound to.
        bindings: HashMap<NodeId, Vec<(AnimationProp, animation::NodeId)>>,
        /// The node each interpolation reads.
        inputs: HashMap<animation::NodeId, animation::NodeId>,
    }

    /// The views that were created, and their children in order.
    #[derive(Default)]
    struct Tree {
        kinds: HashMap<NodeId, ViewKind>,
        children: HashMap<NodeId, Vec<NodeId>>,
        parents: HashMap<NodeId, NodeId>,
    }

    impl Tree {
        fn insert(&mut self, parent: NodeId, child: NodeId, before: Option<NodeId>) {
            let siblings = self.children.entry(parent).or_default();
            let at = before
                .and_then(|before| siblings.iter().position(|id| *id == before))
                .unwrap_or(siblings.len());
            siblings.insert(at, child);
            self.parents.insert(child, parent);
        }

        fn descendants(&self, id: NodeId, out: &mut Vec<NodeId>) {
            for child in self.children.get(&id).into_iter().flatten() {
                out.push(*child);
                self.descendants(*child, out);
            }
        }

        /// The views of the item whose press target is `tab`.
        fn item(&self, tab: NodeId) -> Item {
            let mut all = Vec::new();
            self.descendants(tab, &mut all);
            let of_kind = |kind: ViewKind| -> Vec<NodeId> {
                all.iter()
                    .copied()
                    .filter(|id| self.kinds.get(id) == Some(&kind))
                    .collect()
            };
            let [pill, icon] = of_kind(ViewKind::Canvas)[..] else {
                panic!("two canvases in an item");
            };
            let labels = of_kind(ViewKind::Text);
            assert!(labels.len() <= 1, "{labels:?}");
            // The state layer is the one view of the item that holds nothing.
            let layers: Vec<NodeId> = of_kind(ViewKind::View)
                .into_iter()
                .filter(|id| self.children.get(id).is_none_or(Vec::is_empty))
                .collect();
            let [layer] = layers[..] else {
                panic!("one state layer in an item: {layers:?}");
            };
            Item {
                tab,
                area: self.parents[&pill],
                pill,
                icon,
                layer,
                label: labels.first().copied(),
            }
        }
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut tree = Tree::default();
            let mut accessibility = HashMap::new();
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            let mut texts = HashMap::new();
            let mut bindings = HashMap::new();
            let mut inputs = HashMap::new();
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        tree.kinds.insert(*id, *kind);
                    }
                    MountItem::Insert {
                        parent,
                        child,
                        before,
                    } => tree.insert(*parent, *child, *before),
                    MountItem::ViewProps { id, props } => {
                        accessibility.insert(*id, props.accessibility.clone());
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
                    MountItem::AnimationBindings {
                        id,
                        bindings: bound,
                    } => {
                        bindings.insert(*id, bound.clone());
                    }
                    MountItem::Animation(AnimationOp::CreateInterpolate {
                        node, input, ..
                    }) => {
                        inputs.insert(*node, *input);
                    }
                    _ => {}
                }
            }
            let role = |id: &NodeId| accessibility.get(id).map(|a| a.role);
            let bar = *accessibility
                .keys()
                .find(|id| role(id) == Some(AccessibilityRole::Tablist))
                .expect("a view with the tab list role");
            let container = tree.parents[&bar];
            let items = tree.children[&bar]
                .iter()
                .filter(|id| role(id) == Some(AccessibilityRole::Tab))
                .map(|&tab| tree.item(tab))
                .collect();
            Self {
                container,
                bar,
                items,
                parents: tree.parents,
                styles,
                frames,
                drawings,
                texts,
                accessibility,
                bindings,
                inputs,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        fn size(&self, id: NodeId) -> Size<Points> {
            self.frames[&id].size
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
            let size = self.size(id);
            (x, y, size.width.0, size.height.0)
        }

        fn label_text(&self, item: &Item) -> &AttributedText {
            &self.texts[&item.label.expect("a label")]
        }

        fn label_color(&self, item: &Item) -> Color {
            self.label_text(item).spans[0].style.color
        }

        fn icon_color(&self, item: &Item) -> Color {
            fill(&self.drawing(item.icon).ops).expect("the icon's fill")
        }

        /// The animated value that says how far the item's pill has grown: what its fade reads.
        fn progress(&self, item: &Item) -> animation::NodeId {
            self.inputs[&self.fade(item)]
        }

        /// The animated value the pill's opacity is bound to.
        fn fade(&self, item: &Item) -> animation::NodeId {
            self.bindings[&item.pill]
                .iter()
                .find_map(|(prop, node)| (*prop == AnimationProp::Opacity).then_some(*node))
                .expect("an opacity bound to the pill")
        }

        /// How many of `items` lay out one of the bar's views again.
        fn frames_in(&self, items: &[MountItem]) -> usize {
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Frame { id, .. } if self.parents.contains_key(id)))
                .count()
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

    /// `found` is `wanted`, give or take the rounding of layout: NSH lays out on whole points, so an edge is within half
    /// a point of where the arithmetic puts it, and a size within one.
    fn assert_near(found: f32, wanted: f32, what: &str) {
        assert!(
            (found - wanted).abs() <= 1.0,
            "{what}: {found} is not {wanted}"
        );
    }

    const POSITIONS: [NavigationItemIconPosition; 2] = [
        NavigationItemIconPosition::Top,
        NavigationItemIconPosition::Start,
    ];

    #[test]
    fn the_bar_is_a_tab_list_of_tabs_named_by_their_labels() {
        let mounted = mount(Config {
            selected: 2,
            ..Config::default()
        });
        assert_eq!(mounted.items.len(), 3);
        for (index, item) in mounted.items.iter().enumerate() {
            let access = &mounted.accessibility[&item.tab];
            assert_eq!(access.role, AccessibilityRole::Tab);
            assert_eq!(access.label.as_deref(), Some(LABELS[index]));
            assert_eq!(access.state.selected, index == 2, "{index}");
            assert!(!access.state.disabled, "{index}");
            assert_eq!(mounted.label_text(item).text, LABELS[index]);
        }
    }

    #[test]
    fn a_disabled_item_says_so_to_screen_readers() {
        let mounted = mount(Config {
            disabled: true,
            ..Config::default()
        });
        let disabled: Vec<bool> = mounted
            .items
            .iter()
            .map(|item| mounted.accessibility[&item.tab].state.disabled)
            .collect();
        assert_eq!(disabled, [false, true, false]);
    }

    #[test]
    fn the_container_is_surface_container_with_no_shadow_light_and_dark() {
        for dark in [false, true] {
            let mounted = mount(Config {
                dark,
                ..Config::default()
            });
            let container = mounted.style(mounted.container);
            assert_eq!(
                container.background_color,
                solid(ColorRole::SurfaceContainer, dark),
                "dark: {dark}"
            );
            // androidx draws it with a `Surface` that takes no elevation: no shadow, whatever the token's level says.
            assert_eq!(container.elevation, px(0.0));
            assert!(container.effects.is_none());
            // Nothing else paints over it, and the corners are square (`NavShape` is `CornerNone`).
            let mut views = vec![mounted.bar];
            views.extend(mounted.items.iter().map(|item| item.tab));
            for id in views {
                let style = mounted.style(id);
                assert_eq!(style.background_color, Color::TRANSPARENT, "{id:?}");
                assert_eq!(style.elevation, px(0.0), "{id:?}");
            }
            assert_eq!(container.border_radius, px(0.0));
        }
    }

    /// The icon's and the label's colors an item should have, in the baseline schemes: androidx
    /// `defaultShortNavigationBarItemColors` and `NavigationItemColors`.
    fn expected_colors(
        selected: bool,
        disabled: bool,
        position: NavigationItemIconPosition,
        dark: bool,
    ) -> (Color, Color) {
        let faded = |role| color(scheme_color(role, dark), ALPHA_38);
        match (disabled, selected, position) {
            // The unselected colors at 38 %, whatever the selection.
            (true, ..) => (
                faded(ColorRole::OnSurfaceVariant),
                faded(ColorRole::OnSurfaceVariant),
            ),
            (false, true, NavigationItemIconPosition::Start) => (
                solid(ColorRole::OnSecondaryContainer, dark),
                solid(ColorRole::OnSecondaryContainer, dark),
            ),
            (false, true, _) => (
                solid(ColorRole::OnSecondaryContainer, dark),
                solid(ColorRole::Secondary, dark),
            ),
            (false, false, _) => (
                solid(ColorRole::OnSurfaceVariant, dark),
                solid(ColorRole::OnSurfaceVariant, dark),
            ),
        }
    }

    #[test]
    fn every_item_has_its_colors_in_each_state_light_and_dark() {
        for position in POSITIONS {
            for dark in [false, true] {
                for disabled in [false, true] {
                    for selected in [false, true] {
                        let what = format!(
                            "{position:?} dark: {dark} disabled: {disabled} selected: {selected}"
                        );
                        // The second item is the one that is disabled, and the one selected here.
                        let mounted = mount(Config {
                            icon_position: position,
                            selected: usize::from(selected),
                            disabled,
                            dark,
                            ..Config::default()
                        });
                        let item = mounted.items[1];
                        let (icon, label) = expected_colors(selected, disabled, position, dark);
                        assert_eq!(mounted.icon_color(&item), icon, "{what}: icon");
                        assert_eq!(mounted.label_color(&item), label, "{what}: label");
                        // The pill is the same color in every state: only its growth shows the selection. The state
                        // layer is the bar's content color, not the icon's.
                        assert_eq!(
                            fill(&mounted.drawing(item.pill).ops),
                            Some(solid(ColorRole::SecondaryContainer, dark)),
                            "{what}: pill"
                        );
                        assert_eq!(
                            mounted.style(item.layer).background_color,
                            solid(ColorRole::OnSurface, dark),
                            "{what}: state layer"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_label_is_label_medium_centered() {
        for position in POSITIONS {
            let mounted = mount(Config {
                icon_position: position,
                ..Config::default()
            });
            let text = mounted.label_text(&mounted.items[0]);
            let style = &text.spans[0].style;
            // `NavigationBarTokens.LabelTextFont` is `LabelMedium`: 12 points, a 16 point line, medium weight, 0.5
            // of tracking.
            assert_eq!(style.font_size, px(12.0), "{position:?}");
            assert_eq!(style.line_height, px(16.0), "{position:?}");
            assert_eq!(style.font_weight, 500, "{position:?}");
            assert_eq!(style.letter_spacing, px(0.5), "{position:?}");
            assert_eq!(text.align, TextAlign::Center, "{position:?}");
        }
    }

    #[test]
    fn the_icon_is_its_path_at_the_token_size() {
        for position in POSITIONS {
            let mounted = mount(Config {
                icon_position: position,
                ..Config::default()
            });
            let item = mounted.items[0];
            // The Material Symbol's 960-point view box, scaled to the icon's 24 points and moved down by its height.
            let ops = &mounted.drawing(item.icon).ops;
            let expected = Icon::material_symbol(ICON).drawing(px(24.0)).finish();
            let fills = |ops: &[CanvasOp]| -> Vec<CanvasOp> {
                ops.iter()
                    .filter(|op| {
                        matches!(
                            op,
                            CanvasOp::Fill(..) | CanvasOp::Translate(..) | CanvasOp::Scale(..)
                        )
                    })
                    .cloned()
                    .collect()
            };
            assert_eq!(fills(ops), fills(&expected.ops), "{position:?}");
            assert_eq!(fills(ops).len(), 3, "{position:?}");
            assert_eq!(mounted.size(item.icon), Size::new(px(24.0), px(24.0)));
        }
    }

    #[test]
    fn equal_weight_items_share_the_bar_and_hang_their_content_from_the_top() {
        for count in [3_usize, 4, 5] {
            let mounted = mount(Config {
                count,
                ..Config::default()
            });
            #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
            let share = WINDOW / count as f32;
            assert_eq!(mounted.rect(mounted.bar), (0.0, 0.0, WINDOW, 64.0));
            for (index, item) in mounted.items.iter().enumerate() {
                #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
                let left = share * index as f32;
                let what = format!("{count} items, item {index}");
                let (x, y, width, height) = mounted.rect(item.tab);
                assert_near(x, left, &what);
                assert_near(y, 0.0, &what);
                assert_near(width, share, &what);
                assert_near(height, 64.0, &what);
                // The pill's area: 56 × 32, centered in the item, 6 points below its top
                // (`ContainerBetweenSpace`).
                let (x, y, width, height) = mounted.rect(item.area);
                assert_near(x, left + (share - 56.0) / 2.0, &what);
                assert_near(y, 6.0, &what);
                assert_near(width, 56.0, &what);
                assert_near(height, 32.0, &what);
                // The icon is centered in it, and the state layer is as big as it.
                let (x, y, width, height) = mounted.rect(item.icon);
                assert_near(x, left + (share - 24.0) / 2.0, &what);
                assert_near(y, 6.0 + 4.0, &what);
                assert_near(width, 24.0, &what);
                assert_near(height, 24.0, &what);
                assert_eq!(mounted.rect(item.layer).2, 56.0, "{what}");
                assert_eq!(mounted.rect(item.layer).3, 32.0, "{what}");
                assert_eq!(mounted.rect(item.pill).2, 56.0, "{what}");
                // The label is centered, 4 points below the pill.
                let (x, y, width, _) = mounted.rect(item.label.expect("a label"));
                assert_near(y, 6.0 + 32.0 + 4.0, &what);
                assert_near(x + width / 2.0, left + share / 2.0, &what);
            }
        }
    }

    #[test]
    fn a_top_item_without_a_label_centers_its_pill_in_the_bar() {
        let mounted = mount(Config {
            label_visibility: NavigationItemLabel::Hidden,
            ..Config::default()
        });
        for (index, item) in mounted.items.iter().enumerate() {
            assert!(item.label.is_none(), "{index}");
            // The item is still named by its label.
            assert_eq!(
                mounted.accessibility[&item.tab].label.as_deref(),
                Some(LABELS[index])
            );
            let (_, tab_y, _, tab_height) = mounted.rect(item.tab);
            let (_, y, width, height) = mounted.rect(item.area);
            assert_near(tab_height, 64.0, "item");
            assert_near(y, tab_y + (64.0 - 32.0) / 2.0, "pill");
            assert_near(width, 56.0, "pill");
            assert_near(height, 32.0, "pill");
        }
    }

    #[test]
    fn a_start_item_wraps_its_icon_and_label_in_the_pill() {
        let mounted = mount(Config {
            icon_position: NavigationItemIconPosition::Start,
            ..Config::default()
        });
        let share = WINDOW / 3.0;
        for (index, item) in mounted.items.iter().enumerate() {
            let what = format!("item {index}");
            let (_, _, label_width, _) = mounted.rect(item.label.expect("a label"));
            // The pill: 16 points before the icon and after the label, 8 above and below, 4 between the icon and the
            // label; the item is centered in the bar.
            let wide = 16.0 + 24.0 + 4.0 + label_width + 16.0;
            let (x, y, width, height) = mounted.rect(item.area);
            assert_near(width, wide, &what);
            assert_near(height, 8.0 + 24.0 + 8.0, &what);
            #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
            let left = share * index as f32;
            assert_near(x, left + (share - wide) / 2.0, &what);
            assert_near(y, (64.0 - 40.0) / 2.0, &what);
            let (icon_x, icon_y, ..) = mounted.rect(item.icon);
            assert_near(icon_x, x + 16.0, &what);
            assert_near(icon_y, y + 8.0, &what);
            let (label_x, label_y, _, label_height) = mounted.rect(item.label.expect("a label"));
            assert_near(label_x, x + 16.0 + 24.0 + 4.0, &what);
            assert_near(label_y + label_height / 2.0, y + 20.0, &what);
            assert_eq!(mounted.size(item.layer), mounted.size(item.area), "{what}");
            assert_eq!(mounted.size(item.pill), mounted.size(item.area), "{what}");
        }
    }

    #[test]
    fn a_start_item_without_a_label_is_a_pill_around_the_icon() {
        let mounted = mount(Config {
            icon_position: NavigationItemIconPosition::Start,
            label_visibility: NavigationItemLabel::Hidden,
            ..Config::default()
        });
        let item = mounted.items[0];
        assert!(item.label.is_none());
        let (x, y, width, height) = mounted.rect(item.area);
        assert_near(width, 16.0 + 24.0 + 16.0, "pill");
        assert_near(height, 8.0 + 24.0 + 8.0, "pill");
        assert_near(x, (WINDOW / 3.0 - 56.0) / 2.0, "pill");
        assert_near(y, 12.0, "pill");
        let (icon_x, icon_y, ..) = mounted.rect(item.icon);
        assert_near(icon_x, x + 16.0, "icon");
        assert_near(icon_y, y + 8.0, "icon");
    }

    #[test]
    fn every_item_is_a_48_point_touch_target() {
        for position in POSITIONS {
            for visibility in [NavigationItemLabel::Shown, NavigationItemLabel::Hidden] {
                // Seven items in a window this narrow share less than 48 points each: they keep 48.
                let mounted = mount(Config {
                    icon_position: position,
                    label_visibility: visibility,
                    count: 7,
                    ..Config::default()
                });
                for item in &mounted.items {
                    let (_, _, width, height) = mounted.rect(item.tab);
                    assert!(
                        width >= 48.0 && height >= 48.0,
                        "{position:?} {width}×{height}"
                    );
                }
            }
        }
    }

    /// What each item's width should be when `count` of them are centered in the probe's window with their icon on
    /// top, and how wide a window's share of them is, least and most, in points (androidx
    /// `CenteredContentMeasurePolicy`): the items are as wide as their 56 point pill, within their shares.
    const CENTERED: [(usize, f32, f32); 5] = [
        (3, 78.0, 130.0),
        (4, 68.25, 97.5),
        (5, 62.4, 78.0),
        (6, 58.5, 65.0),
        // Seven items share the whole window: the least and the most are the same.
        (7, 390.0 / 7.0, 390.0 / 7.0),
    ];

    #[test]
    fn centered_items_sit_together_in_the_middle_each_as_wide_as_its_share() {
        for (count, least, most) in CENTERED {
            let mounted = mount(Config {
                arrangement: NavigationBarArrangement::Centered,
                count,
                ..Config::default()
            });
            let width = 56.0_f32.clamp(least, most);
            #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
            let group = width * count as f32;
            for (index, item) in mounted.items.iter().enumerate() {
                let what = format!("{count} items, item {index}");
                let (x, _, found, height) = mounted.rect(item.tab);
                assert_near(found, width, &what);
                assert_near(height, 64.0, &what);
                #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
                let left = (WINDOW - group) / 2.0 + width * index as f32;
                assert_near(x, left, &what);
            }
        }
    }

    #[test]
    fn a_centered_item_grows_with_its_content_up_to_its_share_of_the_bar() {
        // Three start items: each is as wide as its pill, which is more than the least and less than the most.
        let mounted = mount(Config {
            arrangement: NavigationBarArrangement::Centered,
            icon_position: NavigationItemIconPosition::Start,
            ..Config::default()
        });
        let mut widths = Vec::new();
        for item in &mounted.items {
            let (_, _, tab, _) = mounted.rect(item.tab);
            let (_, _, pill, _) = mounted.rect(item.area);
            assert_near(tab, pill, "item");
            assert!(tab > 78.0 && tab < 130.0, "{tab}");
            widths.push(tab);
        }
        // They still sit in the middle of the bar, each as wide as it needs: the room is the same on both sides.
        let first = mounted.rect(mounted.items[0].tab).0;
        let (last_x, _, last_width, _) = mounted.rect(mounted.items[2].tab);
        assert_near(first, WINDOW - (last_x + last_width), "room on each side");
        assert!(widths[0] < widths[1] && widths[1] < widths[2], "{widths:?}");

        // Six of them cannot grow past a sixth of the bar.
        let mounted = mount(Config {
            arrangement: NavigationBarArrangement::Centered,
            icon_position: NavigationItemIconPosition::Start,
            count: 6,
            ..Config::default()
        });
        for item in &mounted.items {
            assert_near(mounted.rect(item.tab).2, 65.0, "item");
        }
    }

    #[test]
    fn equal_weight_is_the_default_and_the_centered_share_is_androidxs() {
        assert_eq!(
            NavigationBarArrangement::default(),
            NavigationBarArrangement::EqualWeight
        );
        assert_eq!(
            NavigationItemIconPosition::default(),
            NavigationItemIconPosition::Top
        );
        assert_eq!(NavigationItemLabel::default(), NavigationItemLabel::Shown);
        // No items: nothing to divide by.
        assert_eq!(centered_share(0), (1.0, 1.0));
        let table = [
            (1, 0.0, 0.0),
            (3, 0.2, 1.0 / 3.0),
            (4, 0.175, 0.25),
            (5, 0.16, 0.2),
            (6, 0.15, 1.0 / 6.0),
            (7, 1.0 / 7.0, 1.0 / 7.0),
            (8, 1.0 / 8.0, 1.0 / 8.0),
        ];
        for (count, least, most) in table {
            if count == 1 {
                // One or two items get a group of 90 % or 80 %... of nothing a formula says: not a case androidx
                // recommends, but the formula still gives the bar a share.
                continue;
            }
            let (found_least, found_most) = centered_share(count);
            assert!((found_least - least).abs() < 1e-6, "{count}: {found_least}");
            assert!((found_most - most).abs() < 1e-6, "{count}: {found_most}");
        }
    }

    #[test]
    fn the_container_stays_clear_of_the_system_bars() {
        let mounted = mount(Config {
            safe_area: Edges {
                top: px(47.0),
                right: px(12.0),
                bottom: px(34.0),
                left: px(10.0),
            },
            ..Config::default()
        });
        // The container is as tall as the row and the bottom bar, and as wide as the window; the row is as tall as
        // always, and as wide as what is between the side bars; the top bar is not its concern.
        assert_eq!(
            mounted.rect(mounted.container),
            (0.0, 0.0, WINDOW, 64.0 + 34.0)
        );
        assert_eq!(mounted.rect(mounted.bar), (10.0, 0.0, WINDOW - 22.0, 64.0));
        let share = (WINDOW - 22.0) / 3.0;
        for (index, item) in mounted.items.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "a handful of items")]
            let left = 10.0 + share * index as f32;
            let (x, _, width, _) = mounted.rect(item.tab);
            assert_near(x, left, "item");
            assert_near(width, share, "item");
        }
    }

    #[test]
    fn an_item_alone_takes_the_whole_row() {
        // No bar tells it to be centered: it shares the row equally, with no one.
        let (_app, recording) = headless(Root::of::<Lone>(), Config::default());
        let items = recording.items();
        let tab = items
            .iter()
            .find_map(|item| match item {
                MountItem::ViewProps { id, props }
                    if props.accessibility.role == AccessibilityRole::Tab =>
                {
                    Some(*id)
                }
                _ => None,
            })
            .expect("the item");
        let width = items.iter().find_map(|item| match item {
            MountItem::Frame { id, frame } if *id == tab => Some(frame.size.width.0),
            _ => None,
        });
        assert_eq!(width, Some(WINDOW));
    }

    #[test]
    fn a_press_reports_and_the_app_selects_the_item() {
        let (mut app, recording) = start(Config::default());
        app.press("label:Library").unwrap();
        assert!(app.shows("presses: 1"));
        assert!(app.shows("selected: 2"));
        let mounted = Mounted::of(&recording.items());
        let selected: Vec<bool> = mounted
            .items
            .iter()
            .map(|item| mounted.accessibility[&item.tab].state.selected)
            .collect();
        assert_eq!(selected, [false, false, true]);
        // The press of the selected item is reported too.
        app.press("label:Library").unwrap();
        assert!(app.shows("presses: 2"));
        assert!(app.shows("selected: 2"));
    }

    #[test]
    fn a_disabled_item_takes_no_press() {
        let (mut app, _recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Search");
        assert!(app.shows("presses: 0"));
        assert!(app.shows("selected: 0"));
        // The others still do.
        app.press("label:Library").unwrap();
        assert!(app.shows("selected: 2"));
    }

    #[test]
    fn an_item_disabled_later_takes_no_press_either() {
        let (mut app, recording) = start(Config::default());
        app.press("Disable").unwrap();
        let _ = app.press("label:Search");
        assert!(app.shows("presses: 0"));
        let mounted = Mounted::of(&recording.items());
        assert!(mounted.accessibility[&mounted.items[1].tab].state.disabled);
        // Its colors are the disabled ones from then on.
        assert_eq!(
            mounted.icon_color(&mounted.items[1]),
            color(scheme_color(ColorRole::OnSurfaceVariant, false), ALPHA_38)
        );
    }

    #[test]
    fn the_arrow_keys_move_between_the_items_and_select_them() {
        let (mut app, _recording) = start(Config::default());
        app.press_key("tab").unwrap();
        app.press_key("right").unwrap();
        assert!(app.shows("selected: 1"));
        assert!(app.shows("presses: 1"));
    }

    /// The opacity the state layer of item `index` was last given.
    fn layer_opacity(recording: &Recording, index: usize) -> f32 {
        let mounted = Mounted::of(&recording.items());
        mounted.style(mounted.items[index].layer).opacity
    }

    #[test]
    fn hover_shows_the_state_layer_and_changes_nothing_else() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        for index in 0..3 {
            assert_eq!(layer_opacity(&recording, index), 0.0, "{index}");
        }
        app.hover("label:Search")?;
        let hover = recording.in_step("hover(label:Search)");
        // `StateTokens.HoverStateLayerOpacity`, on the pill's area of that item, and nothing else is sent: no frame,
        // no drawing, and no other view's style.
        assert_eq!(layer_opacity(&recording, 1), 0.08);
        assert_eq!(layer_opacity(&recording, 0), 0.0);
        assert_eq!(layer_opacity(&recording, 2), 0.0);
        assert!(
            hover.iter().all(|item| matches!(
                item,
                MountItem::ViewStyle { id, .. } if *id == mounted.items[1].layer
            )),
            "{hover:?}"
        );
        assert_eq!(mounted.frames_in(&hover), 0);
        Ok(())
    }

    /// The platform reports `states` of view `id` (the headless driver reports hover and focus itself, not a press: a
    /// platform tracks the pressed state of the views whose `TrackStates` ask for it).
    fn report(app: &mut Headless, id: NodeId, states: u16) {
        app.step("report");
        app.engine().control().set_states(id, states);
        app.settle();
    }

    #[test]
    fn a_press_shows_the_pressed_state_layer_over_the_hover_one_until_it_ends() {
        use nsh_core::tailwind::state::{HOVERED, PRESSED};
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let tab = mounted.items[2].tab;
        // The item asks the platform for both states: the pointer over it, and a press on it.
        let tracked = recording.items().iter().find_map(|item| match item {
            MountItem::TrackStates { id, states } if *id == tab => Some(*states),
            _ => None,
        });
        assert_eq!(tracked, Some(HOVERED | PRESSED));
        // A pointer that is over it shows the hover state; pressed, the pressed one, as `StateTokens` says: 0.1.
        report(&mut app, tab, HOVERED);
        assert_eq!(layer_opacity(&recording, 2), 0.08);
        report(&mut app, tab, HOVERED | PRESSED);
        assert_eq!(layer_opacity(&recording, 2), 0.1);
        report(&mut app, tab, HOVERED);
        assert_eq!(layer_opacity(&recording, 2), 0.08);
        report(&mut app, tab, 0);
        assert_eq!(layer_opacity(&recording, 2), 0.0);
        // The other items are not touched by it.
        assert_eq!(layer_opacity(&recording, 0), 0.0);
        assert_eq!(layer_opacity(&recording, 1), 0.0);
    }

    #[test]
    fn keyboard_focus_shows_the_focused_state_layer() {
        let (mut app, recording) = start(Config::default());
        app.press_key("tab").unwrap();
        // `StateTokens.FocusStateLayerOpacity`, on the selected item, which is the one the keyboard reaches first.
        assert_eq!(layer_opacity(&recording, 0), 0.1);
        assert_eq!(layer_opacity(&recording, 1), 0.0);
    }

    #[test]
    fn a_disabled_item_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        // Its state layer is not laid out (`hidden`, CSS's `display: none`): it has no size to show a state in.
        assert_eq!(mounted.size(mounted.items[1].layer), Size::default());
        // The others have theirs.
        assert_eq!(
            mounted.size(mounted.items[0].layer),
            Size::new(px(56.0), px(32.0))
        );
        // Hovered, it stays out of the layout: a state with no size shows nothing.
        let _ = app.hover("label:Search");
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.size(mounted.items[1].layer), Size::default());
    }

    #[test]
    fn the_state_layer_is_round_and_sits_over_the_pill_not_under_the_icon() {
        let mounted = mount(Config::default());
        let item = mounted.items[0];
        // The pill, the icon, then the state layer: the layer is drawn over both, as androidx's ripple is.
        let mut order = Vec::new();
        for (id, name) in [
            (item.pill, "pill"),
            (item.icon, "icon"),
            (item.layer, "layer"),
        ] {
            let siblings: Vec<NodeId> = mounted
                .parents
                .iter()
                .filter(|(_, parent)| **parent == item.area)
                .map(|(child, _)| *child)
                .collect();
            assert!(siblings.contains(&id), "{name}");
            order.push(name);
        }
        assert_eq!(order, ["pill", "icon", "layer"]);
        assert_eq!(mounted.style(item.layer).border_radius, px(9999.0));
    }

    /// The step of the press of `selector`.
    fn press_step(selector: &str) -> String {
        format!("press({selector})")
    }

    /// `spring` is the indicator's: `DefaultSpatial`, converted as ADR 0004 says, to `to`, from where the value is
    /// and as fast as it moves, at the screen's rate.
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

    fn spring_of(springs: &[StartedSpring], node: animation::NodeId) -> StartedSpring {
        let ours: Vec<StartedSpring> = springs.iter().copied().filter(|s| s.node == node).collect();
        let [spring] = ours[..] else {
            panic!("one spring of {node:?}, not {springs:?}");
        };
        spring
    }

    /// The pill of an area of `area`, grown to `progress`, as ADR 0011 traces it: as wide as the progress says,
    /// centered, its ends half circles.
    fn pill_at(progress: f32, area: Size<Points>) -> Vec<Step> {
        let height = area.height.0;
        let wide = (area.width.0 * progress).max(0.0);
        let radius = wide.min(height) / 2.0;
        let dx = (area.width.0 - wide) / 2.0;
        rounded_rectangle(Size::new(Points(wide), area.height), 0.0, radius)
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

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for position in POSITIONS {
            for selected in [0, 1] {
                let (_app, recording) = start(Config {
                    icon_position: position,
                    selected,
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{position:?}");
                // Each pill is created where it rests, and layout sets nothing again.
                assert_eq!(set_values(&items), 0, "{position:?}");
                assert!(!replay(&recording).is_animating(), "{position:?}");
            }
        }
    }

    #[test]
    fn a_pill_at_rest_is_whole_when_selected_and_nothing_when_not() {
        for position in POSITIONS {
            let (_app, recording) = start(Config {
                icon_position: position,
                selected: 1,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let rest = replay(&recording);
            for (index, item) in mounted.items.iter().enumerate() {
                let what = format!("{position:?} item {index}");
                let area = mounted.size(item.area);
                let drawn = paths(mounted.drawing(item.pill), &rest);
                // The drawing has the size the layout gave the area, even when it is not the one the item expected
                // before its layout (a start item's with its label).
                let progress = f32::from(u8::from(index == 1));
                assert_eq!(drawn.len(), 1, "{what}");
                assert_traces(&drawn[0], &pill_at(progress, area), &what);
                assert_eq!(rest.value(mounted.fade(item)), progress, "{what}");
            }
        }
    }

    #[test]
    fn a_top_pill_is_drawn_once() {
        // Its area is the size the item expected: the layout changes nothing, so nothing is drawn again.
        let (_app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let drawings = recording
            .items()
            .iter()
            .filter(
                |item| matches!(item, MountItem::Canvas { id, .. } if *id == mounted.items[0].pill),
            )
            .count();
        assert_eq!(drawings, 1);
    }

    #[test]
    fn selecting_an_item_springs_its_pill_open_and_the_old_one_shut_and_the_drawing_follows() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let (home, search) = (mounted.items[0], mounted.items[1]);
        app.press("label:Search").unwrap();
        let step = press_step("label:Search");
        let items = recording.in_step(&step);
        // Two springs and no more: the pill of Search opens, the pill of Home closes, and Library, which stays as it
        // is, is not sent anything.
        let springs = started_springs(&items);
        assert_eq!(springs.len(), 2, "{springs:?}");
        let open = spring_of(&springs, mounted.progress(&search));
        let shut = spring_of(&springs, mounted.progress(&home));
        assert_indicator_spring(&open, 1.0, "open");
        assert_indicator_spring(&shut, 0.0, "shut");
        assert_eq!(set_values(&items), 0, "{items:?}");
        // Nothing of the bar is laid out again: the width of the pill is a number the canvas draws with.
        assert_eq!(mounted.frames_in(&items), 0, "{items:?}");
        assert!(!items.iter().any(|item| matches!(item, MountItem::Canvas { id, .. } if *id == search.pill || *id == home.pill)));
        // While they play, the engine sends nothing (ADR 0010).
        app.step("run");
        app.run_for(Duration::from_secs(1)).unwrap();
        assert!(recording.in_step("run").is_empty());

        let area = mounted.size(search.area);
        let curve = |node, from: f32, spring: &StartedSpring| {
            let mut played = replay(&recording);
            let samples = played.run(node, 180);
            let gap = worst_gap(&samples, f64::from(from), spring);
            assert!(
                gap < tolerance(f64::from(from), f64::from(spring.to)),
                "{gap}"
            );
            samples
        };
        // The curve is the closed-form spring, within 0.1 % of its travel, and the open pill overshoots its area.
        let opening = curve(open.node, 0.0, &open);
        let peak = opening.iter().copied().fold(f32::MIN, f32::max);
        assert!(peak > 1.01, "{peak}");
        // The closing one goes a little below nothing before it settles on it.
        let closing = curve(shut.node, 1.0, &shut);
        let trough = closing.iter().copied().fold(f32::MAX, f32::min);
        assert!(trough < -0.01, "{trough}");

        // Along the way the canvas draws the pill as wide as the value says, and the fade is the value between 0 and 1.
        for (item, samples) in [(search, &opening), (home, &closing)] {
            let node = mounted.progress(&item);
            for frame in [2_usize, 5, 9, 14, 30] {
                let mut played = replay(&recording);
                let _ = played.run(node, frame + 1);
                let now = played.value(node);
                assert_eq!(now.to_bits(), samples[frame].to_bits());
                let what = format!("frame {frame}: {now}");
                let drawn = paths(mounted.drawing(item.pill), &played);
                assert_traces(&drawn[0], &pill_at(now, area), &what);
                assert_near(
                    played.value(mounted.fade(&item)),
                    now.clamp(0.0, 1.0),
                    &what,
                );
            }
        }
        // At the peak the pill is wider than its area, and fully shown; at the trough it is nothing, and hidden.
        let at = |node, samples: &[f32], wanted: f32| {
            let frame = samples
                .iter()
                .position(|value| value.to_bits() == wanted.to_bits())
                .expect("the extreme");
            let mut played = replay(&recording);
            let _ = played.run(node, frame + 1);
            played
        };
        let played = at(open.node, &opening, peak);
        let drawn = paths(mounted.drawing(search.pill), &played);
        assert_traces(&drawn[0], &pill_at(peak, area), "peak");
        // Its right side is past the area's.
        assert!(
            drawn[0]
                .iter()
                .any(|step| matches!(step, Step::Line(x, _) if *x > area.width.0)),
            "{drawn:?}"
        );
        assert_eq!(played.value(mounted.fade(&search)), 1.0);
        let played = at(shut.node, &closing, trough);
        let drawn = paths(mounted.drawing(home.pill), &played);
        assert_traces(&drawn[0], &pill_at(0.0, area), "trough");
        assert_eq!(played.value(mounted.fade(&home)), 0.0);

        // They settle exactly where they go, and the graph stops.
        let mut played = replay(&recording);
        let settled = played.run(open.node, 600);
        assert_eq!(settled.last().map(|v| v.to_bits()), Some(1.0_f32.to_bits()));
        assert!(!played.is_animating());
        let mut played = replay(&recording);
        let settled = played.run(shut.node, 600);
        assert_eq!(settled.last().map(|v| v.to_bits()), Some(0.0_f32.to_bits()));
        assert!(!played.is_animating());
    }

    #[test]
    fn selecting_another_item_while_one_moves_goes_from_where_and_as_fast_as_it_moves() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let (search, library) = (mounted.items[1], mounted.items[2]);
        app.press("label:Search").unwrap();
        let first = spring_of(
            &started_springs(&recording.in_step(&press_step("label:Search"))),
            mounted.progress(&search),
        );
        let mut played = replay(&recording);
        // The finger moves on 50 ms in, while the pill of Search still opens.
        let before = played.run(first.node, 6);
        app.press("label:Library").unwrap();
        let items = recording.in_step(&press_step("label:Library"));
        let springs = started_springs(&items);
        // Library opens, and Search, which was opening, closes: Home stays shut, and is not sent anything.
        assert_eq!(springs.len(), 2, "{springs:?}");
        assert_indicator_spring(
            &spring_of(&springs, mounted.progress(&library)),
            1.0,
            "library",
        );
        let back = spring_of(&springs, mounted.progress(&search));
        assert_indicator_spring(&back, 0.0, "search");
        assert_eq!(set_values(&items), 0);
        played.apply(&items);
        let after = played.run(back.node, 180);

        // The graph last ticked 5 frames into the first spring: the second starts there, at that value and that speed.
        let press = parameters(&first);
        let at = 5.0 / REPLAY_HZ;
        let position = |t: f64| spring_position(0.0, 1.0, 0.0, press, t);
        let from = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed > 1.0, "{speed}");
        let expected = |velocity: f64| {
            after
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                    let closed = spring_position(from, 0.0, velocity, parameters(&back), t);
                    (f64::from(value) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        let tolerance = tolerance(from, 0.0);
        assert!(expected(speed) < tolerance, "{}", expected(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(expected(0.0) > 10.0 * tolerance, "{}", expected(0.0));
        let settled = played.run(back.node, 600);
        assert!(!played.is_animating());
        assert_eq!(settled.last().map(|v| v.to_bits()), Some(0.0_f32.to_bits()));
    }

    #[test]
    fn with_reduce_motion_the_pills_go_where_they_go_at_once() {
        // ADR 0011, "Reduce motion": no spring.
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        app.press("label:Search").unwrap();
        let items = recording.in_step(&press_step("label:Search"));
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 2);
        let played = replay(&recording);
        assert!(!played.is_animating());
        let area = mounted.size(mounted.items[1].area);
        for (index, progress) in [(0, 0.0), (1, 1.0), (2, 0.0)] {
            let item = mounted.items[index];
            assert_eq!(played.value(mounted.progress(&item)), progress, "{index}");
            let drawn = paths(mounted.drawing(item.pill), &played);
            assert_traces(&drawn[0], &pill_at(progress, area), &format!("{index}"));
        }
    }

    #[test]
    fn pressing_the_selected_item_moves_nothing() {
        let (mut app, recording) = start(Config::default());
        app.press("label:Home").unwrap();
        let items = recording.in_step(&press_step("label:Home"));
        assert!(app.shows("presses: 1"));
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 0);
    }

    #[test]
    fn a_new_item_count_moves_the_centered_shares() {
        // An item that goes away leaves the others the share of the items that remain.
        let (mut app, recording) = start(Config {
            arrangement: NavigationBarArrangement::Centered,
            count: 4,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        assert_near(mounted.rect(mounted.items[0].tab).2, 68.25, "four items");
        app.press("Hide last").unwrap();
        let mounted = Mounted::of(&recording.items());
        let widths: Vec<f32> = mounted
            .items
            .iter()
            .filter(|item| mounted.frames.contains_key(&item.tab))
            .take(3)
            .map(|item| mounted.rect(item.tab).2)
            .collect();
        assert_eq!(widths.len(), 3);
        for width in widths {
            assert_near(width, 78.0, "three items");
        }
    }

    /// The vendored androidx files, at the pinned commit (ADR 0012).
    const SHORT_NAVIGATION_BAR_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ShortNavigationBar.kt");
    const NAVIGATION_ITEM_KT: &str =
        include_str!("../../../reference/androidx-material3-components/NavigationItem.kt");
    const COLOR_SCHEME_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ColorScheme.kt");
    const RIPPLE_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Ripple.kt");
    const NAVIGATION_BAR_TOKENS_KT: &str =
        include_str!("../../../reference/androidx-material3-tokens/NavigationBarTokens.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_indicators_area_and_the_spaces_the_bar_writes_are_androidxs() {
        // The indicator's area around the icon, and the spaces the bar writes itself.
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "internal val TopIconIndicatorVerticalPadding =",
                "(NavigationBarVerticalItemTokens.ActiveIndicatorHeight -",
                "NavigationBarVerticalItemTokens.IconSize) / 2",
                "/*@VisibleForTesting*/",
                "internal val TopIconIndicatorHorizontalPadding =",
                "(NavigationBarVerticalItemTokens.ActiveIndicatorWidth -",
                "NavigationBarVerticalItemTokens.IconSize) / 2",
                "/*@VisibleForTesting*/",
                "internal val StartIconIndicatorVerticalPadding =",
                "(NavigationBarHorizontalItemTokens.ActiveIndicatorHeight -",
                "NavigationBarHorizontalItemTokens.IconSize) / 2",
                "/*@VisibleForTesting*/",
                "internal val TopIconIndicatorToLabelPadding: Dp",
                "get() = 4.dp",
                "/*@VisibleForTesting*/",
                "internal val StartIconIndicatorHorizontalPadding =",
                "NavigationBarHorizontalItemTokens.ActiveIndicatorLeadingSpace",
                "/*@VisibleForTesting*/",
                "internal val StartIconToLabelPadding",
                "get() = NavigationBarTokens.ItemActiveIndicatorIconLabelSpace",
            ]
        ));
        assert_eq!(TOP_INDICATOR_TO_LABEL, Points(4.0));
        assert_eq!(START_ICON_TO_LABEL, Points(4.0));
        assert_eq!(top_indicator(), Size::new(Points(56.0), Points(32.0)));
        assert_eq!(start_indicator_padding(), (Points(16.0), Points(8.0)));
    }

    #[test]
    fn the_label_type_and_the_colors_are_androidxs() {
        // The space above and below the content of an item with a label, and none without one.
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "internal val TopIconItemVerticalPadding",
                "get() = NavigationBarVerticalItemTokens.ContainerBetweenSpace",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "topIconItemPadding =",
                "if (label != null) PaddingValues(vertical = TopIconItemVerticalPadding)",
                "else PaddingValues(0.dp),",
                "startIconItemPadding = PaddingValues(0.dp),",
            ]
        ));
        assert_eq!(vertical::CONTAINER_BETWEEN_SPACE, Points(6.0));
        // The label's type, for both positions.
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &["labelTextStyle = NavigationBarTokens.LabelTextFont.value,"]
        ));
        assert!(cites(
            NAVIGATION_BAR_TOKENS_KT,
            &[
                "inline val LabelTextFont: TypographyToken",
                "get() = TypographyKeyTokens.LabelMedium",
            ]
        ));
        // The colors, and the text color of a selected item with its icon at the start, which has no token.
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "selectedIconColor = fromToken(NavigationBarTokens.ItemActiveIconColor),",
                "selectedTextColorTopIconPosition =",
                "fromToken(NavigationBarTokens.ItemActiveLabelTextColor),",
                "// TODO: Replace with the correct token once it is available in",
                "// NavigationBarTokens",
                "selectedTextColorStartIconPosition =",
                "fromToken(NavigationBarTokens.ItemActiveIconColor),",
                "selectedIndicatorColor =",
                "fromToken(NavigationBarTokens.ItemActiveIndicatorColor),",
                "unselectedIconColor = fromToken(NavigationBarTokens.ItemInactiveIconColor),",
                "unselectedTextColor =",
                "fromToken(NavigationBarTokens.ItemInactiveLabelTextColor),",
                "disabledIconColor =",
                "fromToken(NavigationBarTokens.ItemInactiveIconColor)",
                ".copy(alpha = DisabledAlpha),",
                "disabledTextColor =",
                "fromToken(NavigationBarTokens.ItemInactiveLabelTextColor)",
                ".copy(alpha = DisabledAlpha),",
            ]
        ));
        assert!(cites(
            COLOR_SCHEME_KT,
            &["internal const val DisabledAlpha = 0.38f"]
        ));
        assert_eq!(DISABLED_OPACITY.to_bits(), 0.38_f32.to_bits());
        // A selected item's colors switch at once in the bar (the rail's item animates them).
        assert!(cites(
            NAVIGATION_ITEM_KT,
            &[
                "labelTextStyle = labelTextStyle,",
                "color = textColor,",
                "enabled = enabled,",
                "animateColor = false,",
            ]
        ));
    }

    #[test]
    fn the_container_its_insets_and_its_height_are_androidxs() {
        // The container: its colors, its insets and its height.
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "public val containerColor: Color",
                "@Composable get() = NavigationBarTokens.ContainerColor.value",
                "",
                "/** Default content color for a short navigation bar. */",
                "public val contentColor: Color",
                "@Composable get() = contentColorFor(containerColor)",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "WindowInsets.systemBarsForVisualComponents.only(",
                "WindowInsetsSides.Horizontal + WindowInsetsSides.Bottom",
                ")",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "Surface(color = containerColor, contentColor = contentColor, modifier = modifier) {",
                "Layout(",
                "modifier =",
                "Modifier.windowInsetsPadding(windowInsets)",
                ".defaultMinSize(minHeight = containerHeight)",
                ".selectableGroup(),",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &["containerHeight = NavigationBarTokens.ContainerHeight,"]
        ));
    }

    #[test]
    fn the_state_layer_and_the_touch_target_are_androidxs() {
        // The state layer is the ripple's, in the content color the bar gives.
        assert!(cites(
            NAVIGATION_ITEM_KT,
            &[".indication(interactionSource, ripple(focusRingShape = indicatorShape))"]
        ));
        assert!(cites(
            RIPPLE_KT,
            &[
                "*   used will be [LocalContentColor] instead. If inset focus rings are enabled, their colors will"
            ]
        ));
        // The touch target of an item.
        assert!(cites(
            NAVIGATION_ITEM_KT,
            &[
                ".defaultMinSize(",
                "minWidth = LocalMinimumInteractiveComponentSize.current,",
                "minHeight = LocalMinimumInteractiveComponentSize.current,",
                ")",
            ]
        ));
    }

    #[test]
    fn the_indicators_spring_and_the_centered_shares_are_androidxs() {
        // The indicator's spring, and the shares of the centered arrangement.
        assert!(cites(
            NAVIGATION_ITEM_KT,
            &[
                "animateFloatAsState(",
                "targetValue = if (selected) 1f else 0f,",
                "// TODO Load the motionScheme tokens from the component tokens file",
                "animationSpec = MotionSchemeKeyTokens.DefaultSpatial.value(),",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "if (itemsCount > 6) return 0",
                "// Formula to calculate the padding percentage based on the number of items and bar width.",
                "val paddingPercentage = ((100 - 10 * (itemsCount + 3)) / 2f) / 100",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &[
                "val itemMaxWidth = width / itemsCount",
                "barHorizontalPadding = calculateCenteredContentHorizontalPadding(itemsCount, width)",
                "val itemMinWidth = (width - (barHorizontalPadding * 2)) / itemsCount",
            ]
        ));
        assert!(cites(
            SHORT_NAVIGATION_BAR_KT,
            &["currentItemWidth = measurableWidth.coerceAtMost(itemMaxWidth)"]
        ));
    }

    #[test]
    fn a_line_that_androidx_does_not_have_is_not_found() {
        // The rail's item animates its colors; the bar's does not.
        assert!(!cites(SHORT_NAVIGATION_BAR_KT, &["animateColor = true,"]));
        assert!(!cites(NAVIGATION_ITEM_KT, &["animateColor = false"]));
    }
}
