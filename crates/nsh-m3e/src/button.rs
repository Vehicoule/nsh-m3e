//! The Material 3 Expressive button: its five sizes, two shapes and four color styles, the state layer of hover,
//! focus and press, and the shape it morphs into while pressed.
//!
//! The button follows androidx's Expressive `Button(onClick, shapes, …)` and its siblings in `Button.kt`, at the
//! pinned commit, as [ADR 0012] says: it reads the tokens they read, and the few values they write in place of a
//! token they call wrong, each cited where it is used. Its container is drawn by a canvas whose corner radius the
//! M3E spring moves ([ADR 0011]).
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::color::{ColorRole, ColorScheme};
use crate::morph::{Corners, DrawnShape};
use crate::tokens::androidx::{
    button_large_tokens, button_medium_tokens, button_small_tokens, button_xlarge_tokens,
    button_xsmall_tokens, filled_button_tokens, filled_tonal_button_tokens, outlined_button_tokens,
    state_tokens, text_button_tokens, typography_tokens,
};
use crate::tokens::{Shape, TypeStyle};

/// The size of a [`Button`], from extra small (32 points tall) to extra large (136).
///
/// Each size takes its height, side spaces, shapes and label style from its androidx token file
/// (`ButtonXSmallTokens` to `ButtonXLargeTokens`), as androidx's `ButtonDefaults.*For(buttonHeight)` helpers give
/// them.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ButtonSize {
    /// 32 points tall.
    ExtraSmall,
    /// 40 points tall: androidx's default button.
    #[default]
    Small,
    /// 56 points tall.
    Medium,
    /// 96 points tall.
    Large,
    /// 136 points tall.
    ExtraLarge,
}

/// The color style of a [`Button`], from the most emphasis to the least.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ButtonStyle {
    /// A container in the primary color: androidx `Button`.
    #[default]
    Filled,
    /// A container in the secondary container color: androidx `FilledTonalButton`.
    Tonal,
    /// No container, an outline: androidx `OutlinedButton`.
    Outlined,
    /// No container and no outline: androidx `TextButton`.
    Text,
}

/// The shape of a [`Button`] at rest.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ButtonShape {
    /// Fully rounded ends: the size's `ContainerShapeRound`.
    #[default]
    Round,
    /// Rounded corners: the size's `ContainerShapeSquare`.
    Square,
}

/// What a size gives the button.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's height for one line of label.
    height: Points,
    /// The space before the label.
    leading: Points,
    /// The space after the label.
    trailing: Points,
    /// The label's type style.
    label: TypeStyle,
    /// The shape at rest when round.
    round: Shape,
    /// The shape at rest when square.
    square: Shape,
    /// The shape while pressed, round or square: androidx `ButtonDefaults.shapesFor(buttonHeight)` gives the size's
    /// `PressedContainerShape` as the pressed shape of both.
    pressed: Shape,
}

/// androidx `Button.kt`, `ButtonDefaults.ExtraSmallContentPadding` (line 972): 12 points on each side, under "TODO
/// update with the value from ButtonXSmallTokens.kt once it's been corrected"; `ButtonXSmallTokens.LeadingSpace` and
/// `TrailingSpace` say 16 (ADR 0012).
const EXTRA_SMALL_SIDE_SPACE: Points = Points(12.0);

/// The values of `size`, from its token file and, for the extra small sides, from `Button.kt`.
///
/// The label styles are androidx `ButtonDefaults.textStyleFor`: label large below the medium height, title medium
/// below the large one, headline small below the extra large one, and headline large from it.
fn size_spec(size: ButtonSize) -> SizeSpec {
    match size {
        ButtonSize::ExtraSmall => SizeSpec {
            height: button_xsmall_tokens::CONTAINER_HEIGHT,
            leading: EXTRA_SMALL_SIDE_SPACE,
            trailing: EXTRA_SMALL_SIDE_SPACE,
            label: typography_tokens::LABEL_LARGE,
            round: button_xsmall_tokens::CONTAINER_SHAPE_ROUND,
            square: button_xsmall_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: button_xsmall_tokens::PRESSED_CONTAINER_SHAPE,
        },
        ButtonSize::Small => SizeSpec {
            height: button_small_tokens::CONTAINER_HEIGHT,
            leading: button_small_tokens::LEADING_SPACE,
            trailing: button_small_tokens::TRAILING_SPACE,
            label: typography_tokens::LABEL_LARGE,
            round: button_small_tokens::CONTAINER_SHAPE_ROUND,
            square: button_small_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: button_small_tokens::PRESSED_CONTAINER_SHAPE,
        },
        ButtonSize::Medium => SizeSpec {
            height: button_medium_tokens::CONTAINER_HEIGHT,
            leading: button_medium_tokens::LEADING_SPACE,
            trailing: button_medium_tokens::TRAILING_SPACE,
            label: typography_tokens::TITLE_MEDIUM,
            round: button_medium_tokens::CONTAINER_SHAPE_ROUND,
            square: button_medium_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: button_medium_tokens::PRESSED_CONTAINER_SHAPE,
        },
        ButtonSize::Large => SizeSpec {
            height: button_large_tokens::CONTAINER_HEIGHT,
            leading: button_large_tokens::LEADING_SPACE,
            trailing: button_large_tokens::TRAILING_SPACE,
            label: typography_tokens::HEADLINE_SMALL,
            round: button_large_tokens::CONTAINER_SHAPE_ROUND,
            square: button_large_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: button_large_tokens::PRESSED_CONTAINER_SHAPE,
        },
        ButtonSize::ExtraLarge => SizeSpec {
            height: button_xlarge_tokens::CONTAINER_HEIGHT,
            leading: button_xlarge_tokens::LEADING_SPACE,
            trailing: button_xlarge_tokens::TRAILING_SPACE,
            label: typography_tokens::HEADLINE_LARGE,
            round: button_xlarge_tokens::CONTAINER_SHAPE_ROUND,
            square: button_xlarge_tokens::CONTAINER_SHAPE_SQUARE,
            pressed: button_xlarge_tokens::PRESSED_CONTAINER_SHAPE,
        },
    }
}

/// A color role, made more transparent by `opacity` (1 for the role as it is).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tint {
    role: ColorRole,
    opacity: f32,
}

impl Tint {
    const fn solid(role: ColorRole) -> Self {
        Self { role, opacity: 1.0 }
    }

    const fn faded(role: ColorRole, opacity: f32) -> Self {
        Self { role, opacity }
    }
}

/// What a color style gives the button, enabled and disabled. No container is a transparent one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StyleSpec {
    container: Option<Tint>,
    content: Tint,
    disabled_container: Option<Tint>,
    disabled_content: Tint,
    outline: Option<Tint>,
    disabled_outline: Option<Tint>,
}

/// androidx `Button.kt`, `ButtonDefaults.defaultTextButtonColors` (line 1423): the text button's label is the primary
/// color, under "TODO replace with the token value once it's corrected"; `TextButtonTokens.LabelColor` says on
/// surface variant (ADR 0012).
const TEXT_BUTTON_LABEL: ColorRole = ColorRole::Primary;

/// The colors of `style`, as androidx `ButtonDefaults` builds them: `defaultButtonColors`,
/// `defaultFilledTonalButtonColors` (from `FilledTonalButtonTokens`, ADR 0012), `defaultOutlinedButtonColors` with
/// `outlinedButtonBorder`, and `defaultTextButtonColors`.
fn style_spec(style: ButtonStyle) -> StyleSpec {
    match style {
        ButtonStyle::Filled => StyleSpec {
            container: Some(Tint::solid(filled_button_tokens::CONTAINER_COLOR)),
            content: Tint::solid(filled_button_tokens::LABEL_TEXT_COLOR),
            disabled_container: Some(Tint::faded(
                filled_button_tokens::DISABLED_CONTAINER_COLOR,
                filled_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
            disabled_content: Tint::faded(
                filled_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                filled_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        ButtonStyle::Tonal => StyleSpec {
            container: Some(Tint::solid(filled_tonal_button_tokens::CONTAINER_COLOR)),
            content: Tint::solid(filled_tonal_button_tokens::LABEL_TEXT_COLOR),
            disabled_container: Some(Tint::faded(
                filled_tonal_button_tokens::DISABLED_CONTAINER_COLOR,
                filled_tonal_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
            disabled_content: Tint::faded(
                filled_tonal_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                filled_tonal_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        ButtonStyle::Outlined => StyleSpec {
            container: None,
            content: Tint::solid(outlined_button_tokens::LABEL_TEXT_COLOR),
            disabled_container: None,
            disabled_content: Tint::faded(
                outlined_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                outlined_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            // `outlinedButtonBorder` fades the outline color by the container's disabled opacity, where
            // `OutlinedButtonTokens.DisabledOutlineColor` names another role with no opacity (ADR 0012).
            outline: Some(Tint::solid(outlined_button_tokens::OUTLINE_COLOR)),
            disabled_outline: Some(Tint::faded(
                outlined_button_tokens::OUTLINE_COLOR,
                outlined_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
        },
        ButtonStyle::Text => StyleSpec {
            container: None,
            content: Tint::solid(TEXT_BUTTON_LABEL),
            disabled_container: None,
            disabled_content: Tint::faded(
                text_button_tokens::DISABLED_LABEL_COLOR,
                text_button_tokens::DISABLED_LABEL_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
    }
}

/// The outline's width at every size: androidx `outlinedButtonBorder` reads `ButtonSmallTokens.OutlinedOutlineWidth`
/// whatever the height (ADR 0012).
const OUTLINE_WIDTH: Points = button_small_tokens::OUTLINED_OUTLINE_WIDTH;

/// The smallest touch target, 48 points on each side: `docs/app-requirements.md` (Accessibility), from Material 3's
/// touch target guidance. No androidx token holds it.
const MIN_TOUCH_TARGET: Points = Points(48.0);

/// How far presses also count around a container of `height`, so that a button shorter than [`MIN_TOUCH_TARGET`]
/// still has a 48-point target, without changing its M3 size or the layout (NSH's accessibility guide: extend the touch
/// area with the hit slop, not the size). The same on every side: a label wider than 8 points makes the extra small and
/// small buttons 48 points wide too, their side spaces being 12 and 16.
fn touch_slop(height: Points) -> EdgeInsets {
    // Half of the missing height above, half below.
    EdgeInsets::all(px(((MIN_TOUCH_TARGET.0 - height.0) / 2.0).max(0.0)))
}

/// The color of `role` in the app's NSH design tokens (ADR 0008), or else in the baseline schemes, so that a button
/// shows Material's colors before the app sets its own.
fn role_color(role: ColorRole) -> AdaptiveColor {
    tokens().color(role.token_name()).unwrap_or_else(|| {
        AdaptiveColor::rgb(
            ColorScheme::BASELINE_LIGHT.get(role).to_hex(),
            ColorScheme::BASELINE_DARK.get(role).to_hex(),
        )
    })
}

/// The color of a tint, or transparent for none.
fn tint_color(tint: Option<Tint>) -> AdaptiveColor {
    tint.map_or(AdaptiveColor::TRANSPARENT, |tint| {
        role_color(tint.role).opacity(tint.opacity)
    })
}

/// The corners the drawn container gives `shape`. Every shape of the button tokens has the same four corners
/// (`each_button_shape_can_morph` tests it), so the fully rounded fallback is never taken; it is there so that a
/// token with uneven corners, which no drawn container can draw, still gives a button, not a panic.
fn corners(shape: Shape) -> Corners {
    Corners::of(shape).unwrap_or(Corners::Full)
}

/// The props of a [`Button`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ButtonProps {
    /// What the button says.
    pub label: Prop<SharedString>,
    /// Its size.
    #[prop(default)]
    pub size: Prop<ButtonSize>,
    /// Its color style.
    #[prop(default)]
    pub style: Prop<ButtonStyle>,
    /// Its shape at rest.
    #[prop(default)]
    pub shape: Prop<ButtonShape>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`Button`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ButtonEvent {
    /// The button was pressed.
    Press,
}

/// A Material 3 Expressive button: a label in a container of the size, shape and color style given.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, the button role) and the look of
/// androidx's Expressive button. Hover, focus and press lay the label's color over it at the state layer opacities
/// of `StateTokens`, as androidx's ripple does, and change nothing else, so nothing is laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::button::{ButtonSize, ButtonStyle, button};
///
/// fn save() -> impl IntoElement {
///     button()
///         .label("Save")
///         .size(ButtonSize::Medium)
///         .style(ButtonStyle::Tonal)
///         .on_press(|| println!("saved"))
/// }
/// ```
#[component]
pub fn Button(props: ButtonProps, cx: Scope<ButtonEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || style_spec(props.style.get());
    let shape = move || match props.shape.get() {
        ButtonShape::Round => size().round,
        ButtonShape::Square => size().square,
    };
    let content = move || {
        let colors = colors();
        tint_color(Some(if props.disabled.get() {
            colors.disabled_content
        } else {
            colors.content
        }))
    };
    // Before its first layout, the container is taken to be as wide as it is tall: the fully rounded radius it starts
    // at is then already right for any label at least that wide, and its layout corrects the rest.
    let drawn = untracked(|| {
        let height = size().height;
        DrawnShape::new(
            corners(shape()),
            corners(size().pressed),
            Size::new(height, height),
        )
    });
    effect(move || drawn.set_corners(corners(shape()), corners(size().pressed)));
    effect(move || {
        // A button disabled while pressed may get no end of its press: it goes back to its shape at rest.
        if props.disabled.get() {
            drawn.press(false);
        }
    });
    let outline = move || colors().outline.map(|_| OUTLINE_WIDTH);
    nsh_base::button()
        .disabled(move || props.disabled.get())
        .on_press(move || cx.emit(ButtonEvent::Press))
        .on_press_down(move || drawn.press(true))
        .on_press_up(move || drawn.press(false))
        .hit_slop(move || touch_slop(size().height))
        .group()
        .flex_row()
        .items_center()
        .justify_center()
        .min_h(move || size().height)
        .ps(move || size().leading)
        .pe(move || size().trailing)
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
            // The state layer: the label's color over the whole container, in its shape, shown by the button's own
            // states with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
            drawn
                .state_layer()
                .bg(content)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| {
                    // A disabled button takes no press and shows no state.
                    if props.disabled.get() { s.hidden() } else { s }
                }),
        )
        .child(
            text(move || props.label.get())
                .text_color(content)
                .text_size(move || size().label.size)
                .line_height(move || LineHeight::Points(size().label.line_height))
                .letter_spacing(move || size().label.tracking)
                .style_with(move |s| s.font_weight(size().label.weight)),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::f32::consts::{FRAC_PI_2, PI};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::AnimationOp;
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num, PathOp};
    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, layout_items, spring_position,
        started_springs, worst_gap,
    };
    use crate::motion::{SPRING_MASS, SpringToken};

    /// What the probe places: one button with these props, a line counting its presses, and two lines that change
    /// its shape and disable it when pressed.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        label: Prop<SharedString>,
        #[prop(default)]
        size: Prop<ButtonSize>,
        #[prop(default)]
        style: Prop<ButtonStyle>,
        #[prop(default)]
        shape: Prop<ButtonShape>,
        #[prop(default)]
        disabled: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        let shape = signal(props.shape.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        // A row, so that the button is as wide as its content, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    button()
                        .label(props.label)
                        .size(props.size)
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
                        ButtonShape::Round => ButtonShape::Square,
                        _ => ButtonShape::Round,
                    };
                });
            }))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        label: &'static str,
        size: ButtonSize,
        style: ButtonStyle,
        shape: ButtonShape,
        disabled: bool,
        dark: bool,
        reduce_motion: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                label: "Save",
                size: ButtonSize::default(),
                style: ButtonStyle::default(),
                shape: ButtonShape::default(),
                disabled: false,
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

    /// The button's views, and the last style, frame, drawing and text each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole button.
        button: NodeId,
        /// The canvas that draws the container.
        container: NodeId,
        /// The canvas that draws the state layer.
        layer: NodeId,
        label: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        text: Option<nsh_core::ResolvedTextStyle>,
        disabled: Option<bool>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut button = None;
            let mut disabled = None;
            let mut hit_slop = None;
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
                        disabled = Some(props.accessibility.state.disabled);
                        hit_slop = Some(props.hit_slop);
                    }
                    MountItem::ViewStyle { id, style } => {
                        styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    MountItem::Text {
                        id, text: content, ..
                    } => {
                        texts.insert(*id, content.spans.first().map(|span| span.style));
                    }
                    MountItem::Canvas { id, drawing } => {
                        drawings.insert(*id, Arc::clone(drawing));
                    }
                    _ => {}
                }
            }
            let button = button.expect("a view with the button role");
            let children = |kind: ViewKind| -> Vec<NodeId> {
                children
                    .get(&button)
                    .into_iter()
                    .flatten()
                    .filter(|child| kinds.get(child) == Some(&kind))
                    .copied()
                    .collect()
            };
            // The container first, under the state layer.
            let [container, layer] = children(ViewKind::Canvas)[..] else {
                panic!("two canvases in the button: {items:?}");
            };
            let [label] = children(ViewKind::Text)[..] else {
                panic!("one text in the button: {items:?}");
            };
            Self {
                button,
                container,
                layer,
                label,
                styles,
                frames,
                drawings,
                text: texts.get(&label).copied().flatten(),
                disabled,
                hit_slop,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        /// How many of `items` lay the button out again: one of its views' frames, or its label's text.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.button, self.container, self.layer, self.label];
            items
                .iter()
                .filter(|item| match item {
                    MountItem::Frame { id, .. } | MountItem::Text { id, .. } => ours.contains(id),
                    _ => false,
                })
                .count()
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// The button's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.button].size
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

    const STYLES: [ButtonStyle; 4] = [
        ButtonStyle::Filled,
        ButtonStyle::Tonal,
        ButtonStyle::Outlined,
        ButtonStyle::Text,
    ];

    const SIZES: [ButtonSize; 5] = [
        ButtonSize::ExtraSmall,
        ButtonSize::Small,
        ButtonSize::Medium,
        ButtonSize::Large,
        ButtonSize::ExtraLarge,
    ];

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// The color `ops` fill with, or that they stroke with, if any.
    fn paint(ops: &[CanvasOp], stroke: bool) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) if !stroke => Some(*color),
            CanvasOp::StrokeStyle(canvas::Paint::Color(color)) if stroke => Some(*color),
            _ => None,
        })
    }

    /// The container's color, its outline's width and color (0 and transparent with no outline), and the label's
    /// color: the container's as its canvas paints it (transparent when nothing fills it).
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
        let label = mounted.text.as_ref().expect("the label's style").color;
        (paint(ops, false).unwrap_or(Color(0)), width, outline, label)
    }

    #[test]
    fn each_style_has_its_colors_at_rest() {
        let light = |role| scheme_color(role, false);
        for (style, expected) in [
            // androidx `FilledButtonTokens`: primary, on primary.
            (
                ButtonStyle::Filled,
                (
                    color(light(ColorRole::Primary), 255),
                    0.0,
                    color(light(ColorRole::OnPrimary), 255),
                ),
            ),
            // androidx `FilledTonalButtonTokens` (ADR 0012): secondary container, on secondary container.
            (
                ButtonStyle::Tonal,
                (
                    color(light(ColorRole::SecondaryContainer), 255),
                    0.0,
                    color(light(ColorRole::OnSecondaryContainer), 255),
                ),
            ),
            // androidx `OutlinedButtonTokens`: no container, a 1-point outline variant outline, on surface variant.
            (
                ButtonStyle::Outlined,
                (
                    Color(0),
                    1.0,
                    color(light(ColorRole::OnSurfaceVariant), 255),
                ),
            ),
            // `Button.kt` line 1423: the text button's label is primary, not `TextButtonTokens.LabelColor`.
            (
                ButtonStyle::Text,
                (Color(0), 0.0, color(light(ColorRole::Primary), 255)),
            ),
        ] {
            let mounted = mount(Config {
                style,
                ..Config::default()
            });
            let (background, width, outline, label) = colors(&mounted);
            assert_eq!((background, width, label), expected, "{style:?}");
            // The canvases draw the container: no view paints a background or a border of its own under them.
            for id in [mounted.button, mounted.container, mounted.layer] {
                let style = mounted.style(id);
                assert_eq!(style.background_color, Color::TRANSPARENT, "{style:?}");
                assert_eq!(style.borders.widths.top, px(0.0), "{style:?}");
            }
            if width > 0.0 {
                assert_eq!(
                    outline,
                    color(light(ColorRole::OutlineVariant), 255),
                    "{style:?}"
                );
            }
        }
    }

    #[test]
    fn the_dark_appearance_takes_the_dark_scheme() {
        let (background, _, _, label) = colors(&mount(Config {
            dark: true,
            ..Config::default()
        }));
        assert_eq!(
            background,
            color(scheme_color(ColorRole::Primary, true), 255)
        );
        assert_eq!(label, color(scheme_color(ColorRole::OnPrimary, true), 255));
        assert_ne!(
            background,
            color(scheme_color(ColorRole::Primary, false), 255)
        );
    }

    #[test]
    fn a_disabled_button_fades_takes_no_press_and_shows_no_state() {
        let light = |role| scheme_color(role, false);
        // 0.1, 0.12 and 0.38 of 255, rounded: 26, 31 and 97.
        for (style, expected) in [
            (
                ButtonStyle::Filled,
                (
                    color(light(ColorRole::OnSurface), 26),
                    0.0,
                    color(light(ColorRole::OnSurfaceVariant), 97),
                ),
            ),
            (
                ButtonStyle::Tonal,
                (
                    color(light(ColorRole::OnSurface), 31),
                    0.0,
                    color(light(ColorRole::OnSurface), 97),
                ),
            ),
            (
                ButtonStyle::Outlined,
                (Color(0), 1.0, color(light(ColorRole::OnSurfaceVariant), 97)),
            ),
            (
                ButtonStyle::Text,
                (Color(0), 0.0, color(light(ColorRole::OnSurfaceVariant), 97)),
            ),
        ] {
            let (mut app, recording) = start(Config {
                style,
                disabled: true,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let (background, width, outline, label) = colors(&mounted);
            assert_eq!((background, width, label), expected, "{style:?}");
            if width > 0.0 {
                // `outlinedButtonBorder`: the outline color at the container's disabled opacity (ADR 0012).
                assert_eq!(
                    outline,
                    color(light(ColorRole::OutlineVariant), 26),
                    "{style:?}"
                );
            }
            assert_eq!(mounted.disabled, Some(true), "{style:?}");
            // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
            assert_eq!(
                mounted.frames.get(&mounted.layer).map(|frame| frame.size),
                Some(Size::default()),
                "{style:?}"
            );
            // A press on it does nothing.
            let _ = app.press("Save");
            assert!(app.shows("presses: 0"), "{style:?}");
        }
        // Enabled, the same button is not dimmed and its state layer is there.
        let mounted = mount(Config::default());
        assert_eq!(mounted.disabled, Some(false));
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(mounted.size())
        );
        assert_ne!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::default())
        );
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press("Save")?;
        app.wait_for("text:presses: 1")?;
        app.press("role:button")?;
        app.wait_for("text:presses: 2")?;
        Ok(())
    }

    #[test]
    fn each_size_has_its_height_side_spaces_and_label_style() {
        // androidx `Button{XSmall,Small,Medium,Large,XLarge}Tokens`: height, leading and trailing space; the extra
        // small sides are `Button.kt`'s 12 (line 972). The labels are `textStyleFor`'s: label large, label large,
        // title medium, headline small, headline large.
        for (size, height, side, label) in [
            (ButtonSize::ExtraSmall, 32.0, 12.0, (14.0, 20.0, 500, 0.1)),
            (ButtonSize::Small, 40.0, 16.0, (14.0, 20.0, 500, 0.1)),
            (ButtonSize::Medium, 56.0, 24.0, (16.0, 24.0, 500, 0.2)),
            (ButtonSize::Large, 96.0, 48.0, (24.0, 32.0, 400, 0.0)),
            (ButtonSize::ExtraLarge, 136.0, 64.0, (32.0, 40.0, 400, 0.0)),
        ] {
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let container = mounted.frames[&mounted.button];
            let text = mounted.frames[&mounted.label];
            assert_eq!(container.size.height.0, height, "{size:?}");
            // The label sits after the leading space, and the trailing space follows it.
            assert_eq!(text.origin.x.0, side, "{size:?}");
            assert_eq!(
                container.size.width.0 - text.origin.x.0 - text.size.width.0,
                side,
                "{size:?}"
            );
            let style = mounted.text.as_ref().expect("the label's style");
            assert_eq!(
                (
                    style.font_size.0,
                    style.line_height.0,
                    style.font_weight,
                    style.letter_spacing.0
                ),
                label,
                "{size:?}"
            );
            // The canvases of the container and of the state layer cover the button.
            for canvas in [mounted.container, mounted.layer] {
                assert_eq!(
                    mounted.frames[&canvas],
                    Bounds::new(point(px(0.0), px(0.0)), container.size),
                    "{size:?}"
                );
            }
        }
    }

    #[test]
    fn every_size_has_a_48_point_touch_target_without_growing() {
        // `docs/app-requirements.md`: 48 points at least. The extra small and small buttons reach it with the hit
        // slop, the taller ones need none; the frame keeps the size's height, so the layout does not change.
        for (size, height, slop) in [
            (ButtonSize::ExtraSmall, 32.0, 8.0),
            (ButtonSize::Small, 40.0, 4.0),
            (ButtonSize::Medium, 56.0, 0.0),
            (ButtonSize::Large, 96.0, 0.0),
            (ButtonSize::ExtraLarge, 136.0, 0.0),
        ] {
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            assert_eq!(
                mounted.hit_slop,
                Some(EdgeInsets::all(px(slop))),
                "{size:?}"
            );
            let frame = mounted.size();
            assert_eq!(frame.height.0, height, "{size:?}");
            assert!(frame.height.0 + 2.0 * slop >= 48.0, "{size:?}");
            assert!(frame.width.0 + 2.0 * slop >= 48.0, "{size:?}");
        }
    }

    /// The radii of a view's corners, top left first, clockwise.
    fn radii(style: &ViewStyle) -> [f32; 4] {
        let corners = style.corner_radius;
        [
            corners.top_left,
            corners.top_right,
            corners.bottom_right,
            corners.bottom_left,
        ]
        .map(|corner| corner.unwrap_or(style.border_radius).0)
    }

    /// One step of a path, its numbers read.
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Step {
        Move(f32, f32),
        Line(f32, f32),
        Arc {
            x: f32,
            y: f32,
            radius: f32,
            start: f32,
            end: f32,
            ccw: bool,
        },
        Close,
    }

    /// The paths `drawing` traces, each begun by `beginPath`, the animated values among their numbers read in
    /// `replay` as they are now.
    fn paths(drawing: &Drawing, replay: &Replay) -> Vec<Vec<Step>> {
        let slot = |slot: u16| replay.value(drawing.slots[usize::from(slot)]);
        let mut paths: Vec<Vec<Step>> = Vec::new();
        for op in &drawing.ops {
            let step = match op {
                CanvasOp::BeginPath => {
                    paths.push(Vec::new());
                    continue;
                }
                CanvasOp::Path(PathOp::MoveTo(x, y)) => Step::Move(x.get(slot), y.get(slot)),
                CanvasOp::Path(PathOp::LineTo(x, y)) => Step::Line(x.get(slot), y.get(slot)),
                CanvasOp::Path(PathOp::Arc {
                    x,
                    y,
                    radius,
                    start,
                    end,
                    ccw,
                }) => Step::Arc {
                    x: x.get(slot),
                    y: y.get(slot),
                    radius: radius.get(slot),
                    start: start.get(slot),
                    end: end.get(slot),
                    ccw: *ccw,
                },
                CanvasOp::Path(PathOp::Close) => Step::Close,
                CanvasOp::Path(other) => panic!("not a step of a rounded rectangle: {other:?}"),
                _ => continue,
            };
            paths.last_mut().expect("a path begun").push(step);
        }
        paths
    }

    /// A rectangle `inset` points inside a container of `size`, its corners rounded by `radius`, as ADR 0011 traces
    /// it: clockwise from the end of the top left corner, a line and a quarter circle for each side.
    fn rounded_rectangle(size: Size<Points>, inset: f32, radius: f32) -> Vec<Step> {
        let (left, top) = (inset, inset);
        let (right, bottom) = (size.width.0 - inset, size.height.0 - inset);
        let quarter = |x, y, start: f32| Step::Arc {
            x,
            y,
            radius,
            start,
            end: start + FRAC_PI_2,
            ccw: false,
        };
        vec![
            Step::Move(left + radius, top),
            Step::Line(right - radius, top),
            quarter(right - radius, top + radius, -FRAC_PI_2),
            Step::Line(right, bottom - radius),
            quarter(right - radius, bottom - radius, 0.0),
            Step::Line(left + radius, bottom),
            quarter(left + radius, bottom - radius, FRAC_PI_2),
            Step::Line(left, top + radius),
            quarter(left + radius, top + radius, PI),
            Step::Close,
        ]
    }

    /// The kind of `step`, and its numbers.
    fn numbers(step: &Step) -> (&'static str, Vec<f32>) {
        match *step {
            Step::Move(x, y) => ("move", vec![x, y]),
            Step::Line(x, y) => ("line", vec![x, y]),
            Step::Arc {
                x,
                y,
                radius,
                start,
                end,
                ccw,
            } => (
                if ccw { "counterclockwise arc" } else { "arc" },
                vec![x, y, radius, start, end],
            ),
            Step::Close => ("close", Vec::new()),
        }
    }

    /// `path` is `expected`, each number within a thousandth of a point (the `f32` sums a slot goes through).
    fn assert_traces(path: &[Step], expected: &[Step], what: &str) {
        let same = path.len() == expected.len()
            && path.iter().zip(expected).all(|(step, wanted)| {
                let (kind, found) = numbers(step);
                let (wanted_kind, wanted_numbers) = numbers(wanted);
                kind == wanted_kind
                    && found
                        .iter()
                        .zip(&wanted_numbers)
                        .all(|(a, b)| (a - b).abs() < 1e-3)
            });
        assert!(same, "{what}:\n{path:?}\nis not\n{expected:?}");
    }

    /// The animation graph with every operation `recording` holds so far.
    fn replay(recording: &Recording) -> Replay {
        let mut replay = Replay::default();
        replay.apply(&recording.items());
        replay
    }

    /// The paths the container's canvas and the state layer's draw now.
    fn drawn(recording: &Recording) -> (Vec<Vec<Step>>, Vec<Vec<Step>>, Size<Points>) {
        let mounted = Mounted::of(&recording.items());
        let replay = replay(recording);
        (
            paths(mounted.drawing(mounted.container), &replay),
            paths(mounted.drawing(mounted.layer), &replay),
            mounted.size(),
        )
    }

    /// The outline's width, halved: it is stroked that far inside the container's edge, so that it lies within the
    /// container as a view's border does, its corners concentric with the container's.
    const HALF_OUTLINE: f32 = 0.5;

    /// `ContainerShapeSquare`'s radius at each size: `CornerMedium` (12) for the extra small and small sizes,
    /// `CornerLarge` (16) for the medium, `CornerExtraLarge` (28) above. `PressedContainerShape`'s, for both shapes:
    /// `CornerSmall` (8) for the extra small and small, `CornerMedium` (12) for the medium, `CornerLarge` (16) above.
    const CORNERS: [(ButtonSize, f32, f32); 5] = [
        (ButtonSize::ExtraSmall, 12.0, 8.0),
        (ButtonSize::Small, 12.0, 8.0),
        (ButtonSize::Medium, 16.0, 12.0),
        (ButtonSize::Large, 28.0, 16.0),
        (ButtonSize::ExtraLarge, 28.0, 16.0),
    ];

    /// The radius of the corners at rest: `ContainerShapeRound` is `CornerFull` at every size, half of the shorter
    /// side, the height for a label as wide as "Save".
    fn rest_radius(shape: ButtonShape, square: f32, size: Size<Points>) -> f32 {
        match shape {
            ButtonShape::Square => square,
            _ => size.width.0.min(size.height.0) / 2.0,
        }
    }

    #[test]
    fn each_shape_draws_its_corners_on_the_container_and_its_state_layer() {
        for (size, square, _) in CORNERS {
            for shape in [ButtonShape::Round, ButtonShape::Square] {
                for style in [ButtonStyle::Filled, ButtonStyle::Outlined] {
                    let what = format!("{size:?} {shape:?} {style:?}");
                    let (_app, recording) = start(Config {
                        size,
                        style,
                        shape,
                        ..Config::default()
                    });
                    let (container, layer, frame) = drawn(&recording);
                    assert!(frame.width > frame.height, "{what}: {frame:?}");
                    let radius = rest_radius(shape, square, frame);
                    // The container is filled inside its edge; the outlined one's outline is stroked inside that.
                    let mut expected = vec![rounded_rectangle(frame, 0.0, radius)];
                    if style == ButtonStyle::Outlined {
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
                    // The state layer keeps the container's corners.
                    assert_eq!(layer.len(), 1, "{what}: {layer:?}");
                    assert_traces(&layer[0], &expected[0], &what);
                    // No view rounds itself: the drawing gives the corners.
                    let mounted = Mounted::of(&recording.items());
                    for id in [mounted.button, mounted.container, mounted.layer] {
                        assert_eq!(radii(&mounted.style(id)), [0.0; 4], "{what}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_full_radius_is_half_the_shorter_side_whichever_it_is() {
        // With no label, the small button is its side spaces wide, 16 and 16: narrower than its 40 points of height.
        // Its round corners take half the width once its layout says so, and never more than the side can hold.
        let (_app, recording) = start(Config {
            label: "",
            ..Config::default()
        });
        let (container, layer, frame) = drawn(&recording);
        assert_eq!((frame.width.0, frame.height.0), (32.0, 40.0));
        let expected = rounded_rectangle(frame, 0.0, 16.0);
        assert_traces(&container[0], &expected, "container");
        assert_traces(&layer[0], &expected, "layer");
    }

    #[test]
    fn every_button_shape_can_morph() {
        // A shape with uneven corners would fall back to full corners: none of the button's does.
        for (size, ..) in CORNERS {
            let spec = size_spec(size);
            for shape in [spec.round, spec.square, spec.pressed] {
                assert!(Corners::of(shape).is_some(), "{size:?}: {shape:?}");
            }
            assert_eq!(corners(spec.round), Corners::Full, "{size:?}");
        }
        for (size, square, pressed) in CORNERS {
            assert_eq!(
                corners(size_spec(size).square),
                Corners::Radius(Points(square)),
                "{size:?}"
            );
            assert_eq!(
                corners(size_spec(size).pressed),
                Corners::Radius(Points(pressed)),
                "{size:?}"
            );
        }
    }

    /// The step of a press that goes down on the button and stays down: a finger that lands and does not move.
    fn press_down(app: &mut Headless) -> String {
        let still = point(px(0.0), px(0.0));
        app.drag_and_hold("Save", still).unwrap();
        format!("dragAndHold(Save, {}, {})", still.x, still.y)
    }

    /// Step `step` springs the corners once, with the morph's spring (`DefaultEffects`, androidx `Button.kt`'s
    /// `shapeByInteraction`, converted as ADR 0004 says), to `to`, from where they are and as fast as they move, at
    /// the screen's rate; and nothing else in the step starts or sets a value.
    fn spring_in(items: &[MountItem], step: &str, to: f32) -> StartedSpring {
        let springs = started_springs(items);
        let [spring] = springs[..] else {
            panic!("{step}: one spring, not {springs:?}");
        };
        let spec = SpringToken::DefaultEffects.spec();
        assert_eq!(spring.to.to_bits(), to.to_bits(), "{step}: {spring:?}");
        assert_eq!(spring.stiffness.to_bits(), spec.stiffness.to_bits());
        assert_eq!(
            spring.damping.to_bits(),
            spec.damping_coefficient().to_bits()
        );
        assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
        assert_eq!(spring.velocity, None, "{step}");
        assert_eq!(spring.frame_rate, None, "{step}");
        assert_eq!(set_values(items), 0, "{step}: {items:?}");
        spring
    }

    /// The press of `step` springs the corners as [`spring_in`] says, and changes nothing else but the state
    /// layer's opacity.
    fn morph_in(recording: &Recording, step: &str, to: f32) -> StartedSpring {
        let items = recording.in_step(step);
        let spring = spring_in(&items, step, to);
        // The platform moves the corners: the drawing is not sent again, and nothing is laid out.
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

    /// How many values `items` set at once.
    fn set_values(items: &[MountItem]) -> usize {
        items
            .iter()
            .filter(|item| matches!(item, MountItem::Animation(AnimationOp::Set { .. })))
            .count()
    }

    /// The spring's `k`, `c` and `m`, as the closed form takes them.
    fn parameters(spring: &StartedSpring) -> (f64, f64, f64) {
        (
            f64::from(spring.stiffness),
            f64::from(spring.damping),
            f64::from(spring.mass),
        )
    }

    /// Tolerance on a replayed curve: 0.1 % of its travel (ADR 0010).
    fn tolerance(from: f64, to: f64) -> f64 {
        (to - from).abs() / 1000.0
    }

    #[test]
    fn a_press_springs_the_corners_to_the_pressed_shape_and_the_drawing_follows() {
        for (size, square, pressed) in CORNERS {
            for shape in [ButtonShape::Round, ButtonShape::Square] {
                let what = format!("{size:?} {shape:?}");
                let (mut app, recording) = start(Config {
                    size,
                    shape,
                    style: ButtonStyle::Outlined,
                    ..Config::default()
                });
                let frame = Mounted::of(&recording.items()).size();
                let rest = rest_radius(shape, square, frame);
                let step = press_down(&mut app);
                let spring = morph_in(&recording, &step, pressed);
                // While it plays, the engine sends nothing (ADR 0010).
                app.step("run");
                app.run_for(Duration::from_secs(1)).unwrap();
                assert!(recording.in_step("run").is_empty(), "{what}");

                let mut replay = replay(&recording);
                let mut samples = replay.run(spring.node, 9);
                // Mid-way, the container, its outline and the state layer are drawn with the corners where the
                // spring has them.
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
                assert_traces(&layer[0], &rounded_rectangle(frame, 0.0, now), &what);
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
        let rest = rest_radius(
            ButtonShape::Round,
            12.0,
            Mounted::of(&recording.items()).size(),
        );
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
        for (size, ..) in CORNERS {
            for shape in [ButtonShape::Round, ButtonShape::Square] {
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
        let at_rest = rounded_rectangle(frame, 0.0, frame.height.0 / 2.0);
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
        // androidx restarts the shape animation when the shapes change (`key(shapes)`): the corners go to the new
        // ones with no spring.
        let (mut app, recording) = start(Config::default());
        let (_, _, frame) = drawn(&recording);
        for (shape, radius) in [(ButtonShape::Square, 12.0), (ButtonShape::Round, 20.0)] {
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
    fn a_button_disabled_while_pressed_goes_back_to_its_shape_at_rest() {
        let (mut app, recording) = start(Config::default());
        let step = press_down(&mut app);
        let _ = morph_in(&recording, &step, 8.0);
        app.press("Disable").unwrap();
        let _ = spring_in(&recording.in_step("press(Disable)"), "press(Disable)", 20.0);
    }
    #[test]
    fn hover_shows_the_state_layer_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("Save")?;
        let hover = recording.in_step("hover(Save)");
        // The state layer at `StateTokens.HoverStateLayerOpacity`, in the label's color.
        let hovered = Mounted {
            styles: HashMap::new(),
            ..Mounted::of(&recording.items())
        };
        let layer: Vec<&ViewStyle> = hover
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == hovered.layer => Some(&**style),
                _ => None,
            })
            .collect();
        assert_eq!(layer.len(), 1);
        assert_eq!(layer[0].opacity, 0.08);
        assert_eq!(
            paint(&mounted.drawing(mounted.layer).ops, false),
            Some(color(scheme_color(ColorRole::OnPrimary, false), 255))
        );
        // Nothing else: no frame, no text, and neither the button nor its container is styled or drawn again. (NSH
        // paints a canvas with its view's style, so the state layer's drawing comes again with its opacity.)
        assert_eq!(layout_items(&hover), 0);
        assert!(
            hover.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } | MountItem::Canvas { id, .. } =>
                    *id == hovered.layer,
                _ => true,
            }),
            "{hover:?}"
        );
        Ok(())
    }

    #[test]
    fn the_sizes_and_styles_come_from_their_tokens() {
        for size in SIZES {
            let spec = size_spec(size);
            assert!(spec.leading.0 > 0.0 && spec.trailing.0 > 0.0, "{size:?}");
            assert_eq!(
                spec.round,
                crate::tokens::androidx::shape_tokens::CORNER_FULL,
                "{size:?}"
            );
        }
        assert_eq!(
            size_spec(ButtonSize::default()),
            size_spec(ButtonSize::Small)
        );
        assert_eq!(ButtonStyle::default(), ButtonStyle::Filled);
        assert_eq!(ButtonShape::default(), ButtonShape::Round);
        for style in STYLES {
            let spec = style_spec(style);
            // Only the outlined button has an outline, and only the filled and tonal buttons a container.
            assert_eq!(
                spec.outline.is_some(),
                style == ButtonStyle::Outlined,
                "{style:?}"
            );
            assert_eq!(
                spec.disabled_outline.is_some(),
                style == ButtonStyle::Outlined,
                "{style:?}"
            );
            assert_eq!(
                spec.container.is_some(),
                matches!(style, ButtonStyle::Filled | ButtonStyle::Tonal),
                "{style:?}"
            );
            assert_eq!(
                spec.container.is_some(),
                spec.disabled_container.is_some(),
                "{style:?}"
            );
        }
        assert_eq!(OUTLINE_WIDTH, Points(1.0));
    }

    /// The vendored `Button.kt`, at the pinned commit (ADR 0012).
    const BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Button.kt");

    /// Whether `Button.kt` holds `lines`, each trimmed, one after the other.
    fn cites(lines: &[&str]) -> bool {
        let file: Vec<&str> = BUTTON_KT.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_button_kt_is_still_there() {
        // When androidx corrects a token, `Button.kt` drops its literal, and this fails in the bump's pull request.
        assert!(cites(&[
            "// TODO update with the value from ButtonXSmallTokens.kt once it's been corrected",
            "start = 12.dp,",
            "end = 12.dp,",
        ]));
        assert_eq!(EXTRA_SMALL_SIDE_SPACE, Points(12.0));
        assert_ne!(EXTRA_SMALL_SIDE_SPACE, button_xsmall_tokens::LEADING_SPACE);
        assert!(cites(&[
            "// TODO replace with the token value once it's corrected",
            "contentColor = fromToken(ColorSchemeKeyTokens.Primary),",
        ]));
        assert_eq!(TEXT_BUTTON_LABEL, ColorRole::Primary);
        assert_ne!(TEXT_BUTTON_LABEL, text_button_tokens::LABEL_COLOR);
        // The tonal button's file, and the outline's width and disabled opacity.
        assert!(cites(&[
            "containerColor = fromToken(FilledTonalButtonTokens.ContainerColor),"
        ]));
        assert!(cites(&[
            "width = ButtonSmallTokens.OutlinedOutlineWidth,",
            "color =",
            "if (enabled) {",
            "OutlinedButtonTokens.OutlineColor.value",
            "} else {",
            "OutlinedButtonTokens.OutlineColor.value.copy(",
            "alpha = OutlinedButtonTokens.DisabledContainerOpacity",
        ]));
        // The label styles by height, and the state layer's color is the content color the ripple takes.
        assert!(cites(&[
            "buttonHeight < mediumHeight -> MaterialTheme.typography.labelLarge",
            "buttonHeight < largeHeight -> MaterialTheme.typography.titleMedium",
            "buttonHeight < xLargeHeight -> MaterialTheme.typography.headlineSmall",
            "else -> MaterialTheme.typography.headlineLarge",
        ]));
        // A line that is not there is not found.
        assert!(!cites(&["start = 16.dp,", "end = 16.dp,", "top = 6.dp,"]));
    }
}
