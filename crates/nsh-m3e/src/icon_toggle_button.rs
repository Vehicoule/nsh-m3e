//! The Material 3 Expressive icon toggle button: an icon button ([`crate::icon_button`]) that is checked or not, which
//! morphs between its shape at rest, its shape while pressed and its shape while checked.
//!
//! The icon toggle button follows androidx's Expressive `IconToggleButton(checked, onCheckedChange, shapes, …)` and its
//! siblings (`FilledIconToggleButton`, `FilledTonalIconToggleButton`, `OutlinedIconToggleButton`) in `IconButton.kt` and
//! `IconButtonDefaults.kt`, at the pinned commit, as [ADR 0012] says. It has the icon button's sizes, widths, shapes and
//! color styles; checked, it takes the size's selected shape (`IconButtonDefaults.*SelectedRoundShape` or
//! `*SelectedSquareShape`) and the style's selected colors. Its container is drawn by a canvas whose corner radius a
//! spring moves ([ADR 0011]), with `MotionSchemeKeyTokens.DefaultEffects` as `IconButton.kt` animates it, converted as
//! [ADR 0004] says; its icon is drawn from path data ([ADR 0017]).
//!
//! As the icon button does, it takes the colors androidx names "vibrant" for the standard and outlined styles
//! (`iconToggleButtonVibrantColors`, `outlinedIconToggleButtonVibrantColors` and `outlinedIconToggleButtonVibrantBorder`,
//! the defaults of the Expressive overloads), since NSH has no content color of the component around it.
//!
//! Its behavior is `nsh-base`'s `Toggle`: presses, Space and Return, focus, the disabled state, and for screen readers a
//! button with a pressed state (androidx says `Role.Checkbox`), named by its label. It reports the value a press asks
//! for and shows the one the app passes back.
//!
//! [ADR 0004]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0004-spring-damping-conversion.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use nsh_core::geometry::Size;
use nsh_core::prelude::*;

use crate::icon::Icon;
use crate::icon_button::{
    IconButtonShape, IconButtonSize, IconButtonStyle, IconButtonWidth, OUTLINE_WIDTH, size_spec,
    touch_slop,
};
use crate::morph::{Corners, DrawnShape, corners};
use crate::motion::SpringToken;
use crate::tint::{Tint, tint_color};
use crate::tokens::Shape;
use crate::tokens::androidx::{
    filled_icon_button_tokens, filled_tonal_icon_button_tokens, large_icon_button_tokens,
    medium_icon_button_tokens, outlined_icon_button_tokens, small_icon_button_tokens,
    standard_icon_button_tokens, state_tokens, xlarge_icon_button_tokens,
    xsmall_icon_button_tokens,
};

/// What a color style gives the icon toggle button: androidx `IconToggleButtonColors` and the outline.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorSpec {
    /// The container when enabled and not checked; none is transparent.
    container: Option<Tint>,
    /// The icon when enabled and not checked.
    content: Tint,
    /// The container when enabled and checked; none is transparent.
    checked_container: Option<Tint>,
    /// The icon when enabled and checked.
    checked_content: Tint,
    /// The container when disabled, checked or not; none is transparent.
    disabled_container: Option<Tint>,
    /// The icon when disabled, checked or not.
    disabled_content: Tint,
    /// The outline when enabled and not checked; none is no outline.
    outline: Option<Tint>,
    /// The outline when disabled and not checked.
    disabled_outline: Option<Tint>,
}

/// The colors of `style`, as androidx builds them: `defaultIconToggleButtonVibrantColors`,
/// `defaultOutlinedIconToggleButtonVibrantColors` with `outlinedIconToggleButtonVibrantBorder`,
/// `defaultFilledTonalIconToggleButtonColors` and `defaultFilledIconToggleButtonColors`.
fn color_spec(style: IconButtonStyle) -> ColorSpec {
    match style {
        IconButtonStyle::Filled => ColorSpec {
            container: Some(Tint::solid(
                filled_icon_button_tokens::UNSELECTED_CONTAINER_COLOR,
            )),
            content: Tint::solid(filled_icon_button_tokens::UNSELECTED_COLOR),
            checked_container: Some(Tint::solid(
                filled_icon_button_tokens::SELECTED_CONTAINER_COLOR,
            )),
            checked_content: Tint::solid(filled_icon_button_tokens::SELECTED_COLOR),
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
        IconButtonStyle::Tonal => ColorSpec {
            container: Some(Tint::solid(
                filled_tonal_icon_button_tokens::UNSELECTED_CONTAINER_COLOR,
            )),
            content: Tint::solid(filled_tonal_icon_button_tokens::UNSELECTED_COLOR),
            checked_container: Some(Tint::solid(
                filled_tonal_icon_button_tokens::SELECTED_CONTAINER_COLOR,
            )),
            checked_content: Tint::solid(filled_tonal_icon_button_tokens::SELECTED_COLOR),
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
        IconButtonStyle::Outlined => ColorSpec {
            container: None,
            content: Tint::solid(outlined_icon_button_tokens::UNSELECTED_COLOR),
            checked_container: Some(Tint::solid(
                outlined_icon_button_tokens::SELECTED_CONTAINER_COLOR,
            )),
            checked_content: Tint::solid(outlined_icon_button_tokens::SELECTED_COLOR),
            // The vibrant colors leave a disabled one transparent, checked or not, where the token file names
            // `SelectedDisabledContainerColor` (ADR 0012).
            disabled_container: None,
            disabled_content: Tint::faded(
                outlined_icon_button_tokens::DISABLED_COLOR,
                outlined_icon_button_tokens::DISABLED_OPACITY,
            ),
            // `outlinedIconButtonVibrantBorder`, as for the icon button.
            outline: Some(Tint::solid(outlined_icon_button_tokens::OUTLINE_COLOR)),
            disabled_outline: Some(Tint::faded(
                outlined_icon_button_tokens::OUTLINE_COLOR,
                outlined_icon_button_tokens::DISABLED_OPACITY,
            )),
        },
        IconButtonStyle::Standard => ColorSpec {
            container: None,
            content: Tint::solid(standard_icon_button_tokens::UNSELECTED_COLOR),
            checked_container: None,
            checked_content: Tint::solid(standard_icon_button_tokens::SELECTED_COLOR),
            disabled_container: None,
            disabled_content: Tint::faded(
                standard_icon_button_tokens::DISABLED_COLOR,
                standard_icon_button_tokens::DISABLED_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
    }
}

impl ColorSpec {
    /// The container's color: androidx `IconToggleButtonColors.containerColor(enabled, checked)`.
    fn container(self, disabled: bool, checked: bool) -> Option<Tint> {
        match (disabled, checked) {
            (true, _) => self.disabled_container,
            (false, true) => self.checked_container,
            (false, false) => self.container,
        }
    }

    /// The icon's color: androidx `IconToggleButtonColors.contentColor(enabled, checked)`.
    fn content(self, disabled: bool, checked: bool) -> Tint {
        match (disabled, checked) {
            (true, _) => self.disabled_content,
            (false, true) => self.checked_content,
            (false, false) => self.content,
        }
    }

    /// The outline's color: androidx `outlinedIconToggleButtonVibrantBorder(enabled, checked)` draws none when
    /// checked.
    fn outline(self, disabled: bool, checked: bool) -> Option<Tint> {
        match (disabled, checked) {
            (_, true) => None,
            (true, false) => self.disabled_outline,
            (false, false) => self.outline,
        }
    }
}

/// The shapes of `size` while checked, round and square: androidx `IconButtonDefaults.*SelectedRoundShape` and
/// `*SelectedSquareShape`, the size's `SelectedContainerShapeRound` and `SelectedContainerShapeSquare`.
fn selected_shapes(size: IconButtonSize) -> (Shape, Shape) {
    match size {
        IconButtonSize::ExtraSmall => (
            xsmall_icon_button_tokens::SELECTED_CONTAINER_SHAPE_ROUND,
            xsmall_icon_button_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        ),
        IconButtonSize::Small => (
            small_icon_button_tokens::SELECTED_CONTAINER_SHAPE_ROUND,
            small_icon_button_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        ),
        IconButtonSize::Medium => (
            medium_icon_button_tokens::SELECTED_CONTAINER_SHAPE_ROUND,
            medium_icon_button_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        ),
        IconButtonSize::Large => (
            large_icon_button_tokens::SELECTED_CONTAINER_SHAPE_ROUND,
            large_icon_button_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        ),
        IconButtonSize::ExtraLarge => (
            xlarge_icon_button_tokens::SELECTED_CONTAINER_SHAPE_ROUND,
            xlarge_icon_button_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        ),
    }
}

/// The `[rest, pressed, checked]` corners of `size` at rest in `shape`: androidx `IconButtonDefaults.toggleableShapes`
/// with the size's round or square shape, its pressed shape, and its selected round or square shape.
fn shapes(size: IconButtonSize, shape: IconButtonShape) -> [Corners; 3] {
    let (selected_round, selected_square) = selected_shapes(size);
    let size = size_spec(size);
    let (rest, checked) = match shape {
        IconButtonShape::Round => (size.round, selected_round),
        IconButtonShape::Square => (size.square, selected_square),
    };
    [corners(rest), corners(size.pressed), corners(checked)]
}

/// The props of an [`IconToggleButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct IconToggleButtonProps {
    /// The icon it shows.
    pub icon: Prop<Icon>,
    /// What screen readers say: an icon toggle button shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// Whether it is checked. The app owns the value: a press reports the next one with
    /// [`IconToggleButtonEvent::Change`], and the icon toggle button shows the value passed back.
    #[prop(default)]
    pub checked: Prop<bool>,
    /// Its size: the icon button's sizes.
    #[prop(default)]
    pub size: Prop<IconButtonSize>,
    /// Its width: the icon button's widths.
    #[prop(default)]
    pub width: Prop<IconButtonWidth>,
    /// Its color style: the icon button's styles.
    #[prop(default)]
    pub style: Prop<IconButtonStyle>,
    /// Its shape at rest, which also gives its shape while checked.
    #[prop(default)]
    pub shape: Prop<IconButtonShape>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of an [`IconToggleButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum IconToggleButtonEvent {
    /// It was pressed: the value it asks for, the opposite of `checked` (androidx `onCheckedChange`).
    #[args(checked)]
    Change(bool),
}

/// A Material 3 Expressive icon toggle button: an icon in a container of the size, width, shape and color style given,
/// checked or not.
///
/// Hover, focus and press lay the icon's color over it at the state layer opacities of `StateTokens`, as androidx's
/// ripple does, and change nothing else. A press morphs its container into the pressed shape; being checked, into the
/// checked one.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::icon::Icon;
/// use nsh_m3e::icon_toggle_button::icon_toggle_button;
///
/// fn favorite() -> impl IntoElement {
///     let favorite = signal(false);
///     icon_toggle_button()
///         // Material Symbols' `add`.
///         .icon(Icon::material_symbol(
///             "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z",
///         ))
///         .label("Favorite")
///         .checked(move || favorite.get())
///         .on_change(move |checked: &bool| favorite.set(*checked))
/// }
/// ```
#[component]
pub fn IconToggleButton(
    props: IconToggleButtonProps,
    cx: Scope<IconToggleButtonEvent>,
) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || color_spec(props.style.get());
    let frame = move || {
        let size = size();
        Size::new(size.width(props.width.get()), size.height)
    };
    let content = move || {
        tint_color(Some(
            colors().content(props.disabled.get(), props.checked.get()),
        ))
    };
    let drawn = untracked(|| {
        DrawnShape::toggle(
            shapes(props.size.get(), props.shape.get()),
            props.checked.get(),
            frame(),
            SpringToken::DefaultEffects,
        )
    });
    effect(move || {
        let [rest, pressed, checked] = shapes(props.size.get(), props.shape.get());
        drawn.set_corners(rest, pressed);
        drawn.set_checked_corners(checked);
    });
    effect(move || drawn.set_checked(props.checked.get()));
    effect(move || {
        // An icon toggle button disabled while pressed may get no end of its press: it leaves the pressed shape.
        if props.disabled.get() {
            drawn.press(false);
        }
    });
    let outline = move || {
        colors()
            .outline(props.disabled.get(), props.checked.get())
            .map(|_| OUTLINE_WIDTH)
    };
    nsh_base::toggle()
        .pressed(move || props.checked.get())
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_change(move |checked: &bool| cx.emit(IconToggleButtonEvent::Change(*checked)))
        .on_press_down(move || drawn.press(true))
        .on_press_up(move || drawn.press(false))
        .hit_slop(move || touch_slop(frame()))
        .group()
        .items_center()
        .justify_center()
        .w(move || frame().width)
        .h(move || frame().height)
        .child(
            drawn
                .container(outline)
                .bg(move || {
                    tint_color(colors().container(props.disabled.get(), props.checked.get()))
                })
                .stroke_color(move || {
                    tint_color(colors().outline(props.disabled.get(), props.checked.get()))
                }),
        )
        .child(
            // The state layer, as the icon button's: androidx's ripple in the content color.
            drawn
                .state_layer(outline)
                .bg(content)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| if props.disabled.get() { s.hidden() } else { s }),
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
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::{Bounds, Points};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Checked, Color, MountItem, NodeId, ViewKind};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        Recording, StartedSpring, layout_items, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph_probe::{
        Step, assert_traces, paths, replay, rounded_rectangle, set_values, tolerance,
    };
    use crate::motion::SPRING_MASS;

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ADD: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";

    /// The selector of the icon toggle button: its label.
    const BUTTON: &str = "label:Favorite";

    /// What the probe places: one icon toggle button, lines saying whether it is checked and how many changes it asked
    /// for, and lines that check, uncheck, reshape and disable it from the app.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        checked: Prop<bool>,
        #[prop(default)]
        size: Prop<IconButtonSize>,
        #[prop(default)]
        style: Prop<IconButtonStyle>,
        #[prop(default)]
        shape: Prop<IconButtonShape>,
        #[prop(default)]
        disabled: Prop<bool>,
        /// The app does not pass back the value a press asks for.
        #[prop(default)]
        ignore: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let checked = signal(props.checked.get_untracked());
        let changes = signal(0_u32);
        let disabled = signal(props.disabled.get_untracked());
        let shape = signal(props.shape.get_untracked());
        let ignore = props.ignore.get_untracked();
        // A row, so that the icon toggle button keeps its own size.
        view()
            .child(
                view().flex_row().child(
                    icon_toggle_button()
                        .icon(Icon::material_symbol(ADD))
                        .label("Favorite")
                        .checked(move || checked.get())
                        .size(props.size)
                        .style(props.style)
                        .shape(move || shape.get())
                        .disabled(move || disabled.get())
                        .on_change(move |next: &bool| {
                            changes.update(|n| *n += 1);
                            if !ignore {
                                checked.set(*next);
                            }
                        }),
                ),
            )
            .child(text(move || format!("checked: {}", checked.get())))
            .child(text(move || format!("changes: {}", changes.get())))
            .child(text("Check").on_press(move |(): &()| checked.set(true)))
            .child(text("Uncheck").on_press(move |(): &()| checked.set(false)))
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
        checked: bool,
        size: IconButtonSize,
        style: IconButtonStyle,
        shape: IconButtonShape,
        disabled: bool,
        answer: Answer,
        appearance: Appearance,
        reduce_motion: bool,
    }

    /// What the app does with the value a press asks for.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Answer {
        /// It passes it back: the icon toggle button shows it.
        #[default]
        PassesBack,
        /// It ignores it: the icon toggle button stays as it was.
        Ignores,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            checked: Prop::new(config.checked),
            size: Prop::new(config.size),
            style: Prop::new(config.style),
            shape: Prop::new(config.shape),
            disabled: Prop::new(config.disabled),
            ignore: Prop::new(config.answer == Answer::Ignores),
        });
        // The web has a mouse to hover with.
        let environment = Environment {
            os: Os::Web,
            color_scheme: config.appearance,
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
        app.wait_for("text:changes: 0").unwrap();
        (app, recording)
    }

    /// The icon toggle button's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole icon toggle button.
        button: NodeId,
        /// The canvas that draws the container.
        container: NodeId,
        /// The canvas that draws the state layer.
        layer: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        opacities: HashMap<NodeId, f32>,
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
            let mut opacities = HashMap::new();
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
                        opacities.insert(*id, style.opacity);
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
                panic!("three canvases in the icon toggle button: {items:?}");
            };
            assert_eq!(children.get(&button).map(Vec::len), Some(3), "{items:?}");
            Self {
                button,
                container,
                layer,
                icon,
                opacities,
                frames,
                drawings,
                label,
                hit_slop,
            }
        }

        fn drawing(&self, id: NodeId) -> &Drawing {
            &self.drawings[&id]
        }

        /// The icon toggle button's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.button].size
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

    /// The 8-bit alphas NSH rounds the token opacities to: 0.38 and 0.1 of 255.
    const ALPHA_38: u32 = 97;
    const ALPHA_10: u32 = 26;

    const STYLES: [IconButtonStyle; 4] = [
        IconButtonStyle::Standard,
        IconButtonStyle::Outlined,
        IconButtonStyle::Tonal,
        IconButtonStyle::Filled,
    ];

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

    /// The colors each style should paint, in the baseline schemes: the container, the outline's width and color, and
    /// the icon.
    fn expected_colors(
        style: IconButtonStyle,
        checked: bool,
        disabled: bool,
        dark: bool,
    ) -> (Color, f32, Color, Color) {
        let solid = |role| color(scheme_color(role, dark), 255);
        let faded = |role, alpha| color(scheme_color(role, dark), alpha);
        let none = Color(0);
        match (style, checked, disabled) {
            // androidx `defaultIconToggleButtonVibrantColors` (`StandardIconButtonTokens`): on surface variant,
            // primary checked; never a container.
            (IconButtonStyle::Standard, false, false) => {
                (none, 0.0, none, solid(ColorRole::OnSurfaceVariant))
            }
            (IconButtonStyle::Standard, true, false) => {
                (none, 0.0, none, solid(ColorRole::Primary))
            }
            // `defaultOutlinedIconToggleButtonVibrantColors` and `outlinedIconToggleButtonVibrantBorder`: an outline
            // variant outline 1 point wide, faded to 38 % disabled; inverse surface and inverse on surface checked,
            // with no outline; never a container when disabled.
            (IconButtonStyle::Outlined, false, false) => (
                none,
                1.0,
                solid(ColorRole::OutlineVariant),
                solid(ColorRole::OnSurfaceVariant),
            ),
            (IconButtonStyle::Outlined, true, false) => (
                solid(ColorRole::InverseSurface),
                0.0,
                none,
                solid(ColorRole::InverseOnSurface),
            ),
            (IconButtonStyle::Outlined, false, true) => (
                none,
                1.0,
                faded(ColorRole::OutlineVariant, ALPHA_38),
                faded(ColorRole::OnSurface, ALPHA_38),
            ),
            // Disabled with no outline: no container either, the icon on surface at 38 %.
            (IconButtonStyle::Standard, _, true) | (IconButtonStyle::Outlined, true, true) => {
                (none, 0.0, none, faded(ColorRole::OnSurface, ALPHA_38))
            }
            // `defaultFilledTonalIconToggleButtonColors`: secondary container and on secondary container, secondary
            // and on secondary checked.
            (IconButtonStyle::Tonal, false, false) => (
                solid(ColorRole::SecondaryContainer),
                0.0,
                none,
                solid(ColorRole::OnSecondaryContainer),
            ),
            (IconButtonStyle::Tonal, true, false) => (
                solid(ColorRole::Secondary),
                0.0,
                none,
                solid(ColorRole::OnSecondary),
            ),
            // `defaultFilledIconToggleButtonColors`: surface container and on surface variant, primary and on primary
            // checked.
            (IconButtonStyle::Filled, false, false) => (
                solid(ColorRole::SurfaceContainer),
                0.0,
                none,
                solid(ColorRole::OnSurfaceVariant),
            ),
            (IconButtonStyle::Filled, true, false) => (
                solid(ColorRole::Primary),
                0.0,
                none,
                solid(ColorRole::OnPrimary),
            ),
            // Both disabled, checked or not: on surface at 10 % and 38 %.
            (_, _, true) => (
                faded(ColorRole::OnSurface, ALPHA_10),
                0.0,
                none,
                faded(ColorRole::OnSurface, ALPHA_38),
            ),
        }
    }

    #[test]
    fn each_style_has_its_colors_checked_or_not_enabled_or_not_light_and_dark() {
        for style in STYLES {
            for checked in [false, true] {
                for disabled in [false, true] {
                    for appearance in [Appearance::Light, Appearance::Dark] {
                        let dark = appearance == Appearance::Dark;
                        let what =
                            format!("{style:?} checked {checked} disabled {disabled} dark {dark}");
                        let mounted = mount(Config {
                            checked,
                            style,
                            disabled,
                            appearance,
                            ..Config::default()
                        });
                        assert_eq!(
                            colors(&mounted),
                            expected_colors(style, checked, disabled, dark),
                            "{what}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_press_asks_for_the_other_value_and_shows_the_one_passed_back() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        assert_eq!(
            Mounted::of(&recording.items()).label.as_deref(),
            Some("Favorite")
        );
        let pressed = |app: &Headless| {
            app.screen()
                .accessibility_state("role:button[Favorite]")
                .map(|state| state.pressed)
        };
        assert_eq!(pressed(&app)?, Some(Checked::False));
        app.press(BUTTON)?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("checked: true"));
        assert_eq!(pressed(&app)?, Some(Checked::True));
        app.press(BUTTON)?;
        assert!(app.shows("changes: 2"));
        assert!(app.shows("checked: false"));
        assert_eq!(pressed(&app)?, Some(Checked::False));
        Ok(())
    }

    #[test]
    fn an_app_that_ignores_the_change_keeps_it_as_it_was() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            style: IconButtonStyle::Filled,
            ..Config::default()
        });
        let before = colors(&Mounted::of(&recording.items()));
        app.press(BUTTON)?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("checked: false"));
        assert_eq!(colors(&Mounted::of(&recording.items())), before);
        Ok(())
    }

    #[test]
    fn a_disabled_icon_toggle_button_takes_no_press_and_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press(BUTTON);
        assert!(app.shows("changes: 0"));
        let mounted = Mounted::of(&recording.items());
        // The state layer is hidden: laid out at no size.
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::new(Points(0.0), Points(0.0)))
        );
        assert_eq!(
            app.screen().enabled("role:button[Favorite]"),
            Ok(false),
            "screen readers say it is dimmed"
        );
    }

    /// Every size: its height, and its round, square, pressed, selected round and selected square radii (`None` is
    /// fully rounded). androidx `XSmallIconButtonTokens` to `XLargeIconButtonTokens`.
    type SizeRow = (IconButtonSize, f32, [Option<f32>; 5]);

    const SIZES: [SizeRow; 5] = [
        (
            IconButtonSize::ExtraSmall,
            32.0,
            [None, Some(12.0), Some(8.0), Some(12.0), None],
        ),
        (
            IconButtonSize::Small,
            40.0,
            [None, Some(12.0), Some(8.0), Some(12.0), None],
        ),
        (
            IconButtonSize::Medium,
            56.0,
            [None, Some(16.0), Some(12.0), Some(16.0), None],
        ),
        (
            IconButtonSize::Large,
            96.0,
            [None, Some(28.0), Some(16.0), Some(28.0), None],
        ),
        (
            IconButtonSize::ExtraLarge,
            136.0,
            [None, Some(28.0), Some(16.0), Some(28.0), None],
        ),
    ];

    fn radius(radius: Option<f32>) -> Corners {
        radius.map_or(Corners::Full, |radius| Corners::Radius(Points(radius)))
    }

    #[test]
    fn each_size_and_shape_rests_presses_and_checks_in_its_own_corners() {
        for (size, height, [round, square, pressed, selected_round, selected_square]) in SIZES {
            assert_eq!(
                shapes(size, IconButtonShape::Round),
                [radius(round), radius(pressed), radius(selected_round)],
                "{size:?}"
            );
            assert_eq!(
                shapes(size, IconButtonShape::Square),
                [radius(square), radius(pressed), radius(selected_square)],
                "{size:?}"
            );
            // The icon button's frame and its 48-point target.
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let frame = mounted.size();
            assert_eq!(frame.height, Points(height), "{size:?}");
            assert_eq!(
                frame.width,
                size_spec(size).width(IconButtonWidth::Uniform),
                "{size:?}"
            );
            assert_eq!(mounted.hit_slop, Some(touch_slop(frame)), "{size:?}");
        }
    }

    /// The container's and the state layer's paths, and the icon toggle button's size.
    fn drawn(recording: &Recording) -> (Vec<Vec<Step>>, Vec<Vec<Step>>, Size<Points>) {
        let mounted = Mounted::of(&recording.items());
        let replay = replay(recording);
        (
            paths(mounted.drawing(mounted.container), &replay),
            paths(mounted.drawing(mounted.layer), &replay),
            mounted.size(),
        )
    }

    /// The radius `corners` give a container of `frame`.
    fn radius_of(corners: Corners, frame: Size<Points>) -> f32 {
        match corners {
            Corners::Radius(radius) => radius.0,
            Corners::Full => frame.height.0.min(frame.width.0) / 2.0,
        }
    }

    #[test]
    fn it_mounts_with_the_corners_it_has_and_starts_nothing() {
        for (size, ..) in SIZES {
            for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                for checked in [false, true] {
                    let what = format!("{size:?} {shape:?} checked {checked}");
                    let (_app, recording) = start(Config {
                        checked,
                        size,
                        shape,
                        ..Config::default()
                    });
                    let items = recording.items();
                    assert_eq!(started_springs(&items), Vec::new(), "{what}");
                    assert_eq!(set_values(&items), 0, "{what}");
                    assert!(!replay(&recording).is_animating(), "{what}");
                    let (container, layer, frame) = drawn(&recording);
                    let [rest, _, checked_corners] = shapes(size, shape);
                    let corners = if checked { checked_corners } else { rest };
                    let expected = rounded_rectangle(frame, 0.0, radius_of(corners, frame));
                    assert_traces(&container[0], &expected, &what);
                    assert_traces(&layer[0], &expected, &what);
                }
            }
        }
    }

    /// A press that goes down on the icon toggle button and stays down: what it sent.
    fn press_down(app: &mut Headless, recording: &Recording) -> Vec<MountItem> {
        let (pressed, sent) =
            recording.during(|| app.drag_and_hold(BUTTON, point(px(0.0), px(0.0))));
        pressed.unwrap();
        sent
    }

    /// The press held ends: what it sent.
    fn release(app: &mut Headless, recording: &Recording) -> Vec<MountItem> {
        let (dropped, sent) = recording.during(|| app.drop_held());
        dropped.unwrap();
        sent
    }

    /// A press on `selector` (a line of the probe): what it sent.
    fn press(app: &mut Headless, recording: &Recording, selector: &str) -> Vec<MountItem> {
        let (pressed, sent) = recording.during(|| app.press(selector));
        pressed.unwrap();
        sent
    }

    /// `items` spring the corners to `to` with `DefaultEffects`, from where they are and as fast as they move, set
    /// nothing at once, and resize neither the icon toggle button nor its container. The last spring wins: a press's
    /// end and the change the app makes for it come in the same batch, each to its own corners.
    fn morph_in(recording: &Recording, items: &[MountItem], to: f32, what: &str) -> StartedSpring {
        let springs = started_springs(items);
        assert!(
            !springs.is_empty() && springs.len() <= 2,
            "{what}: {springs:?}"
        );
        let spec = SpringToken::DefaultEffects.spec();
        let last = springs[springs.len() - 1];
        for spring in &springs {
            assert_eq!(spring.node, last.node, "{what}");
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
        assert_eq!(last.to.to_bits(), to.to_bits(), "{what}: {springs:?}");
        assert_eq!(set_values(items), 0, "{what}: {items:?}");
        let mounted = Mounted::of(&recording.items());
        assert!(
            !items.iter().any(|item| matches!(
                item,
                MountItem::Frame { id, .. } if *id == mounted.button || *id == mounted.container
            )),
            "{what}: {items:?}"
        );
        last
    }

    #[test]
    fn a_press_springs_to_the_pressed_corners_and_its_end_to_the_checked_ones() {
        for (size, ..) in SIZES {
            for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                let what = format!("{size:?} {shape:?}");
                let (mut app, recording) = start(Config {
                    size,
                    shape,
                    ..Config::default()
                });
                let frame = Mounted::of(&recording.items()).size();
                let [rest, pressed, checked] = shapes(size, shape).map(|c| radius_of(c, frame));
                let sent = press_down(&mut app, &recording);
                let down = morph_in(&recording, &sent, pressed, &what);
                assert_eq!(started_springs(&sent).len(), 1, "{what}");
                // A press alone draws nothing again: the platform moves the corners.
                assert!(
                    !sent
                        .iter()
                        .any(|item| matches!(item, MountItem::Canvas { .. })),
                    "{what}: {sent:?}"
                );
                let mut replay = replay(&recording);
                let samples = replay.run(down.node, 180);
                let gap = worst_gap(&samples, f64::from(rest), &down);
                assert!(
                    gap < tolerance(f64::from(rest), f64::from(pressed)),
                    "{what}: {gap}"
                );
                // The press ends: the app checks it, and the corners go from the pressed ones to the checked ones.
                let sent = release(&mut app, &recording);
                assert!(app.shows("checked: true"), "{what}");
                let up = morph_in(&recording, &sent, checked, &what);
                assert_eq!(up.node, down.node, "{what}");
                replay.apply(&sent);
                let _ = replay.run(up.node, 600);
                assert!(!replay.is_animating(), "{what}");
                assert_eq!(replay.value(up.node).to_bits(), checked.to_bits(), "{what}");
                // Its next press and end take it back through the pressed corners to rest.
                let sent = press_down(&mut app, &recording);
                let _ = morph_in(&recording, &sent, pressed, &what);
                let sent = release(&mut app, &recording);
                assert!(app.shows("checked: false"), "{what}");
                let _ = morph_in(&recording, &sent, rest, &what);
            }
        }
    }

    #[test]
    fn a_change_from_the_app_springs_the_corners_and_waits_for_a_press_to_end() {
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            ..Config::default()
        });
        let sent = press(&mut app, &recording, "Check");
        let _ = morph_in(&recording, &sent, 12.0, "checked by the app");
        let sent = press(&mut app, &recording, "Uncheck");
        let _ = morph_in(&recording, &sent, 20.0, "unchecked by the app");
        // Checked by the app while a finger holds it down: the pressed corners stay until the press ends.
        let sent = press_down(&mut app, &recording);
        let _ = morph_in(&recording, &sent, 8.0, "pressed");
        let sent = press(&mut app, &recording, "Check");
        assert_eq!(started_springs(&sent), Vec::new(), "{sent:?}");
        assert_eq!(set_values(&sent), 0, "{sent:?}");
        let sent = release(&mut app, &recording);
        assert!(app.shows("checked: true"));
        let _ = morph_in(&recording, &sent, 12.0, "released checked");
    }

    #[test]
    fn with_reduce_motion_a_change_shows_at_once_and_a_press_moves_nothing() {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let (_, _, frame) = drawn(&recording);
        let sent = press_down(&mut app, &recording);
        assert_eq!(started_springs(&sent), Vec::new());
        assert_eq!(set_values(&sent), 0);
        let sent = release(&mut app, &recording);
        assert!(app.shows("checked: true"));
        assert_eq!(started_springs(&sent), Vec::new());
        assert_eq!(set_values(&sent), 1, "{sent:?}");
        let (container, layer, _) = drawn(&recording);
        let checked = rounded_rectangle(frame, 0.0, 12.0);
        assert_traces(&container[0], &checked, "container");
        assert_traces(&layer[0], &checked, "layer");
        // And unchecked by the app, at once too.
        let sent = press(&mut app, &recording, "Uncheck");
        assert_eq!(started_springs(&sent), Vec::new());
        assert_eq!(set_values(&sent), 1, "{sent:?}");
        let (container, ..) = drawn(&recording);
        assert_traces(
            &container[0],
            &rounded_rectangle(frame, 0.0, 20.0),
            "unchecked",
        );
    }

    #[test]
    fn a_new_shape_moves_the_corners_at_once_checked_or_not() {
        // androidx starts the shape animation again when the shapes change (`key(shapes)`): no spring.
        for (checked, square, round) in [(false, 12.0, 20.0), (true, 20.0, 12.0)] {
            let what = format!("checked {checked}");
            let (mut app, recording) = start(Config {
                checked,
                ..Config::default()
            });
            let (_, _, frame) = drawn(&recording);
            for radius in [square, round] {
                let sent = press(&mut app, &recording, "Reshape");
                assert_eq!(started_springs(&sent), Vec::new(), "{what}");
                assert!(set_values(&sent) >= 1, "{what}: {sent:?}");
                let (container, layer, _) = drawn(&recording);
                let expected = rounded_rectangle(frame, 0.0, radius);
                assert_traces(&container[0], &expected, &what);
                assert_traces(&layer[0], &expected, &what);
                assert!(!replay(&recording).is_animating(), "{what}");
            }
        }
    }

    #[test]
    fn an_icon_toggle_button_disabled_while_pressed_leaves_the_pressed_shape() {
        let (mut app, recording) = start(Config::default());
        let sent = press_down(&mut app, &recording);
        let _ = morph_in(&recording, &sent, 8.0, "pressed");
        let sent = press(&mut app, &recording, "Disable");
        let _ = morph_in(&recording, &sent, 20.0, "disabled");
    }

    #[test]
    fn the_outline_is_drawn_inside_the_container_and_only_unchecked() {
        let (mut app, recording) = start(Config {
            style: IconButtonStyle::Outlined,
            answer: Answer::Ignores,
            ..Config::default()
        });
        let (container, layer, frame) = drawn(&recording);
        assert_eq!(container.len(), 2, "the fill and the outline");
        assert_traces(
            &container[1],
            &rounded_rectangle(frame, 0.5, 19.5),
            "outline",
        );
        assert_traces(&layer[0], &rounded_rectangle(frame, 1.0, 19.0), "layer");
        let sent = press(&mut app, &recording, "Check");
        let spring = morph_in(&recording, &sent, 12.0, "checked");
        let mounted = Mounted::of(&recording.items());
        assert_eq!(colors(&mounted).1, 0.0);
        // Once the corners settle, the fill alone, and the state layer over all of it.
        let mut replayed = replay(&recording);
        let _ = replayed.run(spring.node, 600);
        let container = paths(mounted.drawing(mounted.container), &replayed);
        let layer = paths(mounted.drawing(mounted.layer), &replayed);
        assert_eq!(container.len(), 1, "the fill alone");
        let checked = rounded_rectangle(frame, 0.0, 12.0);
        assert_traces(&container[0], &checked, "checked container");
        assert_traces(&layer[0], &checked, "checked layer");
    }

    #[test]
    fn hover_shows_the_state_layer_in_the_icon_color_and_lays_out_nothing() -> Result<(), String> {
        for checked in [false, true] {
            let (mut app, recording) = start(Config {
                checked,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            assert_eq!(mounted.opacities.get(&mounted.layer), Some(&0.0));
            // In the icon's color.
            assert_eq!(
                paint(&mounted.drawing(mounted.layer).ops, false),
                Some(colors(&mounted).3),
                "checked {checked}"
            );
            app.hover(BUTTON)?;
            let hover = recording.in_step(&format!("hover({BUTTON})"));
            let opacity = Mounted::of(&recording.items()).opacities[&mounted.layer];
            assert!(
                (opacity - state_tokens::HOVER_STATE_LAYER_OPACITY).abs() < 1e-6,
                "checked {checked}: {opacity}"
            );
            assert_eq!(layout_items(&hover), 0, "checked {checked}");
        }
        Ok(())
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
        // The shape morph: `DefaultEffects`, and pressed over checked over rest.
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultEffects.value<Float>()",
                "val pressed by interactionSource.collectIsPressedAsState()",
                "",
                "return shapeByInteraction(shapes, pressed, checked, defaultAnimationSpec)",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "if (pressed) {",
                "shapes.pressedShape",
                "} else if (checked) {",
                "shapes.checkedShape",
                "} else shapes.shape",
            ]
        ));
        // The Expressive overloads' colors and border: the vibrant ones for the standard and outlined styles.
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "colors: IconToggleButtonColors = IconButtonDefaults.iconToggleButtonVibrantColors(),"
            ]
        ));
        assert!(cites(
            ICON_BUTTON_KT,
            &[
                "colors: IconToggleButtonColors = IconButtonDefaults.outlinedIconToggleButtonVibrantColors(),",
                "border: BorderStroke? =",
                "IconButtonDefaults.outlinedIconToggleButtonVibrantBorder(enabled, checked),",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "containerColor = Color.Transparent,",
                "contentColor = fromToken(StandardIconButtonTokens.UnselectedColor),",
                "disabledContainerColor = Color.Transparent,",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "checkedContainerColor = Color.Transparent,",
                "checkedContentColor = fromToken(StandardIconButtonTokens.SelectedColor),",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "containerColor = Color.Transparent,",
                "contentColor = fromToken(OutlinedIconButtonTokens.UnselectedColor),",
                "disabledContainerColor = Color.Transparent,",
            ]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "if (checked) {",
                "return null",
                "}",
                "return outlinedIconButtonVibrantBorder(enabled)",
            ]
        ));
        // The checked shapes.
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &["checkedShape = fromToken(SmallIconButtonTokens.SelectedContainerShapeRound),"]
        ));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &["@Composable get() = XLargeIconButtonTokens.SelectedContainerShapeSquare.value"]
        ));
    }
}
