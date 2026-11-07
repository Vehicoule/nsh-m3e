//! What the tests of the four chips share: one app that places any of them, what its views are, the colors they paint
//! and the paths they trace, read from the mount items the app sends. The icon button's tests are the model.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
use nsh_core::prelude::*;
use nsh_core::{
    AccessibilityRole, AccessibilityState, Color, MountItem, NodeId, ResolvedTextStyle, ViewKind,
    ViewStyle,
};
use nsh_engine::{Clock, EngineConfig, Root};
use nsh_testing::Headless;

use crate::animation_probe::{Recording, StartedSpring};
use crate::assist_chip::assist_chip;
use crate::color::{ColorRole, ColorScheme};
use crate::filter_chip::filter_chip;
use crate::input_chip::input_chip;
use crate::material_symbols::outlined;
use crate::morph_probe::{Step, paths, replay, spring_in_with};
use crate::motion::SpringToken;
use crate::suggestion_chip::suggestion_chip;

/// The kind of chip a probe places.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub(crate) enum Kind {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

pub(crate) const KINDS: [Kind; 4] = [Kind::Assist, Kind::Filter, Kind::Input, Kind::Suggestion];

/// The icons a chip is given.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Icons {
    #[default]
    None,
    Leading,
    Trailing,
    Both,
}

/// What a chip is in: selected or not, and enabled or not.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Condition {
    #[default]
    Rest,
    Selected,
    Disabled,
    SelectedDisabled,
}

/// What the probe places, and the environment it runs in. Each property is set with the method of its name, which
/// takes whether the chip has it: `Config::of(kind).selected(true).dark(true)`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Config {
    pub(crate) kind: Kind,
    /// What the chip says.
    pub(crate) label: &'static str,
    /// The leading icon (the suggestion chip's only icon) and the trailing one (no suggestion chip has one).
    icons: Icons,
    /// An avatar (only the input chip has one).
    avatar: bool,
    condition: Condition,
    dark: bool,
    reduce_motion: bool,
}

impl Config {
    pub(crate) fn of(kind: Kind) -> Self {
        Self {
            kind,
            label: LABEL,
            icons: Icons::None,
            avatar: false,
            condition: Condition::Rest,
            dark: false,
            reduce_motion: false,
        }
    }

    pub(crate) fn label(self, label: &'static str) -> Self {
        Self { label, ..self }
    }

    pub(crate) fn leading(self, on: bool) -> Self {
        let icons = match (on, self.has_trailing()) {
            (false, false) => Icons::None,
            (true, false) => Icons::Leading,
            (false, true) => Icons::Trailing,
            (true, true) => Icons::Both,
        };
        Self { icons, ..self }
    }

    pub(crate) fn trailing(self, on: bool) -> Self {
        let icons = match (self.has_leading(), on) {
            (false, false) => Icons::None,
            (true, false) => Icons::Leading,
            (false, true) => Icons::Trailing,
            (true, true) => Icons::Both,
        };
        Self { icons, ..self }
    }

    pub(crate) fn avatar(self, on: bool) -> Self {
        Self { avatar: on, ..self }
    }

    pub(crate) fn selected(self, on: bool) -> Self {
        let condition = match (on, self.is_disabled()) {
            (false, false) => Condition::Rest,
            (true, false) => Condition::Selected,
            (false, true) => Condition::Disabled,
            (true, true) => Condition::SelectedDisabled,
        };
        Self { condition, ..self }
    }

    pub(crate) fn disabled(self, on: bool) -> Self {
        let condition = match (self.is_selected(), on) {
            (false, false) => Condition::Rest,
            (true, false) => Condition::Selected,
            (false, true) => Condition::Disabled,
            (true, true) => Condition::SelectedDisabled,
        };
        Self { condition, ..self }
    }

    pub(crate) fn dark(self, on: bool) -> Self {
        Self { dark: on, ..self }
    }

    pub(crate) fn reduce_motion(self, on: bool) -> Self {
        Self {
            reduce_motion: on,
            ..self
        }
    }

    fn has_leading(self) -> bool {
        matches!(self.icons, Icons::Leading | Icons::Both)
    }

    fn has_trailing(self) -> bool {
        matches!(self.icons, Icons::Trailing | Icons::Both)
    }

    fn is_selected(self) -> bool {
        matches!(
            self.condition,
            Condition::Selected | Condition::SelectedDisabled
        )
    }

    fn is_disabled(self) -> bool {
        matches!(
            self.condition,
            Condition::Disabled | Condition::SelectedDisabled
        )
    }
}

/// What the label says: the text the probe's presses and hovers name the chip by.
pub(crate) const LABEL: &str = "Chip";

#[derive(Debug, Clone, PartialEq, Props)]
struct ProbeProps {
    label: Prop<SharedString>,
    #[prop(default)]
    kind: Prop<Kind>,
    #[prop(default)]
    leading: Prop<bool>,
    #[prop(default)]
    trailing: Prop<bool>,
    #[prop(default)]
    avatar: Prop<bool>,
    #[prop(default)]
    selected: Prop<bool>,
    #[prop(default)]
    disabled: Prop<bool>,
}

/// One chip, a line counting its presses (or, for the filter chip, the changes it asked for), and lines that select
/// it, disable it and give it a leading icon.
#[component]
fn Probe(props: ProbeProps) -> impl IntoElement {
    let presses = signal(0_u32);
    let selected = signal(props.selected.get_untracked());
    let disabled = signal(props.disabled.get_untracked());
    let leading = signal(props.leading.get_untracked());
    let trailing = props.trailing.get_untracked();
    let avatar = props.avatar.get_untracked();
    let count = move || presses.update(|n| *n += 1);
    let chip = match props.kind.get_untracked() {
        Kind::Assist => assist_chip()
            .label(props.label)
            .leading_icon(move || leading.get().then_some(outlined::ADD))
            .trailing_icon(move || trailing.then_some(outlined::CLOSE))
            .disabled(move || disabled.get())
            .on_press(count)
            .into_element(),
        Kind::Suggestion => suggestion_chip()
            .label(props.label)
            .icon(move || leading.get().then_some(outlined::ADD))
            .disabled(move || disabled.get())
            .on_press(count)
            .into_element(),
        Kind::Filter => filter_chip()
            .label(props.label)
            .leading_icon(move || leading.get().then_some(outlined::CHECK))
            .trailing_icon(move || trailing.then_some(outlined::CLOSE))
            .selected(move || selected.get())
            .disabled(move || disabled.get())
            .on_change(move |next: &bool| {
                presses.update(|n| *n += 1);
                selected.set(*next);
            })
            .into_element(),
        Kind::Input => {
            let chip = input_chip()
                .label(props.label)
                .leading_icon(move || leading.get().then_some(outlined::CHECK))
                .trailing_icon(move || trailing.then_some(outlined::CLOSE))
                .selected(move || selected.get())
                .disabled(move || disabled.get())
                .on_press(count);
            if avatar {
                chip.avatar(view().bg(Hsla::red()))
            } else {
                chip
            }
            .into_element()
        }
    };
    // A row, so that the chip is as wide as its content, not stretched across a column.
    view()
        .child(view().flex_row().child(chip))
        .child(text(move || format!("presses: {}", presses.get())))
        .child(
            text("Select")
                .on_press(move |(): &()| selected.update(|selected| *selected = !*selected)),
        )
        .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
        .child(text("Icon").on_press(move |(): &()| leading.update(|leading| *leading = !*leading)))
}

pub(crate) fn start(config: Config) -> (Headless, Recording) {
    let root = Root::new::<Probe>(move || ProbeProps {
        label: Prop::new(SharedString::from(config.label)),
        kind: Prop::new(config.kind),
        leading: Prop::new(config.has_leading()),
        trailing: Prop::new(config.has_trailing()),
        avatar: Prop::new(config.avatar),
        selected: Prop::new(config.is_selected()),
        disabled: Prop::new(config.is_disabled()),
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

/// The chip's views, and the last style, frame and drawing each was sent.
pub(crate) struct Mounted {
    /// The press target with the button role: the whole chip.
    pub(crate) button: NodeId,
    /// The canvas that draws the container.
    pub(crate) container: NodeId,
    /// The canvas that draws the state layer.
    pub(crate) layer: NodeId,
    /// The label's text.
    pub(crate) label: NodeId,
    /// The canvas that draws the leading icon.
    pub(crate) leading: Option<NodeId>,
    /// The view the avatar fills.
    pub(crate) avatar: Option<NodeId>,
    /// The canvas that draws the trailing icon.
    pub(crate) trailing: Option<NodeId>,
    pub(crate) styles: HashMap<NodeId, ViewStyle>,
    pub(crate) frames: HashMap<NodeId, Bounds<Points>>,
    pub(crate) drawings: HashMap<NodeId, Arc<Drawing>>,
    pub(crate) text: Option<ResolvedTextStyle>,
    pub(crate) state: AccessibilityState,
    pub(crate) hit_slop: EdgeInsets,
}

/// The views of a chip's press target, told apart by their kind and their order.
struct Parts {
    container: NodeId,
    layer: NodeId,
    label: NodeId,
    leading: Option<NodeId>,
    avatar: Option<NodeId>,
    trailing: Option<NodeId>,
}

impl Parts {
    /// The container first, under the state layer; then the leading icon or avatar, the label and the trailing icon,
    /// as many as the chip has; and nothing else.
    fn of(own: &[(NodeId, ViewKind)]) -> Self {
        let [
            (container, ViewKind::Canvas),
            (layer, ViewKind::Canvas),
            rest @ ..,
        ] = own
        else {
            panic!("a container and a state layer first: {own:?}");
        };
        let label_at = rest
            .iter()
            .position(|(_, kind)| *kind == ViewKind::Text)
            .unwrap_or_else(|| panic!("a label: {own:?}"));
        let (before, after) = rest.split_at(label_at);
        assert!(before.len() <= 1 && after.len() <= 2, "{own:?}");
        let of_kind = |views: &[(NodeId, ViewKind)], wanted| {
            views
                .iter()
                .find(|(_, kind)| *kind == wanted)
                .map(|(id, _)| *id)
        };
        Self {
            container: *container,
            layer: *layer,
            label: after[0].0,
            leading: of_kind(before, ViewKind::Canvas),
            avatar: of_kind(before, ViewKind::View),
            trailing: after[1..].first().map(|(id, _)| *id),
        }
    }
}

impl Mounted {
    pub(crate) fn of(items: &[MountItem]) -> Self {
        let mut kinds = HashMap::new();
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut button = None;
        let mut state = AccessibilityState::default();
        let mut hit_slop = EdgeInsets::default();
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
                    // An insert of a view that is already in a parent moves it.
                    for siblings in children.values_mut() {
                        siblings.retain(|id| id != child);
                    }
                    let siblings: &mut Vec<NodeId> = children.entry(*parent).or_default();
                    let at = before
                        .and_then(|before| siblings.iter().position(|id| *id == before))
                        .unwrap_or(siblings.len());
                    siblings.insert(at, *child);
                }
                MountItem::Remove { id } | MountItem::Delete { id } => {
                    for siblings in children.values_mut() {
                        siblings.retain(|child| child != id);
                    }
                }
                MountItem::ViewProps { id, props }
                    if props.accessibility.role == AccessibilityRole::Button =>
                {
                    button = Some(*id);
                    state = props.accessibility.state;
                    hit_slop = props.hit_slop;
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
        let own: Vec<(NodeId, ViewKind)> = children
            .get(&button)
            .into_iter()
            .flatten()
            .map(|child| (*child, kinds[child]))
            .collect();
        let parts = Parts::of(&own);
        Self {
            button,
            container: parts.container,
            layer: parts.layer,
            label: parts.label,
            leading: parts.leading,
            avatar: parts.avatar,
            trailing: parts.trailing,
            text: texts.get(&parts.label).copied().flatten(),
            styles,
            frames,
            drawings,
            state,
            hit_slop,
        }
    }

    /// The last style `id` was sent; the default one if it was sent none.
    pub(crate) fn style(&self, id: NodeId) -> ViewStyle {
        self.styles.get(&id).cloned().unwrap_or_default()
    }

    pub(crate) fn drawing(&self, id: NodeId) -> &Drawing {
        &self.drawings[&id]
    }

    /// The chip's size, as its layout gave it.
    pub(crate) fn size(&self) -> Size<Points> {
        self.frames[&self.button].size
    }

    /// How many of `items` lay the chip out again: one of its views' frames, or its label's text.
    pub(crate) fn layout_items(&self, items: &[MountItem]) -> usize {
        let mut ours = vec![self.button, self.container, self.layer, self.label];
        ours.extend(self.leading);
        ours.extend(self.avatar);
        ours.extend(self.trailing);
        items
            .iter()
            .filter(|item| match item {
                MountItem::Frame { id, .. } | MountItem::Text { id, .. } => ours.contains(id),
                _ => false,
            })
            .count()
    }
}

pub(crate) fn mount(config: Config) -> Mounted {
    let (_app, recording) = start(config);
    Mounted::of(&recording.items())
}

/// NSH's color of `hex` (`0xRRGGBB`) at alpha `alpha` (0 to 255).
pub(crate) fn color(hex: u32, alpha: u32) -> Color {
    Color((hex << 8) | alpha)
}

pub(crate) fn scheme_color(role: ColorRole, dark: bool) -> u32 {
    let scheme = if dark {
        ColorScheme::BASELINE_DARK
    } else {
        ColorScheme::BASELINE_LIGHT
    };
    scheme.get(role).to_hex()
}

/// A role in the baseline scheme, opaque.
pub(crate) fn solid(role: ColorRole, dark: bool) -> Color {
    color(scheme_color(role, dark), 255)
}

/// A role in the baseline scheme at 8-bit alpha `alpha`.
pub(crate) fn faded(role: ColorRole, alpha: u32, dark: bool) -> Color {
    color(scheme_color(role, dark), alpha)
}

/// The 8-bit alphas NSH rounds the token opacities to: 0.38 of 255 and 0.12 of 255.
pub(crate) const ALPHA_38: u32 = 97;
pub(crate) const ALPHA_12: u32 = 31;

/// The color `ops` fill with, or that they stroke with, if any.
pub(crate) fn paint(ops: &[CanvasOp], stroke: bool) -> Option<Color> {
    ops.iter().find_map(|op| match op {
        CanvasOp::FillStyle(canvas::Paint::Color(color)) if !stroke => Some(*color),
        CanvasOp::StrokeStyle(canvas::Paint::Color(color)) if stroke => Some(*color),
        _ => None,
    })
}

/// What a chip paints: the container's color, the outline's width and color (0 and transparent with no outline), the
/// state layer's color, the label's color, and the colors of the leading and trailing icons (transparent for an icon
/// the chip does not have).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Painted {
    pub(crate) container: Color,
    pub(crate) outline_width: f32,
    pub(crate) outline: Color,
    pub(crate) layer: Color,
    pub(crate) label: Color,
    pub(crate) leading: Color,
    pub(crate) trailing: Color,
}

impl Painted {
    /// The same chip without the icons: they paint nothing.
    pub(crate) fn without_icons(self) -> Self {
        Self {
            leading: Color(0),
            trailing: Color(0),
            ..self
        }
    }
}

pub(crate) fn painted(mounted: &Mounted) -> Painted {
    let ops = &mounted.drawing(mounted.container).ops;
    let (outline_width, outline) = if ops.iter().any(|op| matches!(op, CanvasOp::Stroke(None))) {
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
    let icon = |id: Option<NodeId>| {
        id.map_or(Color(0), |id| {
            paint(&mounted.drawing(id).ops, false).expect("the icon's fill")
        })
    };
    Painted {
        container: paint(ops, false).unwrap_or(Color(0)),
        outline_width,
        outline,
        layer: paint(&mounted.drawing(mounted.layer).ops, false).expect("the state layer's fill"),
        label: mounted.text.as_ref().expect("the label's style").color,
        leading: icon(mounted.leading),
        trailing: icon(mounted.trailing),
    }
}

/// Whether `file` holds `lines`, each trimmed, one after the other.
pub(crate) fn cites(file: &str, lines: &[&str]) -> bool {
    let file: Vec<&str> = file.lines().map(str::trim).collect();
    file.windows(lines.len()).any(|window| window == lines)
}

/// The vendored `Chip.kt`, at the pinned commit (ADR 0012).
pub(crate) const CHIP_KT: &str =
    include_str!("../../../reference/androidx-material3-components/Chip.kt");

/// The step of a press that goes down on the chip and stays down: a finger that lands and does not move.
pub(crate) fn press_down(app: &mut Headless) -> String {
    let still = point(px(0.0), px(0.0));
    app.drag_and_hold(LABEL, still).unwrap();
    format!("dragAndHold({LABEL}, {}, {})", still.x, still.y)
}

/// The paths the container's canvas and the state layer's draw now, and the chip's size.
pub(crate) fn drawn(recording: &Recording) -> (Vec<Vec<Step>>, Vec<Vec<Step>>, Size<Points>) {
    let mounted = Mounted::of(&recording.items());
    let replay = replay(recording);
    (
        paths(mounted.drawing(mounted.container), &replay),
        paths(mounted.drawing(mounted.layer), &replay),
        mounted.size(),
    )
}

/// The items sent by one more press of `selector`: a step name is the same each time, so the items of the step are
/// what the recording holds after the press and did not before it.
pub(crate) fn press_items(
    app: &mut Headless,
    recording: &Recording,
    selector: &str,
) -> Vec<MountItem> {
    let before = recording.items().len();
    app.press(selector).unwrap();
    recording.items().split_off(before)
}

/// The items sent by a finger that lands on the chip and stays down.
pub(crate) fn down_items(app: &mut Headless, recording: &Recording) -> Vec<MountItem> {
    let before = recording.items().len();
    press_down(app);
    recording.items().split_off(before)
}

/// The items sent by dropping the finger a [`press_down`] left down.
pub(crate) fn drop_items(app: &mut Headless, recording: &Recording) -> Vec<MountItem> {
    let before = recording.items().len();
    app.drop_held().unwrap();
    recording.items().split_off(before)
}

/// What the chips' corners do in `items`: one spring to `to` with the chips' `FastSpatial`, from where the corners are
/// and as fast as they move, and no view laid out again (the label's text may be sent again in another color: its
/// layout does not change).
pub(crate) fn morph_in(recording: &Recording, items: &[MountItem], to: f32) -> StartedSpring {
    let spring = spring_in_with(items, "the morph", to, SpringToken::FastSpatial);
    let mounted = Mounted::of(&recording.items());
    let frames = items
        .iter()
        .filter(
            |item| matches!(item, MountItem::Frame { id, .. } if mounted.frames.contains_key(id)),
        )
        .count();
    assert_eq!(frames, 0, "{items:?}");
    spring
}
