//! The Material 3 Expressive button, at rest: its five sizes, two shapes and four color styles, with the state layer
//! of hover, focus and press.
//!
//! The button follows androidx's Expressive `Button(onClick, shapes, …)` and its siblings in `Button.kt`, at the
//! pinned commit, as [ADR 0012] says: it reads the tokens they read, and the few values they write in place of a
//! token they call wrong, each cited where it is used. The shape morph on press comes in its own pull request
//! ([ADR 0011]).
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::color::{ColorRole, ColorScheme};
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
        },
        ButtonSize::Small => SizeSpec {
            height: button_small_tokens::CONTAINER_HEIGHT,
            leading: button_small_tokens::LEADING_SPACE,
            trailing: button_small_tokens::TRAILING_SPACE,
            label: typography_tokens::LABEL_LARGE,
            round: button_small_tokens::CONTAINER_SHAPE_ROUND,
            square: button_small_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Medium => SizeSpec {
            height: button_medium_tokens::CONTAINER_HEIGHT,
            leading: button_medium_tokens::LEADING_SPACE,
            trailing: button_medium_tokens::TRAILING_SPACE,
            label: typography_tokens::TITLE_MEDIUM,
            round: button_medium_tokens::CONTAINER_SHAPE_ROUND,
            square: button_medium_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::Large => SizeSpec {
            height: button_large_tokens::CONTAINER_HEIGHT,
            leading: button_large_tokens::LEADING_SPACE,
            trailing: button_large_tokens::TRAILING_SPACE,
            label: typography_tokens::HEADLINE_SMALL,
            round: button_large_tokens::CONTAINER_SHAPE_ROUND,
            square: button_large_tokens::CONTAINER_SHAPE_SQUARE,
        },
        ButtonSize::ExtraLarge => SizeSpec {
            height: button_xlarge_tokens::CONTAINER_HEIGHT,
            leading: button_xlarge_tokens::LEADING_SPACE,
            trailing: button_xlarge_tokens::TRAILING_SPACE,
            label: typography_tokens::HEADLINE_LARGE,
            round: button_xlarge_tokens::CONTAINER_SHAPE_ROUND,
            square: button_xlarge_tokens::CONTAINER_SHAPE_SQUARE,
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

/// `view` with the corners of `shape`.
fn with_shape<E: Styled>(view: E, shape: Shape) -> E {
    match shape {
        Shape::Rectangle => view.rounded(px(0.0)),
        Shape::Circle => view.rounded_full(),
        Shape::Rounded(corners) => view
            .rounded_ss(corners.top_start)
            .rounded_se(corners.top_end)
            .rounded_ee(corners.bottom_end)
            .rounded_es(corners.bottom_start),
    }
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
    nsh_base::button()
        .disabled(move || props.disabled.get())
        .on_press(move || cx.emit(ButtonEvent::Press))
        .group()
        .flex_row()
        .items_center()
        .justify_center()
        .min_h(move || size().height)
        .ps(move || size().leading)
        .pe(move || size().trailing)
        .bg(move || {
            let colors = colors();
            tint_color(if props.disabled.get() {
                colors.disabled_container
            } else {
                colors.container
            })
        })
        .border(move || {
            if colors().outline.is_some() {
                OUTLINE_WIDTH
            } else {
                px(0.0)
            }
        })
        .border_color(move || {
            let colors = colors();
            tint_color(if props.disabled.get() {
                colors.disabled_outline
            } else {
                colors.outline
            })
        })
        .style_with(move |s| with_shape(s, shape()))
        .child(
            // The state layer: the label's color over the whole container, shown by the button's own states with
            // nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
            view()
                .absolute()
                .inset(px(0.0))
                .style_with(move |s| with_shape(s, shape()))
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
    use std::time::SystemTime;

    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{Recording, layout_items};

    /// What the probe places: one button with these props, and a line counting its presses.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
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
        // A row, so that the button is as wide as its content, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    button()
                        .label("Save")
                        .size(props.size)
                        .style(props.style)
                        .shape(props.shape)
                        .disabled(props.disabled)
                        .on_press(move || presses.update(|n| *n += 1)),
                ),
            )
            .child(text(move || format!("presses: {}", presses.get())))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        size: ButtonSize,
        style: ButtonStyle,
        shape: ButtonShape,
        disabled: bool,
        dark: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
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
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("Save").unwrap();
        (app, recording)
    }

    /// The button's three views, and the last style, frame and text each was sent.
    struct Mounted {
        container: NodeId,
        layer: NodeId,
        label: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        text: Option<nsh_core::ResolvedTextStyle>,
        disabled: Option<bool>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut parents = Vec::new();
            let mut container = None;
            let mut disabled = None;
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut text = None;
            let mut label = None;
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        kinds.insert(*id, *kind);
                    }
                    MountItem::Insert { parent, child, .. } => parents.push((*parent, *child)),
                    MountItem::ViewProps { id, props }
                        if props.accessibility.role == AccessibilityRole::Button =>
                    {
                        container = Some(*id);
                        disabled = Some(props.accessibility.state.disabled);
                    }
                    MountItem::ViewStyle { id, style } => {
                        styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    MountItem::Text {
                        id, text: content, ..
                    } if content.text == "Save" => {
                        label = Some(*id);
                        text = content.spans.first().map(|span| span.style);
                    }
                    _ => {}
                }
            }
            let container = container.expect("a view with the button role");
            let layer = parents
                .iter()
                .find(|(parent, child)| {
                    *parent == container && matches!(kinds.get(child), Some(ViewKind::View))
                })
                .map(|(_, child)| *child)
                .expect("a view in the button");
            Self {
                container,
                layer,
                label: label.expect("the label"),
                styles,
                frames,
                text,
                disabled,
            }
        }

        fn style(&self, id: NodeId) -> &ViewStyle {
            &self.styles[&id]
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

    /// The container's background, its outline's width and color, and the label's color.
    fn colors(mounted: &Mounted) -> (Color, f32, Color, Color) {
        let container = mounted.style(mounted.container);
        let borders = &container.borders;
        let widths = [
            borders.widths.top,
            borders.widths.right,
            borders.widths.bottom,
            borders.widths.left,
        ];
        assert!(widths.iter().all(|w| *w == widths[0]), "{widths:?}");
        assert!(
            borders.colors.iter().all(|c| *c == borders.colors[0]),
            "{:?}",
            borders.colors
        );
        let label = mounted.text.as_ref().expect("the label's style").color;
        (
            container.background_color,
            widths[0].0,
            borders.colors[0],
            label,
        )
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
            let (background, width, outline, label) = colors(&mount(Config {
                style,
                ..Config::default()
            }));
            assert_eq!((background, width, label), expected, "{style:?}");
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
            mounted
                .frames
                .get(&mounted.container)
                .map(|frame| frame.size)
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
            let container = mounted.frames[&mounted.container];
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
            // The state layer covers the container.
            assert_eq!(
                mounted.frames[&mounted.layer].size, container.size,
                "{size:?}"
            );
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

    #[test]
    fn each_shape_rounds_the_container_and_its_state_layer() {
        // `ContainerShapeRound` is `CornerFull` at every size; `ContainerShapeSquare` is `CornerMedium` (12) for
        // the extra small and small sizes, `CornerLarge` (16) for the medium, `CornerExtraLarge` (28) above.
        for (size, square) in [
            (ButtonSize::ExtraSmall, 12.0),
            (ButtonSize::Small, 12.0),
            (ButtonSize::Medium, 16.0),
            (ButtonSize::Large, 28.0),
            (ButtonSize::ExtraLarge, 28.0),
        ] {
            let round = mount(Config {
                size,
                ..Config::default()
            });
            let squared = mount(Config {
                size,
                shape: ButtonShape::Square,
                ..Config::default()
            });
            for mounted in [&round, &squared] {
                assert_eq!(
                    radii(mounted.style(mounted.layer)),
                    radii(mounted.style(mounted.container)),
                    "{size:?}"
                );
            }
            // NSH draws `rounded_full` as a radius larger than any side, which the platform caps at half the
            // height.
            let full = radii(round.style(round.container));
            assert!(full.iter().all(|r| *r >= 136.0 / 2.0), "{size:?}: {full:?}");
            assert_eq!(
                radii(squared.style(squared.container)),
                [square; 4],
                "{size:?}"
            );
        }
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
            layer[0].background_color,
            color(scheme_color(ColorRole::OnPrimary, false), 255)
        );
        // Nothing else: no frame, no text, and the container is not styled again.
        assert_eq!(layout_items(&hover), 0);
        assert!(
            hover.iter().all(
                |item| !matches!(item, MountItem::ViewStyle { id, .. } if *id == mounted.container)
            ),
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
