//! The Material 3 Expressive toggle button: a button that is checked or not, in the button's five sizes and three of
//! its color styles, which morphs between its shape at rest, its shape while pressed and its shape while checked.
//!
//! The toggle button follows androidx's `ToggleButton`, `FilledTonalToggleButton` and `OutlinedToggleButton` in
//! `ToggleButton.kt`, at the pinned commit, as [ADR 0012] says: the tokens they read, and the spaces they write in place
//! of a token, each cited where it is used. Its container is drawn by a canvas whose corner radius a spring moves
//! ([ADR 0011]), with `MotionSchemeKeyTokens.FastSpatial` as `ToggleButton.kt` animates it (line 178), converted as
//! [ADR 0004] says.
//!
//! Its behavior is `nsh-base`'s `Toggle`: presses, Space and Return, focus, the disabled state, and for screen readers a
//! button with a pressed state (androidx says `Role.Checkbox`), whose label is the toggle button's text. The toggle
//! button reports the value a press asks for and shows the one the app passes back.
//!
//! Left out, each for its own pull request: the leading icon (`icon`, which [`crate::button`] has not either);
//! `ElevatedToggleButton`; the shadow that `ToggleButtonDefaults.elevation()` and its tonal sibling give a toggle button
//! under the pointer (`HoveredContainerElevation`, level 1, which [`crate::button`] leaves out too); and the change of
//! the outline's width and color, which `ToggleButton.kt` springs (`animateBorderStrokeAsState`, line 1731) and this one
//! makes at once.
//!
//! [ADR 0004]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0004-spring-damping-conversion.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::button::ButtonSize;
use crate::morph::{DrawnShape, corners};
use crate::motion::SpringToken;
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::androidx::{
    button_large_tokens, button_medium_tokens, button_small_tokens, button_xlarge_tokens,
    button_xsmall_tokens, filled_button_tokens, outlined_button_tokens, state_tokens,
    tonal_button_tokens, typography_tokens,
};
use crate::tokens::{Shape, TypeStyle};

/// The color style of a [`ToggleButton`], from the most emphasis to the least.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum ToggleButtonStyle {
    /// A surface container, primary when checked: androidx `ToggleButton`.
    #[default]
    Filled,
    /// A secondary container, secondary when checked: androidx `FilledTonalToggleButton`.
    Tonal,
    /// An outline and no container, inverse surface when checked: androidx `OutlinedToggleButton`.
    Outlined,
}

/// What a size gives the toggle button.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's height for one line of label.
    height: Points,
    /// The space before the label.
    leading: Points,
    /// The space after the label.
    trailing: Points,
    /// The space above and below the label.
    vertical: Points,
    /// The label's type style.
    label: TypeStyle,
    /// The shape while pressed.
    pressed: Shape,
    /// The shape while checked.
    checked: Shape,
}

/// The shape at rest of every size: androidx `ToggleButtonDefaults.shapesFor(buttonHeight)` gives each size
/// `ToggleButtonDefaults.shape`, `ButtonSmallTokens.ContainerShapeRound` (`ToggleButton.kt` lines 714-715).
const REST: Shape = button_small_tokens::CONTAINER_SHAPE_ROUND;

/// androidx `ToggleButton.kt`, `ExtraSmallContentPadding` (line 604): 12 points on each side, 6 above and below.
const EXTRA_SMALL_SIDE_SPACE: Points = Points(12.0);
/// See [`EXTRA_SMALL_SIDE_SPACE`].
const EXTRA_SMALL_VERTICAL_SPACE: Points = Points(6.0);
/// androidx `ToggleButton.kt`, `smallVerticalPadding` (lines 657-658) on a touch screen: 10 points; 8 where androidx
/// sizes components for a precise pointer, which [`crate::button`] does not do either.
const SMALL_VERTICAL_SPACE: Points = Points(10.0);
/// androidx `ToggleButton.kt`, `MediumVerticalPadding` (line 666).
const MEDIUM_VERTICAL_SPACE: Points = Points(16.0);
/// androidx `ToggleButton.kt`, `LargeVerticalPadding` (line 669).
const LARGE_VERTICAL_SPACE: Points = Points(32.0);
/// androidx `ToggleButton.kt`, `ExtraLargeContentPadding` (line 645).
const EXTRA_LARGE_VERTICAL_SPACE: Points = Points(48.0);

/// The values of `size`: androidx `ToggleButtonDefaults.contentPaddingFor` and `shapesFor` for its height, with no icon,
/// and `ButtonDefaults.textStyleFor` (label large below the medium height, title medium below the large one, headline
/// small below the extra large one, headline large from it).
fn size_spec(size: ButtonSize) -> SizeSpec {
    match size {
        ButtonSize::ExtraSmall => SizeSpec {
            height: button_xsmall_tokens::CONTAINER_HEIGHT,
            leading: EXTRA_SMALL_SIDE_SPACE,
            trailing: EXTRA_SMALL_SIDE_SPACE,
            vertical: EXTRA_SMALL_VERTICAL_SPACE,
            label: typography_tokens::LABEL_LARGE,
            pressed: button_xsmall_tokens::PRESSED_CONTAINER_SHAPE,
            checked: button_xsmall_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Small => SizeSpec {
            height: button_small_tokens::CONTAINER_HEIGHT,
            leading: button_small_tokens::LEADING_SPACE,
            trailing: button_small_tokens::TRAILING_SPACE,
            vertical: SMALL_VERTICAL_SPACE,
            label: typography_tokens::LABEL_LARGE,
            pressed: button_small_tokens::PRESSED_CONTAINER_SHAPE,
            checked: button_small_tokens::SELECTED_CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Medium => SizeSpec {
            height: button_medium_tokens::CONTAINER_HEIGHT,
            leading: button_medium_tokens::LEADING_SPACE,
            trailing: button_medium_tokens::TRAILING_SPACE,
            vertical: MEDIUM_VERTICAL_SPACE,
            label: typography_tokens::TITLE_MEDIUM,
            pressed: button_medium_tokens::PRESSED_CONTAINER_SHAPE,
            checked: button_medium_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Large => SizeSpec {
            height: button_large_tokens::CONTAINER_HEIGHT,
            leading: button_large_tokens::LEADING_SPACE,
            trailing: button_large_tokens::TRAILING_SPACE,
            vertical: LARGE_VERTICAL_SPACE,
            label: typography_tokens::HEADLINE_SMALL,
            pressed: button_large_tokens::PRESSED_CONTAINER_SHAPE,
            checked: button_large_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::ExtraLarge => SizeSpec {
            height: button_xlarge_tokens::CONTAINER_HEIGHT,
            leading: button_xlarge_tokens::LEADING_SPACE,
            trailing: button_xlarge_tokens::TRAILING_SPACE,
            vertical: EXTRA_LARGE_VERTICAL_SPACE,
            label: typography_tokens::HEADLINE_LARGE,
            pressed: button_xlarge_tokens::PRESSED_CONTAINER_SHAPE,
            checked: button_xlarge_tokens::CONTAINER_SHAPE_SQUARE,
        },
    }
}

/// What a color style gives the toggle button: androidx `ToggleButtonColors` and the outline.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorSpec {
    /// The container when enabled and not checked; none is transparent.
    container: Option<Tint>,
    /// The label when enabled and not checked.
    content: Tint,
    /// The container when enabled and checked.
    checked_container: Tint,
    /// The label when enabled and checked.
    checked_content: Tint,
    /// The container when disabled, checked or not.
    disabled_container: Tint,
    /// The label when disabled, checked or not.
    disabled_content: Tint,
    /// The outline when enabled and not checked; none is no outline.
    outline: Option<Tint>,
    /// The outline when disabled and not checked.
    disabled_outline: Option<Tint>,
}

/// The colors of `style`, as androidx builds them: `defaultToggleButtonColors`, `defaultFilledTonalToggleButtonColors`
/// and `defaultOutlinedToggleButtonColors`, with `OutlinedToggleButtonDefaults.border`.
fn color_spec(style: ToggleButtonStyle) -> ColorSpec {
    match style {
        ToggleButtonStyle::Filled => ColorSpec {
            container: Some(Tint::solid(
                filled_button_tokens::UNSELECTED_CONTAINER_COLOR,
            )),
            // androidx reads the pressed label color here, the token file holding no other unselected one.
            content: Tint::solid(filled_button_tokens::UNSELECTED_PRESSED_LABEL_TEXT_COLOR),
            checked_container: Tint::solid(filled_button_tokens::SELECTED_CONTAINER_COLOR),
            checked_content: Tint::solid(filled_button_tokens::SELECTED_PRESSED_LABEL_TEXT_COLOR),
            disabled_container: Tint::faded(
                filled_button_tokens::DISABLED_CONTAINER_COLOR,
                filled_button_tokens::DISABLED_CONTAINER_OPACITY,
            ),
            disabled_content: Tint::faded(
                filled_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                filled_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        ToggleButtonStyle::Tonal => ColorSpec {
            container: Some(Tint::solid(tonal_button_tokens::UNSELECTED_CONTAINER_COLOR)),
            content: Tint::solid(tonal_button_tokens::UNSELECTED_LABEL_TEXT_COLOR),
            checked_container: Tint::solid(tonal_button_tokens::SELECTED_CONTAINER_COLOR),
            checked_content: Tint::solid(tonal_button_tokens::SELECTED_LABEL_TEXT_COLOR),
            disabled_container: Tint::faded(
                tonal_button_tokens::DISABLED_CONTAINER_COLOR,
                tonal_button_tokens::DISABLED_CONTAINER_OPACITY,
            ),
            disabled_content: Tint::faded(
                tonal_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                tonal_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            outline: None,
            disabled_outline: None,
        },
        ToggleButtonStyle::Outlined => ColorSpec {
            container: None,
            content: Tint::solid(outlined_button_tokens::UNSELECTED_LABEL_TEXT_COLOR),
            checked_container: Tint::solid(outlined_button_tokens::SELECTED_CONTAINER_COLOR),
            checked_content: Tint::solid(outlined_button_tokens::SELECTED_LABEL_TEXT_COLOR),
            // androidx fills a disabled outlined toggle button with the disabled outline color at the container's
            // disabled opacity.
            disabled_container: Tint::faded(
                outlined_button_tokens::DISABLED_OUTLINE_COLOR,
                outlined_button_tokens::DISABLED_CONTAINER_OPACITY,
            ),
            disabled_content: Tint::faded(
                outlined_button_tokens::DISABLED_LABEL_TEXT_COLOR,
                outlined_button_tokens::DISABLED_LABEL_TEXT_OPACITY,
            ),
            // `ButtonDefaults.outlinedButtonBorder(enabled)`, as for the button.
            outline: Some(Tint::solid(outlined_button_tokens::OUTLINE_COLOR)),
            disabled_outline: Some(Tint::faded(
                outlined_button_tokens::OUTLINE_COLOR,
                outlined_button_tokens::DISABLED_CONTAINER_OPACITY,
            )),
        },
    }
}

impl ColorSpec {
    /// The container's color: androidx `ToggleButtonColors.containerColor(enabled, checked)`.
    fn container(self, disabled: bool, checked: bool) -> Option<Tint> {
        match (disabled, checked) {
            (true, _) => Some(self.disabled_container),
            (false, true) => Some(self.checked_container),
            (false, false) => self.container,
        }
    }

    /// The label's color: androidx `ToggleButtonColors.contentColor(enabled, checked)`.
    fn content(self, disabled: bool, checked: bool) -> Tint {
        match (disabled, checked) {
            (true, _) => self.disabled_content,
            (false, true) => self.checked_content,
            (false, false) => self.content,
        }
    }

    /// The outline's color: androidx `OutlinedToggleButtonDefaults.border(enabled, checked)` draws none when checked.
    fn outline(self, disabled: bool, checked: bool) -> Option<Tint> {
        match (disabled, checked) {
            (_, true) => None,
            (true, false) => self.disabled_outline,
            (false, false) => self.outline,
        }
    }
}

/// The outline's width at every size: `ButtonDefaults.outlinedButtonBorder` reads
/// `ButtonSmallTokens.OutlinedOutlineWidth` whatever the height.
const OUTLINE_WIDTH: Points = button_small_tokens::OUTLINED_OUTLINE_WIDTH;

/// How far presses also count around a container of `height`, so that it has a 48-point target with no change to its
/// size or the layout.
fn touch_slop(height: Points) -> EdgeInsets {
    EdgeInsets::all(px(((MIN_TOUCH_TARGET.0 - height.0) / 2.0).max(0.0)))
}

/// The props of a [`ToggleButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ToggleButtonProps {
    /// What the toggle button says, which screen readers say too.
    pub label: Prop<SharedString>,
    /// Whether it is checked. The app owns the value: a press reports the next one with
    /// [`ToggleButtonEvent::Change`], and the toggle button shows the value passed back.
    #[prop(default)]
    pub checked: Prop<bool>,
    /// Its size: the button's sizes.
    #[prop(default)]
    pub size: Prop<ButtonSize>,
    /// Its color style.
    #[prop(default)]
    pub style: Prop<ToggleButtonStyle>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`ToggleButton`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ToggleButtonEvent {
    /// It was pressed: the value it asks for, the opposite of `checked` (androidx `onCheckedChange`).
    #[args(checked)]
    Change(bool),
}

/// A Material 3 Expressive toggle button: a label in a container of the size and color style given, checked or not.
///
/// Hover, focus and press lay the label's color over it at the state layer opacities of `StateTokens`, as androidx's
/// ripple does, and change nothing else. A press morphs its container into the pressed shape; being checked, into the
/// checked one.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::toggle_button::toggle_button;
///
/// fn starred() -> impl IntoElement {
///     let starred = signal(false);
///     toggle_button()
///         .label("Starred")
///         .checked(move || starred.get())
///         .on_change(move |checked: &bool| starred.set(*checked))
/// }
/// ```
#[component]
pub fn ToggleButton(props: ToggleButtonProps, cx: Scope<ToggleButtonEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || color_spec(props.style.get());
    let shapes = move || {
        let size = size();
        [corners(REST), corners(size.pressed), corners(size.checked)]
    };
    let content = move || {
        tint_color(Some(
            colors().content(props.disabled.get(), props.checked.get()),
        ))
    };
    // Before its first layout, the container is taken to be as wide as it is tall, as the button's is.
    let drawn = untracked(|| {
        let height = size().height;
        DrawnShape::toggle(
            shapes(),
            props.checked.get(),
            Size::new(height, height),
            SpringToken::FastSpatial,
        )
    });
    effect(move || {
        let [rest, pressed, checked] = shapes();
        drawn.set_corners(rest, pressed);
        drawn.set_checked_corners(checked);
    });
    effect(move || drawn.set_checked(props.checked.get()));
    effect(move || {
        // A toggle button disabled while pressed may get no end of its press: it leaves the pressed shape.
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
        .on_change(move |checked: &bool| cx.emit(ToggleButtonEvent::Change(*checked)))
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
        .py(move || size().vertical)
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
            // The state layer, as the button's: androidx's ripple in the content color.
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
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Checked, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        Recording, StartedSpring, layout_items, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph::Corners;
    use crate::morph_probe::{
        Step, assert_traces, paths, replay, rounded_rectangle, set_values, tolerance,
    };
    use crate::motion::SPRING_MASS;

    /// What the probe places: one toggle button, lines saying whether it is checked and how many changes it asked
    /// for, and lines that check, uncheck and disable it from the app.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        label: Prop<SharedString>,
        #[prop(default)]
        checked: Prop<bool>,
        #[prop(default)]
        size: Prop<ButtonSize>,
        #[prop(default)]
        style: Prop<ToggleButtonStyle>,
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
        let size = signal(props.size.get_untracked());
        let ignore = props.ignore.get_untracked();
        // A row, so that the toggle button is as wide as its content.
        view()
            .child(
                view().flex_row().child(
                    toggle_button()
                        .label(props.label)
                        .checked(move || checked.get())
                        .size(move || size.get())
                        .style(props.style)
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
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Grow").on_press(move |(): &()| size.set(ButtonSize::Medium)))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        checked: bool,
        size: ButtonSize,
        style: ToggleButtonStyle,
        disabled: bool,
        answer: Answer,
        appearance: Appearance,
        reduce_motion: bool,
    }

    /// What the app does with the value a press asks for.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Answer {
        /// It passes it back: the toggle button shows it.
        #[default]
        PassesBack,
        /// It ignores it: the toggle button stays as it was.
        Ignores,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            label: Prop::new(SharedString::from("Wi-Fi")),
            checked: Prop::new(config.checked),
            size: Prop::new(config.size),
            style: Prop::new(config.style),
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

    /// The toggle button's views, and the last style, frame, drawing and text each was sent.
    struct Mounted {
        /// The press target, with the button role.
        button: NodeId,
        /// The canvas that draws the container.
        container: NodeId,
        /// The canvas that draws the state layer.
        layer: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        text: Option<nsh_core::ResolvedTextStyle>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut button = None;
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
                panic!("two canvases in the toggle button: {items:?}");
            };
            let [label] = children(ViewKind::Text)[..] else {
                panic!("one text in the toggle button: {items:?}");
            };
            Self {
                button,
                container,
                layer,
                styles,
                frames,
                drawings,
                text: texts.get(&label).copied().flatten(),
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

        /// The toggle button's size, as its layout gave it.
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

    const STYLES: [ToggleButtonStyle; 3] = [
        ToggleButtonStyle::Filled,
        ToggleButtonStyle::Tonal,
        ToggleButtonStyle::Outlined,
    ];

    /// The color `ops` fill with, or that they stroke with, if any.
    fn paint(ops: &[CanvasOp], stroke: bool) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) if !stroke => Some(*color),
            CanvasOp::StrokeStyle(canvas::Paint::Color(color)) if stroke => Some(*color),
            _ => None,
        })
    }

    /// The container's color, its outline's width and color (0 and transparent with no outline), and the label's
    /// color.
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

    /// The colors each style should paint, in the baseline schemes: the container, the outline's width and color, and
    /// the label.
    fn expected_colors(
        style: ToggleButtonStyle,
        checked: bool,
        disabled: bool,
        dark: bool,
    ) -> (Color, f32, Color, Color) {
        let solid = |role| color(scheme_color(role, dark), 255);
        let faded = |role, alpha| color(scheme_color(role, dark), alpha);
        let none = Color(0);
        match (style, checked, disabled) {
            // androidx `defaultToggleButtonColors`: surface container and on surface variant, primary and on primary.
            (ToggleButtonStyle::Filled, false, false) => (
                solid(ColorRole::SurfaceContainer),
                0.0,
                none,
                solid(ColorRole::OnSurfaceVariant),
            ),
            (ToggleButtonStyle::Filled, true, false) => (
                solid(ColorRole::Primary),
                0.0,
                none,
                solid(ColorRole::OnPrimary),
            ),
            // `defaultFilledTonalToggleButtonColors`: secondary container and on secondary container, secondary and on
            // secondary.
            (ToggleButtonStyle::Tonal, false, false) => (
                solid(ColorRole::SecondaryContainer),
                0.0,
                none,
                solid(ColorRole::OnSecondaryContainer),
            ),
            (ToggleButtonStyle::Tonal, true, false) => (
                solid(ColorRole::Secondary),
                0.0,
                none,
                solid(ColorRole::OnSecondary),
            ),
            // Both disabled, checked or not: on surface at 10 %, on surface variant at 38 %.
            (ToggleButtonStyle::Filled | ToggleButtonStyle::Tonal, _, true) => (
                faded(ColorRole::OnSurface, ALPHA_10),
                0.0,
                none,
                faded(ColorRole::OnSurfaceVariant, ALPHA_38),
            ),
            // `defaultOutlinedToggleButtonColors` and `border`: an outline variant outline 1 point wide and on surface
            // variant, inverse surface and inverse on surface with no outline.
            (ToggleButtonStyle::Outlined, false, false) => (
                none,
                1.0,
                solid(ColorRole::OutlineVariant),
                solid(ColorRole::OnSurfaceVariant),
            ),
            (ToggleButtonStyle::Outlined, true, false) => (
                solid(ColorRole::InverseSurface),
                0.0,
                none,
                solid(ColorRole::InverseOnSurface),
            ),
            // Disabled: outline variant at 10 % fills it, and the outline, unchecked only, is outline variant at 10 %.
            (ToggleButtonStyle::Outlined, false, true) => (
                faded(ColorRole::OutlineVariant, ALPHA_10),
                1.0,
                faded(ColorRole::OutlineVariant, ALPHA_10),
                faded(ColorRole::OnSurfaceVariant, ALPHA_38),
            ),
            (ToggleButtonStyle::Outlined, true, true) => (
                faded(ColorRole::OutlineVariant, ALPHA_10),
                0.0,
                none,
                faded(ColorRole::OnSurfaceVariant, ALPHA_38),
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
                        // No view paints a background of its own under the canvases.
                        assert_eq!(
                            mounted.style(mounted.button).background_color,
                            Color::TRANSPARENT,
                            "{what}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_press_asks_for_the_other_value_and_shows_the_one_passed_back() -> Result<(), String> {
        let (mut app, _) = start(Config::default());
        let pressed = |app: &Headless| {
            app.screen()
                .accessibility_state("role:button[Wi-Fi]")
                .map(|state| state.pressed)
        };
        assert_eq!(pressed(&app)?, Some(Checked::False));
        app.press("Wi-Fi")?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("checked: true"));
        assert_eq!(pressed(&app)?, Some(Checked::True));
        app.press("Wi-Fi")?;
        assert!(app.shows("changes: 2"));
        assert!(app.shows("checked: false"));
        assert_eq!(pressed(&app)?, Some(Checked::False));
        Ok(())
    }

    #[test]
    fn an_app_that_ignores_the_change_keeps_it_as_it_was() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            ..Config::default()
        });
        let before = colors(&Mounted::of(&recording.items()));
        app.press("Wi-Fi")?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("checked: false"));
        assert_eq!(colors(&Mounted::of(&recording.items())), before);
        Ok(())
    }

    #[test]
    fn a_disabled_toggle_button_takes_no_press_and_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("Wi-Fi");
        assert!(app.shows("changes: 0"));
        let mounted = Mounted::of(&recording.items());
        // The state layer is hidden: laid out at no size.
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::new(Points(0.0), Points(0.0)))
        );
        assert_eq!(
            app.screen().enabled("role:button[Wi-Fi]"),
            Ok(false),
            "screen readers say it is dimmed"
        );
    }

    /// Every size: its height, its spaces before, after, above and below the label, the label's size and line height,
    /// and its pressed and checked corners. androidx `ToggleButton.kt` (`contentPaddingFor`, `shapesFor`) and the size
    /// token files: `PressedContainerShape` and `ContainerShapeSquare` (`SelectedContainerShapeSquare` at the small
    /// size).
    /// A size, its height, side space, vertical space, font size, line height, pressed radius and checked radius.
    type SizeRow = (ButtonSize, f32, f32, f32, f32, f32, f32, f32);

    const SIZES: [SizeRow; 5] = [
        (
            ButtonSize::ExtraSmall,
            32.0,
            12.0,
            6.0,
            14.0,
            20.0,
            8.0,
            12.0,
        ),
        (ButtonSize::Small, 40.0, 16.0, 10.0, 14.0, 20.0, 8.0, 12.0),
        (ButtonSize::Medium, 56.0, 24.0, 16.0, 16.0, 24.0, 12.0, 16.0),
        (ButtonSize::Large, 96.0, 48.0, 32.0, 24.0, 32.0, 16.0, 28.0),
        (
            ButtonSize::ExtraLarge,
            136.0,
            64.0,
            48.0,
            32.0,
            40.0,
            16.0,
            28.0,
        ),
    ];

    #[test]
    fn each_size_has_its_height_spaces_label_and_corners() {
        for (size, height, side, vertical, font, line, pressed, checked) in SIZES {
            let spec = size_spec(size);
            assert_eq!(spec.height, Points(height), "{size:?}");
            assert_eq!(
                (spec.leading, spec.trailing),
                (Points(side), Points(side)),
                "{size:?}"
            );
            assert_eq!(spec.vertical, Points(vertical), "{size:?}");
            // One line of label and its spaces above and below make the height.
            assert_eq!(
                spec.label.line_height.0 + 2.0 * vertical,
                height,
                "{size:?}"
            );
            assert_eq!(
                corners(spec.pressed),
                Corners::Radius(Points(pressed)),
                "{size:?}"
            );
            assert_eq!(
                corners(spec.checked),
                Corners::Radius(Points(checked)),
                "{size:?}"
            );
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let frame = mounted.size();
            assert_eq!(frame.height, Points(height), "{size:?}");
            // The headless engine's characters are 0.55 times the font size wide.
            let label = 5.0 * font * 0.55;
            assert!(
                (frame.width.0 - (label + 2.0 * side)).abs() <= 1.0,
                "{size:?} {frame:?}"
            );
            let text = mounted.text.as_ref().expect("the label's style");
            assert_eq!(text.font_size, Points(font), "{size:?}");
            assert_eq!(text.line_height, Points(line), "{size:?}");
            // Pressing counts within 48 points.
            let slop = ((48.0 - height) / 2.0).max(0.0);
            assert_eq!(
                mounted.hit_slop,
                Some(EdgeInsets::all(px(slop))),
                "{size:?}"
            );
        }
        // Every size rests fully rounded.
        assert_eq!(corners(REST), Corners::Full);
    }

    /// The container's and the state layer's paths, and the toggle button's size.
    fn drawn(recording: &Recording) -> (Vec<Vec<Step>>, Vec<Vec<Step>>, Size<Points>) {
        let mounted = Mounted::of(&recording.items());
        let replay = replay(recording);
        (
            paths(mounted.drawing(mounted.container), &replay),
            paths(mounted.drawing(mounted.layer), &replay),
            mounted.size(),
        )
    }

    #[test]
    fn it_mounts_with_the_corners_it_has_and_starts_nothing() {
        for (size, .., checked_radius) in SIZES {
            for checked in [false, true] {
                let what = format!("{size:?} checked {checked}");
                let (_app, recording) = start(Config {
                    checked,
                    size,
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{what}");
                assert_eq!(set_values(&items), 0, "{what}");
                assert!(!replay(&recording).is_animating(), "{what}");
                let (container, layer, frame) = drawn(&recording);
                let radius = if checked {
                    checked_radius
                } else {
                    frame.height.0 / 2.0
                };
                let expected = rounded_rectangle(frame, 0.0, radius);
                assert_traces(&container[0], &expected, &what);
                assert_traces(&layer[0], &expected, &what);
            }
        }
    }

    /// A press that goes down on the toggle button and stays down: what it sent.
    fn press_down(app: &mut Headless, recording: &Recording) -> Vec<MountItem> {
        let (pressed, sent) =
            recording.during(|| app.drag_and_hold("Wi-Fi", point(px(0.0), px(0.0))));
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

    /// `items`, a press going down or the app changing what the toggle button is, spring the corners to `to` with
    /// `FastSpatial`, from where they are and as fast as they move, and set nothing at once. The last spring wins: a
    /// press's end and the change the app makes for it come in the same batch, each to its own corners, and the second,
    /// started in the same frame, takes over from the first where it is. Nothing is resized: a new color draws the
    /// container again with the same animated corners.
    fn morph_in(recording: &Recording, items: &[MountItem], to: f32, what: &str) -> StartedSpring {
        let springs = started_springs(items);
        assert!(
            !springs.is_empty() && springs.len() <= 2,
            "{what}: {springs:?}"
        );
        let spec = SpringToken::FastSpatial.spec();
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
        for (size, .., pressed, checked) in SIZES {
            let what = format!("{size:?}");
            let (mut app, recording) = start(Config {
                size,
                ..Config::default()
            });
            let frame = Mounted::of(&recording.items()).size();
            let rest = frame.height.0 / 2.0;
            let sent = press_down(&mut app, &recording);
            let down = morph_in(&recording, &sent, pressed, &what);
            assert_eq!(started_springs(&sent).len(), 1, "{what}");
            // A press alone draws nothing again and lays nothing out: the platform moves the corners.
            assert!(
                !sent
                    .iter()
                    .any(|item| matches!(item, MountItem::Canvas { .. })),
                "{what}: {sent:?}"
            );
            assert_eq!(layout_items(&sent), 0, "{what}: {sent:?}");
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
            let after = replay.run(up.node, 180);
            let gap = worst_gap(&after, f64::from(pressed), &up);
            assert!(
                gap < tolerance(f64::from(pressed), f64::from(checked)),
                "{what}: {gap}"
            );
            let settled = replay.run(up.node, 600);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(checked.to_bits()),
                "{what}"
            );
            // Its next press and end take it back through the pressed corners to rest.
            let sent = press_down(&mut app, &recording);
            let _ = morph_in(&recording, &sent, pressed, &what);
            let sent = release(&mut app, &recording);
            assert!(app.shows("checked: false"), "{what}");
            let _ = morph_in(&recording, &sent, rest, &what);
        }
    }

    #[test]
    fn a_change_from_the_app_springs_the_corners_and_a_press_left_alone_ends_where_it_was() {
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            ..Config::default()
        });
        let frame = Mounted::of(&recording.items()).size();
        let rest = frame.height.0 / 2.0;
        let sent = press(&mut app, &recording, "Check");
        let _ = morph_in(&recording, &sent, 12.0, "checked by the app");
        let sent = press(&mut app, &recording, "Uncheck");
        let _ = morph_in(&recording, &sent, rest, "unchecked by the app");
        // A press the app ignores: down to the pressed corners, then back to rest, with one spring each.
        let sent = press_down(&mut app, &recording);
        let _ = morph_in(&recording, &sent, 8.0, "pressed");
        let sent = release(&mut app, &recording);
        assert!(app.shows("changes: 1"));
        assert!(app.shows("checked: false"));
        let _ = morph_in(&recording, &sent, rest, "released");
        assert_eq!(started_springs(&sent).len(), 1, "{sent:?}");
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
            &rounded_rectangle(frame, 0.0, frame.height.0 / 2.0),
            "unchecked",
        );
    }

    #[test]
    fn a_change_from_the_app_during_a_press_waits_for_its_end_unless_motion_is_reduced() {
        // The app checks the toggle button while a finger holds it down: the pressed corners stay, and the press's end
        // springs to the checked ones.
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            ..Config::default()
        });
        let sent = press_down(&mut app, &recording);
        let _ = morph_in(&recording, &sent, 8.0, "pressed");
        let sent = press(&mut app, &recording, "Check");
        assert!(app.shows("checked: true"));
        assert_eq!(started_springs(&sent), Vec::new(), "{sent:?}");
        assert_eq!(set_values(&sent), 0, "{sent:?}");
        let sent = release(&mut app, &recording);
        assert!(app.shows("checked: true"));
        let _ = morph_in(&recording, &sent, 12.0, "released checked");
        // With reduce motion, the press shows nothing and the change shows at once, press or not.
        let (mut app, recording) = start(Config {
            answer: Answer::Ignores,
            reduce_motion: true,
            ..Config::default()
        });
        let (_, _, frame) = drawn(&recording);
        let _ = press_down(&mut app, &recording);
        let sent = press(&mut app, &recording, "Check");
        assert_eq!(started_springs(&sent), Vec::new(), "{sent:?}");
        assert_eq!(set_values(&sent), 1, "{sent:?}");
        let (container, ..) = drawn(&recording);
        assert_traces(
            &container[0],
            &rounded_rectangle(frame, 0.0, 12.0),
            "checked",
        );
    }

    #[test]
    fn a_new_size_moves_the_corners_at_once() {
        // androidx restarts the shape animation when the shapes change (`key(shapes)`): checked or not, the corners go
        // to the new size's at once, with no spring.
        for (checked, radius) in [(true, 16.0), (false, 28.0)] {
            let (mut app, recording) = start(Config {
                checked,
                ..Config::default()
            });
            let sent = press(&mut app, &recording, "Grow");
            assert_eq!(started_springs(&sent), Vec::new(), "checked {checked}");
            assert!(set_values(&sent) >= 1, "checked {checked}: {sent:?}");
            let (container, layer, frame) = drawn(&recording);
            assert_eq!(frame.height, Points(56.0));
            let expected = rounded_rectangle(frame, 0.0, radius);
            assert_traces(&container[0], &expected, &format!("checked {checked}"));
            assert_traces(&layer[0], &expected, &format!("checked {checked}"));
            assert!(!replay(&recording).is_animating(), "checked {checked}");
        }
    }

    #[test]
    fn a_toggle_button_disabled_while_pressed_leaves_the_pressed_shape() {
        let (mut app, recording) = start(Config::default());
        let frame = Mounted::of(&recording.items()).size();
        let sent = press_down(&mut app, &recording);
        let _ = morph_in(&recording, &sent, 8.0, "pressed");
        let sent = press(&mut app, &recording, "Disable");
        let _ = morph_in(&recording, &sent, frame.height.0 / 2.0, "disabled");
    }

    #[test]
    fn the_outline_is_drawn_inside_the_container_and_only_unchecked() {
        let (mut app, recording) = start(Config {
            style: ToggleButtonStyle::Outlined,
            answer: Answer::Ignores,
            ..Config::default()
        });
        let (container, layer, frame) = drawn(&recording);
        let rest = frame.height.0 / 2.0;
        assert_eq!(container.len(), 2, "the fill and the outline");
        assert_traces(
            &container[1],
            &rounded_rectangle(frame, 0.5, rest - 0.5),
            "outline",
        );
        assert_traces(
            &layer[0],
            &rounded_rectangle(frame, 1.0, rest - 1.0),
            "layer",
        );
        let sent = press(&mut app, &recording, "Check");
        let spring = morph_in(&recording, &sent, 12.0, "checked");
        let mounted = Mounted::of(&recording.items());
        assert_eq!(colors(&mounted).1, 0.0);
        // Once the corners settle, the fill alone, and the state layer over all of it, with no outline to keep clear
        // of.
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
    fn hover_shows_the_state_layer_in_the_label_color_and_lays_out_nothing() -> Result<(), String> {
        for checked in [false, true] {
            let (mut app, recording) = start(Config {
                checked,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
            // In the label's color.
            assert_eq!(
                paint(&mounted.drawing(mounted.layer).ops, false),
                Some(colors(&mounted).3),
                "checked {checked}"
            );
            app.hover("Wi-Fi")?;
            let hover = recording.in_step("hover(Wi-Fi)");
            let opacity = Mounted::of(&recording.items()).style(mounted.layer).opacity;
            assert!(
                (opacity - state_tokens::HOVER_STATE_LAYER_OPACITY).abs() < 1e-6,
                "checked {checked}: {opacity}"
            );
            assert_eq!(layout_items(&hover), 0, "checked {checked}");
        }
        Ok(())
    }

    /// The vendored `ToggleButton.kt`, at the pinned commit (ADR 0012).
    const TOGGLE_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ToggleButton.kt");

    /// Whether `ToggleButton.kt` holds `lines`, each trimmed, one after the other.
    fn cites(lines: &[&str]) -> bool {
        let file: Vec<&str> = TOGGLE_BUTTON_KT.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_toggle_button_kt_is_still_there() {
        assert!(cites(&[
            "val defaultAnimationSpec = MotionSchemeKeyTokens.FastSpatial.value<Float>()"
        ]));
        assert!(cites(&[
            "if (pressed) {",
            "shapes.pressedShape",
            "} else if (checked) {",
            "shapes.checkedShape",
            "} else {",
            "shapes.shape",
        ]));
        assert!(cites(&[
            "@Composable get() = ButtonSmallTokens.ContainerShapeRound.value"
        ]));
        assert!(cites(&[
            "get() = PaddingValues(start = 12.dp, end = 12.dp, top = 6.dp, bottom = 6.dp)"
        ]));
        assert_eq!(
            (EXTRA_SMALL_SIDE_SPACE, EXTRA_SMALL_VERTICAL_SPACE),
            (Points(12.0), Points(6.0))
        );
        assert!(cites(&[
            "get() = if (shouldUsePrecisionPointerComponentSizing.value) 8.dp else 10.dp"
        ]));
        assert_eq!(SMALL_VERTICAL_SPACE, Points(10.0));
        assert!(cites(&["private val MediumVerticalPadding = 16.dp"]));
        assert_eq!(MEDIUM_VERTICAL_SPACE, Points(16.0));
        assert!(cites(&["private val LargeVerticalPadding = 32.dp"]));
        assert_eq!(LARGE_VERTICAL_SPACE, Points(32.0));
        assert!(cites(&["top = 48.dp,", "bottom = 48.dp,"]));
        assert_eq!(EXTRA_LARGE_VERTICAL_SPACE, Points(48.0));
        // The colors: the filled style's unselected label is its pressed one, and the outline goes when checked.
        assert!(cites(&[
            "contentColor =",
            "fromToken(FilledButtonTokens.UnselectedPressedLabelTextColor),"
        ]));
        assert!(cites(&[
            "return if (checked) {",
            "null",
            "} else {",
            "ButtonDefaults.outlinedButtonBorder(enabled)",
        ]));
        assert!(cites(&[
            "fromToken(OutlinedButtonTokens.DisabledOutlineColor)",
            ".copy(alpha = OutlinedButtonTokens.DisabledContainerOpacity),"
        ]));
        // The checked shapes, from each size's square container shape.
        assert!(cites(&[
            "checkedShape = fromToken(ButtonSmallTokens.SelectedContainerShapeSquare),"
        ]));
        assert!(cites(&[
            "@Composable get() = ButtonXLargeTokens.ContainerShapeSquare.value"
        ]));
        assert!(!cites(&["private val MediumVerticalPadding = 12.dp"]));
    }
}
