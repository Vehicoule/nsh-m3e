//! The Material 3 Expressive extended floating action button: a label, and an optional icon before it, in a container
//! of three sizes and two color styles, that collapses to its icon.
//!
//! The extended FAB follows androidx's `SmallExtendedFloatingActionButton`, `MediumExtendedFloatingActionButton` and
//! `LargeExtendedFloatingActionButton` in `FloatingActionButton.kt`, at the pinned commit, as [ADR 0012] says. Its
//! container, colors, state layer and elevation are the FAB's ([`crate::fab`]): androidx builds both on the same
//! private `FloatingActionButton`, with the same default container color and the same elevations
//! (`FloatingActionButtonDefaults`). The baseline extended FAB (`ExtendedFloatingActionButton`, 56 points high with
//! its own paddings and its own expand and collapse motion) is the one that predates Expressive; it is not ported.
//!
//! # Where this FAB follows `FloatingActionButton.kt` and not the token files
//!
//! Each of these is a value of `FloatingActionButton.kt` that is not the token of its size ([ADR 0012]). The module's
//! tests pin each one to the line it cites.
//!
//! - **The medium icon-to-label space is 12 points**, not the 16 of `ExtendedFabMediumTokens.IconLabelSpace`:
//!   `MediumExtendedFabIconPadding` is written `12.dp` under "ExtendedFabMediumTokens.IconLabelSpace is incorrect"
//!   (line 1454).
//! - **The large icon-to-label space is 16 points**, not the 20 of `ExtendedFabLargeTokens.IconLabelSpace`
//!   (`LargeExtendedFabIconPadding`, line 1467, under the same TODO).
//! - **The medium shape is `CornerLargeIncreased`** (20 points): `ExtendedFabMediumTokens` has no container shape (its
//!   `ContainerShape` is commented out), and `mediumExtendedFabShape` is `ShapeDefaults.LargeIncreased` under "TODO:
//!   update to use token" (line 1038). `CornerLargeIncreased` is also the shape of the medium FAB.
//!
//! The small and the large shapes, the icon sizes, the paddings at the sides and the label styles are the tokens'
//! (`ExtendedFabSmallTokens`, `ExtendedFabMediumTokens`, `ExtendedFabLargeTokens`) and androidx's text styles
//! (`SmallExtendedFabTextStyle` title medium, `MediumExtendedFabTextStyle` title large, `LargeExtendedFabTextStyle`
//! headline small, lines 1444 to 1470), which `FloatingActionButton.kt` writes and the token files confirm.
//!
//! # Layout
//!
//! The container is as tall as its size says and at least as wide ([`ExtendedFabSize`] gives both): `minWidth` and
//! `minHeight` of `FloatingActionButton.kt` are both `ContainerHeight`, so a collapsed extended FAB is a square.
//! Its content is a row: the icon (if it has one) after the leading space, then the label after the icon-to-label
//! space, then the trailing space. The container is as wide as that row is, or as its minimum if the row is narrower.
//!
//! - **Without an icon** there is nothing to collapse to, so the FAB is always expanded, whatever
//!   [`ExtendedFabProps::expansion`] says, and its content is centered in the container (androidx's icon-less overloads
//!   take only the content, with no `expanded`).
//! - **A collapsed FAB keeps its leading space**, as androidx's does (its `Row` is padded by the leading space whatever
//!   the width its `layout` modifier reports, lines 981 to 996): the icon is centered in the square when the leading
//!   space is half of what the icon leaves, which is the case of the small and the medium FAB, and 4 points left of
//!   the middle in the large one, whose icon is 32 points in a 96-point square with 28 points before it.
//!
//! # Collapsing and expanding
//!
//! [`ExtendedFabExpansion`] collapses the FAB to its icon or expands it to its label, with the two motions of
//! androidx's `ExtendedFloatingActionButton` for the small, medium and large sizes (lines 961 to 968): the width
//! springs with the fast spatial spring and the label's opacity with the fast effects spring.
//!
//! - **The width is a layout animation.** NSH moves a frame with a layout animation, run natively
//!   (`ElementRef::animate_layout`), and not with a spring that sets a width from the engine. Its curve takes a damping
//!   ratio and a duration where M3E gives a stiffness and a ratio, so [`crate::motion::SpringSpec::layout_transition`]
//!   converts exactly: the same spring, which NSH fits into the duration. Like androidx's width, it overshoots: the
//!   container is for a moment narrower than its collapsed width when it collapses and wider than its label needs when
//!   it expands.
//! - **The label is clipped by the container while the width moves** (a view with the container's shape that clips what
//!   it holds), as androidx's `Surface` clips its content to the shape. The label stays in the layout, so the width the
//!   FAB expands to is its content's, as androidx's `maxIntrinsicWidth`; it is invisible, and still the FAB's name for
//!   screen readers (the button's own label), once it has faded out.
//! - **The label's opacity** springs from where it is, as the FAB's visibility does ([`crate::fab`]): a change that
//!   reverses another in flight goes on from there.
//! - **With reduce motion on**, the FAB collapses and expands at once.
//! - **A FAB made collapsed or expanded starts there**, with nothing animating.
//!
//! # Not ported
//!
//! The show and hide animation (`Modifier.animateFloatingActionButton`), which androidx applies to any FAB from
//! outside, is [`crate::fab::FabVisibility`]'s and is not offered here yet: an extended FAB that an app hides is
//! not common, and the FAB's own is the reference when it is wanted.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::element::LayoutTransition;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::fab::{FabState, FabStyle, reveal_spring, rounded, style_spec};
use crate::icon::Icon;
use crate::motion::SpringToken;
use crate::tint::role_color;
use crate::tokens::androidx::{
    extended_fab_large_tokens, extended_fab_medium_tokens, extended_fab_small_tokens, shape_tokens,
    state_tokens, typography_tokens,
};
use crate::tokens::{Shape, TypeStyle};

/// The size of an [`ExtendedFab`]: the three of androidx's `FloatingActionButton.kt`, from 56 to 96 points high.
///
/// Each size takes its container, icon size, paddings and shape from its token file (`ExtendedFabSmallTokens`,
/// `ExtendedFabMediumTokens` and `ExtendedFabLargeTokens`), except where `FloatingActionButton.kt` writes its own
/// (this module's documentation lists them).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ExtendedFabSize {
    /// 56 points high, a 24-point icon, a title medium label: androidx `SmallExtendedFloatingActionButton`.
    #[default]
    Small,
    /// 80 points high, a 28-point icon, a title large label: androidx `MediumExtendedFloatingActionButton`.
    Medium,
    /// 96 points high, a 32-point icon, a headline small label: androidx `LargeExtendedFloatingActionButton`.
    Large,
}

/// Whether an [`ExtendedFab`] shows its label: androidx's `expanded`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ExtendedFabExpansion {
    /// The icon and the label, in a container as wide as they are.
    #[default]
    Expanded,
    /// The icon alone, in a square as wide as the container is high. Asking for it animates the width and the label's
    /// opacity; a FAB with no icon stays expanded.
    Collapsed,
}

impl ExtendedFabExpansion {
    /// The label's opacity once the FAB has settled: 1 expanded, 0 collapsed (androidx `if (expanded) 1f else 0f`,
    /// line 961, which the alpha progress follows).
    const fn opacity(self) -> f32 {
        match self {
            ExtendedFabExpansion::Expanded => 1.0,
            ExtendedFabExpansion::Collapsed => 0.0,
        }
    }
}

/// What a size gives the extended FAB.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's height, which is also its minimum width: androidx's `minWidth` and `minHeight`.
    height: Points,
    /// The icon's side.
    icon: Points,
    /// The space between the icon and the label.
    gap: Points,
    /// The space before the icon, or before the label of a FAB with no icon.
    leading: Points,
    /// The space after the label.
    trailing: Points,
    /// The label's type style.
    label: TypeStyle,
    /// The container's shape, which no state changes.
    shape: Shape,
}

/// The medium icon-to-label space: androidx `MediumExtendedFabIconPadding`, written `12.dp` under "TODO:
/// ExtendedFabMediumTokens.IconLabelSpace is incorrect" (`FloatingActionButton.kt` line 1454), where the token says 16
/// (ADR 0012).
const MEDIUM_GAP: Points = Points(12.0);

/// The large icon-to-label space: androidx `LargeExtendedFabIconPadding`, written `16.dp` under the same TODO (line
/// 1467), where the token says 20 (ADR 0012).
const LARGE_GAP: Points = Points(16.0);

/// The medium shape: androidx `FloatingActionButtonDefaults.mediumExtendedFabShape`, which is
/// `ShapeDefaults.LargeIncreased` under "TODO: update to use token" (`FloatingActionButton.kt` line 1038). That is
/// `ShapeTokens.CornerLargeIncreased` (ADR 0012, as the medium FAB's shape is).
const MEDIUM_SHAPE: Shape = shape_tokens::CORNER_LARGE_INCREASED;

/// The values of `size`, from its token file, or from the component where it overrides the token.
fn size_spec(size: ExtendedFabSize) -> SizeSpec {
    match size {
        ExtendedFabSize::Small => SizeSpec {
            height: extended_fab_small_tokens::CONTAINER_HEIGHT,
            icon: extended_fab_small_tokens::ICON_SIZE,
            gap: extended_fab_small_tokens::ICON_LABEL_SPACE,
            leading: extended_fab_small_tokens::LEADING_SPACE,
            trailing: extended_fab_small_tokens::TRAILING_SPACE,
            label: typography_tokens::TITLE_MEDIUM,
            shape: extended_fab_small_tokens::CONTAINER_SHAPE,
        },
        ExtendedFabSize::Medium => SizeSpec {
            height: extended_fab_medium_tokens::CONTAINER_HEIGHT,
            icon: extended_fab_medium_tokens::ICON_SIZE,
            gap: MEDIUM_GAP,
            leading: extended_fab_medium_tokens::LEADING_SPACE,
            trailing: extended_fab_medium_tokens::TRAILING_SPACE,
            label: typography_tokens::TITLE_LARGE,
            shape: MEDIUM_SHAPE,
        },
        ExtendedFabSize::Large => SizeSpec {
            height: extended_fab_large_tokens::CONTAINER_HEIGHT,
            icon: extended_fab_large_tokens::ICON_SIZE,
            gap: LARGE_GAP,
            leading: extended_fab_large_tokens::LEADING_SPACE,
            trailing: extended_fab_large_tokens::TRAILING_SPACE,
            label: typography_tokens::HEADLINE_SMALL,
            shape: extended_fab_large_tokens::CONTAINER_SHAPE,
        },
    }
}

/// The expansion a FAB has: the one asked for, or expanded for a FAB with no icon, which has nothing to collapse to.
fn effective(asked: ExtendedFabExpansion, has_icon: bool) -> ExtendedFabExpansion {
    if has_icon {
        asked
    } else {
        ExtendedFabExpansion::Expanded
    }
}

/// The layout transition of the width: androidx plays the width with the motion scheme's `FastSpatial` spring both
/// ways (`sizeAnimationSpec`, `FloatingActionButton.kt` line 959), converted so that NSH's curve is that spring.
fn width_transition() -> LayoutTransition {
    SpringToken::FastSpatial.spec().layout_transition()
}

/// The props of an [`ExtendedFab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ExtendedFabProps {
    /// The label: what the FAB shows when it is expanded, and what screen readers say whether it is or not.
    pub label: Prop<SharedString>,
    /// The icon before the label, none by default: a FAB with no icon is always expanded.
    #[prop(default)]
    pub icon: Prop<Option<Icon>>,
    /// Its size.
    #[prop(default)]
    pub size: Prop<ExtendedFabSize>,
    /// Its color style, the FAB's.
    #[prop(default)]
    pub style: Prop<FabStyle>,
    /// Whether it shows its label. Asking for the other expansion moves its width and fades its label with M3E's
    /// springs.
    #[prop(default)]
    pub expansion: Prop<ExtendedFabExpansion>,
}

/// The events of an [`ExtendedFab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ExtendedFabEvent {
    /// The FAB was pressed.
    Press,
}

/// Runs the change of expansion: each time `expansion` becomes another, the frames its container moves animate with
/// [`width_transition`] and the label's `opacity` springs to where the expansion puts it. With reduce motion on, the
/// opacity goes there at once and no frame animates. Nothing runs for the expansion the FAB is made with.
///
/// The layout animation is asked for before the commit that moves the width (`ElementRef::animate_layout` takes the
/// next commit), which the effect that sets the width runs in too, as `nsh-base`'s `Collapsible` does.
fn follow_expansion(
    expansion: Memo<ExtendedFabExpansion>,
    container: ElementRef,
    opacity: AnimatedValue,
) {
    let shown = stored(untracked(|| expansion.get()));
    effect(move || {
        let to = expansion.get();
        untracked(|| {
            if shown.replace(to) == to {
                return;
            }
            if reduce_motion() {
                opacity.set(to.opacity());
                return;
            }
            container.animate_layout(width_transition());
            // The end is not waited for: the label stays in the layout, and the width's end is the platform's.
            drop(opacity.spring(to.opacity(), reveal_spring(SpringToken::FastEffects)));
        });
    });
}

/// A Material 3 extended floating action button: a label, with an icon before it, in a container of the size and color
/// style given.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the button role, named by its label for screen
/// readers) and the look of androidx's `ExtendedFloatingActionButton`, which is the [`crate::fab::Fab`]'s (container
/// color, state layer, elevation) with a label. Hover, focus and press lay the content's color over it at the state
/// layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so nothing is laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::extended_fab::{ExtendedFabExpansion, ExtendedFabSize, extended_fab};
/// use nsh_m3e::icon::Icon;
///
/// fn compose(scrolled_down: ReadSignal<bool>) -> impl IntoElement {
///     extended_fab()
///         .label("Compose")
///         // Material Symbols' `edit`.
///         .icon(Some(Icon::material_symbol(
///             "M200-200h57l391-391-57-57-391 391v57Zm-80 80v-170l528-527q12-11 26.5-17t30.5-6q16 0 31 6t26 18l55 56q12 11 17.5 26t5.5 30q0 16-5.5 30.5T817-647L290-120H120Z",
///         )))
///         .size(ExtendedFabSize::Medium)
///         // Collapsed to its icon, with M3E's springs, while the content scrolls down.
///         .expansion(move || {
///             if scrolled_down.get() {
///                 ExtendedFabExpansion::Collapsed
///             } else {
///                 ExtendedFabExpansion::Expanded
///             }
///         })
///         .on_press(|| println!("compose"))
/// }
/// ```
#[component]
pub fn ExtendedFab(props: ExtendedFabProps, cx: Scope<ExtendedFabEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || style_spec(props.style.get());
    // The label's and the icon's color, and the state layer's: androidx's content color.
    let content_color = move || role_color(colors().icon);
    let has_icon = move || props.icon.get().is_some();
    // A memo, so that the effect below runs when the expansion changes and not when it is only set again.
    let expansion = computed(move || effective(props.expansion.get(), has_icon()));
    let container = element_ref();
    let label_opacity = animated_value(untracked(|| expansion.get()).opacity());
    follow_expansion(expansion, container, label_opacity);
    // The two shadow layers of the container's level in `state` (ADR 0018), read from the style as it is now.
    let shadows = move |state| props.style.get().level(state).shadows();
    nsh_base::button()
        .element_ref(&container)
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(ExtendedFabEvent::Press))
        .group()
        .justify_center()
        .h(move || size().height)
        .min_w(move || size().height)
        .bg(move || role_color(colors().container))
        .style_with(move |s| {
            let s = rounded(s, size().shape).shadow(shadows(FabState::Rest));
            match expansion.get() {
                ExtendedFabExpansion::Expanded => s,
                // The width the layout animation moves: the minimum, which a container cannot be narrower than at
                // rest.
                ExtendedFabExpansion::Collapsed => s.w(size().height),
            }
        })
        // The shadow of each state the container is in: NSH resolves them natively, with nothing running, and a
        // change of level snaps, as the FAB's does ([`crate::fab`] says why).
        .hover(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Hovered))))
        .focus_within(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Focused))))
        .active(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Pressed))))
        .child(
            // The state layer: the content's color over the container, in its shape, shown by the button's own states
            // with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
            view()
                .absolute()
                .inset(px(0.0))
                .bg(content_color)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| rounded(s, size().shape)),
        )
        .child(
            // The content, clipped to the container's shape: its row is as wide as its icon and label, and the
            // container is narrower while it collapses.
            view()
                .flex_row()
                .items_center()
                .overflow_hidden()
                .h(move || size().height)
                .ps(move || size().leading)
                .pe(move || size().trailing)
                .style_with(move |s| {
                    let s = rounded(s, size().shape);
                    if has_icon() { s } else { s.justify_center() }
                })
                .child(when(has_icon, move || {
                    // The icon, at its size, in the content color.
                    canvas(move || {
                        props
                            .icon
                            .get()
                            .map(|icon| icon.drawing(size().icon))
                            .unwrap_or_default()
                    })
                    .w(move || size().icon)
                    .h(move || size().icon)
                    .fill_color(content_color)
                }))
                .child(
                    // The label, after the icon-to-label space (none without an icon), fading out as the FAB
                    // collapses: androidx's `graphicsLayer { alpha }` on the row of the space and the text.
                    view()
                        .flex_row()
                        .style_with(move |s| if has_icon() { s.ps(size().gap) } else { s })
                        .bind_opacity(&label_opacity)
                        .child(
                            text(move || props.label.get())
                                .text_color(content_color)
                                .text_size(move || size().label.size)
                                .line_height(move || LineHeight::Points(size().label.line_height))
                                .letter_spacing(move || size().label.tracking)
                                .style_with(move |s| s.font_weight(size().label.weight)),
                        ),
                ),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{AnimationProp, NodeId as AnimationNode};
    use nsh_core::canvas::{self, CanvasOp, Drawing};
    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{
        AccessibilityRole, BoxShadow, Color, LayoutCurve, MountItem, NodeId, ViewKind, ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{Recording, layout_items, started_springs};
    use crate::color::{ColorRole, ColorScheme};
    use crate::elevation::Elevation;
    use crate::morph::Corners;
    use crate::morph_probe::{replay, set_values};
    use crate::motion::SPRING_MASS;

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ADD: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";

    /// What the probe places: one extended FAB with these props, a line counting its presses, and lines that change
    /// its expansion and its label.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        label: Prop<SharedString>,
        #[prop(default)]
        size: Prop<ExtendedFabSize>,
        #[prop(default)]
        style: Prop<FabStyle>,
        #[prop(default)]
        expansion: Prop<ExtendedFabExpansion>,
        #[prop(default)]
        icon: Prop<bool>,
    }

    /// The expansion other than `expansion`.
    fn other(expansion: ExtendedFabExpansion) -> ExtendedFabExpansion {
        match expansion {
            ExtendedFabExpansion::Expanded => ExtendedFabExpansion::Collapsed,
            _ => ExtendedFabExpansion::Expanded,
        }
    }

    /// How long `Blink` waits before it undoes what it did: 6 display frames, so that a spring is still moving.
    const BLINK: Duration = Duration::from_millis(50);

    /// The label `Relabel` gives the FAB: longer than any the tests start with.
    const LONGER: &str = "Compose a much longer message";

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        let expansion = signal(props.expansion.get_untracked());
        let label = signal(props.label.get_untracked());
        // A row, so that the FAB is as wide as its content, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    extended_fab()
                        .label(move || label.get())
                        .icon(move || props.icon.get().then(|| Icon::material_symbol(ADD)))
                        .size(props.size)
                        .style(props.style)
                        .expansion(move || expansion.get())
                        .on_press(move || presses.update(|n| *n += 1)),
                ),
            )
            .child(text(move || format!("presses: {}", presses.get())))
            // Asks for the other expansion.
            .child(text("Flip").on_press(move |(): &()| {
                expansion.update(|expansion| *expansion = other(*expansion));
            }))
            // Asks for the other expansion, and for the first again `BLINK` later.
            .child(text("Blink").on_press(move |(): &()| {
                let before = expansion.get_untracked();
                expansion.set(other(before));
                drop(set_timeout(BLINK, move || expansion.set(before)));
            }))
            .child(text("Relabel").on_press(move |(): &()| label.set(SharedString::from(LONGER))))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        label: &'static str,
        size: ExtendedFabSize,
        style: FabStyle,
        expansion: ExtendedFabExpansion,
        icon: bool,
        dark: bool,
        reduce_motion: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                label: "Compose",
                size: ExtendedFabSize::default(),
                style: FabStyle::default(),
                expansion: ExtendedFabExpansion::default(),
                icon: true,
                dark: false,
                reduce_motion: false,
            }
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            label: Prop::new(SharedString::from(config.label)),
            size: Prop::new(config.size),
            style: Prop::new(config.style),
            expansion: Prop::new(config.expansion),
            icon: Prop::new(config.icon),
        });
        // The web has a mouse to hover with, whatever its idiom.
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
        app.wait_for("text:presses: 0").unwrap();
        (app, recording)
    }

    /// The extended FAB's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole FAB, which is its container.
        button: NodeId,
        /// The view of the state layer.
        layer: NodeId,
        /// The view that clips the content to the container's shape.
        clip: NodeId,
        /// The canvas that draws the icon, if it has one.
        icon: Option<NodeId>,
        /// The row of the icon-to-label space and the label, whose opacity is animated.
        label_row: NodeId,
        /// The label's text.
        label: NodeId,
        /// The animated value the label row's opacity is bound to.
        opacity: AnimationNode,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        text: nsh_core::ResolvedTextStyle,
        name: Option<Arc<str>>,
        hit_slop: Option<EdgeInsets>,
    }

    /// Which view is which, found from how `items` nested them.
    struct Parts {
        layer: NodeId,
        clip: NodeId,
        icon: Option<NodeId>,
        label_row: NodeId,
        label: NodeId,
        opacity: AnimationNode,
    }

    impl Parts {
        fn of(
            button: NodeId,
            kinds: &HashMap<NodeId, ViewKind>,
            children: &HashMap<NodeId, Vec<NodeId>>,
            bindings: &HashMap<NodeId, Vec<(AnimationProp, AnimationNode)>>,
            items: &[MountItem],
        ) -> Self {
            let nested = |id: NodeId| children.get(&id).cloned().unwrap_or_default();
            // The state layer, under the content; and nothing else in the button.
            let [layer, clip] = nested(button)[..] else {
                panic!("two views in the extended FAB: {items:?}");
            };
            assert_eq!(kinds.get(&layer), Some(&ViewKind::View), "{items:?}");
            assert_eq!(kinds.get(&clip), Some(&ViewKind::View), "{items:?}");
            // The icon if there is one, then the label's row, and nothing else in the clip.
            let (icon, label_row) = match nested(clip)[..] {
                [icon, label_row] => (Some(icon), label_row),
                [label_row] => (None, label_row),
                _ => panic!("an icon and a label, or a label, in the clip: {items:?}"),
            };
            if let Some(icon) = icon {
                assert_eq!(kinds.get(&icon), Some(&ViewKind::Canvas), "{items:?}");
            }
            let [label] = nested(label_row)[..] else {
                panic!("the label in its row: {items:?}");
            };
            assert_eq!(kinds.get(&label), Some(&ViewKind::Text), "{items:?}");
            let opacity: Vec<AnimationNode> = bindings
                .get(&label_row)
                .into_iter()
                .flatten()
                .filter(|(prop, _)| matches!(prop, AnimationProp::Opacity))
                .map(|(_, node)| *node)
                .collect();
            let [opacity] = opacity[..] else {
                panic!("one opacity bound to the label's row: {bindings:?}");
            };
            Self {
                layer,
                clip,
                icon,
                label_row,
                label,
                opacity,
            }
        }
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut button = None;
            let mut name = None;
            let mut hit_slop = None;
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            let mut texts = HashMap::new();
            let mut bindings = HashMap::new();
            let mut labels = HashMap::new();
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
                        let siblings: &mut Vec<NodeId> = children.entry(*parent).or_default();
                        let at = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(at, *child);
                    }
                    MountItem::ViewProps { id, props }
                        if props.accessibility.role == AccessibilityRole::Button =>
                    {
                        button = Some(*id);
                        name = props.accessibility.label.clone();
                        hit_slop = Some(props.hit_slop);
                    }
                    MountItem::AccessibilityLabel { id, label } => {
                        labels.insert(*id, label.clone());
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
                    MountItem::Text {
                        id, text: content, ..
                    } => {
                        texts.insert(*id, content.spans.first().map(|span| span.style));
                    }
                    MountItem::AnimationBindings {
                        id,
                        bindings: found,
                    } => {
                        bindings.insert(*id, found.clone());
                    }
                    _ => {}
                }
            }
            let button = button.expect("a view with the button role");
            // A new label is sent as its own item, after the props the FAB was made with.
            let name = labels.get(&button).cloned().unwrap_or(name);
            let Parts {
                layer,
                clip,
                icon,
                label_row,
                label,
                opacity,
            } = Parts::of(button, &kinds, &children, &bindings, items);
            Self {
                button,
                layer,
                clip,
                icon,
                label_row,
                label,
                opacity,
                styles,
                frames,
                drawings,
                text: texts
                    .get(&label)
                    .copied()
                    .flatten()
                    .expect("the label's style"),
                name,
                hit_slop,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }

        /// The FAB's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frame(self.button).size
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// How many of `items` lay the FAB out again: one of its views' frames or its label's text.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let icon = self.icon.into_iter();
            let ours: Vec<NodeId> = [
                self.button,
                self.layer,
                self.clip,
                self.label_row,
                self.label,
            ]
            .into_iter()
            .chain(icon)
            .collect();
            items
                .iter()
                .filter(|item| match item {
                    MountItem::Frame { id, .. } | MountItem::Text { id, .. } => ours.contains(id),
                    _ => false,
                })
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

    /// The opaque color of `role` in the baseline scheme.
    fn solid(role: ColorRole, dark: bool) -> Color {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        color(scheme.get(role).to_hex(), 255)
    }

    /// The color `ops` fill with, if any.
    fn fill(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    const STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

    const SIZES: [ExtendedFabSize; 3] = [
        ExtendedFabSize::Small,
        ExtendedFabSize::Medium,
        ExtendedFabSize::Large,
    ];

    /// Every size: its height, its icon's side, the space between icon and label, the space before and after, and its
    /// corners' radius. androidx `ExtendedFabSmallTokens`, `ExtendedFabMediumTokens` and `ExtendedFabLargeTokens`,
    /// and `FloatingActionButton.kt` where it overrides them: the medium and the large icon-to-label spaces, and the
    /// medium shape.
    const EXPECTED: [(ExtendedFabSize, f32, f32, f32, f32, f32, f32); 3] = [
        (ExtendedFabSize::Small, 56.0, 24.0, 8.0, 16.0, 16.0, 16.0),
        (ExtendedFabSize::Medium, 80.0, 28.0, 12.0, 26.0, 26.0, 20.0),
        (ExtendedFabSize::Large, 96.0, 32.0, 16.0, 28.0, 28.0, 28.0),
    ];

    /// The vendored `FloatingActionButton.kt`.
    const FLOATING_ACTION_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/FloatingActionButton.kt");
    /// The vendored `ExtendedFabMediumTokens.kt`, whose `ContainerShape` androidx comments out.
    const EXTENDED_FAB_MEDIUM_TOKENS_KT: &str =
        include_str!("../../../reference/androidx-material3-tokens/ExtendedFabMediumTokens.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn each_size_is_its_token_file_and_where_androidx_overrides_it_androidx() {
        for (size, height, icon, gap, leading, trailing, radius) in EXPECTED {
            let spec = size_spec(size);
            let what = format!("{size:?}");
            assert_eq!(spec.height, Points(height), "{what}");
            assert_eq!(spec.icon, Points(icon), "{what}");
            assert_eq!(spec.gap, Points(gap), "{what}");
            assert_eq!(spec.leading, Points(leading), "{what}");
            assert_eq!(spec.trailing, Points(trailing), "{what}");
            assert_eq!(
                Corners::of(spec.shape),
                Some(Corners::Radius(Points(radius))),
                "{what}"
            );
        }
        // Each size's height, icon, paddings are its tokens'.
        assert_eq!(
            size_spec(ExtendedFabSize::Small).height,
            extended_fab_small_tokens::CONTAINER_HEIGHT
        );
        assert_eq!(
            size_spec(ExtendedFabSize::Medium).height,
            extended_fab_medium_tokens::CONTAINER_HEIGHT
        );
        assert_eq!(
            size_spec(ExtendedFabSize::Large).height,
            extended_fab_large_tokens::CONTAINER_HEIGHT
        );
        assert_eq!(
            size_spec(ExtendedFabSize::Small).gap,
            extended_fab_small_tokens::ICON_LABEL_SPACE
        );
    }

    #[test]
    fn the_overrides_are_the_lines_of_floating_action_button_kt_they_cite() {
        // The icon-to-label spaces of the medium and the large size, the literals and the tokens they disagree with.
        for (property, literal, gap, token) in [
            (
                "private val MediumExtendedFabIconPadding",
                "get() = 12.dp",
                MEDIUM_GAP,
                extended_fab_medium_tokens::ICON_LABEL_SPACE,
            ),
            (
                "private val LargeExtendedFabIconPadding",
                "get() = 16.dp",
                LARGE_GAP,
                extended_fab_large_tokens::ICON_LABEL_SPACE,
            ),
        ] {
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[property, literal]),
                "{property}"
            );
            assert_ne!(gap, token, "{property}");
        }
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "// TODO: ExtendedFabMediumTokens.IconLabelSpace is incorrect",
                "private val MediumExtendedFabIconPadding",
            ]
        ));
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "// TODO: ExtendedFabLargeTokens.IconLabelSpace is incorrect",
                "private val LargeExtendedFabIconPadding",
            ]
        ));
        assert_eq!(extended_fab_medium_tokens::ICON_LABEL_SPACE, Points(16.0));
        assert_eq!(extended_fab_large_tokens::ICON_LABEL_SPACE, Points(20.0));
        // The small size's is its token.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "private val SmallExtendedFabIconPadding",
                "get() = ExtendedFabSmallTokens.IconLabelSpace",
            ]
        ));
        // The medium shape: `ShapeDefaults.LargeIncreased`, and the key the commented-out token names.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "public val mediumExtendedFabShape: Shape",
                "@Composable get() = ShapeDefaults.LargeIncreased // TODO: update to use token",
            ]
        ));
        assert!(cites(
            EXTENDED_FAB_MEDIUM_TOKENS_KT,
            &["// val ContainerShape = ShapeKeyTokens.CornerLargeIncreased"]
        ));
        assert_eq!(MEDIUM_SHAPE, shape_tokens::CORNER_LARGE_INCREASED);
        // The other shapes are their tokens'.
        for (name, token, shape, size) in [
            (
                "smallExtendedFabShape",
                "ExtendedFabSmallTokens.ContainerShape.value",
                extended_fab_small_tokens::CONTAINER_SHAPE,
                ExtendedFabSize::Small,
            ),
            (
                "largeExtendedFabShape",
                "ExtendedFabLargeTokens.ContainerShape.value",
                extended_fab_large_tokens::CONTAINER_SHAPE,
                ExtendedFabSize::Large,
            ),
        ] {
            let property = format!("public val {name}: Shape");
            let getter = format!("@Composable get() = {token}");
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&property, &getter]),
                "{name}"
            );
            assert_eq!(size_spec(size).shape, shape, "{name}");
        }
        assert_eq!(size_spec(ExtendedFabSize::Medium).shape, MEDIUM_SHAPE);
        assert_eq!(size_spec(ExtendedFabSize::Medium).gap, MEDIUM_GAP);
        assert_eq!(size_spec(ExtendedFabSize::Large).gap, LARGE_GAP);
    }

    #[test]
    fn the_sizes_are_androidxs_minimum_of_their_height_and_their_paddings_and_label_styles() {
        for (token, size) in [
            ("Small", ExtendedFabSize::Small),
            ("Medium", ExtendedFabSize::Medium),
            ("Large", ExtendedFabSize::Large),
        ] {
            // The container's minimum width and height are both its height token.
            let width = format!("private val {token}ExtendedFabMinimumWidth");
            let height = format!("private val {token}ExtendedFabMinimumHeight");
            let tokens = format!("get() = ExtendedFab{token}Tokens.ContainerHeight");
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&width, &tokens]),
                "{token}"
            );
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&height, &tokens]),
                "{token}"
            );
            // The paddings at the sides are the tokens'.
            for (side, leading) in [("Start", true), ("End", false)] {
                let property = format!("private val {token}ExtendedFabPadding{side}");
                let getter = format!(
                    "get() = ExtendedFab{token}Tokens.{}",
                    if leading {
                        "LeadingSpace"
                    } else {
                        "TrailingSpace"
                    }
                );
                assert!(
                    cites(FLOATING_ACTION_BUTTON_KT, &[&property, &getter]),
                    "{token}"
                );
            }
            assert_eq!(size_spec(size).leading, size_spec(size).trailing, "{token}");
        }
        // The label styles.
        for (token, style, expected) in [
            ("Small", "TitleMedium", typography_tokens::TITLE_MEDIUM),
            ("Medium", "TitleLarge", typography_tokens::TITLE_LARGE),
            ("Large", "HeadlineSmall", typography_tokens::HEADLINE_SMALL),
        ] {
            let line =
                format!("private val {token}ExtendedFabTextStyle = TypographyKeyTokens.{style}");
            assert!(cites(FLOATING_ACTION_BUTTON_KT, &[&line]), "{token}");
            let size = match token {
                "Small" => ExtendedFabSize::Small,
                "Medium" => ExtendedFabSize::Medium,
                _ => ExtendedFabSize::Large,
            };
            assert_eq!(size_spec(size).label, expected, "{token}");
        }
    }

    #[test]
    fn each_style_has_its_colors_light_and_dark() {
        for style in STYLES {
            for dark in [false, true] {
                let what = format!("{style:?} dark: {dark}");
                let mounted = mount(Config {
                    style,
                    dark,
                    ..Config::default()
                });
                let (container, content) = match style {
                    // androidx `FabPrimaryContainerTokens`: primary container, on primary container.
                    FabStyle::PrimaryContainer => {
                        (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer)
                    }
                    // androidx `FabSecondaryContainerTokens`.
                    _ => (
                        ColorRole::SecondaryContainer,
                        ColorRole::OnSecondaryContainer,
                    ),
                };
                let icon = mounted.icon.expect("an icon");
                assert_eq!(
                    mounted.style(mounted.button).background_color,
                    solid(container, dark),
                    "{what}"
                );
                // The icon, the label and the state layer take the content color, and not the container's.
                assert_eq!(
                    fill(&mounted.drawing(icon).ops),
                    Some(solid(content, dark)),
                    "{what}"
                );
                assert_eq!(mounted.text.color, solid(content, dark), "{what}");
                assert_eq!(
                    mounted.style(mounted.layer).background_color,
                    solid(content, dark),
                    "{what}"
                );
                assert_ne!(solid(container, dark), solid(content, dark), "{what}");
            }
        }
    }

    #[test]
    fn the_label_has_the_size_s_type_style() {
        for size in SIZES {
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let style = size_spec(size).label;
            assert_eq!(mounted.text.font_size, style.size, "{size:?}");
            assert_eq!(mounted.text.line_height, style.line_height, "{size:?}");
            assert_eq!(mounted.text.letter_spacing, style.tracking, "{size:?}");
            assert_eq!(mounted.text.font_weight, style.weight.0, "{size:?}");
        }
        // The three sizes do not share a style.
        let sizes: Vec<_> = SIZES
            .map(|size| size_spec(size).label.size.0.to_bits())
            .to_vec();
        assert_eq!(sizes.len(), 3);
        assert!(sizes[0] < sizes[1] && sizes[1] < sizes[2]);
    }

    /// The width of the label's text, as its layout gave it.
    fn label_width(mounted: &Mounted) -> f32 {
        mounted.frame(mounted.label).size.width.0
    }

    #[test]
    fn an_expanded_fab_is_as_wide_as_its_icon_its_label_and_the_spaces_around_them() {
        for (size, height, icon, gap, leading, trailing, _) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let frame = mounted.size();
            assert_eq!(frame.height.0, height, "{what}");
            assert_eq!(
                frame.width.0,
                leading + icon + gap + label_width(&mounted) + trailing,
                "{what}"
            );
            assert!(frame.width.0 > height, "{what}: wider than it is tall");
            // The state layer and the clip cover the FAB.
            for id in [mounted.layer, mounted.clip] {
                let covering = mounted.frame(id);
                assert_eq!(
                    (covering.origin.x.0, covering.origin.y.0),
                    (0.0, 0.0),
                    "{what}"
                );
                assert_eq!(covering.size, frame, "{what}");
            }
            // The icon after the leading space, centered in the height; then the space, then the label.
            let bounds = mounted.frame(mounted.icon.expect("an icon"));
            assert_eq!(
                (bounds.size.width.0, bounds.size.height.0),
                (icon, icon),
                "{what}"
            );
            assert_eq!(bounds.origin.x.0, leading, "{what}");
            assert_eq!(bounds.origin.y.0, (height - icon) / 2.0, "{what}");
            let row = mounted.frame(mounted.label_row);
            assert_eq!(row.origin.x.0, leading + icon, "{what}");
            assert_eq!(mounted.frame(mounted.label).origin.x.0, gap, "{what}");
        }
    }

    #[test]
    fn a_collapsed_fab_is_a_square_with_its_icon_where_the_leading_space_puts_it() {
        for (size, height, icon, _, leading, _, _) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                expansion: ExtendedFabExpansion::Collapsed,
                ..Config::default()
            });
            assert_eq!(mounted.size(), Size::new(px(height), px(height)), "{what}");
            let bounds = mounted.frame(mounted.icon.expect("an icon"));
            // androidx keeps the leading space when it collapses, so the icon is centered in the square only when
            // the leading space is half of what the icon leaves: the small and the medium FAB. The large one's
            // icon is 4 points left of the middle.
            assert_eq!(bounds.origin.x.0, leading, "{what}");
            let off_center = height / 2.0 - (bounds.origin.x.0 + icon / 2.0);
            let expected = if size == ExtendedFabSize::Large {
                4.0
            } else {
                0.0
            };
            assert_eq!(off_center, expected, "{what}");
            // The label is still laid out, past the icon, where the clip hides it.
            let label = mounted.frame(mounted.label_row);
            assert!(label.origin.x.0 >= leading + icon, "{what}");
            // The label is fully transparent, and the FAB is still named by it.
            assert_eq!(
                replay(
                    &start(Config {
                        size,
                        expansion: ExtendedFabExpansion::Collapsed,
                        ..Config::default()
                    })
                    .1
                )
                .value(mounted.opacity)
                .to_bits(),
                0.0_f32.to_bits(),
                "{what}"
            );
        }
    }

    #[test]
    fn an_expanded_fab_shows_its_label_at_full_opacity() {
        let (_app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(
            replay(&recording).value(mounted.opacity).to_bits(),
            1.0_f32.to_bits()
        );
    }

    #[test]
    fn a_fab_with_no_icon_is_always_expanded_and_centers_its_content() {
        for (size, height, _, _, leading, trailing, _) in EXPECTED {
            for expansion in [
                ExtendedFabExpansion::Expanded,
                ExtendedFabExpansion::Collapsed,
            ] {
                let what = format!("{size:?} {expansion:?}");
                let mounted = mount(Config {
                    // A label so short that its content is narrower than the container's minimum width.
                    label: ".",
                    size,
                    expansion,
                    icon: false,
                    ..Config::default()
                });
                assert!(mounted.icon.is_none(), "{what}");
                let frame = mounted.size();
                let content = leading + label_width(&mounted) + trailing;
                assert!(content < height, "{what}: the label is short enough");
                // The container is its minimum and the content is centered in it (to the half point layout
                // rounds to): the label has no icon-to-label space, and the leading and trailing spaces are the
                // same.
                assert_eq!(frame, Size::new(px(height), px(height)), "{what}");
                let label = mounted.frame(mounted.label_row);
                assert_eq!(label.size.width.0, label_width(&mounted), "{what}");
                let start = (height - content) / 2.0 + leading;
                assert!(
                    (label.origin.x.0 - start).abs() <= 0.5,
                    "{what}: {label:?} at {start}"
                );
                assert_eq!(mounted.frame(mounted.label).origin.x.0, 0.0, "{what}");
                // It is not faded out: there is nothing else to show.
                assert_eq!(
                    replay(&start_recording(size, expansion))
                        .value(mounted.opacity)
                        .to_bits(),
                    1.0_f32.to_bits(),
                    "{what}"
                );
            }
        }
    }

    /// The recording of a FAB with no icon.
    fn start_recording(size: ExtendedFabSize, expansion: ExtendedFabExpansion) -> Recording {
        start(Config {
            label: ".",
            size,
            expansion,
            icon: false,
            ..Config::default()
        })
        .1
    }

    #[test]
    fn only_the_container_casts_a_shadow_and_no_view_has_a_border_or_another_effect() {
        for (size, ..) in EXPECTED {
            for style in STYLES {
                let mounted = mount(Config {
                    size,
                    style,
                    ..Config::default()
                });
                let mut views = vec![
                    mounted.button,
                    mounted.layer,
                    mounted.clip,
                    mounted.label_row,
                ];
                views.extend(mounted.icon);
                for id in views {
                    let view = mounted.style(id);
                    let what = format!("{size:?} {style:?}");
                    // Android's own material shadow is not used: the shadow is ADR 0018's layers.
                    assert_eq!(view.elevation, Points(0.0), "{what}");
                    assert_eq!(view.borders.widths.top, px(0.0), "{what}");
                    assert_eq!(view.outline, nsh_core::Outline::default(), "{what}");
                    if id == mounted.button {
                        // The shadow is the only effect: no gradient, filter or mask.
                        let effects = view.effects.as_deref().expect("the container's shadow");
                        assert!(effects.background_images.is_empty(), "{what}");
                        assert!(effects.filters.is_empty(), "{what}");
                        assert!(effects.backdrop_filters.is_empty(), "{what}");
                        assert!(effects.mask_images.is_empty(), "{what}");
                    } else {
                        // Neither the state layer nor the content casts one: the container's is the FAB's only.
                        assert!(view.effects.is_none(), "{what}: {:?}", view.effects);
                    }
                }
            }
        }
    }

    #[test]
    fn each_size_has_its_corners_on_the_container_the_state_layer_and_the_clip() {
        for (size, .., radius) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            for id in [mounted.button, mounted.layer, mounted.clip] {
                let style = mounted.style(id);
                assert_eq!(style.border_radius.0, radius, "{what}");
                // No corner of its own: the four are the same.
                assert_eq!(style.corner_radius, nsh_core::Corners::default(), "{what}");
            }
            // Nothing else has a radius: not the icon, not the label's row.
            for id in mounted.icon.into_iter().chain([mounted.label_row]) {
                assert_eq!(mounted.style(id).border_radius.0, 0.0, "{what}");
            }
        }
    }

    #[test]
    fn the_clip_clips_the_content_and_the_container_does_not_clip_its_shadow() {
        for expansion in [
            ExtendedFabExpansion::Expanded,
            ExtendedFabExpansion::Collapsed,
        ] {
            let mounted = mount(Config {
                expansion,
                ..Config::default()
            });
            // The shadow is drawn outside the container's box: a clip on the container would cut it off.
            assert!(
                !mounted.style(mounted.button).overflow_hidden,
                "{expansion:?}"
            );
            assert!(mounted.style(mounted.clip).overflow_hidden, "{expansion:?}");
            assert!(
                !mounted.style(mounted.layer).overflow_hidden,
                "{expansion:?}"
            );
            assert!(
                !mounted.style(mounted.label_row).overflow_hidden,
                "{expansion:?}"
            );
        }
    }

    #[test]
    fn no_size_needs_a_touch_slop() {
        // Every container is at least 56 points high and wide, past the 48 of a touch target: no slop.
        for (size, height, ..) in EXPECTED {
            for expansion in [
                ExtendedFabExpansion::Expanded,
                ExtendedFabExpansion::Collapsed,
            ] {
                let mounted = mount(Config {
                    size,
                    expansion,
                    ..Config::default()
                });
                assert!(height >= 48.0);
                assert_eq!(
                    mounted.hit_slop,
                    Some(EdgeInsets::default()),
                    "{size:?} {expansion:?}"
                );
            }
        }
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_button_expanded_collapsed_or_with_no_icon() {
        for (icon, expansion) in [
            (true, ExtendedFabExpansion::Expanded),
            (true, ExtendedFabExpansion::Collapsed),
            (false, ExtendedFabExpansion::Expanded),
        ] {
            let mounted = mount(Config {
                expansion,
                icon,
                ..Config::default()
            });
            assert_eq!(
                mounted.name.as_deref(),
                Some("Compose"),
                "{icon} {expansion:?}"
            );
        }
    }

    #[test]
    fn a_new_label_is_the_name_and_the_text_and_widens_the_fab_at_once() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let before = Mounted::of(&recording.items());
        app.press("Relabel")?;
        let items = recording.in_step("press(Relabel)");
        let after = Mounted::of(&recording.items());
        assert_eq!(after.name.as_deref(), Some(LONGER));
        assert!(after.size().width > before.size().width);
        assert_eq!(after.size().height, before.size().height);
        // The width of the label is where it is going at once, as androidx's intrinsic width is: no animation.
        assert_eq!(layout_animations(&items), Vec::new());
        assert_eq!(started_springs(&items), Vec::new());
        Ok(())
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press("label:Compose")?;
        assert!(app.shows("presses: 1"));
        assert!(!app.shows("presses: 2"));
        Ok(())
    }

    /// The opacities the state layer was sent in `items`, in order.
    fn layer_opacities(mounted: &Mounted, items: &[MountItem]) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    /// Nothing but the styles of the FAB's container and state layer changed in `items`, anywhere on screen: no
    /// frame, no text, no drawing, and no other view styled again.
    fn assert_only_the_styles_change(mounted: &Mounted, items: &[MountItem], what: &str) {
        assert_eq!(layout_items(items), 0, "{what}");
        assert_eq!(mounted.layout_items(items), 0, "{what}");
        assert!(
            items.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } => *id == mounted.layer || *id == mounted.button,
                MountItem::Canvas { .. } | MountItem::LayoutAnimation(_) => false,
                _ => true,
            }),
            "{what}: {items:?}"
        );
        assert_eq!(layout_animations(items), Vec::new(), "{what}");
        assert_eq!(started_springs(items), Vec::new(), "{what}");
    }

    #[test]
    fn hover_shows_the_state_layer_in_the_content_color_and_lays_out_nothing() -> Result<(), String>
    {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("label:Compose")?;
        let hover = recording.in_step("hover(label:Compose)");
        // The state layer at `StateTokens.HoverStateLayerOpacity`, once.
        assert_eq!(layer_opacities(&mounted, &hover), [0.08]);
        assert_only_the_styles_change(&mounted, &hover, "hover");
        assert_eq!(
            mounted.style(mounted.layer).background_color,
            solid(ColorRole::OnPrimaryContainer, false)
        );
        // A pointer that leaves takes it away.
        app.step("leave");
        app.hover("text:presses: 0")?;
        assert_eq!(
            layer_opacities(&mounted, &recording.in_step("hover(text:presses: 0)")),
            [0.0]
        );
        Ok(())
    }

    #[test]
    fn focus_shows_the_state_layer_at_the_focus_opacity() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("Tab")?;
        let focus = recording.in_step("keyboard.press(Tab)");
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &focus), [0.1]);
        assert_only_the_styles_change(&mounted, &focus, "focus");
        Ok(())
    }

    #[test]
    fn the_button_asks_the_platform_for_its_hover_and_press_so_that_its_state_layer_shows_them() {
        let (_app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let tracked = recording.items().iter().find_map(|item| match item {
            MountItem::TrackStates { id, states } if *id == mounted.button => Some(*states),
            _ => None,
        });
        let tracked = tracked.expect("the FAB's group tracks states");
        assert_ne!(tracked & nsh_core::tailwind::state::HOVERED, 0);
        assert_ne!(tracked & nsh_core::tailwind::state::PRESSED, 0);
    }

    /// The box shadows `style` has, top layer first.
    fn shadows_of(style: &ViewStyle) -> Vec<BoxShadow> {
        style
            .effects
            .as_deref()
            .map(|effects| effects.box_shadows.clone())
            .unwrap_or_default()
    }

    /// The two layers of `level`, as a list.
    fn layers(level: Elevation) -> Vec<BoxShadow> {
        level.shadows().to_vec()
    }

    /// The shadow the container has once the platform reports `states` for it (bits of
    /// `nsh_core::tailwind::state`), as it does a hover, a focus or a press.
    fn shadows_in(app: &mut Headless, recording: &Recording, states: u16) -> Vec<BoxShadow> {
        let button = Mounted::of(&recording.items()).button;
        app.engine().control().set_states(button, states);
        assert!(app.settle());
        shadows_of(&Mounted::of(&recording.items()).style(button))
    }

    #[test]
    fn the_container_rests_at_level_3_and_the_state_the_platform_reports_decides_the_level() {
        use nsh_core::tailwind::state::{FOCUSED, HOVERED, PRESSED};
        for style in STYLES {
            let (mut app, recording) = start(Config {
                style,
                ..Config::default()
            });
            let rest = shadows_of(
                &Mounted::of(&recording.items()).style(Mounted::of(&recording.items()).button),
            );
            let level_3 = layers(Elevation::Level3);
            let level_4 = layers(Elevation::Level4);
            // The levels differ: a test of level 3 is not a test of level 4.
            assert_ne!(level_3, level_4);
            assert_eq!(rest, level_3, "{style:?} at rest");
            // The same levels the FAB has, which are androidx's `FloatingActionButtonDefaults.elevation()` for both.
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED),
                level_4,
                "{style:?}"
            );
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED | PRESSED),
                level_3,
                "{style:?}"
            );
            assert_eq!(
                shadows_in(&mut app, &recording, PRESSED),
                level_3,
                "{style:?}"
            );
            assert_eq!(
                shadows_in(&mut app, &recording, FOCUSED),
                level_3,
                "{style:?}"
            );
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED | FOCUSED),
                level_4,
                "{style:?}"
            );
            assert_eq!(shadows_in(&mut app, &recording, 0), level_3, "{style:?}");
            // Each layer is a drop shadow, none inset.
            assert!(layers(Elevation::Level3).iter().all(|layer| !layer.inset));
        }
    }

    #[test]
    fn a_collapsed_fab_has_the_same_shadows_and_states_as_an_expanded_one() {
        use nsh_core::tailwind::state::HOVERED;
        let (mut app, recording) = start(Config {
            expansion: ExtendedFabExpansion::Collapsed,
            ..Config::default()
        });
        assert_eq!(
            shadows_of(
                &Mounted::of(&recording.items()).style(Mounted::of(&recording.items()).button)
            ),
            layers(Elevation::Level3)
        );
        assert_eq!(
            shadows_in(&mut app, &recording, HOVERED),
            layers(Elevation::Level4)
        );
    }

    /// The layout animations `items` carry, in order.
    fn layout_animations(items: &[MountItem]) -> Vec<nsh_core::LayoutAnimation> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::LayoutAnimation(animation) => Some(*animation),
                _ => None,
            })
            .collect()
    }

    /// What a layout animation of the width is: the update asks for `width_transition()`, and nothing else is asked.
    fn width_animation() -> nsh_core::LayoutAnimation {
        nsh_core::LayoutAnimation {
            update: Some(width_transition()),
            ..nsh_core::LayoutAnimation::default()
        }
    }

    #[test]
    fn the_width_springs_with_the_fast_spatial_spring_converted_for_a_layout_animation() {
        let transition = width_transition();
        // `FastSpatial`: stiffness 800 and damping ratio 0.6, which NSH's curve takes as the ratio and a duration.
        assert_eq!(
            transition,
            SpringToken::FastSpatial.spec().layout_transition()
        );
        assert_eq!(transition.curve, LayoutCurve::Spring { damping: 0.6 });
        assert!(
            (transition.duration_ms - 407.04).abs() < 0.01,
            "{transition:?}"
        );
        // A spatial spring: it overshoots, as the motion scheme's do.
        assert!(transition.duration_ms > 0.0);
        assert_ne!(
            transition,
            SpringToken::DefaultSpatial.spec().layout_transition()
        );
        assert_ne!(
            transition,
            SpringToken::FastEffects.spec().layout_transition()
        );
    }

    #[test]
    fn collapsing_and_expanding_animate_the_width_and_spring_the_label_s_opacity()
    -> Result<(), String> {
        for size in SIZES {
            let what = format!("{size:?}");
            let (mut app, recording) = start(Config {
                size,
                ..Config::default()
            });
            let expanded = Mounted::of(&recording.items());
            let height = size_spec(size).height;
            // Collapse.
            let (pressed, collapse) = recording.during(|| app.press("Flip"));
            pressed?;
            assert_eq!(layout_animations(&collapse), [width_animation()], "{what}");
            let spring = fast_effects_spring(&collapse, expanded.opacity, 0.0, &what);
            assert_eq!(spring.node, expanded.opacity, "{what}");
            // The frame it moves is the container's, to a square.
            assert_eq!(
                Mounted::of(&recording.items()).size(),
                Size::new(height, height),
                "{what}"
            );
            // Expand again.
            let (pressed, expand) = recording.during(|| app.press("Flip"));
            pressed?;
            assert_eq!(layout_animations(&expand), [width_animation()], "{what}");
            fast_effects_spring(&expand, expanded.opacity, 1.0, &what);
            assert_eq!(
                Mounted::of(&recording.items()).size(),
                expanded.size(),
                "{what}"
            );
        }
        Ok(())
    }

    /// The one spring `items` start, on `opacity` toward `to`, with the fast effects spring and nothing else set.
    fn fast_effects_spring(
        items: &[MountItem],
        opacity: AnimationNode,
        to: f32,
        what: &str,
    ) -> crate::animation_probe::StartedSpring {
        let springs = started_springs(items);
        let [spring] = springs[..] else {
            panic!("{what}: one spring, not {springs:?}");
        };
        let spec = SpringToken::FastEffects.spec();
        assert_eq!(spring.node, opacity, "{what}");
        assert_eq!(spring.to.to_bits(), to.to_bits(), "{what}");
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
        assert_eq!(set_values(items), 0, "{what}: {items:?}");
        spring
    }

    #[test]
    fn a_fab_is_made_where_it_is_and_starts_nothing() {
        for expansion in [
            ExtendedFabExpansion::Expanded,
            ExtendedFabExpansion::Collapsed,
        ] {
            let what = format!("{expansion:?}");
            let (_app, recording) = start(Config {
                expansion,
                ..Config::default()
            });
            let items = recording.items();
            assert_eq!(started_springs(&items), Vec::new(), "{what}");
            assert_eq!(set_values(&items), 0, "{what}");
            assert_eq!(layout_animations(&items), Vec::new(), "{what}");
            let replay = replay(&recording);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                replay.value(Mounted::of(&items).opacity).to_bits(),
                expansion.opacity().to_bits(),
                "{what}"
            );
        }
    }

    #[test]
    fn a_fab_with_no_icon_asks_for_nothing_when_it_is_asked_to_collapse() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            icon: false,
            ..Config::default()
        });
        let before = Mounted::of(&recording.items());
        let (pressed, items) = recording.during(|| app.press("Flip"));
        pressed?;
        assert_eq!(layout_animations(&items), Vec::new());
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(Mounted::of(&recording.items()).size(), before.size());
        assert_eq!(
            replay(&recording).value(before.opacity).to_bits(),
            1.0_f32.to_bits()
        );
        Ok(())
    }

    #[test]
    fn a_collapse_that_is_undone_in_flight_ends_expanded_and_springs_back_from_where_it_is()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press("Blink")?;
        let items = recording.in_step("press(Blink)");
        // Both changes ask for the width, and both spring the opacity: the second from where the first has taken it,
        // keeping its speed (`velocity: None`).
        assert_eq!(
            layout_animations(&items),
            [width_animation(), width_animation()]
        );
        let springs = started_springs(&items);
        let [down, up] = springs[..] else {
            panic!("two springs, not {springs:?}");
        };
        assert_eq!((down.to, up.to), (0.0, 1.0));
        assert_eq!((down.node, up.node), (mounted.opacity, mounted.opacity));
        assert_eq!((down.velocity, up.velocity), (None, None));
        assert_eq!(Mounted::of(&recording.items()).size(), mounted.size());
        Ok(())
    }

    #[test]
    fn with_reduce_motion_the_fab_collapses_and_expands_at_once() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let expanded = Mounted::of(&recording.items());
        for (to, size) in [
            (0.0_f32, Size::new(px(56.0), px(56.0))),
            (1.0, expanded.size()),
        ] {
            let (pressed, items) = recording.during(|| app.press("Flip"));
            pressed?;
            // The opacity is set where it goes, nothing springs and no frame animates.
            assert_eq!(started_springs(&items), Vec::new());
            assert_eq!(layout_animations(&items), Vec::new());
            assert_eq!(set_values(&items), 1, "{items:?}");
            let replay = replay(&recording);
            assert!(!replay.is_animating());
            assert_eq!(replay.value(expanded.opacity).to_bits(), to.to_bits());
            assert_eq!(Mounted::of(&recording.items()).size(), size);
        }
        Ok(())
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        assert_eq!(ExtendedFabSize::default(), ExtendedFabSize::Small);
        assert_eq!(
            ExtendedFabExpansion::default(),
            ExtendedFabExpansion::Expanded
        );
        assert_eq!(FabStyle::default(), FabStyle::PrimaryContainer);
        assert_eq!(
            effective(ExtendedFabExpansion::Collapsed, true),
            ExtendedFabExpansion::Collapsed
        );
        assert_eq!(
            effective(ExtendedFabExpansion::Collapsed, false),
            ExtendedFabExpansion::Expanded
        );
        assert_eq!(
            effective(ExtendedFabExpansion::Expanded, true),
            ExtendedFabExpansion::Expanded
        );
        assert_eq!(
            ExtendedFabExpansion::Expanded.opacity().to_bits(),
            1.0_f32.to_bits()
        );
        assert_eq!(
            ExtendedFabExpansion::Collapsed.opacity().to_bits(),
            0.0_f32.to_bits()
        );
    }
}
