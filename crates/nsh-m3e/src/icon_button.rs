//! The Material 3 Expressive icon button: an [`Icon`] in a container of five sizes, three widths, two shapes and four
//! color styles, the state layer of hover, focus and press, and the shape it morphs into while pressed.
//!
//! The icon button follows androidx's Expressive `IconButton(onClick, shapes, …)` and its siblings
//! (`FilledIconButton`, `FilledTonalIconButton`, `OutlinedIconButton`) in `IconButton.kt` and `IconButtonDefaults.kt`,
//! at the pinned commit, as [ADR 0012] says. Its container is drawn by a canvas whose corner radius the M3E spring
//! moves, as the button's is ([ADR 0011]), and its icon is drawn from path data ([ADR 0017]).
//!
//! androidx's default colors for the standard and outlined icon buttons are the content color of the component
//! around them (`LocalContentColor`), which NSH does not have. This icon button takes the colors androidx names
//! "vibrant", the ones of the token files: `iconButtonVibrantColors`, `outlinedIconButtonVibrantColors` and
//! `outlinedIconButtonVibrantBorder`. The filled and tonal ones have only those.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::icon::Icon;
use crate::morph::{DrawnShape, corners};
use crate::tint::{MIN_TOUCH_TARGET, StyleSpec, Tint, tint_color};
use crate::tokens::Shape;
use crate::tokens::androidx::{
    filled_icon_button_tokens, filled_tonal_icon_button_tokens, large_icon_button_tokens,
    medium_icon_button_tokens, outlined_icon_button_tokens, small_icon_button_tokens,
    standard_icon_button_tokens, state_tokens, xlarge_icon_button_tokens,
    xsmall_icon_button_tokens,
};

/// The size of an [`IconButton`], from extra small (32 points tall, a 20-point icon) to extra large (136 points tall,
/// a 40-point icon).
///
/// Each size takes its height, icon size, side spaces and shapes from its androidx token file
/// (`XSmallIconButtonTokens` to `XLargeIconButtonTokens`), as androidx's `IconButtonDefaults.*ContainerSize`,
/// `*IconSize` and `*Shape` give them.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum IconButtonSize {
    /// 32 points tall, a 20-point icon.
    ExtraSmall,
    /// 40 points tall, a 24-point icon: androidx's default icon button.
    #[default]
    Small,
    /// 56 points tall, a 24-point icon.
    Medium,
    /// 96 points tall, a 32-point icon.
    Large,
    /// 136 points tall, a 40-point icon.
    ExtraLarge,
}

/// The width of an [`IconButton`]: the space on each side of its icon (androidx `IconButtonWidthOption`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum IconButtonWidth {
    /// The size's narrow spaces.
    Narrow,
    /// The size's default spaces: androidx's default.
    #[default]
    Uniform,
    /// The size's wide spaces.
    Wide,
}

/// The color style of an [`IconButton`], from the least emphasis to the most.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum IconButtonStyle {
    /// No container and no outline: androidx `IconButton`, its default.
    #[default]
    Standard,
    /// No container, an outline: androidx `OutlinedIconButton`.
    Outlined,
    /// A container in the secondary container color: androidx `FilledTonalIconButton`.
    Tonal,
    /// A container in the primary color: androidx `FilledIconButton`.
    Filled,
}

/// The shape of an [`IconButton`] at rest.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum IconButtonShape {
    /// Fully rounded: the size's `ContainerShapeRound`.
    #[default]
    Round,
    /// Rounded corners: the size's `ContainerShapeSquare`.
    Square,
}

/// What a size gives the icon button.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's height.
    height: Points,
    /// The icon's side.
    icon: Points,
    /// The space before and after the icon for each width.
    narrow: (Points, Points),
    uniform: (Points, Points),
    wide: (Points, Points),
    /// The shape at rest when round.
    round: Shape,
    /// The shape at rest when square.
    square: Shape,
    /// The shape while pressed, round or square: androidx `IconButtonDefaults.*PressedShape`.
    pressed: Shape,
}

impl SizeSpec {
    /// The container's width at `width`: the icon and the spaces on each side of it, as androidx
    /// `IconButtonDefaults.*ContainerSize(widthOption)` adds them.
    fn width(&self, width: IconButtonWidth) -> Points {
        let (leading, trailing) = match width {
            IconButtonWidth::Narrow => self.narrow,
            IconButtonWidth::Uniform => self.uniform,
            IconButtonWidth::Wide => self.wide,
        };
        Points(self.icon.0 + leading.0 + trailing.0)
    }
}

/// The values of `size`, from its token file.
///
/// For the uniform width, androidx's `*ContainerSize` adds the leading space twice (`DefaultLeadingSpace +
/// DefaultLeadingSpace`, `UniformLeadingSpace + UniformLeadingSpace` for the large size), so this does too. Every
/// size's leading and trailing spaces are the same in the tokens; `uniform_width_adds_the_leading_space_twice` would
/// say if that changed.
fn size_spec(size: IconButtonSize) -> SizeSpec {
    match size {
        IconButtonSize::ExtraSmall => SizeSpec {
            height: xsmall_icon_button_tokens::CONTAINER_HEIGHT,
            icon: xsmall_icon_button_tokens::ICON_SIZE,
            narrow: (
                xsmall_icon_button_tokens::NARROW_LEADING_SPACE,
                xsmall_icon_button_tokens::NARROW_TRAILING_SPACE,
            ),
            uniform: (
                xsmall_icon_button_tokens::DEFAULT_LEADING_SPACE,
                xsmall_icon_button_tokens::DEFAULT_LEADING_SPACE,
            ),
            wide: (
                xsmall_icon_button_tokens::WIDE_LEADING_SPACE,
                xsmall_icon_button_tokens::WIDE_TRAILING_SPACE,
            ),
            round: xsmall_icon_button_tokens::CONTAINER_SHAPE_ROUND,
            square: xsmall_icon_button_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: xsmall_icon_button_tokens::PRESSED_CONTAINER_SHAPE,
        },
        IconButtonSize::Small => SizeSpec {
            height: small_icon_button_tokens::CONTAINER_HEIGHT,
            icon: small_icon_button_tokens::ICON_SIZE,
            narrow: (
                small_icon_button_tokens::NARROW_LEADING_SPACE,
                small_icon_button_tokens::NARROW_TRAILING_SPACE,
            ),
            uniform: (
                small_icon_button_tokens::DEFAULT_LEADING_SPACE,
                small_icon_button_tokens::DEFAULT_LEADING_SPACE,
            ),
            wide: (
                small_icon_button_tokens::WIDE_LEADING_SPACE,
                small_icon_button_tokens::WIDE_TRAILING_SPACE,
            ),
            round: small_icon_button_tokens::CONTAINER_SHAPE_ROUND,
            square: small_icon_button_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: small_icon_button_tokens::PRESSED_CONTAINER_SHAPE,
        },
        IconButtonSize::Medium => SizeSpec {
            height: medium_icon_button_tokens::CONTAINER_HEIGHT,
            icon: medium_icon_button_tokens::ICON_SIZE,
            narrow: (
                medium_icon_button_tokens::NARROW_LEADING_SPACE,
                medium_icon_button_tokens::NARROW_TRAILING_SPACE,
            ),
            uniform: (
                medium_icon_button_tokens::DEFAULT_LEADING_SPACE,
                medium_icon_button_tokens::DEFAULT_LEADING_SPACE,
            ),
            wide: (
                medium_icon_button_tokens::WIDE_LEADING_SPACE,
                medium_icon_button_tokens::WIDE_TRAILING_SPACE,
            ),
            round: medium_icon_button_tokens::CONTAINER_SHAPE_ROUND,
            square: medium_icon_button_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: medium_icon_button_tokens::PRESSED_CONTAINER_SHAPE,
        },
        IconButtonSize::Large => SizeSpec {
            height: large_icon_button_tokens::CONTAINER_HEIGHT,
            icon: large_icon_button_tokens::ICON_SIZE,
            narrow: (
                large_icon_button_tokens::NARROW_LEADING_SPACE,
                large_icon_button_tokens::NARROW_TRAILING_SPACE,
            ),
            uniform: (
                large_icon_button_tokens::UNIFORM_LEADING_SPACE,
                large_icon_button_tokens::UNIFORM_LEADING_SPACE,
            ),
            wide: (
                large_icon_button_tokens::WIDE_LEADING_SPACE,
                large_icon_button_tokens::WIDE_TRAILING_SPACE,
            ),
            round: large_icon_button_tokens::CONTAINER_SHAPE_ROUND,
            square: large_icon_button_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: large_icon_button_tokens::PRESSED_CONTAINER_SHAPE,
        },
        IconButtonSize::ExtraLarge => SizeSpec {
            height: xlarge_icon_button_tokens::CONTAINER_HEIGHT,
            icon: xlarge_icon_button_tokens::ICON_SIZE,
            narrow: (
                xlarge_icon_button_tokens::NARROW_LEADING_SPACE,
                xlarge_icon_button_tokens::NARROW_TRAILING_SPACE,
            ),
            uniform: (
                xlarge_icon_button_tokens::DEFAULT_LEADING_SPACE,
                xlarge_icon_button_tokens::DEFAULT_LEADING_SPACE,
            ),
            wide: (
                xlarge_icon_button_tokens::WIDE_LEADING_SPACE,
                xlarge_icon_button_tokens::WIDE_TRAILING_SPACE,
            ),
            round: xlarge_icon_button_tokens::CONTAINER_SHAPE_ROUND,
            square: xlarge_icon_button_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: xlarge_icon_button_tokens::PRESSED_CONTAINER_SHAPE,
        },
    }
}

/// The colors of `style`, as androidx `IconButtonDefaults` builds its vibrant ones: `defaultIconButtonVibrantColors`,
/// `defaultOutlinedIconButtonVibrantColors` with `outlinedIconButtonVibrantBorder`, `filledTonalIconButtonColors` and
/// `filledIconButtonColors`.
fn style_spec(style: IconButtonStyle) -> StyleSpec {
    match style {
        IconButtonStyle::Standard => StyleSpec {
            container: None,
            content: Tint::solid(standard_icon_button_tokens::COLOR),
            disabled_container: None,
            disabled_content: Tint::faded(
                standard_icon_button_tokens::DISABLED_COLOR,
                standard_icon_button_tokens::DISABLED_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        IconButtonStyle::Outlined => StyleSpec {
            container: None,
            content: Tint::solid(outlined_icon_button_tokens::COLOR),
            disabled_container: None,
            disabled_content: Tint::faded(
                outlined_icon_button_tokens::DISABLED_COLOR,
                outlined_icon_button_tokens::DISABLED_OPACITY,
            ),
            // `outlinedIconButtonVibrantBorder` fades the outline color by the icon's disabled opacity, where
            // `OutlinedIconButtonTokens.DisabledOutlineColor` names the same role with no opacity (ADR 0012).
            outline: Some(Tint::solid(outlined_icon_button_tokens::OUTLINE_COLOR)),
            disabled_outline: Some(Tint::faded(
                outlined_icon_button_tokens::OUTLINE_COLOR,
                outlined_icon_button_tokens::DISABLED_OPACITY,
            )),
        },
        IconButtonStyle::Tonal => StyleSpec {
            container: Some(Tint::solid(
                filled_tonal_icon_button_tokens::CONTAINER_COLOR,
            )),
            content: Tint::solid(filled_tonal_icon_button_tokens::COLOR),
            disabled_container: Some(Tint::faded(
                filled_tonal_icon_button_tokens::DISABLED_CONTAINER_COLOR,
                filled_tonal_icon_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
            disabled_content: Tint::faded(
                filled_tonal_icon_button_tokens::DISABLED_COLOR,
                filled_tonal_icon_button_tokens::DISABLED_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        IconButtonStyle::Filled => StyleSpec {
            container: Some(Tint::solid(filled_icon_button_tokens::CONTAINER_COLOR)),
            content: Tint::solid(filled_icon_button_tokens::COLOR),
            disabled_container: Some(Tint::faded(
                filled_icon_button_tokens::DISABLED_CONTAINER_COLOR,
                filled_icon_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
            disabled_content: Tint::faded(
                filled_icon_button_tokens::DISABLED_COLOR,
                filled_icon_button_tokens::DISABLED_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
    }
}

/// The outline's width at every size: androidx `outlinedIconButtonVibrantBorder` reads
/// `SmallIconButtonTokens.OutlinedOutlineWidth` whatever the size, where the large and extra large token files say 2
/// and 3 (ADR 0012).
const OUTLINE_WIDTH: Points = small_icon_button_tokens::OUTLINED_OUTLINE_WIDTH;

/// How far presses also count around a container of `size`, so that an icon button smaller than [`MIN_TOUCH_TARGET`]
/// on a side still has a 48-point target there (androidx `minimumInteractiveComponentSize`), without changing its M3
/// size or the layout (NSH's accessibility guide: extend the touch area with the hit slop, not the size).
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

/// The props of an [`IconButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct IconButtonProps {
    /// The icon it shows.
    pub icon: Prop<Icon>,
    /// What screen readers say: an icon button shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// Its size.
    #[prop(default)]
    pub size: Prop<IconButtonSize>,
    /// Its width.
    #[prop(default)]
    pub width: Prop<IconButtonWidth>,
    /// Its color style.
    #[prop(default)]
    pub style: Prop<IconButtonStyle>,
    /// Its shape at rest.
    #[prop(default)]
    pub shape: Prop<IconButtonShape>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of an [`IconButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum IconButtonEvent {
    /// The icon button was pressed.
    Press,
}

/// A Material 3 Expressive icon button: an icon in a container of the size, width, shape and color style given.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, the button role, named by its
/// label for screen readers) and the look of androidx's Expressive icon button. Hover, focus and press lay the icon's
/// color over it at the state layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else,
/// so nothing is laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::icon::Icon;
/// use nsh_m3e::icon_button::{IconButtonStyle, icon_button};
///
/// fn add() -> impl IntoElement {
///     icon_button()
///         // Material Symbols' `add`.
///         .icon(Icon::material_symbol(
///             "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z",
///         ))
///         .label("Add")
///         .style(IconButtonStyle::Filled)
///         .on_press(|| println!("added"))
/// }
/// ```
#[component]
pub fn IconButton(props: IconButtonProps, cx: Scope<IconButtonEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || style_spec(props.style.get());
    let frame = move || {
        let size = size();
        Size::new(size.width(props.width.get()), size.height)
    };
    let shape = move || match props.shape.get() {
        IconButtonShape::Round => size().round,
        IconButtonShape::Square => size().square,
    };
    let content = move || {
        let colors = colors();
        tint_color(Some(if props.disabled.get() {
            colors.disabled_content
        } else {
            colors.content
        }))
    };
    let drawn = untracked(|| DrawnShape::new(corners(shape()), corners(size().pressed), frame()));
    effect(move || drawn.set_corners(corners(shape()), corners(size().pressed)));
    effect(move || {
        // An icon button disabled while pressed may get no end of its press: it goes back to its shape at rest.
        if props.disabled.get() {
            drawn.press(false);
        }
    });
    let outline = move || colors().outline.map(|_| OUTLINE_WIDTH);
    nsh_base::button()
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(IconButtonEvent::Press))
        .on_press_down(move || drawn.press(true))
        .on_press_up(move || drawn.press(false))
        .hit_slop(move || touch_slop(frame()))
        .group()
        .items_center()
        .justify_center()
        .w(move || frame().width)
        .h(move || frame().height)
        .child(
            // The container: its color, and its outline drawn inside its edge as a view's border is.
            drawn
                .container(outline)
                .bg(move || {
                    let colors = colors();
                    tint_color(if props.disabled.get() {
                        colors.disabled_container
                    } else {
                        colors.container
                    })
                })
                .stroke_color(move || {
                    let colors = colors();
                    tint_color(if props.disabled.get() {
                        colors.disabled_outline
                    } else {
                        colors.outline
                    })
                }),
        )
        .child(
            // The state layer: the icon's color over the container inside its outline, in its shape, shown by the
            // button's own states with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the
            // content color).
            drawn
                .state_layer(outline)
                .bg(content)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| {
                    // A disabled icon button takes no press and shows no state.
                    if props.disabled.get() { s.hidden() } else { s }
                }),
        )
        .child(
            // The icon, centered at its token size, in the content color.
            canvas(move || props.icon.get().drawing(size().icon))
                .w(move || size().icon)
                .h(move || size().icon)
                .fill_color(content),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, StartedSpring, layout_items, spring_position, started_springs,
        worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph::Corners;
    use crate::morph_probe::{
        Step, assert_traces, parameters, paths, replay, rounded_rectangle, set_values, spring_in,
        tolerance,
    };

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ADD: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";

    /// What the probe places: one icon button with these props, a line counting its presses, and two lines that
    /// change its shape and disable it when pressed.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        size: Prop<IconButtonSize>,
        #[prop(default)]
        width: Prop<IconButtonWidth>,
        #[prop(default)]
        style: Prop<IconButtonStyle>,
        #[prop(default)]
        shape: Prop<IconButtonShape>,
        #[prop(default)]
        disabled: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        let shape = signal(props.shape.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        // A row, so that the icon button keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    icon_button()
                        .icon(Icon::material_symbol(ADD))
                        .label("Add")
                        .size(props.size)
                        .width(props.width)
                        .style(props.style)
                        .shape(move || shape.get())
                        .disabled(move || disabled.get())
                        .on_press(move || presses.update(|n| *n += 1)),
                ),
            )
            .child(text(move || format!("presses: {}", presses.get())))
            .child(text("Reshape").on_press(move |(): &()| {
                shape.update(|shape| {
                    *shape = match *shape {
                        IconButtonShape::Round => IconButtonShape::Square,
                        _ => IconButtonShape::Round,
                    };
                });
            }))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        size: IconButtonSize,
        width: IconButtonWidth,
        style: IconButtonStyle,
        shape: IconButtonShape,
        disabled: bool,
        dark: bool,
        reduce_motion: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            size: Prop::new(config.size),
            width: Prop::new(config.width),
            style: Prop::new(config.style),
            shape: Prop::new(config.shape),
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

    /// The icon button's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole icon button.
        button: NodeId,
        /// The canvas that draws the container.
        container: NodeId,
        /// The canvas that draws the state layer.
        layer: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        label: Option<Arc<str>>,
        disabled: Option<bool>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut button = None;
            let mut label = None;
            let mut disabled = None;
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
                        disabled = Some(props.accessibility.state.disabled);
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
            let canvases: Vec<NodeId> = children
                .get(&button)
                .into_iter()
                .flatten()
                .filter(|child| kinds.get(child) == Some(&ViewKind::Canvas))
                .copied()
                .collect();
            // The container first, under the state layer, under the icon; and nothing else in the button.
            let [container, layer, icon] = canvases[..] else {
                panic!("three canvases in the icon button: {items:?}");
            };
            assert_eq!(children.get(&button).map(Vec::len), Some(3), "{items:?}");
            Self {
                button,
                container,
                layer,
                icon,
                styles,
                frames,
                drawings,
                label,
                disabled,
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

        /// The icon button's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.button].size
        }

        /// How many of `items` lay the icon button out again: one of its views' frames.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.button, self.container, self.layer, self.icon];
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

    /// The 8-bit alphas NSH rounds the token opacities to: 0.38 of 255 and 0.1 of 255.
    const ALPHA_38: u32 = 97;
    const ALPHA_10: u32 = 26;

    /// The color `ops` fill with, or that they stroke with, if any.
    fn paint(ops: &[CanvasOp], stroke: bool) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) if !stroke => Some(*color),
            CanvasOp::StrokeStyle(canvas::Paint::Color(color)) if stroke => Some(*color),
            _ => None,
        })
    }

    /// The container's color, its outline's width and color (0 and transparent with no outline), and the icon's
    /// color, as their canvases paint them (transparent when nothing fills the container).
    fn colors(mounted: &Mounted) -> (Color, f32, Color, Color) {
        let ops = &mounted.drawing(mounted.container).ops;
        let (width, outline) = if ops.iter().any(|op| matches!(op, CanvasOp::Stroke(None))) {
            let width = ops.iter().find_map(|op| match op {
                CanvasOp::LineWidth(Num::Const(width)) => Some(*width),
                _ => None,
            });
            (
                width.expect("a line width"),
                paint(ops, true).expect("a stroke color"),
            )
        } else {
            (0.0, Color(0))
        };
        let icon = paint(&mounted.drawing(mounted.icon).ops, false).expect("the icon's fill");
        (paint(ops, false).unwrap_or(Color(0)), width, outline, icon)
    }

    const STYLES: [IconButtonStyle; 4] = [
        IconButtonStyle::Standard,
        IconButtonStyle::Outlined,
        IconButtonStyle::Tonal,
        IconButtonStyle::Filled,
    ];

    /// The colors each style should paint, at rest and disabled, in the baseline schemes: the container, the outline's
    /// width and color, and the icon.
    fn expected_colors(
        style: IconButtonStyle,
        disabled: bool,
        dark: bool,
    ) -> (Color, f32, Color, Color) {
        let solid = |role| color(scheme_color(role, dark), 255);
        let faded = |role, alpha| color(scheme_color(role, dark), alpha);
        let none = Color(0);
        match (style, disabled) {
            // androidx `StandardIconButtonTokens`: on surface variant; on surface at 38 % disabled.
            (IconButtonStyle::Standard, false) => {
                (none, 0.0, none, solid(ColorRole::OnSurfaceVariant))
            }
            (IconButtonStyle::Standard, true) => {
                (none, 0.0, none, faded(ColorRole::OnSurface, ALPHA_38))
            }
            // androidx `OutlinedIconButtonTokens` and `outlinedIconButtonVibrantBorder`: an outline variant outline,
            // 1 point wide, faded to 38 % disabled.
            (IconButtonStyle::Outlined, false) => (
                none,
                1.0,
                solid(ColorRole::OutlineVariant),
                solid(ColorRole::OnSurfaceVariant),
            ),
            (IconButtonStyle::Outlined, true) => (
                none,
                1.0,
                faded(ColorRole::OutlineVariant, ALPHA_38),
                faded(ColorRole::OnSurface, ALPHA_38),
            ),
            // androidx `FilledTonalIconButtonTokens`: secondary container, on secondary container.
            (IconButtonStyle::Tonal, false) => (
                solid(ColorRole::SecondaryContainer),
                0.0,
                none,
                solid(ColorRole::OnSecondaryContainer),
            ),
            // androidx `FilledIconButtonTokens`: primary, on primary.
            (IconButtonStyle::Filled, false) => (
                solid(ColorRole::Primary),
                0.0,
                none,
                solid(ColorRole::OnPrimary),
            ),
            // Both `FilledTonalIconButtonTokens` and `FilledIconButtonTokens`, disabled: on surface at 10 % and 38 %.
            (IconButtonStyle::Tonal | IconButtonStyle::Filled, true) => (
                faded(ColorRole::OnSurface, ALPHA_10),
                0.0,
                none,
                faded(ColorRole::OnSurface, ALPHA_38),
            ),
        }
    }

    #[test]
    fn each_style_has_its_colors_enabled_and_disabled_light_and_dark() {
        for style in STYLES {
            for disabled in [false, true] {
                for dark in [false, true] {
                    let what = format!("{style:?} disabled: {disabled} dark: {dark}");
                    let mounted = mount(Config {
                        style,
                        disabled,
                        dark,
                        ..Config::default()
                    });
                    assert_eq!(
                        colors(&mounted),
                        expected_colors(style, disabled, dark),
                        "{what}"
                    );
                    // No view paints a background or a border of its own under the canvases.
                    for id in [
                        mounted.button,
                        mounted.container,
                        mounted.layer,
                        mounted.icon,
                    ] {
                        let style = mounted.style(id);
                        assert_eq!(style.background_color, Color::TRANSPARENT, "{what}");
                        assert_eq!(style.borders.widths.top, px(0.0), "{what}");
                    }
                }
            }
        }
    }

    /// Every size: its height, its icon's side, its spaces at each width (narrow, uniform, wide), and its square and
    /// pressed corners. androidx `XSmallIconButtonTokens` to `XLargeIconButtonTokens`; the uniform width adds the
    /// leading space twice, which equals the trailing one at every size.
    const SIZES: [(IconButtonSize, f32, f32, [f32; 3], f32, f32); 5] = [
        (
            IconButtonSize::ExtraSmall,
            32.0,
            20.0,
            [4.0, 6.0, 10.0],
            12.0,
            8.0,
        ),
        (
            IconButtonSize::Small,
            40.0,
            24.0,
            [4.0, 8.0, 14.0],
            12.0,
            8.0,
        ),
        (
            IconButtonSize::Medium,
            56.0,
            24.0,
            [12.0, 16.0, 24.0],
            16.0,
            12.0,
        ),
        (
            IconButtonSize::Large,
            96.0,
            32.0,
            [16.0, 32.0, 48.0],
            28.0,
            16.0,
        ),
        (
            IconButtonSize::ExtraLarge,
            136.0,
            40.0,
            [32.0, 48.0, 72.0],
            28.0,
            16.0,
        ),
    ];

    const WIDTHS: [IconButtonWidth; 3] = [
        IconButtonWidth::Narrow,
        IconButtonWidth::Uniform,
        IconButtonWidth::Wide,
    ];

    #[test]
    fn each_size_and_width_has_its_frame_and_centers_its_icon() {
        for (size, height, icon, spaces, ..) in SIZES {
            for (width, space) in WIDTHS.into_iter().zip(spaces) {
                let what = format!("{size:?} {width:?}");
                let mounted = mount(Config {
                    size,
                    width,
                    ..Config::default()
                });
                let frame = mounted.size();
                let expected = icon + 2.0 * space;
                assert_eq!(
                    (frame.width.0, frame.height.0),
                    (expected, height),
                    "{what}"
                );
                // The container and the state layer cover the icon button; the icon is centered at its side.
                for id in [mounted.container, mounted.layer] {
                    let bounds = mounted.frames[&id];
                    assert_eq!((bounds.origin.x.0, bounds.origin.y.0), (0.0, 0.0), "{what}");
                    assert_eq!(bounds.size, frame, "{what}");
                }
                let bounds = mounted.frames[&mounted.icon];
                assert_eq!(
                    (bounds.size.width.0, bounds.size.height.0),
                    (icon, icon),
                    "{what}"
                );
                assert_eq!(bounds.origin.x.0, space, "{what}");
                assert_eq!(bounds.origin.y.0, (height - icon) / 2.0, "{what}");
            }
        }
    }

    #[test]
    fn every_size_has_a_48_point_touch_target_without_growing() {
        for (size, height, icon, spaces, ..) in SIZES {
            for (width, space) in WIDTHS.into_iter().zip(spaces) {
                let what = format!("{size:?} {width:?}");
                let mounted = mount(Config {
                    size,
                    width,
                    ..Config::default()
                });
                let slop = mounted.hit_slop.expect("the button's props");
                let frame_width = icon + 2.0 * space;
                // Each side reaches 48 points with the slop, and a side already 48 or more gets none.
                let reach = |side: f32| (48.0 - side).max(0.0) / 2.0;
                assert_eq!(slop.left.0, reach(frame_width), "{what}");
                assert_eq!(slop.right.0, reach(frame_width), "{what}");
                assert_eq!(slop.top.0, reach(height), "{what}");
                assert_eq!(slop.bottom.0, reach(height), "{what}");
                assert!(frame_width + slop.left.0 + slop.right.0 >= 48.0, "{what}");
                assert!(height + slop.top.0 + slop.bottom.0 >= 48.0, "{what}");
                assert_eq!(mounted.size().height.0, height, "{what}");
            }
        }
    }

    #[test]
    fn the_icon_is_its_path_at_the_token_size_in_the_content_color() {
        for (size, _, icon, ..) in SIZES {
            let mounted = mount(Config {
                size,
                style: IconButtonStyle::Filled,
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
            assert_eq!(
                paint(ops, false),
                Some(color(scheme_color(ColorRole::OnPrimary, false), 255)),
                "{size:?}"
            );
        }
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_button() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.label.as_deref(), Some("Add"));
        assert_eq!(mounted.disabled, Some(false));
        assert_eq!(
            mount(Config {
                disabled: true,
                ..Config::default()
            })
            .disabled,
            Some(true)
        );
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press("label:Add")?;
        assert!(app.shows("presses: 1"));
        assert!(!app.shows("presses: 2"));
        Ok(())
    }

    #[test]
    fn a_disabled_icon_button_takes_no_press_and_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Add");
        assert!(app.shows("presses: 0"));
        let mounted = Mounted::of(&recording.items());
        // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::default())
        );
        // Its container keeps its size: only the container's canvas reports it, not the hidden state layer.
        assert_eq!(mounted.size(), Size::new(Points(40.0), Points(40.0)));
        assert_eq!(
            mounted
                .frames
                .get(&mounted.container)
                .map(|frame| frame.size),
            Some(mounted.size())
        );
        // Enabled, its state layer is there.
        let enabled = mount(Config::default());
        assert_eq!(
            enabled.frames.get(&enabled.layer).map(|frame| frame.size),
            Some(enabled.size())
        );
    }

    /// The paths the container's canvas and the state layer's draw now, and the icon button's size.
    fn drawn(recording: &Recording) -> (Vec<Vec<Step>>, Vec<Vec<Step>>, Size<Points>) {
        let mounted = Mounted::of(&recording.items());
        let replay = replay(recording);
        (
            paths(mounted.drawing(mounted.container), &replay),
            paths(mounted.drawing(mounted.layer), &replay),
            mounted.size(),
        )
    }

    /// The outline's width, halved: it is stroked that far inside the container's edge.
    const HALF_OUTLINE: f32 = 0.5;

    /// The radius of the corners at rest: `ContainerShapeRound` is `CornerFull` at every size, half of the shorter
    /// side.
    fn rest_radius(shape: IconButtonShape, square: f32, size: Size<Points>) -> f32 {
        match shape {
            IconButtonShape::Square => square,
            _ => size.width.0.min(size.height.0) / 2.0,
        }
    }

    #[test]
    fn each_shape_draws_its_corners_on_the_container_and_its_state_layer() {
        for (size, .., square, _) in SIZES {
            for width in WIDTHS {
                for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                    for style in [IconButtonStyle::Filled, IconButtonStyle::Outlined] {
                        let what = format!("{size:?} {width:?} {shape:?} {style:?}");
                        let (_app, recording) = start(Config {
                            size,
                            width,
                            style,
                            shape,
                            ..Config::default()
                        });
                        let (container, layer, frame) = drawn(&recording);
                        let radius = rest_radius(shape, square, frame);
                        let mut expected = vec![rounded_rectangle(frame, 0.0, radius)];
                        if style == IconButtonStyle::Outlined {
                            expected.push(rounded_rectangle(
                                frame,
                                HALF_OUTLINE,
                                radius - HALF_OUTLINE,
                            ));
                        }
                        assert_eq!(container.len(), expected.len(), "{what}: {container:?}");
                        for (path, expected) in container.iter().zip(&expected) {
                            assert_traces(path, expected, &what);
                        }
                        // The state layer keeps the container's corners, inside the outline: the outline is not
                        // tinted.
                        let inside = if style == IconButtonStyle::Outlined {
                            OUTLINE_WIDTH.0
                        } else {
                            0.0
                        };
                        assert_eq!(layer.len(), 1, "{what}: {layer:?}");
                        assert_traces(
                            &layer[0],
                            &rounded_rectangle(frame, inside, radius - inside),
                            &what,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_icon_button_shape_can_morph() {
        // A shape with uneven corners would fall back to full corners: none of the icon button's does.
        for (size, .., square, pressed) in SIZES {
            let spec = size_spec(size);
            assert_eq!(corners(spec.round), Corners::Full, "{size:?}");
            assert_eq!(Corners::of(spec.round), Some(Corners::Full), "{size:?}");
            assert_eq!(
                Corners::of(spec.square),
                Some(Corners::Radius(Points(square))),
                "{size:?}"
            );
            assert_eq!(
                Corners::of(spec.pressed),
                Some(Corners::Radius(Points(pressed))),
                "{size:?}"
            );
        }
    }

    /// The step of a press that goes down on the icon button and stays down: a finger that lands and does not move.
    fn press_down(app: &mut Headless) -> String {
        let still = point(px(0.0), px(0.0));
        app.drag_and_hold("label:Add", still).unwrap();
        format!("dragAndHold(label:Add, {}, {})", still.x, still.y)
    }

    /// The press of `step` springs the corners as [`spring_in`] says, with nothing drawn again or laid out.
    fn morph_in(recording: &Recording, step: &str, to: f32) -> StartedSpring {
        let items = recording.in_step(step);
        let spring = spring_in(&items, step, to);
        assert!(
            !items
                .iter()
                .any(|item| matches!(item, MountItem::Canvas { .. })),
            "{step}: {items:?}"
        );
        assert_eq!(
            Mounted::of(&recording.items()).layout_items(&items),
            0,
            "{step}: {items:?}"
        );
        spring
    }

    #[test]
    fn a_press_springs_the_corners_to_the_pressed_shape_and_the_drawing_follows() {
        for (size, .., square, pressed) in SIZES {
            for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                let what = format!("{size:?} {shape:?}");
                let (mut app, recording) = start(Config {
                    size,
                    shape,
                    style: IconButtonStyle::Outlined,
                    ..Config::default()
                });
                let frame = Mounted::of(&recording.items()).size();
                let rest = rest_radius(shape, square, frame);
                let step = press_down(&mut app);
                let spring = morph_in(&recording, &step, pressed);
                // While it plays, the engine sends nothing (ADR 0010).
                let (ran, sent) = recording.during(|| app.run_for(Duration::from_secs(1)));
                ran.unwrap();
                assert!(sent.is_empty(), "{what}: {sent:?}");

                let mut replay = replay(&recording);
                let mut samples = replay.run(spring.node, 9);
                // Mid-way, the container, its outline and the state layer follow the corners.
                let now = replay.value(spring.node);
                assert!(now < rest - 1.0 && now > pressed + 1.0, "{what}: {now}");
                let mounted = Mounted::of(&recording.items());
                let container = paths(mounted.drawing(mounted.container), &replay);
                assert_traces(&container[0], &rounded_rectangle(frame, 0.0, now), &what);
                assert_traces(
                    &container[1],
                    &rounded_rectangle(frame, HALF_OUTLINE, now - HALF_OUTLINE),
                    &what,
                );
                let layer = paths(mounted.drawing(mounted.layer), &replay);
                assert_traces(
                    &layer[0],
                    &rounded_rectangle(frame, OUTLINE_WIDTH.0, now - OUTLINE_WIDTH.0),
                    &what,
                );
                // The whole curve is the closed-form spring from the corners at rest, within 0.1 % of its travel.
                samples.extend(replay.run(spring.node, 171));
                let gap = worst_gap(&samples, f64::from(rest), &spring);
                assert!(
                    gap < tolerance(f64::from(rest), f64::from(pressed)),
                    "{what}: {gap}"
                );
                // It settles on the pressed corners exactly, and the graph stops.
                let settled = replay.run(spring.node, 600);
                assert!(!replay.is_animating(), "{what}");
                assert_eq!(
                    settled.last().map(|v| v.to_bits()),
                    Some(pressed.to_bits()),
                    "{what}"
                );
            }
        }
    }

    #[test]
    fn a_release_springs_back_from_where_and_as_fast_as_the_corners_move() {
        let (mut app, recording) = start(Config::default());
        // The small icon button is 40 points square: round, its corners are 20.
        let rest = 20.0_f32;
        let step = press_down(&mut app);
        let down = morph_in(&recording, &step, 8.0);
        let mut replay = replay(&recording);
        // The finger lifts 50 ms in, while the corners still move.
        let before = replay.run(down.node, 6);
        app.drop_held().unwrap();
        let up = morph_in(&recording, "drop()", rest);
        assert_eq!(up.node, down.node);
        assert!(app.shows("presses: 1"));
        replay.apply(&recording.in_step("drop()"));
        let after = replay.run(up.node, 180);

        // The graph last ticked 5 frames into the press: the release starts there, at that value and that speed.
        let press = parameters(&down);
        let at = 5.0 / REPLAY_HZ;
        let position = |t: f64| spring_position(f64::from(rest), 8.0, 0.0, press, t);
        let from = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed < -10.0, "{speed}");
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
        assert!(expected(speed) < tolerance, "{}", expected(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(expected(0.0) > 10.0 * tolerance, "{}", expected(0.0));
        let settled = replay.run(up.node, 600);
        assert!(!replay.is_animating());
        assert_eq!(settled.last().map(|v| v.to_bits()), Some(rest.to_bits()));
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for (size, ..) in SIZES {
            for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                let (_app, recording) = start(Config {
                    size,
                    shape,
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{size:?} {shape:?}");
                // The corners are created where they rest: the layout sets nothing again.
                assert_eq!(set_values(&items), 0, "{size:?} {shape:?}");
                assert!(!replay(&recording).is_animating(), "{size:?} {shape:?}");
            }
        }
    }

    #[test]
    fn with_reduce_motion_a_press_leaves_the_shape_as_it_is() {
        // ADR 0011, "Reduce motion": no spring, and no jump either.
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let (_, _, frame) = drawn(&recording);
        let at_rest = rounded_rectangle(frame, 0.0, 20.0);
        let step = press_down(&mut app);
        let pressed = recording.in_step(&step);
        assert_eq!(started_springs(&pressed), Vec::new());
        assert_eq!(set_values(&pressed), 0);
        let (container, layer, _) = drawn(&recording);
        assert_traces(&container[0], &at_rest, "pressed");
        assert_traces(&layer[0], &at_rest, "pressed");
        app.drop_held().unwrap();
        assert_eq!(started_springs(&recording.in_step("drop()")), Vec::new());
        assert!(app.shows("presses: 1"));
    }

    #[test]
    fn a_new_shape_moves_the_corners_at_once() {
        // androidx starts the shape animation again when the shapes change (`key(shapes)`): no spring.
        let (mut app, recording) = start(Config::default());
        let (_, _, frame) = drawn(&recording);
        for (shape, radius) in [
            (IconButtonShape::Square, 12.0),
            (IconButtonShape::Round, 20.0),
        ] {
            app.press("Reshape").unwrap();
            let items = recording.in_step("press(Reshape)");
            assert_eq!(started_springs(&items), Vec::new(), "{shape:?}");
            let (container, layer, _) = drawn(&recording);
            let expected = rounded_rectangle(frame, 0.0, radius);
            assert_traces(&container[0], &expected, &format!("{shape:?}"));
            assert_traces(&layer[0], &expected, &format!("{shape:?}"));
            assert!(!replay(&recording).is_animating(), "{shape:?}");
        }
        assert!(app.shows("presses: 0"));
    }

    #[test]
    fn a_new_shape_during_a_press_stops_the_morph_on_the_pressed_corners() {
        let (mut app, recording) = start(Config::default());
        let (_, _, frame) = drawn(&recording);
        let step = press_down(&mut app);
        let spring = morph_in(&recording, &step, 8.0);
        app.press("Reshape").unwrap();
        let items = recording.in_step("press(Reshape)");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 1, "{items:?}");
        let mut replay = replay(&recording);
        let after = replay.run(spring.node, 2);
        assert!(!replay.is_animating());
        assert_eq!(
            after.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            [8.0_f32.to_bits(); 2]
        );
        let (container, layer, _) = drawn(&recording);
        let pressed = rounded_rectangle(frame, 0.0, 8.0);
        assert_traces(&container[0], &pressed, "container");
        assert_traces(&layer[0], &pressed, "layer");
    }

    #[test]
    fn an_icon_button_disabled_while_pressed_goes_back_to_its_shape_at_rest() {
        let (mut app, recording) = start(Config::default());
        let step = press_down(&mut app);
        let _ = morph_in(&recording, &step, 8.0);
        app.press("Disable").unwrap();
        let _ = spring_in(&recording.in_step("press(Disable)"), "press(Disable)", 20.0);
    }

    #[test]
    fn hover_shows_the_state_layer_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            style: IconButtonStyle::Filled,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("label:Add")?;
        let hover = recording.in_step("hover(label:Add)");
        let layer: Vec<&ViewStyle> = hover
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(&**style),
                _ => None,
            })
            .collect();
        // The state layer at `StateTokens.HoverStateLayerOpacity`, in the icon's color.
        assert_eq!(layer.len(), 1);
        assert_eq!(layer[0].opacity, 0.08);
        assert_eq!(
            paint(&mounted.drawing(mounted.layer).ops, false),
            Some(color(scheme_color(ColorRole::OnPrimary, false), 255))
        );
        // Nothing else: no frame, and neither the button, its container nor its icon is styled or drawn again.
        assert_eq!(layout_items(&hover), 0);
        assert!(
            hover.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } | MountItem::Canvas { id, .. } =>
                    *id == mounted.layer,
                _ => true,
            }),
            "{hover:?}"
        );
        Ok(())
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        assert_eq!(
            size_spec(IconButtonSize::default()),
            size_spec(IconButtonSize::Small)
        );
        assert_eq!(IconButtonWidth::default(), IconButtonWidth::Uniform);
        assert_eq!(IconButtonStyle::default(), IconButtonStyle::Standard);
        assert_eq!(IconButtonShape::default(), IconButtonShape::Round);
        for style in STYLES {
            let spec = style_spec(style);
            // Only the outlined icon button has an outline, and only the filled and tonal ones a container.
            assert_eq!(
                spec.outline.is_some(),
                style == IconButtonStyle::Outlined,
                "{style:?}"
            );
            assert_eq!(
                spec.disabled_outline.is_some(),
                style == IconButtonStyle::Outlined,
                "{style:?}"
            );
            assert_eq!(
                spec.container.is_some(),
                matches!(style, IconButtonStyle::Filled | IconButtonStyle::Tonal),
                "{style:?}"
            );
            assert_eq!(
                spec.container.is_some(),
                spec.disabled_container.is_some(),
                "{style:?}"
            );
        }
    }

    #[test]
    fn uniform_width_adds_the_leading_space_twice() {
        // androidx adds `DefaultLeadingSpace` (or `UniformLeadingSpace`) twice for the uniform width: that is the
        // trailing space too, as long as the tokens keep them equal.
        for (size, ..) in SIZES {
            let spec = size_spec(size);
            assert_eq!(spec.uniform.0, spec.uniform.1, "{size:?}");
        }
        assert_eq!(
            xsmall_icon_button_tokens::DEFAULT_LEADING_SPACE,
            xsmall_icon_button_tokens::DEFAULT_TRAILING_SPACE
        );
        assert_eq!(
            small_icon_button_tokens::DEFAULT_LEADING_SPACE,
            small_icon_button_tokens::DEFAULT_TRAILING_SPACE
        );
        assert_eq!(
            medium_icon_button_tokens::DEFAULT_LEADING_SPACE,
            medium_icon_button_tokens::DEFAULT_TRAILING_SPACE
        );
        assert_eq!(
            large_icon_button_tokens::UNIFORM_LEADING_SPACE,
            large_icon_button_tokens::UNIFORM_TRAILING_SPACE
        );
        assert_eq!(
            xlarge_icon_button_tokens::DEFAULT_LEADING_SPACE,
            xlarge_icon_button_tokens::DEFAULT_TRAILING_SPACE
        );
    }

    /// The vendored `IconButton.kt` and `IconButtonDefaults.kt`, at the pinned commit (ADR 0012).
    const ICON_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/IconButton.kt");
    const ICON_BUTTON_DEFAULTS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/IconButtonDefaults.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_choice_taken_from_androidx_is_still_there() {
        // The vibrant colors and border are the token ones; the outline is the small size's at every size.
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "containerColor = Color.Transparent,",
                "contentColor = fromToken(StandardIconButtonTokens.Color),",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "containerColor = Color.Transparent,",
                "contentColor = fromToken(OutlinedIconButtonTokens.Color),",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "val outlineColor = OutlinedIconButtonTokens.OutlineColor.value",
                "val color: Color =",
                "if (enabled) {",
                "outlineColor",
                "} else {",
                "outlineColor.copy(alpha = OutlinedIconButtonTokens.DisabledOpacity)",
                "}",
                "return remember(color) { BorderStroke(SmallIconButtonTokens.OutlinedOutlineWidth, color) }",
            ]
        ));
        assert_eq!(OUTLINE_WIDTH, Points(1.0));
        assert_ne!(
            OUTLINE_WIDTH,
            large_icon_button_tokens::OUTLINED_OUTLINE_WIDTH
        );
        // The uniform width's spaces, and the morph's spring.
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "IconButtonWidthOption.Uniform ->",
                "SmallIconButtonTokens.DefaultLeadingSpace +",
                "SmallIconButtonTokens.DefaultLeadingSpace",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "IconButtonWidthOption.Uniform ->",
                "LargeIconButtonTokens.UniformLeadingSpace +",
                "LargeIconButtonTokens.UniformLeadingSpace",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultEffects.value<Float>()",
                "val pressed by interactionSource.collectIsPressedAsState()",
                "",
                "return shapeByInteraction(shapes, pressed, defaultAnimationSpec)",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "return key(shapes) { rememberAnimatedShape(shape as CornerBasedShape, animationSpec) }",
            ]
        ));
        // A line that is not there is not found.
        assert!(!cites(
            ICON_BUTTON_KT,
            &["contentColor = fromToken(StandardIconButtonTokens.Color),"]
        ));
    }
}
