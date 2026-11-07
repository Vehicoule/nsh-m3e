//! The Material 3 Expressive floating action button (FAB): an [`Icon`] in a container of four sizes and two color
//! styles, and the state layer of hover, focus and press.
//!
//! The FAB follows androidx's `FloatingActionButton`, `SmallFloatingActionButton`, `MediumFloatingActionButton` and
//! `LargeFloatingActionButton` in `FloatingActionButton.kt`, at the pinned commit, as [ADR 0012] says. The extended
//! FAB and the FAB menu are other components.
//!
//! # Where this FAB follows `FloatingActionButton.kt` and not the token files
//!
//! Each of these is a value of `FloatingActionButtonDefaults` that is not the token of its size ([ADR 0012]). The
//! module's tests pin each one to the line it cites.
//!
//! - **The large icon is 36 points**, not the 32 of `FabLargeTokens.IconSize`: `LargeIconSize` is written
//!   `36.dp // TODO: FabLargeTokens.IconSize is incorrect` (line 1010). It is the only literal of this module that
//!   comes from the component, and `LARGE_ICON_SIZE` holds it.
//! - **The medium shape is `CornerLargeIncreased`** (20 points): `FabMediumTokens` has no container shape (its
//!   `ContainerShape` is commented out), and `mediumShape` is `ShapeDefaults.LargeIncreased` under "TODO: update to
//!   use token" (line 1022). `ShapeDefaults.LargeIncreased` is `ShapeTokens.CornerLargeIncreased` in androidx's
//!   `Shapes.kt` (line 292 at the pinned commit). That file is not vendored: it was read once, there, and the
//!   generated token holds the value. The commented-out line of `FabMediumTokens.kt` names the same key.
//!
//! The medium icon is `FabMediumTokens.IconSize` (`MediumIconSize`, line 1007), and the small and the default FAB's
//! icons are their tokens' 24 points, which is also the size of the `Icon` androidx's samples put in them.
//!
//! # Color styles
//!
//! androidx's FAB takes the container color of `FabPrimaryContainerTokens` by default
//! (`FloatingActionButtonDefaults.containerColor`) and the content color that goes with it, the icon color of the
//! same file. [`FabStyle::SecondaryContainer`] is the same with `FabSecondaryContainerTokens`, which the vendored
//! token files have and `FloatingActionButton.kt` does not read. A tertiary or a surface FAB token file: not found
//! at the pinned commit, so there is no such style. Neither style has a disabled state: androidx's FAB has no
//! `enabled`.
//!
//! # Shape
//!
//! The FAB's shape does not change when it is pressed: androidx's FAB has no pressed shape (the button's
//! `shapes.pressedShape` has no counterpart in `FloatingActionButton.kt`). So its container is a plain view with a
//! corner radius, and not the drawn container of [ADR 0011], which exists to move a radius with a spring. The shadow
//! of the elevation will follow that radius with no extra work.
//!
//! # Touch target
//!
//! The 40-point small FAB takes presses 4 points around it, so that its target is 48 points, as androidx's
//! `minimumInteractiveComponentSize` does for the components that are smaller; its layout and look do not change
//! (NSH's accessibility guide: extend the touch area with the hit slop, not the size).
//!
//! # Elevation
//!
//! The tokens give the level of the container in each state ([`FabStyle::elevation`]). **No shadow is drawn yet**:
//! how elevation is drawn is ADR 0018's, and its shadow is built in a change of its own that this one does not wait
//! for; the tokens give levels in points, not shadows, so this module invents no shadow value. [`Fab`] marks the one
//! place where the shadow will be applied.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::icon::Icon;
use crate::morph::{Corners, corners};
use crate::tint::{MIN_TOUCH_TARGET, role_color};
use crate::tokens::Shape;
use crate::tokens::androidx::{
    fab_baseline_tokens, fab_large_tokens, fab_medium_tokens, fab_primary_container_tokens,
    fab_secondary_container_tokens, fab_small_tokens, shape_tokens, state_tokens,
};

/// The size of a [`Fab`]: the four of androidx's `FloatingActionButton.kt`, from 40 to 96 points.
///
/// Each size takes its container, icon size and shape from its token file (`FabSmallTokens`, `FabBaselineTokens`,
/// `FabMediumTokens` and `FabLargeTokens`), except where `FloatingActionButton.kt` writes its own (this module's
/// documentation lists them).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum FabSize {
    /// 40 points square, a 24-point icon: androidx `SmallFloatingActionButton`.
    Small,
    /// 56 points square, a 24-point icon: androidx `FloatingActionButton`, its default.
    #[default]
    Baseline,
    /// 80 points square, a 28-point icon: androidx `MediumFloatingActionButton`.
    Medium,
    /// 96 points square, a 36-point icon: androidx `LargeFloatingActionButton`.
    Large,
}

/// The color style of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum FabStyle {
    /// A container in the primary container color, its icon in the color on it: androidx's default
    /// (`FabPrimaryContainerTokens`).
    #[default]
    PrimaryContainer,
    /// A container in the secondary container color, its icon in the color on it
    /// (`FabSecondaryContainerTokens`).
    SecondaryContainer,
}

/// What a [`Fab`] is going through: the interactions that change its elevation (androidx's
/// `FloatingActionButtonElevation`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FabState {
    /// Nothing: no pointer over it, no focus, no press.
    #[default]
    Rest,
    /// A pointer is over it (androidx `HoverInteraction.Enter`).
    Hovered,
    /// It has the focus (androidx `FocusInteraction.Focus`).
    Focused,
    /// A press is down on it (androidx `PressInteraction.Press`).
    Pressed,
}

impl FabStyle {
    /// How far above the surface the container sits in `state`, from the style's token file
    /// (`ContainerElevation`, `HoveredContainerElevation`, `FocusedContainerElevation` and
    /// `PressedContainerElevation`): the level 3 of the elevation tokens at rest, focused and pressed, and level 4
    /// hovered, for both styles.
    ///
    /// This is data: the FAB draws no shadow yet (this module's documentation says why).
    ///
    /// ```
    /// use nsh_m3e::fab::{FabState, FabStyle};
    ///
    /// let style = FabStyle::PrimaryContainer;
    /// assert_eq!(style.elevation(FabState::Rest).0, 6.0);
    /// assert!(style.elevation(FabState::Hovered) > style.elevation(FabState::Rest));
    /// ```
    #[must_use]
    pub const fn elevation(self, state: FabState) -> Points {
        match (self, state) {
            (FabStyle::PrimaryContainer, FabState::Rest) => {
                fab_primary_container_tokens::CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Hovered) => {
                fab_primary_container_tokens::HOVERED_CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Focused) => {
                fab_primary_container_tokens::FOCUSED_CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Pressed) => {
                fab_primary_container_tokens::PRESSED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Rest) => {
                fab_secondary_container_tokens::CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Hovered) => {
                fab_secondary_container_tokens::HOVERED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Focused) => {
                fab_secondary_container_tokens::FOCUSED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Pressed) => {
                fab_secondary_container_tokens::PRESSED_CONTAINER_ELEVATION
            }
        }
    }
}

/// What a size gives the FAB.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's width: androidx's `sizeIn(minWidth = Fab*Tokens.ContainerWidth)`.
    width: Points,
    /// The container's height.
    height: Points,
    /// The icon's side.
    icon: Points,
    /// The container's shape, which no state changes.
    shape: Shape,
}

/// The large FAB's icon, 36 points: androidx `FloatingActionButtonDefaults.LargeIconSize`, which
/// `FloatingActionButton.kt` writes as `36.dp // TODO: FabLargeTokens.IconSize is incorrect` (line 1010), where the
/// token says 32 (ADR 0012).
const LARGE_ICON_SIZE: Points = Points(36.0);

/// The medium FAB's shape: androidx `FloatingActionButtonDefaults.mediumShape`, which is
/// `ShapeDefaults.LargeIncreased` under "TODO: update to use token" (`FloatingActionButton.kt` line 1022). That is
/// `ShapeTokens.CornerLargeIncreased` (`Shapes.kt` line 292, not vendored), and the key that the commented-out
/// `ContainerShape` of `FabMediumTokens.kt` names (ADR 0012).
const MEDIUM_SHAPE: Shape = shape_tokens::CORNER_LARGE_INCREASED;

/// The values of `size`, from its token file, or from the component where it overrides the token.
fn size_spec(size: FabSize) -> SizeSpec {
    match size {
        FabSize::Small => SizeSpec {
            width: fab_small_tokens::CONTAINER_WIDTH,
            height: fab_small_tokens::CONTAINER_HEIGHT,
            icon: fab_small_tokens::ICON_SIZE,
            shape: fab_small_tokens::CONTAINER_SHAPE,
        },
        FabSize::Baseline => SizeSpec {
            width: fab_baseline_tokens::CONTAINER_WIDTH,
            height: fab_baseline_tokens::CONTAINER_HEIGHT,
            icon: fab_baseline_tokens::ICON_SIZE,
            shape: fab_baseline_tokens::CONTAINER_SHAPE,
        },
        FabSize::Medium => SizeSpec {
            width: fab_medium_tokens::CONTAINER_WIDTH,
            height: fab_medium_tokens::CONTAINER_HEIGHT,
            icon: fab_medium_tokens::ICON_SIZE,
            shape: MEDIUM_SHAPE,
        },
        FabSize::Large => SizeSpec {
            width: fab_large_tokens::CONTAINER_WIDTH,
            height: fab_large_tokens::CONTAINER_HEIGHT,
            icon: LARGE_ICON_SIZE,
            shape: fab_large_tokens::CONTAINER_SHAPE,
        },
    }
}

/// What a color style gives the FAB: the role of its container and of its icon.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StyleSpec {
    container: ColorRole,
    /// The icon, and the state layer over the container, which is the icon's color as androidx's ripple is the
    /// content color.
    icon: ColorRole,
}

/// The colors of `style`. A state does not change them: each style's token file gives the same icon color at rest,
/// hovered, focused and pressed, and `every_state_has_the_same_icon_color` would say if that changed.
fn style_spec(style: FabStyle) -> StyleSpec {
    match style {
        FabStyle::PrimaryContainer => StyleSpec {
            container: fab_primary_container_tokens::CONTAINER_COLOR,
            icon: fab_primary_container_tokens::ICON_COLOR,
        },
        FabStyle::SecondaryContainer => StyleSpec {
            container: fab_secondary_container_tokens::CONTAINER_COLOR,
            icon: fab_secondary_container_tokens::ICON_COLOR,
        },
    }
}

/// How far presses also count around a container of `size`, so that a FAB smaller than [`MIN_TOUCH_TARGET`] on a side
/// still has a 48-point target there (androidx `minimumInteractiveComponentSize`), without changing its M3 size or the
/// layout (NSH's accessibility guide: extend the touch area with the hit slop, not the size). Like the icon button's
/// and the button's.
fn touch_slop(size: Size<Points>) -> EdgeInsets {
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

/// `style` with the corners that `shape` gives a container: rounded by the shape's radius, or by half the shorter
/// side.
fn rounded<S: Styled>(style: S, shape: Shape) -> S {
    match corners(shape) {
        Corners::Full => style.rounded_full(),
        Corners::Radius(radius) => style.rounded(radius),
    }
}

/// The props of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct FabProps {
    /// The icon it shows.
    pub icon: Prop<Icon>,
    /// What screen readers say: a FAB shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// Its size.
    #[prop(default)]
    pub size: Prop<FabSize>,
    /// Its color style.
    #[prop(default)]
    pub style: Prop<FabStyle>,
}

/// The events of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum FabEvent {
    /// The FAB was pressed.
    Press,
}

/// A Material 3 floating action button: an icon in a container of the size and color style given.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the button role, named by its label for screen
/// readers) and the look of androidx's `FloatingActionButton`. Hover, focus and press lay the icon's color over it at
/// the state layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so nothing is laid
/// out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::fab::{FabSize, fab};
/// use nsh_m3e::icon::Icon;
///
/// fn add() -> impl IntoElement {
///     fab()
///         // Material Symbols' `add`.
///         .icon(Icon::material_symbol(
///             "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z",
///         ))
///         .label("Add")
///         .size(FabSize::Large)
///         .on_press(|| println!("added"))
/// }
/// ```
#[component]
pub fn Fab(props: FabProps, cx: Scope<FabEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || style_spec(props.style.get());
    let frame = move || {
        let size = size();
        Size::new(size.width, size.height)
    };
    let icon_color = move || role_color(colors().icon);
    // ELEVATION (ADR 0018): the shadow of `props.style.get().elevation(state)` goes on this view, in each state, and
    // nowhere else. The shadow is built in a change of its own: until it lands nothing is drawn here.
    nsh_base::button()
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(FabEvent::Press))
        .hit_slop(move || touch_slop(frame()))
        .group()
        .items_center()
        .justify_center()
        .w(move || frame().width)
        .h(move || frame().height)
        .bg(move || role_color(colors().container))
        .style_with(move |s| rounded(s, size().shape))
        .child(
            // The state layer: the icon's color over the container, in its shape, shown by the button's own states
            // with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
            view()
                .absolute()
                .inset(px(0.0))
                .bg(icon_color)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| rounded(s, size().shape)),
        )
        .child(
            // The icon, centered at its size, in the content color.
            canvas(move || props.icon.get().drawing(size().icon))
                .w(move || size().icon)
                .h(move || size().icon)
                .fill_color(icon_color),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;
    use proptest::prelude::*;

    use super::*;
    use crate::animation_probe::{Recording, layout_items, started_springs};
    use crate::color::ColorScheme;

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ADD: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";

    /// What the probe places: one FAB with these props, and a line counting its presses.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        size: Prop<FabSize>,
        #[prop(default)]
        style: Prop<FabStyle>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        // A row, so that the FAB keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    fab()
                        .icon(Icon::material_symbol(ADD))
                        .label("Add")
                        .size(props.size)
                        .style(props.style)
                        .on_press(move || presses.update(|n| *n += 1)),
                ),
            )
            .child(text(move || format!("presses: {}", presses.get())))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        size: FabSize,
        style: FabStyle,
        dark: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            size: Prop::new(config.size),
            style: Prop::new(config.style),
        });
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
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

    /// The FAB's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole FAB, which is its container.
        button: NodeId,
        /// The view of the state layer.
        layer: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        label: Option<Arc<str>>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut button = None;
            let mut label = None;
            let mut hit_slop = None;
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
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
                        label = props.accessibility.label.clone();
                        hit_slop = Some(props.hit_slop);
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
                    _ => {}
                }
            }
            let button = button.expect("a view with the button role");
            let inside: Vec<NodeId> = children.get(&button).cloned().unwrap_or_default();
            // The state layer, under the icon; and nothing else in the button.
            let [layer, icon] = inside[..] else {
                panic!("two views in the FAB: {items:?}");
            };
            assert_eq!(kinds.get(&layer), Some(&ViewKind::View), "{items:?}");
            assert_eq!(kinds.get(&icon), Some(&ViewKind::Canvas), "{items:?}");
            Self {
                button,
                layer,
                icon,
                styles,
                frames,
                drawings,
                label,
                hit_slop,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// The FAB's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.button].size
        }

        /// How many of `items` lay the FAB out again: one of its views' frames.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.button, self.layer, self.icon];
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Frame { id, .. } if ours.contains(id)))
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

    /// The opaque color of `role` in the baseline scheme.
    fn solid(role: ColorRole, dark: bool) -> Color {
        color(scheme_color(role, dark), 255)
    }

    /// The color `ops` fill with, if any.
    fn fill(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    const STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

    const SIZES: [FabSize; 4] = [
        FabSize::Small,
        FabSize::Baseline,
        FabSize::Medium,
        FabSize::Large,
    ];

    /// Every size: its side, its icon's side and its corners' radius. androidx `FabSmallTokens`, `FabBaselineTokens`,
    /// `FabMediumTokens` and `FabLargeTokens`, and `FloatingActionButtonDefaults` where it overrides them: the large
    /// icon is 36 and the medium shape is `CornerLargeIncreased`.
    const EXPECTED: [(FabSize, f32, f32, f32); 4] = [
        (FabSize::Small, 40.0, 24.0, 12.0),
        (FabSize::Baseline, 56.0, 24.0, 16.0),
        (FabSize::Medium, 80.0, 28.0, 20.0),
        (FabSize::Large, 96.0, 36.0, 28.0),
    ];

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
                let (container, icon) = match style {
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
                assert_eq!(
                    mounted.style(mounted.button).background_color,
                    solid(container, dark),
                    "{what}"
                );
                assert_eq!(
                    fill(&mounted.drawing(mounted.icon).ops),
                    Some(solid(icon, dark)),
                    "{what}"
                );
                // The state layer is the icon's color, and not the container's.
                assert_eq!(
                    mounted.style(mounted.layer).background_color,
                    solid(icon, dark),
                    "{what}"
                );
                assert_ne!(solid(container, dark), solid(icon, dark), "{what}");
            }
        }
    }

    #[test]
    fn each_size_has_its_frame_and_centers_its_icon() {
        for (size, side, icon, _) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let frame = mounted.size();
            assert_eq!((frame.width.0, frame.height.0), (side, side), "{what}");
            // The state layer covers the FAB; the icon is centered at its side.
            let layer = mounted.frames[&mounted.layer];
            assert_eq!((layer.origin.x.0, layer.origin.y.0), (0.0, 0.0), "{what}");
            assert_eq!(layer.size, frame, "{what}");
            let bounds = mounted.frames[&mounted.icon];
            assert_eq!(
                (bounds.size.width.0, bounds.size.height.0),
                (icon, icon),
                "{what}"
            );
            assert_eq!(bounds.origin.x.0, (side - icon) / 2.0, "{what}");
            assert_eq!(bounds.origin.y.0, (side - icon) / 2.0, "{what}");
        }
    }

    #[test]
    fn each_size_has_its_corners_on_the_container_and_the_state_layer() {
        for (size, .., radius) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            for id in [mounted.button, mounted.layer] {
                let style = mounted.style(id);
                assert_eq!(style.border_radius.0, radius, "{what}");
                // No corner of its own: the four are the same.
                assert_eq!(style.corner_radius, nsh_core::Corners::default(), "{what}");
            }
            // The icon's canvas has none.
            assert_eq!(mounted.style(mounted.icon).border_radius.0, 0.0, "{what}");
        }
    }

    #[test]
    fn no_view_has_a_border_a_shadow_or_an_effect_yet() {
        // The elevation's shadow is ADR 0018's and is built in a change of its own: until it lands, nothing draws one,
        // and this test changes with it.
        for (size, ..) in EXPECTED {
            for style in STYLES {
                let mounted = mount(Config {
                    size,
                    style,
                    ..Config::default()
                });
                for id in [mounted.button, mounted.layer, mounted.icon] {
                    let view = mounted.style(id);
                    let what = format!("{size:?} {style:?}");
                    assert_eq!(view.elevation, Points(0.0), "{what}");
                    assert!(view.effects.is_none(), "{what}: {:?}", view.effects);
                    assert_eq!(view.borders.widths.top, px(0.0), "{what}");
                    assert_eq!(view.outline, nsh_core::Outline::default(), "{what}");
                }
            }
        }
    }

    #[test]
    fn every_size_has_a_48_point_touch_target_without_growing() {
        for (size, side, ..) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let slop = mounted.hit_slop.expect("the button's props");
            // Each side reaches 48 points with the slop, and a side already 48 or more gets none.
            let reach = (48.0 - side).max(0.0) / 2.0;
            for around in [slop.left, slop.right, slop.top, slop.bottom] {
                assert_eq!(around.0, reach, "{what}");
            }
            assert!(side + slop.left.0 + slop.right.0 >= 48.0, "{what}");
            assert!(side + slop.top.0 + slop.bottom.0 >= 48.0, "{what}");
            assert_eq!(mounted.size().height.0, side, "{what}");
        }
        // The small FAB is the only one that needs the slop: 4 points on each side.
        let small = mount(Config {
            size: FabSize::Small,
            ..Config::default()
        });
        assert_eq!(small.hit_slop.expect("props").left.0, 4.0);
    }

    proptest! {
        #[test]
        fn the_slop_always_reaches_the_target_and_never_grows_a_big_container(
            width in 0.0_f32..400.0,
            height in 0.0_f32..400.0,
        ) {
            let slop = touch_slop(Size::new(Points(width), Points(height)));
            for around in [slop.left, slop.right, slop.top, slop.bottom] {
                prop_assert!(around.0 >= 0.0);
            }
            prop_assert!(width + slop.left.0 + slop.right.0 >= MIN_TOUCH_TARGET.0 - 1e-3);
            prop_assert!(height + slop.top.0 + slop.bottom.0 >= MIN_TOUCH_TARGET.0 - 1e-3);
            // The slop is symmetric, and only as large as it must be.
            prop_assert_eq!(slop.left.0.to_bits(), slop.right.0.to_bits());
            prop_assert_eq!(slop.top.0.to_bits(), slop.bottom.0.to_bits());
            if width >= MIN_TOUCH_TARGET.0 {
                prop_assert_eq!(slop.left.0.to_bits(), 0.0_f32.to_bits());
            }
            if height >= MIN_TOUCH_TARGET.0 {
                prop_assert_eq!(slop.top.0.to_bits(), 0.0_f32.to_bits());
            }
        }
    }

    #[test]
    fn the_icon_is_its_path_at_the_size_in_the_content_color() {
        for (size, _, icon, _) in EXPECTED {
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let ops = &mounted.drawing(mounted.icon).ops;
            // The Material Symbol's 960-point view box, scaled to the icon's side and moved down by its height.
            let scale = icon / 960.0;
            let constant = |n: &Num| match n {
                Num::Const(v) => Some(*v),
                Num::Slot(_) => None,
            };
            assert!(
                ops.iter().any(|op| matches!(op, CanvasOp::Translate(x, y)
                    if constant(x) == Some(0.0) && constant(y) == Some(icon))),
                "{size:?}: {ops:?}"
            );
            assert!(
                ops.iter().any(|op| matches!(op, CanvasOp::Scale(x, y)
                    if constant(x) == Some(scale) && constant(y) == Some(scale))),
                "{size:?}: {ops:?}"
            );
            let expected = Icon::material_symbol(ADD).drawing(Points(icon)).finish();
            let fills = |ops: &[CanvasOp]| -> Vec<CanvasOp> {
                ops.iter()
                    .filter(|op| matches!(op, CanvasOp::Fill(..)))
                    .cloned()
                    .collect()
            };
            assert_eq!(fills(ops), fills(&expected.ops), "{size:?}");
            assert_eq!(fills(ops).len(), 1, "{size:?}");
        }
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_button() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.label.as_deref(), Some("Add"));
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press("label:Add")?;
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

    /// Nothing of the FAB but its state layer's style changed in `items`: none of its frames, no drawing, and none of
    /// its other views styled again. What else is on screen (the probe's counter) may change.
    fn assert_only_the_layer_changes_in_the_fab(
        mounted: &Mounted,
        items: &[MountItem],
        what: &str,
    ) {
        assert_eq!(mounted.layout_items(items), 0, "{what}");
        assert!(
            items.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } | MountItem::Canvas { id, .. } =>
                    *id == mounted.layer,
                _ => true,
            }),
            "{what}: {items:?}"
        );
    }

    /// Nothing but the state layer's style changed in `items`, anywhere on screen: no frame, no text, no drawing, and
    /// no other view styled again.
    fn assert_only_the_layer_changes(mounted: &Mounted, items: &[MountItem], what: &str) {
        assert_eq!(layout_items(items), 0, "{what}");
        assert_only_the_layer_changes_in_the_fab(mounted, items, what);
    }

    #[test]
    fn hover_shows_the_state_layer_in_the_icon_color_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("label:Add")?;
        let hover = recording.in_step("hover(label:Add)");
        // The state layer at `StateTokens.HoverStateLayerOpacity`, once.
        assert_eq!(layer_opacities(&mounted, &hover), [0.08]);
        assert_only_the_layer_changes(&mounted, &hover, "hover");
        // It is the icon's color, which the layer already had at rest.
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

    /// The step of a press that goes down on the FAB and stays down: a finger that lands and does not move.
    fn press_down(app: &mut Headless) -> String {
        let still = point(px(0.0), px(0.0));
        app.drag_and_hold("label:Add", still).unwrap();
        format!("dragAndHold(label:Add, {}, {})", still.x, still.y)
    }

    #[test]
    fn a_press_changes_nothing_that_is_laid_out_or_drawn_and_starts_no_animation()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let step = press_down(&mut app);
        let down = recording.in_step(&step);
        // The FAB has no pressed shape (androidx's has none): nothing moves, so no spring starts and nothing is laid
        // out or drawn again. The state layer's pressed opacity is the platform's to apply: the headless runner
        // sends the press to the FAB's handlers, not to its styles.
        assert_eq!(started_springs(&down), Vec::new());
        assert_only_the_layer_changes(&mounted, &down, "down");
        assert!(app.shows("presses: 0"));
        app.drop_held()?;
        let up = recording.in_step("drop()");
        // The release reports the press, so the probe's counter text is laid out again: only the FAB itself is held
        // to nothing.
        assert_only_the_layer_changes_in_the_fab(&mounted, &up, "up");
        assert_eq!(started_springs(&up), Vec::new());
        assert!(app.shows("presses: 1"));
        assert_eq!(Mounted::of(&recording.items()).size(), mounted.size());
        Ok(())
    }

    #[test]
    fn the_button_asks_the_platform_for_its_hover_and_press_so_that_its_state_layer_shows_them() {
        // The state layer's `group_hover` and `group_active` are resolved by the platform, which the button's group
        // tells to report both.
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

    #[test]
    fn the_state_layer_opacities_are_the_state_tokens() {
        // androidx `StateTokens`: hover 0.08, focus 0.1, pressed 0.1.
        let bits = |opacity: f32| opacity.to_bits();
        assert_eq!(bits(state_tokens::HOVER_STATE_LAYER_OPACITY), bits(0.08));
        assert_eq!(bits(state_tokens::FOCUS_STATE_LAYER_OPACITY), bits(0.1));
        assert_eq!(bits(state_tokens::PRESSED_STATE_LAYER_OPACITY), bits(0.1));
    }

    #[test]
    fn focus_shows_the_state_layer_at_the_focus_opacity() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("Tab")?;
        let focus = recording.in_step("keyboard.press(Tab)");
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &focus), [0.1]);
        assert_only_the_layer_changes(&mounted, &focus, "focus");
        Ok(())
    }

    #[test]
    fn mounting_starts_nothing() {
        for (size, ..) in EXPECTED {
            let (_app, recording) = start(Config {
                size,
                ..Config::default()
            });
            assert_eq!(started_springs(&recording.items()), Vec::new(), "{size:?}");
        }
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        // androidx `FloatingActionButton` is the 56-point one, with `FloatingActionButtonDefaults.containerColor`.
        assert_eq!(FabSize::default(), FabSize::Baseline);
        assert_eq!(FabStyle::default(), FabStyle::PrimaryContainer);
        assert_eq!(FabState::default(), FabState::Rest);
        assert_eq!(size_spec(FabSize::default()).width, Points(56.0));
        assert_eq!(
            style_spec(FabStyle::default()).container,
            ColorRole::PrimaryContainer
        );
    }

    #[test]
    fn the_container_is_as_wide_as_it_is_tall_at_every_size() {
        // androidx gives the width and the height of each size their own token: they are the same today.
        for size in SIZES {
            let spec = size_spec(size);
            assert_eq!(spec.width, spec.height, "{size:?}");
        }
    }

    /// `FabStyle::elevation` of each state: level 3 at rest, focused and pressed, level 4 hovered. androidx
    /// `ElevationTokens`: `Level3` is 6 and `Level4` is 8.
    const ELEVATIONS: [(FabState, f32); 4] = [
        (FabState::Rest, 6.0),
        (FabState::Hovered, 8.0),
        (FabState::Focused, 6.0),
        (FabState::Pressed, 6.0),
    ];

    #[test]
    fn each_style_resolves_the_elevation_of_each_state() {
        for style in STYLES {
            for (state, points) in ELEVATIONS {
                assert_eq!(
                    style.elevation(state),
                    Points(points),
                    "{style:?} {state:?}"
                );
            }
        }
        // Only hovering lifts it.
        for style in STYLES {
            for state in [FabState::Rest, FabState::Focused, FabState::Pressed] {
                assert!(style.elevation(FabState::Hovered) > style.elevation(state));
            }
        }
    }

    #[test]
    fn every_state_has_the_same_icon_color() {
        // The component draws one icon color per style, and the state layer in it. If a state ever has another one
        // in the tokens, the component has to show it.
        let primary = [
            fab_primary_container_tokens::ICON_COLOR,
            fab_primary_container_tokens::FOCUSED_ICON_COLOR,
            fab_primary_container_tokens::HOVERED_ICON_COLOR,
            fab_primary_container_tokens::PRESSED_ICON_COLOR,
        ];
        let secondary = [
            fab_secondary_container_tokens::ICON_COLOR,
            fab_secondary_container_tokens::FOCUSED_ICON_COLOR,
            fab_secondary_container_tokens::HOVERED_ICON_COLOR,
            fab_secondary_container_tokens::PRESSED_ICON_COLOR,
        ];
        assert!(
            primary
                .iter()
                .all(|role| *role == ColorRole::OnPrimaryContainer)
        );
        assert!(
            secondary
                .iter()
                .all(|role| *role == ColorRole::OnSecondaryContainer)
        );
    }

    /// Views of one shape each, to read the corners `rounded` gives.
    #[component]
    fn Medium() -> impl IntoElement {
        rounded(view().size(px(40.0)), shape_tokens::CORNER_MEDIUM)
    }

    #[component]
    fn ExtraLarge() -> impl IntoElement {
        rounded(view().size(px(96.0)), shape_tokens::CORNER_EXTRA_LARGE)
    }

    #[component]
    fn Full() -> impl IntoElement {
        rounded(view().size(px(40.0)), shape_tokens::CORNER_FULL)
    }

    /// The largest corner radius of the views `root` mounts.
    fn radius_of(root: Root) -> f32 {
        let mut app = Headless::start(EngineConfig::new(root)).unwrap();
        let recording = Recording::attach(&mut app);
        app.first_render().unwrap();
        recording
            .items()
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { style, .. } => Some(style.border_radius.0),
                _ => None,
            })
            .fold(0.0, f32::max)
    }

    #[test]
    fn a_shape_gives_its_corners_to_a_view() {
        assert_eq!(radius_of(Root::of::<Medium>()), 12.0);
        assert_eq!(radius_of(Root::of::<ExtraLarge>()), 28.0);
        // A pill or a circle takes as much as any container can round.
        assert!(radius_of(Root::of::<Full>()) >= 1000.0);
    }

    /// The vendored `FloatingActionButton.kt`, at the pinned commit (ADR 0012).
    const FLOATING_ACTION_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/FloatingActionButton.kt");
    /// The vendored `FabMediumTokens.kt`, whose `ContainerShape` androidx comments out.
    const FAB_MEDIUM_TOKENS_KT: &str =
        include_str!("../../../reference/androidx-material3-tokens/FabMediumTokens.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_icon_sizes_are_androidxs_and_the_large_one_is_its_override() {
        // The large icon: the literal, and the token it disagrees with.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val LargeIconSize: Dp = 36.dp // TODO: FabLargeTokens.IconSize is incorrect"]
        ));
        assert_eq!(LARGE_ICON_SIZE, Points(36.0));
        assert_ne!(LARGE_ICON_SIZE, fab_large_tokens::ICON_SIZE);
        assert_eq!(fab_large_tokens::ICON_SIZE, Points(32.0));
        // The medium icon is its token.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val MediumIconSize: Dp = FabMediumTokens.IconSize"]
        ));
        assert_eq!(
            size_spec(FabSize::Medium).icon,
            fab_medium_tokens::ICON_SIZE
        );
        assert_eq!(size_spec(FabSize::Large).icon, LARGE_ICON_SIZE);
        // A line that is not there is not found.
        assert!(!cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val LargeIconSize: Dp = FabLargeTokens.IconSize"]
        ));
    }

    #[test]
    fn the_shapes_are_androidxs_and_the_medium_one_is_its_override() {
        // The medium shape: `ShapeDefaults.LargeIncreased`, and the key the commented-out token names.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "public val mediumShape: Shape",
                "@Composable get() = ShapeDefaults.LargeIncreased // TODO: update to use token",
            ]
        ));
        assert!(cites(
            FAB_MEDIUM_TOKENS_KT,
            &["// val ContainerShape = ShapeKeyTokens.CornerLargeIncreased"]
        ));
        assert_eq!(MEDIUM_SHAPE, shape_tokens::CORNER_LARGE_INCREASED);
        assert_eq!(
            Corners::of(MEDIUM_SHAPE),
            Some(Corners::Radius(Points(20.0)))
        );
        // The other sizes' shapes are their tokens.
        for (name, size, token, shape) in [
            (
                "smallShape",
                FabSize::Small,
                "FabSmallTokens.ContainerShape.value",
                fab_small_tokens::CONTAINER_SHAPE,
            ),
            (
                "shape",
                FabSize::Baseline,
                "FabBaselineTokens.ContainerShape.value",
                fab_baseline_tokens::CONTAINER_SHAPE,
            ),
            (
                "largeShape",
                FabSize::Large,
                "FabLargeTokens.ContainerShape.value",
                fab_large_tokens::CONTAINER_SHAPE,
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
        assert_eq!(size_spec(FabSize::Medium).shape, MEDIUM_SHAPE);
    }

    #[test]
    fn the_default_style_is_androidxs_with_its_elevations() {
        // The default container color is the primary container's.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "public val containerColor: Color",
                "@Composable get() = FabPrimaryContainerTokens.ContainerColor.value",
            ]
        ));
        // The elevation of every state is read from the primary container's tokens by default.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "defaultElevation: Dp = FabPrimaryContainerTokens.ContainerElevation,",
                "pressedElevation: Dp = FabPrimaryContainerTokens.PressedContainerElevation,",
                "focusedElevation: Dp = FabPrimaryContainerTokens.FocusedContainerElevation,",
                "hoveredElevation: Dp = FabPrimaryContainerTokens.HoveredContainerElevation,",
            ]
        ));
        assert_eq!(
            style_spec(FabStyle::default()).container,
            fab_primary_container_tokens::CONTAINER_COLOR
        );
    }

    #[test]
    fn each_size_is_its_tokens_width_and_height() {
        // Each size is a minimum of its tokens' width and height.
        for (token, size, width, height) in [
            (
                "FabSmallTokens",
                FabSize::Small,
                fab_small_tokens::CONTAINER_WIDTH,
                fab_small_tokens::CONTAINER_HEIGHT,
            ),
            (
                "FabMediumTokens",
                FabSize::Medium,
                fab_medium_tokens::CONTAINER_WIDTH,
                fab_medium_tokens::CONTAINER_HEIGHT,
            ),
            (
                "FabLargeTokens",
                FabSize::Large,
                fab_large_tokens::CONTAINER_WIDTH,
                fab_large_tokens::CONTAINER_HEIGHT,
            ),
        ] {
            let min_width = format!("minWidth = {token}.ContainerWidth,");
            let min_height = format!("minHeight = {token}.ContainerHeight,");
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&min_width, &min_height]),
                "{token}"
            );
            let spec = size_spec(size);
            assert_eq!((spec.width, spec.height), (width, height), "{token}");
        }
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "FabBaselineTokens.ContainerWidth,",
                "FabBaselineTokens.ContainerHeight,",
            ]
        ));
        let baseline = size_spec(FabSize::Baseline);
        assert_eq!(
            (baseline.width, baseline.height),
            (
                fab_baseline_tokens::CONTAINER_WIDTH,
                fab_baseline_tokens::CONTAINER_HEIGHT
            )
        );
    }
}
