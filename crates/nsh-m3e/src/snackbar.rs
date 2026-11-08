//! The Material 3 snackbar: a short message in an inverse-colored container at elevation level 3, with an optional
//! action and an optional dismiss button, laid out on one row or with the action on a line of its own.
//!
//! The snackbar follows androidx's `Snackbar(snackbarData, …)` in `Snackbar.kt`, at the pinned commit, as [ADR 0012]
//! says: the container, colors and type styles of `SnackbarTokens`, and the spaces and the baseline rules that
//! `Snackbar.kt` writes in place of a token, each cited where it is used.
//!
//! # Which of androidx's two layouts
//!
//! `Snackbar.kt` holds two layouts of each arrangement, chosen by `ComposeMaterial3Flags.isSnackbarStylingFixEnabled`:
//! `OneRowSnackbar` and `NewLineButtonSnackbar` when it is true, `LegacyOneRowSnackbar` and
//! `LegacyNewLineButtonSnackbar` when it is false. At the pinned commit the flag is `false`
//! (`ComposeMaterial3Flags.kt`, line 58), so every snackbar androidx shows takes the legacy layouts, and so does this
//! one.
//!
//! # Its action and its dismiss button
//!
//! androidx builds them from its standard `TextButton` and `IconButton`, not the Expressive ones that
//! [`crate::button`] and [`crate::icon_button`] follow: the action is a text button with the content padding of
//! `ButtonDefaults.TextButtonContentPadding` and no shape that morphs, in `SnackbarTokens.ActionLabelTextColor`; the
//! dismiss button an icon button of `IconButtonDefaults.smallContainerSize()` in `SnackbarTokens.IconColor`. Their
//! colors are the snackbar's, not their own style's. So this module draws both itself, from the same values.
//!
//! The dismiss button shows Material Symbols' `close` ([`crate::material_symbols::outlined::CLOSE`], [ADR 0017]),
//! where androidx draws `Icons.Filled.Close` of its own internal icon set: the same cross, from another path.
//!
//! Left out: the tooltip androidx shows over the dismiss button (`TooltipBox`, line 250), which comes with the
//! tooltip component.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_core::{EventKind, EventPayload, LineHeight, TextLine};

use crate::elevation::Elevation;
use crate::material_symbols::outlined::CLOSE;
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::androidx::{
    button_small_tokens, small_icon_button_tokens, snackbar_tokens, state_tokens,
};
use crate::tokens::{Shape, TypeStyle};

/// Where a [`Snackbar`]'s action goes.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum SnackbarActionPlacement {
    /// On the message's row, after it: androidx's default (`actionOnNewLine = false`).
    #[default]
    SameLine,
    /// On a line of its own, below the message, at the end: for an action whose label is long (`actionOnNewLine =
    /// true`). A snackbar with no action lays its message out on one row whatever this says, as androidx does.
    NewLine,
}

/// androidx `Snackbar.kt`, `Snackbar(snackbarData, …)`'s `modifier.padding(12.dp)` (line 272): the space around the
/// snackbar, on every side, outside its container.
const MARGIN: Points = Points(12.0);

/// androidx `Snackbar.kt`, `ContainerMaxWidth` (line 604): the snackbar fills the width it is given, up to this.
const CONTAINER_MAX_WIDTH: Points = Points(600.0);

/// androidx `Snackbar.kt`, `HeightToFirstLine` (line 606): from the top of a snackbar of more than one line to the
/// first baseline of its message.
const HEIGHT_TO_FIRST_LINE: Points = Points(30.0);

/// androidx `Snackbar.kt`, `HorizontalSpacing` (line 608): the space before the message.
const HORIZONTAL_SPACING: Points = Points(16.0);

/// androidx `Snackbar.kt`, `HorizontalSpacingButtonSide` (line 610): the space after the action when there is no
/// dismiss button, and after the message on its own line.
const HORIZONTAL_SPACING_BUTTON_SIDE: Points = Points(8.0);

/// androidx `Snackbar.kt`, `SeparateButtonExtraY` (line 612): below the action on its own line.
const SEPARATE_BUTTON_EXTRA_Y: Points = Points(2.0);

/// androidx `Snackbar.kt`, `LegacySnackbarVerticalPadding` (line 614): above and below the message on the action's row.
const LEGACY_VERTICAL_PADDING: Points = Points(6.0);

/// androidx `Snackbar.kt`, `TextEndExtraSpacing` (line 616): between the message and the action when there is no
/// dismiss button.
const TEXT_END_EXTRA_SPACING: Points = Points(8.0);

/// androidx `Snackbar.kt`, `LongButtonVerticalOffset` (line 618): from the message's last baseline to the action on
/// its own line.
const LONG_BUTTON_VERTICAL_OFFSET: Points = Points(12.0);

/// androidx `Button.kt`, `ButtonDefaults.TextButtonHorizontalPadding` (line 1025): the action's space on each side of
/// its label.
const ACTION_HORIZONTAL_PADDING: Points = Points(12.0);

/// androidx `Button.kt`, `ButtonDefaults.MinWidth` (line 1063): the action's narrowest width.
const ACTION_MIN_WIDTH: Points = Points(58.0);

/// The action's height: androidx `ButtonDefaults.MinHeight` (`Button.kt`, line 1069), `ButtonSmallTokens.ContainerHeight`
/// on a touch screen. Its label, a line of label large (20 points) between `ButtonVerticalPadding` (8, line 925) on each
/// side, is shorter, so the height is this. `MinHeight` is 36 where androidx sizes components for a precise pointer;
/// [`crate::button`] keeps the token too.
const ACTION_HEIGHT: Points = button_small_tokens::CONTAINER_HEIGHT;

/// The dismiss button's width: androidx `IconButtonDefaults.smallContainerSize()` (`IconButtonDefaults.kt`, line 1105)
/// at its default uniform width, the small icon and `SmallIconButtonTokens.DefaultLeadingSpace` on each side of it.
const DISMISS_WIDTH: Points = Points(
    small_icon_button_tokens::ICON_SIZE.0 + 2.0 * small_icon_button_tokens::DEFAULT_LEADING_SPACE.0,
);

/// The dismiss button's height: `smallContainerSize()`'s, `SmallIconButtonTokens.ContainerHeight`.
const DISMISS_HEIGHT: Points = small_icon_button_tokens::CONTAINER_HEIGHT;

/// The props of a [`Snackbar`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SnackbarProps {
    /// What the snackbar says.
    pub message: Prop<SharedString>,
    /// The label of its action, if it has one: a text button after the message (androidx `SnackbarVisuals.actionLabel`).
    pub action_label: Prop<Option<SharedString>>,
    /// What screen readers say for its dismiss button, if it has one: an icon button at its end (androidx
    /// `SnackbarVisuals.withDismissAction`, named `Strings.SnackbarDismiss` there). Give one to a snackbar that stays
    /// until the user acts, as androidx recommends.
    pub dismiss_label: Prop<Option<SharedString>>,
    /// Where its action goes.
    #[prop(default)]
    pub action_placement: Prop<SnackbarActionPlacement>,
}

/// The events of a [`Snackbar`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SnackbarEvent {
    /// Its action was pressed (androidx `SnackbarData.performAction`).
    Action,
    /// Its dismiss button was pressed (androidx `SnackbarData.dismiss`).
    Dismiss,
}

/// Where the message's baselines are, from the top of its text, and where its text ends.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Baselines {
    /// The first line's baseline.
    first: Points,
    /// The last line's baseline.
    last: Points,
    /// The bottom of the last line: the text's height.
    bottom: Points,
}

impl Baselines {
    /// The baselines of `lines`, each where NSH draws a line's text: centered in the line's height, as androidx centers
    /// it ([`TypeStyle::line_height`]), its ascender above the baseline. `None` with no line, which androidx counts as
    /// one line (`hasText`).
    fn of(lines: &[TextLine]) -> Option<Self> {
        let baseline = |line: &TextLine| {
            let leading = line.height.0 - line.ascender.0 - line.descender.0;
            Points(line.y.0 + leading / 2.0 + line.ascender.0)
        };
        let (first, last) = (lines.first()?, lines.last()?);
        Some(Self {
            first: baseline(first),
            last: baseline(last),
            bottom: Points(last.y.0 + last.height.0),
        })
    }

    /// Whether the message takes more than one line: androidx's `firstTextBaseline == lastTextBaseline` is false.
    fn several_lines(self) -> bool {
        self.first != self.last
    }
}

/// Whether a message laid out as `baselines` (none yet, or none at all) takes more than one line.
fn several_lines(baselines: Option<Baselines>) -> bool {
    baselines.is_some_and(Baselines::several_lines)
}

/// The least height of a snackbar whose message is on the action's row: `SnackbarTokens.SingleLineContainerHeight`
/// for one line, `TwoLinesContainerHeight` for more (`LegacyOneRowSnackbar`).
fn row_min_height(baselines: Option<Baselines>) -> Points {
    if several_lines(baselines) {
        snackbar_tokens::TWO_LINES_CONTAINER_HEIGHT
    } else {
        snackbar_tokens::SINGLE_LINE_CONTAINER_HEIGHT
    }
}

/// Where a message on the action's row starts, below the top of the snackbar, when it takes more than one line: where
/// its first baseline is 30 points down (`LegacyOneRowSnackbar`, `textPlaceY = baselineOffset - firstTextBaseline`),
/// its box's 6 points above it included, never above the top. `None` for one line (or none yet), which is centered.
fn several_lines_top(baselines: Option<Baselines>) -> Option<Points> {
    baselines
        .filter(|b| b.several_lines())
        .map(|b| Points((HEIGHT_TO_FIRST_LINE.0 - LEGACY_VERTICAL_PADDING.0 - b.first.0).max(0.0)))
}

/// The spaces above and below a message on its own line: its first baseline 30 points below the top, its last 12
/// points above the bottom (`LegacyNewLineButtonSnackbar`, `paddingFromBaseline(HeightToFirstLine,
/// LongButtonVerticalOffset)`), each never less than none. None before the message is laid out.
fn new_line_padding(baselines: Option<Baselines>) -> (Points, Points) {
    baselines.map_or((Points(0.0), Points(0.0)), |b| {
        (
            Points((HEIGHT_TO_FIRST_LINE.0 - b.first.0).max(0.0)),
            Points((LONG_BUTTON_VERTICAL_OFFSET.0 - (b.bottom.0 - b.last.0)).max(0.0)),
        )
    })
}

/// The color of `role`, at no opacity.
fn solid(role: crate::color::ColorRole) -> AdaptiveColor {
    tint_color(Some(Tint::solid(role)))
}

/// The radius of a rounded shape's corners, which `SnackbarTokens.ContainerShape` gives the same on every corner.
fn radius(shape: Shape) -> Points {
    match shape {
        Shape::Rounded(corners) => corners.top_start,
        // Neither is a snackbar's shape at the pinned commit; a square corner is the safe reading of either.
        Shape::Rectangle | Shape::Circle => Points(0.0),
    }
}

/// A line of text in `style` and `color`.
fn styled_text(
    content: impl IntoReactive<SharedString> + 'static,
    style: TypeStyle,
    color: AdaptiveColor,
) -> Text {
    text(content)
        .text_color(color)
        .text_size(style.size)
        .line_height(LineHeight::Points(style.line_height))
        .letter_spacing(style.tracking)
        .style_with(move |s| s.font_weight(style.weight))
}

/// How far presses also count around a target of `height`, so that it has a 48-point target (androidx
/// `minimumInteractiveComponentSize`) with no change to its size.
fn touch_slop(height: Points) -> EdgeInsets {
    EdgeInsets::all(px(((MIN_TOUCH_TARGET.0 - height.0) / 2.0).max(0.0)))
}

/// The state layer of a press target: `color` over it, in its fully rounded shape, shown by the target's own hover,
/// focus and press (androidx's ripple, `RippleDefaults.RippleAlpha`, at `StateTokens`' opacities).
fn state_layer(color: AdaptiveColor) -> View {
    view()
        .absolute()
        .inset(px(0.0))
        .rounded_full()
        .bg(color)
        .opacity(0.0)
        .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
        .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
        .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
}

/// The action: androidx's `TextButton` with the snackbar's action color, its label in label large.
fn action(label: SharedString, cx: Scope<SnackbarEvent>) -> impl IntoElement {
    let color = solid(snackbar_tokens::ACTION_LABEL_TEXT_COLOR);
    nsh_base::button()
        .on_press(move || cx.emit(SnackbarEvent::Action))
        .hit_slop(touch_slop(ACTION_HEIGHT))
        .group()
        .flex_row()
        .items_center()
        .justify_center()
        .h(ACTION_HEIGHT)
        .min_w(ACTION_MIN_WIDTH)
        .px(ACTION_HORIZONTAL_PADDING)
        .child(state_layer(color))
        .child(styled_text(
            label,
            snackbar_tokens::ACTION_LABEL_TEXT_FONT,
            color,
        ))
}

/// The dismiss button: androidx's standard `IconButton` with the snackbar's icon color, its icon `close`.
fn dismiss(label: SharedString, cx: Scope<SnackbarEvent>) -> impl IntoElement {
    let color = solid(snackbar_tokens::ICON_COLOR);
    nsh_base::button()
        .accessibility_label(Some(label))
        .on_press(move || cx.emit(SnackbarEvent::Dismiss))
        .hit_slop(touch_slop(DISMISS_HEIGHT))
        .group()
        .items_center()
        .justify_center()
        .w(DISMISS_WIDTH)
        .h(DISMISS_HEIGHT)
        .child(state_layer(color))
        .child(
            canvas(|| CLOSE.drawing(snackbar_tokens::ICON_SIZE))
                .w(snackbar_tokens::ICON_SIZE)
                .h(snackbar_tokens::ICON_SIZE)
                .fill_color(color),
        )
}

/// A Material 3 snackbar: a message, and an optional action and dismiss button.
///
/// It is the snackbar's look and layout only. When it shows, for how long, and how it comes and goes is the app's, or
/// `nsh-base`'s toasts' (`Toast`, `ToastStack`, `ToastManager`), as androidx's `SnackbarHost` is for its own.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::snackbar::snackbar;
///
/// fn archived() -> impl IntoElement {
///     snackbar()
///         .message("Conversation archived")
///         .action_label(Some("Undo".into()))
///         .on_action(|| println!("undone"))
/// }
/// ```
#[component]
pub fn Snackbar(props: SnackbarProps, cx: Scope<SnackbarEvent>) -> impl IntoElement {
    let baselines = signal(None::<Baselines>);
    let has_dismiss = move || props.dismiss_label.get().is_some();
    let new_line = move || {
        props.action_label.get().is_some()
            && props.action_placement.get() == SnackbarActionPlacement::NewLine
    };

    let message = styled_text(
        move || props.message.get(),
        snackbar_tokens::SUPPORTING_TEXT_FONT,
        solid(snackbar_tokens::SUPPORTING_TEXT_COLOR),
    )
    .on(EventKind::TextLayout, move |payload: &EventPayload| {
        if let EventPayload::TextLayout(lines) = payload {
            baselines.set(Baselines::of(lines));
        }
    });

    // The message's box: 6 points above and below it on the action's row, or the spaces from its baselines.
    let message_box =
        view()
            .style_with(move |s| {
                let baselines = baselines.get();
                if new_line() {
                    let (top, bottom) = new_line_padding(baselines);
                    s.w_full()
                        .pt(top)
                        .pb(bottom)
                        .pe(HORIZONTAL_SPACING_BUTTON_SIDE)
                } else {
                    let s = s.flex_1().min_w(px(0.0)).py(LEGACY_VERTICAL_PADDING).me(
                        if has_dismiss() {
                            Points(0.0)
                        } else {
                            TEXT_END_EXTRA_SPACING
                        },
                    );
                    match several_lines_top(baselines) {
                        Some(top) => s.self_start().mt(top),
                        None => s.self_center(),
                    }
                }
            })
            .child(message);

    let buttons = move || {
        let action = props.action_label.get().map(|label| action(label, cx));
        let dismiss = props.dismiss_label.get().map(|label| dismiss(label, cx));
        (action, dismiss)
    };

    let shadows =
        Elevation::from_points(snackbar_tokens::CONTAINER_ELEVATION).map(Elevation::shadows);

    let container = view()
        .w_full()
        .max_w(CONTAINER_MAX_WIDTH)
        .rounded(radius(snackbar_tokens::CONTAINER_SHAPE))
        .bg(solid(snackbar_tokens::CONTAINER_COLOR))
        .shadow(shadows.into_iter().flatten())
        .ps(HORIZONTAL_SPACING)
        .style_with(move |s| {
            if new_line() {
                s.flex_col().pb(SEPARATE_BUTTON_EXTRA_Y)
            } else {
                // A single line is centered in 48 points, more lines take at least 68 (`SnackbarTokens`); the action
                // and the dismiss button are centered on the row. With one line, androidx puts the action's baseline on
                // the message's: both are a line of 20 points centered at the same height (body medium and label
                // large), so centering does the same.
                s.flex_row()
                    .items_center()
                    .min_h(row_min_height(baselines.get()))
                    .pe(if has_dismiss() {
                        Points(0.0)
                    } else {
                        HORIZONTAL_SPACING_BUTTON_SIDE
                    })
            }
        })
        .child(message_box)
        .child(dynamic(move || {
            let (action, dismiss) = buttons();
            let row = view()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .child(action)
                .child(dismiss);
            if new_line() {
                // At the end of its own line, with 8 points after it when there is no dismiss button.
                row.self_end().pe(if has_dismiss() {
                    Points(0.0)
                } else {
                    HORIZONTAL_SPACING_BUTTON_SIDE
                })
            } else {
                row
            }
        }));
    view().w_full().p(MARGIN).child(container)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::canvas::{self, CanvasOp, Drawing};
    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{Recording, layout_items};
    use crate::color::{ColorRole, ColorScheme};

    /// What the probe places: a snackbar with these props, and lines counting its two events.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        message: Prop<SharedString>,
        action_label: Prop<Option<SharedString>>,
        dismiss_label: Prop<Option<SharedString>>,
        #[prop(default)]
        action_placement: Prop<SnackbarActionPlacement>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let actions = signal(0_u32);
        let dismisses = signal(0_u32);
        view()
            .child(
                snackbar()
                    .message(props.message)
                    .action_label(props.action_label)
                    .dismiss_label(props.dismiss_label)
                    .action_placement(props.action_placement)
                    .on_action(move || actions.update(|n| *n += 1))
                    .on_dismiss(move || dismisses.update(|n| *n += 1)),
            )
            .child(text(move || format!("actions: {}", actions.get())))
            .child(text(move || format!("dismisses: {}", dismisses.get())))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        message: &'static str,
        action: Option<&'static str>,
        dismiss: Option<&'static str>,
        placement: SnackbarActionPlacement,
        width: f32,
        dark: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                message: "Saved",
                action: None,
                dismiss: None,
                placement: SnackbarActionPlacement::SameLine,
                width: 390.0,
                dark: false,
            }
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            message: Prop::new(config.message.into()),
            action_label: Prop::new(config.action.map(Into::into)),
            dismiss_label: Prop::new(config.dismiss.map(Into::into)),
            action_placement: Prop::new(config.placement),
        });
        // The web has a mouse to hover with.
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
        app.set_viewport(Size::new(Points(config.width), Points(800.0)))
            .unwrap();
        app.wait_for("text:actions: 0").unwrap();
        app.settle();
        (app, recording)
    }

    /// The views the engine mounted, with the last frame, style, drawing and button props each was sent.
    #[derive(Default)]
    struct Tree {
        parents: HashMap<NodeId, NodeId>,
        frames: HashMap<NodeId, Bounds<Points>>,
        styles: HashMap<NodeId, ViewStyle>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        texts: HashMap<NodeId, String>,
        /// Each view with the button role, its accessibility label and its hit slop.
        buttons: HashMap<NodeId, (Option<String>, EdgeInsets)>,
    }

    impl Tree {
        fn of(items: &[MountItem]) -> Self {
            let mut tree = Self::default();
            for item in items {
                match item {
                    MountItem::Insert { parent, child, .. } => {
                        tree.parents.insert(*child, *parent);
                    }
                    MountItem::Frame { id, frame } => {
                        tree.frames.insert(*id, *frame);
                    }
                    MountItem::ViewStyle { id, style } => {
                        tree.styles.insert(*id, (**style).clone());
                    }
                    MountItem::Canvas { id, drawing } => {
                        tree.drawings.insert(*id, Arc::clone(drawing));
                    }
                    MountItem::Text { id, text, .. } => {
                        tree.texts.insert(*id, text.text.clone());
                    }
                    MountItem::ViewProps { id, props }
                        if props.accessibility.role == AccessibilityRole::Button =>
                    {
                        tree.buttons.insert(
                            *id,
                            (
                                props.accessibility.label.as_deref().map(str::to_owned),
                                props.hit_slop,
                            ),
                        );
                    }
                    _ => {}
                }
            }
            tree
        }

        fn mount(config: Config) -> Self {
            let (_app, recording) = start(config);
            Self::of(&recording.items())
        }

        /// `id`'s frame in the root's coordinates.
        fn bounds(&self, id: NodeId) -> Bounds<Points> {
            let mut bounds = self.frames[&id];
            let mut at = id;
            while let Some(parent) = self.parents.get(&at) {
                if let Some(frame) = self.frames.get(parent) {
                    bounds.origin.x += frame.origin.x;
                    bounds.origin.y += frame.origin.y;
                }
                at = *parent;
            }
            bounds
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        /// The snackbar's container: the one view whose background is inverse surface.
        fn container(&self, dark: bool) -> NodeId {
            let background = color(scheme_color(ColorRole::InverseSurface, dark), 255);
            let found: Vec<NodeId> = self
                .styles
                .iter()
                .filter(|(_, style)| style.background_color == background)
                .map(|(id, _)| *id)
                .collect();
            assert_eq!(
                found.len(),
                1,
                "one view with the background {background:?}"
            );
            found[0]
        }

        /// The text view that says `content`.
        fn text(&self, content: &str) -> NodeId {
            *self
                .texts
                .iter()
                .find(|(_, text)| text.as_str() == content)
                .map(|(id, _)| id)
                .expect("the text")
        }

        /// The press target that holds `id`.
        fn button_of(&self, id: NodeId) -> NodeId {
            let mut at = id;
            while !self.buttons.contains_key(&at) {
                at = self.parents[&at];
            }
            at
        }

        /// The press target that screen readers name `label`.
        fn labelled(&self, label: &str) -> NodeId {
            *self
                .buttons
                .iter()
                .find(|(_, (name, _))| name.as_deref() == Some(label))
                .map(|(id, _)| id)
                .expect("the labelled button")
        }

        /// The children of `parent` the engine mounted.
        fn children(&self, parent: NodeId) -> Vec<NodeId> {
            self.parents
                .iter()
                .filter(|(_, p)| **p == parent)
                .map(|(child, _)| *child)
                .collect()
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

    /// The headless engine's text metrics (`ApproxTextEngine`): a line is 1.2 times the font size tall and a
    /// character 0.55 times as wide, whatever the line height asks.
    fn line(style: TypeStyle) -> f32 {
        style.size.0 * 1.2
    }

    fn width(content: &str, style: TypeStyle) -> f32 {
        f32::from(u16::try_from(content.chars().count()).unwrap()) * style.size.0 * 0.55
    }

    const MESSAGE: TypeStyle = snackbar_tokens::SUPPORTING_TEXT_FONT;
    const LABEL: TypeStyle = snackbar_tokens::ACTION_LABEL_TEXT_FONT;

    /// Whether `a` is `b` once the engine rounds frames to the display's pixels, one point here: a length whose two
    /// edges are rounded can be a point off.
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() <= 1.0
    }

    fn assert_near(actual: f32, expected: f32, what: &str) {
        assert!(near(actual, expected), "{what}: {actual} is not {expected}");
    }

    // The pure rules, with lines a real text engine reports: several lines, a line height taller than the font.

    /// A line of 20 points at `y`, its ascender 15 and descender 4: one point of leading, half above.
    fn text_line(y: f32) -> TextLine {
        TextLine {
            y: Points(y),
            height: Points(20.0),
            ascender: Points(15.0),
            descender: Points(4.0),
            ..TextLine::default()
        }
    }

    #[test]
    fn baselines_are_where_each_line_draws_its_text() {
        assert_eq!(Baselines::of(&[]), None);
        let one = Baselines::of(&[text_line(0.0)]).unwrap();
        assert_eq!(
            one,
            Baselines {
                first: Points(15.5),
                last: Points(15.5),
                bottom: Points(20.0),
            }
        );
        assert!(!one.several_lines());
        let two = Baselines::of(&[text_line(0.0), text_line(20.0)]).unwrap();
        assert_eq!(
            two,
            Baselines {
                first: Points(15.5),
                last: Points(35.5),
                bottom: Points(40.0),
            }
        );
        assert!(two.several_lines());
        // A line further down is counted from its own top.
        let moved = Baselines::of(&[text_line(3.0)]).unwrap();
        assert_eq!(moved.first, Points(18.5));
        assert_eq!(moved.bottom, Points(23.0));
    }

    #[test]
    fn one_line_is_48_points_tall_and_more_lines_68() {
        let one = Baselines::of(&[text_line(0.0)]);
        let two = Baselines::of(&[text_line(0.0), text_line(20.0)]);
        // No text yet counts as one line, as androidx's `hasText` does.
        assert!(!several_lines(None));
        assert!(!several_lines(one));
        assert!(several_lines(two));
        assert_eq!(row_min_height(None), Points(48.0));
        assert_eq!(row_min_height(one), Points(48.0));
        assert_eq!(row_min_height(two), Points(68.0));
        assert_eq!(
            row_min_height(two),
            snackbar_tokens::TWO_LINES_CONTAINER_HEIGHT
        );
        assert_eq!(
            row_min_height(one),
            snackbar_tokens::SINGLE_LINE_CONTAINER_HEIGHT
        );
    }

    #[test]
    fn several_lines_start_where_the_first_baseline_is_30_points_down() {
        // One line, or none yet, is centered instead.
        assert_eq!(several_lines_top(None), None);
        assert_eq!(several_lines_top(Baselines::of(&[text_line(0.0)])), None);
        let two = Baselines::of(&[text_line(0.0), text_line(20.0)]);
        // 30 - 6 (the box's padding) - 15.5: the box starts 8.5 down, its text 14.5, its baseline 30.
        assert_eq!(several_lines_top(two), Some(Points(8.5)));
        // A first baseline lower than 24 points would start above the top: it starts at the top.
        let low = Baselines {
            first: Points(25.0),
            last: Points(45.0),
            bottom: Points(50.0),
        };
        assert_eq!(several_lines_top(Some(low)), Some(Points(0.0)));
    }

    #[test]
    fn a_message_on_its_own_line_is_padded_from_its_baselines() {
        assert_eq!(new_line_padding(None), (Points(0.0), Points(0.0)));
        // First baseline 30 down: 14.5 above 15.5. Last baseline 12 up: 7.5 below a descent of 4.5.
        let two = Baselines::of(&[text_line(0.0), text_line(20.0)]);
        assert_eq!(new_line_padding(two), (Points(14.5), Points(7.5)));
        // Neither is ever negative.
        let tall = Baselines {
            first: Points(31.0),
            last: Points(51.0),
            bottom: Points(64.0),
        };
        assert_eq!(new_line_padding(Some(tall)), (Points(0.0), Points(0.0)));
    }

    // The component, on the headless engine.

    #[test]
    fn the_container_is_inverse_surface_with_extra_small_corners_at_level_3() {
        for dark in [false, true] {
            let tree = Tree::mount(Config {
                dark,
                ..Config::default()
            });
            let style = tree.style(tree.container(dark));
            // `SnackbarTokens.ContainerShape`: `CornerExtraSmall`, 4 points.
            assert_eq!(style.border_radius, Points(4.0), "dark: {dark}");
            // `SnackbarTokens.ContainerElevation`: level 3, as ADR 0018 draws it.
            let shadows = style.effects.map(|e| e.box_shadows.clone());
            assert_eq!(
                shadows,
                Some(Elevation::Level3.shadows().to_vec()),
                "dark: {dark}"
            );
            // The message in inverse on surface and body medium.
            let (app, _) = start(Config {
                dark,
                ..Config::default()
            });
            let span = app.screen().span_style("Saved").unwrap();
            assert_eq!(
                span.color,
                color(scheme_color(ColorRole::InverseOnSurface, dark), 255),
                "dark: {dark}"
            );
            assert_eq!(span.font_size, MESSAGE.size);
            assert_eq!(span.line_height, MESSAGE.line_height);
            assert_eq!(span.letter_spacing, MESSAGE.tracking);
            assert_eq!(span.font_weight, MESSAGE.weight.0);
        }
    }

    #[test]
    fn it_fills_the_width_up_to_600_points_12_points_in() {
        for (viewport, expected) in [(390.0, 366.0), (624.0, 600.0), (800.0, 600.0)] {
            let tree = Tree::mount(Config {
                width: viewport,
                ..Config::default()
            });
            let bounds = tree.bounds(tree.container(false));
            assert_eq!(bounds.size.width, Points(expected), "viewport {viewport}");
            assert_eq!(
                bounds.origin,
                point(Points(12.0), Points(12.0)),
                "viewport {viewport}"
            );
            // And 12 points below it, before the next line.
            let below = tree.bounds(tree.text("actions: 0"));
            assert_near(below.origin.y.0, 12.0 + 48.0 + 12.0, "below");
        }
    }

    #[test]
    fn one_line_is_centered_in_48_points_after_16() {
        let tree = Tree::mount(Config::default());
        let container = tree.bounds(tree.container(false));
        assert_eq!(container.size.height, Points(48.0));
        let message = tree.bounds(tree.text("Saved"));
        assert_near(message.origin.x.0 - container.origin.x.0, 16.0, "start");
        assert_near(
            message.origin.y.0 - container.origin.y.0,
            (48.0 - line(MESSAGE)) / 2.0,
            "top",
        );
        // With no button, the message may run to 16 points before the end: 8 of the row and 8 after the text.
        let message_box = tree.bounds(tree.parents[&tree.text("Saved")]);
        assert_near(
            message_box.origin.x.0 + message_box.size.width.0,
            container.origin.x.0 + container.size.width.0 - 16.0,
            "end",
        );
        assert!(tree.buttons.is_empty());
    }

    #[test]
    fn the_action_is_a_text_button_in_inverse_primary_at_the_end() {
        for dark in [false, true] {
            let (app, recording) = start(Config {
                action: Some("Try again"),
                dark,
                ..Config::default()
            });
            let tree = Tree::of(&recording.items());
            let container = tree.bounds(tree.container(dark));
            let action = tree.bounds(tree.button_of(tree.text("Try again")));
            // 12 points on each side of its label, 40 tall, centered on the row; 8 points after it.
            assert_near(
                action.size.width.0,
                width("Try again", LABEL) + 24.0,
                "width",
            );
            assert_eq!(action.size.height, Points(40.0));
            assert_near(
                action.origin.y.0 - container.origin.y.0,
                4.0,
                "centered in 48",
            );
            assert_near(
                action.origin.x.0 + action.size.width.0,
                container.origin.x.0 + container.size.width.0 - 8.0,
                "end",
            );
            let label = tree.bounds(tree.text("Try again"));
            assert_near(label.origin.x.0 - action.origin.x.0, 12.0, "label start");
            // The message stops 8 points before it.
            let message_box = tree.bounds(tree.parents[&tree.text("Saved")]);
            assert_near(
                message_box.origin.x.0 + message_box.size.width.0,
                action.origin.x.0 - 8.0,
                "message end",
            );
            let span = app.screen().span_style("Try again").unwrap();
            assert_eq!(
                span.color,
                color(scheme_color(ColorRole::InversePrimary, dark), 255)
            );
            assert_eq!(span.font_size, LABEL.size);
            assert_eq!(span.line_height, LABEL.line_height);
            assert_eq!(span.font_weight, LABEL.weight.0);
        }
    }

    #[test]
    fn a_short_action_is_58_points_wide() {
        let tree = Tree::mount(Config {
            action: Some("Undo"),
            ..Config::default()
        });
        let action = tree.bounds(tree.button_of(tree.text("Undo")));
        // 4 characters and their padding are 54.8 points: `ButtonDefaults.MinWidth` widens it.
        assert!(width("Undo", LABEL) + 24.0 < 58.0);
        assert_eq!(action.size.width, Points(58.0));
    }

    /// The color `drawing` fills with.
    fn fill(drawing: &Drawing) -> Option<Color> {
        drawing.ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    #[test]
    fn the_dismiss_button_is_a_40_point_close_icon_at_the_very_end() {
        for dark in [false, true] {
            let tree = Tree::mount(Config {
                action: Some("Undo"),
                dismiss: Some("Dismiss"),
                dark,
                ..Config::default()
            });
            let container = tree.bounds(tree.container(dark));
            let button = tree.labelled("Dismiss");
            let dismiss = tree.bounds(button);
            assert_eq!(dismiss.size, Size::new(Points(40.0), Points(40.0)));
            assert_near(dismiss.origin.y.0 - container.origin.y.0, 4.0, "centered");
            // No space after it, and the action right before it, with no space between.
            assert_near(
                dismiss.origin.x.0 + dismiss.size.width.0,
                container.origin.x.0 + container.size.width.0,
                "end",
            );
            let action = tree.bounds(tree.button_of(tree.text("Undo")));
            assert_near(
                action.origin.x.0 + action.size.width.0,
                dismiss.origin.x.0,
                "action end",
            );
            // The message runs to the action, with no extra space.
            let message_box = tree.bounds(tree.parents[&tree.text("Saved")]);
            assert_near(
                message_box.origin.x.0 + message_box.size.width.0,
                action.origin.x.0,
                "message end",
            );
            // Its icon: `close`, 24 points, centered, in inverse on surface.
            let icons: Vec<NodeId> = tree
                .children(button)
                .into_iter()
                .filter(|id| tree.drawings.contains_key(id))
                .collect();
            let [icon] = icons[..] else {
                panic!("one canvas in the dismiss button: {icons:?}");
            };
            let bounds = tree.bounds(icon);
            assert_eq!(bounds.size, Size::new(Points(24.0), Points(24.0)));
            assert_near(bounds.origin.x.0 - dismiss.origin.x.0, 8.0, "icon x");
            assert_near(bounds.origin.y.0 - dismiss.origin.y.0, 8.0, "icon y");
            // The same path as `close` at 24 points, filled with the view's fill color in place of none.
            let shape = |ops: &[CanvasOp]| -> Vec<CanvasOp> {
                ops.iter()
                    .filter(|op| !matches!(op, CanvasOp::FillStyle(_)))
                    .cloned()
                    .collect()
            };
            assert_eq!(
                shape(&tree.drawings[&icon].ops),
                shape(&CLOSE.drawing(snackbar_tokens::ICON_SIZE).finish().ops)
            );
            assert_eq!(
                fill(&tree.drawings[&icon]),
                Some(color(scheme_color(ColorRole::InverseOnSurface, dark), 255))
            );
        }
    }

    #[test]
    fn a_dismiss_button_alone_ends_the_row_and_the_message_runs_to_it() {
        let tree = Tree::mount(Config {
            dismiss: Some("Dismiss"),
            ..Config::default()
        });
        let container = tree.bounds(tree.container(false));
        let dismiss = tree.bounds(tree.labelled("Dismiss"));
        assert_near(
            dismiss.origin.x.0 + dismiss.size.width.0,
            container.origin.x.0 + container.size.width.0,
            "end",
        );
        let message_box = tree.bounds(tree.parents[&tree.text("Saved")]);
        assert_near(
            message_box.origin.x.0 + message_box.size.width.0,
            dismiss.origin.x.0,
            "message end",
        );
        assert_eq!(container.size.height, Points(48.0));
    }

    #[test]
    fn each_button_reports_its_own_press() -> Result<(), String> {
        let (mut app, _) = start(Config {
            action: Some("Undo"),
            dismiss: Some("Dismiss"),
            ..Config::default()
        });
        app.press("Undo")?;
        assert!(app.shows("actions: 1"));
        assert!(app.shows("dismisses: 0"));
        app.press("label:Dismiss")?;
        assert!(app.shows("actions: 1"));
        assert!(app.shows("dismisses: 1"));
        Ok(())
    }

    #[test]
    fn both_buttons_take_presses_within_48_points() {
        let tree = Tree::mount(Config {
            action: Some("Undo"),
            dismiss: Some("Dismiss"),
            ..Config::default()
        });
        let slop = EdgeInsets::all(px(4.0));
        let action = tree.button_of(tree.text("Undo"));
        assert_eq!(tree.buttons[&action].1, slop);
        assert_eq!(tree.buttons[&tree.labelled("Dismiss")].1, slop);
        assert_eq!(tree.buttons.len(), 2);
    }

    #[test]
    fn hovering_a_button_shows_its_state_layer_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            action: Some("Undo"),
            dismiss: Some("Dismiss"),
            ..Config::default()
        });
        let tree = Tree::of(&recording.items());
        for (selector, button, role) in [
            (
                "Undo",
                tree.button_of(tree.text("Undo")),
                ColorRole::InversePrimary,
            ),
            (
                "label:Dismiss",
                tree.labelled("Dismiss"),
                ColorRole::InverseOnSurface,
            ),
        ] {
            let layers: Vec<NodeId> = tree
                .children(button)
                .into_iter()
                .filter(|id| {
                    tree.style(*id).background_color == color(scheme_color(role, false), 255)
                })
                .collect();
            let [layer] = layers[..] else {
                panic!("one state layer in {selector}: {layers:?}");
            };
            assert_eq!(tree.style(layer).opacity, 0.0, "{selector} at rest");
            assert_eq!(tree.bounds(layer), tree.bounds(button), "{selector}");
            app.hover(selector)?;
            let hover = recording.in_step(&format!("hover({selector})"));
            let opacity = Tree::of(&hover).style(layer).opacity;
            assert!(
                (opacity - state_tokens::HOVER_STATE_LAYER_OPACITY).abs() < 1e-6,
                "{selector} hovered: {opacity}"
            );
            assert_eq!(layout_items(&hover), 0, "{selector}");
        }
        Ok(())
    }

    #[test]
    fn a_new_line_action_goes_below_the_message_at_the_end() {
        for dismiss in [None, Some("Dismiss")] {
            let tree = Tree::mount(Config {
                action: Some("Undo"),
                dismiss,
                placement: SnackbarActionPlacement::NewLine,
                ..Config::default()
            });
            let container = tree.bounds(tree.container(false));
            let message = tree.bounds(tree.text("Saved"));
            let message_box = tree.bounds(tree.parents[&tree.text("Saved")]);
            // The headless engine reports one line whose baseline is at its bottom: 30 points down, 12 above the
            // message box's bottom.
            assert_near(
                message.origin.y.0 - container.origin.y.0,
                30.0 - line(MESSAGE),
                "message top",
            );
            assert_near(
                message.origin.x.0 - container.origin.x.0,
                16.0,
                "message start",
            );
            assert_near(
                message_box.origin.y.0 + message_box.size.height.0,
                message.origin.y.0 + message.size.height.0 + 12.0,
                "message bottom",
            );
            // Its line spans the snackbar, whatever the buttons.
            assert_near(
                message_box.origin.x.0 + message_box.size.width.0,
                container.origin.x.0 + container.size.width.0,
                "message line end",
            );
            let action = tree.bounds(tree.button_of(tree.text("Undo")));
            assert_near(
                action.origin.y.0,
                message_box.origin.y.0 + message_box.size.height.0,
                "action top",
            );
            let end = match dismiss {
                Some(label) => {
                    let dismiss = tree.bounds(tree.labelled(label));
                    assert_near(dismiss.origin.y.0, action.origin.y.0, "dismiss top");
                    assert_near(
                        dismiss.origin.x.0 + dismiss.size.width.0,
                        container.origin.x.0 + container.size.width.0,
                        "dismiss end",
                    );
                    dismiss.origin.x.0
                }
                None => container.origin.x.0 + container.size.width.0 - 8.0,
            };
            assert_near(action.origin.x.0 + action.size.width.0, end, "action end");
            // 2 points below the buttons.
            assert_near(
                container.origin.y.0 + container.size.height.0,
                action.origin.y.0 + 40.0 + 2.0,
                "bottom",
            );
        }
    }

    /// A message that takes more than one line at every width these tests use.
    const LONG: &str =
        "Your photos are backed up and will stay on this phone until you remove them";

    #[test]
    fn a_long_message_wraps_where_the_buttons_leave_room() {
        // On the row: up to the action, which the dismiss button follows.
        let tree = Tree::mount(Config {
            message: LONG,
            action: Some("Undo"),
            dismiss: Some("Dismiss"),
            ..Config::default()
        });
        let message = tree.bounds(tree.text(LONG));
        let action = tree.bounds(tree.button_of(tree.text("Undo")));
        assert!(width(LONG, MESSAGE) > 366.0);
        assert_near(
            message.origin.x.0 + message.size.width.0,
            action.origin.x.0,
            "row",
        );
        assert!(
            message.size.height.0 > line(MESSAGE),
            "wrapped: {message:?}"
        );
        // On its own line: up to 8 points before the end, whatever the buttons.
        for dismiss in [None, Some("Dismiss")] {
            let tree = Tree::mount(Config {
                message: LONG,
                action: Some("Undo"),
                dismiss,
                placement: SnackbarActionPlacement::NewLine,
                ..Config::default()
            });
            let container = tree.bounds(tree.container(false));
            let message = tree.bounds(tree.text(LONG));
            assert_near(
                message.origin.x.0 + message.size.width.0,
                container.origin.x.0 + container.size.width.0 - 8.0,
                "own line",
            );
        }
    }

    #[test]
    fn a_new_line_placement_with_no_action_is_one_row() {
        let tree = Tree::mount(Config {
            dismiss: Some("Dismiss"),
            placement: SnackbarActionPlacement::NewLine,
            ..Config::default()
        });
        let container = tree.bounds(tree.container(false));
        assert_eq!(container.size.height, Points(48.0));
        let dismiss = tree.bounds(tree.labelled("Dismiss"));
        assert_near(dismiss.origin.y.0 - container.origin.y.0, 4.0, "on the row");
    }

    /// The vendored `Snackbar.kt` and `ComposeMaterial3Flags.kt`, at the pinned commit (ADR 0012).
    const SNACKBAR_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Snackbar.kt");
    const FLAGS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ComposeMaterial3Flags.kt");
    const BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Button.kt");
    const ICON_BUTTON_DEFAULTS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/IconButtonDefaults.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_androidx_is_still_there() {
        // The legacy layouts are the ones androidx shows.
        assert!(cites(
            FLAGS_KT,
            &["public var isSnackbarStylingFixEnabled: Boolean = false"]
        ));
        for (name, value, ours) in [
            ("ContainerMaxWidth", "600", CONTAINER_MAX_WIDTH),
            ("HeightToFirstLine", "30", HEIGHT_TO_FIRST_LINE),
            ("HorizontalSpacing", "16", HORIZONTAL_SPACING),
            (
                "HorizontalSpacingButtonSide",
                "8",
                HORIZONTAL_SPACING_BUTTON_SIDE,
            ),
            ("SeparateButtonExtraY", "2", SEPARATE_BUTTON_EXTRA_Y),
            (
                "LegacySnackbarVerticalPadding",
                "6",
                LEGACY_VERTICAL_PADDING,
            ),
            ("TextEndExtraSpacing", "8", TEXT_END_EXTRA_SPACING),
            (
                "LongButtonVerticalOffset",
                "12",
                LONG_BUTTON_VERTICAL_OFFSET,
            ),
        ] {
            let declaration = format!("private val {name}");
            let getter = format!("get() = {value}.dp");
            assert!(cites(SNACKBAR_KT, &[&declaration, &getter]), "{name}");
            assert_eq!(ours.0.to_string(), value, "{name}");
        }
        // The layouts this one follows, and the buttons they are given.
        assert!(cites(SNACKBAR_KT, &["val textPlaceY: Int"]));
        assert!(cites(
            SNACKBAR_KT,
            &[
                "val baselineOffset = HeightToFirstLine.roundToPx()",
                "textPlaceY = baselineOffset - firstTextBaseline"
            ]
        ));
        assert!(cites(
            SNACKBAR_KT,
            &["Modifier.paddingFromBaseline(HeightToFirstLine, LongButtonVerticalOffset)"]
        ));
        assert!(cites(SNACKBAR_KT, &["TextButton("]));
        assert!(cites(
            SNACKBAR_KT,
            &["IconButton(", "onClick = { snackbarData.dismiss() },"]
        ));
        assert!(cites(SNACKBAR_KT, &["modifier = modifier.padding(12.dp),"]));
        assert_eq!(MARGIN, Points(12.0));
        assert!(cites(
            SNACKBAR_KT,
            &["@Composable get() = SnackbarTokens.ActionLabelTextColor.value"]
        ));
        // The text button's padding and least size, and the small icon button's size.
        assert!(cites(
            BUTTON_KT,
            &["private val TextButtonHorizontalPadding = 12.dp"]
        ));
        assert_eq!(ACTION_HORIZONTAL_PADDING, Points(12.0));
        assert!(cites(BUTTON_KT, &["public val MinWidth: Dp = 58.dp"]));
        assert_eq!(ACTION_MIN_WIDTH, Points(58.0));
        assert!(cites(
            BUTTON_KT,
            &[
                "public val MinHeight: Dp",
                "get() =",
                "if (shouldUsePrecisionPointerComponentSizing.value) {",
                "36.dp",
                "} else {",
                "ButtonSmallTokens.ContainerHeight"
            ]
        ));
        assert_eq!(ACTION_HEIGHT, Points(40.0));
        assert!(cites(
            ICON_BUTTON_DEFAULTS_KT,
            &[
                "public fun smallContainerSize(",
                "widthOption: IconButtonWidthOption = IconButtonWidthOption.Uniform"
            ]
        ));
        assert_eq!(
            (DISMISS_WIDTH, DISMISS_HEIGHT),
            (Points(40.0), Points(40.0))
        );
        // A line that is not there is not found.
        assert!(!cites(
            SNACKBAR_KT,
            &["private val HeightToFirstLine", "get() = 31.dp"]
        ));
    }
}
