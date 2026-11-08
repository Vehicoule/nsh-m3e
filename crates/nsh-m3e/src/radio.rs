//! The Material 3 Expressive radio button: a ring and a dot drawn in one color, the state layer of hover, focus and
//! press around them, and the dot that grows when it is selected.
//!
//! The radio button follows androidx's `RadioButton` in `RadioButton.kt` at the pinned commit, as [ADR 0012] says: the
//! tokens give the colors, the ring's size and the state layer's size (`RadioButtonTokens`); the rest is written in the
//! file, and each value below that is not a token cites its line there.
//!
//! # What it draws
//!
//! - **The ring.** A circle of [`radio_button_tokens::ICON_SIZE`] across (`RadioButtonTokens.IconSize`, 20 points),
//!   stroked [`STROKE_WIDTH`] wide inside its edge: androidx strokes a circle of radius `IconSize / 2 - strokeWidth / 2`
//!   (`RadioButton.kt`, line 183 to 187).
//! - **The dot.** A filled circle that grows from nothing when the radio is selected. Its radius is animated from 0 to
//!   [`DOT_SIZE`]` / 2` (6 points, `RadioButton.kt`, line 91) and drawn [`STROKE_WIDTH`]` / 2` smaller, 5 points, only
//!   while it is more than 0 (line 188 and 189).
//! - **The state layer.** A circle of [`radio_button_tokens::STATE_LAYER_SIZE`] across (40 points), centered on the
//!   radio and wider than it, which hover, focus and press show at the opacities of `StateTokens`, as androidx's
//!   unbounded ripple does (`RadioButton.kt`, line 165 to 169).
//! - **The size.** The ring and 2 points of padding on each side, [`FRAME`] across (24 points): androidx pads the
//!   canvas by `RadioButtonPadding` and then requires the icon's size (line 177 to 179, 318 and 319).
//! - **The touch target.** androidx's `minimumInteractiveComponentSize()` (line 156) makes the radio button take 48
//!   points in the layout. Here presses count within 48 points of the center through the hit slop, 12 points on each
//!   side, and the layout stays [`FRAME`] across: the rule of this library for every control (`docs/app-requirements.md`,
//!   Accessibility), as the icon button's.
//!
//! # Colors
//!
//! The ring and the dot take one color, from `RadioButtonColors.radioColor` (line 289 to 295) and the defaults of
//! `RadioButtonDefaults.defaultRadioButtonColors` (line 229 to 243): the primary color when selected, the on surface
//! variant color when not, and on surface at 0.38 when disabled, selected or not (the tokens
//! `RadioButtonTokens.DisabledSelectedIconOpacity` and `DisabledUnselectedIconOpacity`).
//!
//! The state layer's color is the token's, selected or not: primary, or on surface. androidx draws the ripple with no
//! color of its own (`ripple(bounded = false, radius = …, focusRingShape = CircleShape)`, line 165 to 169), which makes
//! it take `LocalContentColor`, a color of the component around the radio button that NSH does not have (`Ripple.kt`,
//! not vendored). The tokens name the layer's colors for hover, focus and press (`SelectedHoverIconColor`,
//! `UnselectedHoverIconColor`, …), and every one of them is the same for the three states, so one layer holds them
//! all (`the_state_layer_has_one_color_for_hover_focus_and_press` fails if the tokens tell them apart).
//!
//! # Motion
//!
//! - **The dot's radius** springs with `MotionSchemeKeyTokens.FastSpatial` (`RadioButton.kt`, line 137 to 142), through
//!   [`SpringToken::FastSpatial`] and the conversion of ADR 0004, from where it is and as fast as it moves. That spring
//!   overshoots (its damping ratio is 0.6): the dot swells a little past its size before it settles, as androidx's
//!   does. It runs whether the radio is enabled or not.
//! - **The color** springs with `MotionSchemeKeyTokens.DefaultEffects` when the radio is enabled (line 144 to 147), and
//!   snaps when it is disabled: "there should be no animations between enabled / disabled" (line 149 to 152). A radio
//!   that becomes enabled starts at the color it has, not from the one before, as androidx's `animateColorAsState` does
//!   when it enters the composition. NSH's animation graph mixes two colors channel by channel, 8 bits each, in a
//!   straight line; androidx's mix is `animateColorAsState`'s, which this file does not say how it mixes (not found in
//!   the vendored files).
//! - **Reduce motion.** With it on, neither the dot nor the color moves: both are set at once to where they go, and no
//!   spring is sent ([ADR 0010], check 6). That check also asks for a short opacity fade "with the duration its spec
//!   gives", and neither `RadioButton.kt` nor `RadioButtonTokens.kt` gives a duration: "not found", so none is made up
//!   and nothing replaces the motion, as the corners' morph leaves the shape as it is ([ADR 0011], "Reduce motion").
//! - **Nothing runs at rest or at mount.** The dot and the color start where they rest.
//!
//! The dot's radius and the color are values of NSH's animation graph, which the platform plays itself; the radio's two
//! canvases draw with them and are not drawn again for each frame by the engine ([ADR 0010]). The dot's radius is a
//! drawing argument, not a transform or an opacity: the one exception to "transform and opacity" that [ADR 0011] makes
//! for a shape that changes.
//!
//! # What it does not do
//!
//! - **It has no label.** androidx's `RadioButton` shows none; apps lay out a radio button and a `Text` in a row
//!   (`RadioButton.kt`, line 58 to 61), and make the row selectable with `onClick = null` on the radio button. NSH's
//!   `Radio` takes presses on its own children, so a label beside this radio button is not part of what presses it:
//!   the row that selects when its label is pressed comes with the lists that use it.
//! - **It has no group of its own.** androidx has no `RadioGroup` (line 58 to 61), and `nsh_base::radio_group()` has no
//!   look: a set of radio buttons is a `nsh_base::radio_group()` around them, which gives the keyboard behavior (the
//!   arrow keys move between them and select) and the group role for screen readers. A styled wrapper would add a
//!   name and nothing else.
//! - **The press does not ripple.** The state layer takes its opacity at once under hover, focus and press, and nothing
//!   runs; androidx's ripple spreads from the center and fades. The focus ring androidx can draw for
//!   `focusRingShape` (line 165 to 169) is not drawn: focus shows as the state layer, as the icon button's does.
//! - **`onClick = null`.** androidx lets a radio button leave its input to something else (line 63 to 64); this one
//!   always takes presses.
//! - **`StyleableRadioButton`** (line 99 to 125) is `internal` in androidx, with a "TODO … add screenshot tests", and is
//!   not followed.
//!
//! # What NSH could not do
//!
//! - **A color for the ripple.** NSH has no `LocalContentColor`, the color androidx's ripple takes here (see "Colors").
//! - **A fade for reduce motion.** The requirements allow a fade, and no duration for one was found (see "Motion").
//! - **Press-down and press-up events.** NSH's `Radio` has none (`nsh-base`, `radio.rs`). This radio does not need
//!   them: the pressed state layer is `group_active`, and nothing else moves with a press.
//! - **A pressed state in the headless driver.** `nsh_testing::Headless` reports hover and focus, and leaves the
//!   pressed state to a platform (`EngineControl::set_states`): the tests report it as a platform does.
//!
//! [ADR 0010]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0010-measure-animations-in-ci.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [`radio_button_tokens::ICON_SIZE`]: crate::tokens::androidx::radio_button_tokens::ICON_SIZE
//! [`radio_button_tokens::STATE_LAYER_SIZE`]: crate::tokens::androidx::radio_button_tokens::STATE_LAYER_SIZE

use std::cell::Cell;
use std::f32::consts::TAU;

use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::motion::{SPRING_MASS, SpringToken};
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::androidx::{radio_button_tokens, state_tokens};

/// The ring's stroke: androidx `RadioStrokeWidth`, 2 dp (`RadioButton.kt`, line 322 and 323). Not in the tokens.
pub const STROKE_WIDTH: Points = Points(2.0);

/// The dot's diameter when selected: androidx `RadioButtonDotSize`, 12 dp (`RadioButton.kt`, line 320 and 321). Not in
/// the tokens.
pub const DOT_SIZE: Points = Points(12.0);

/// The padding on each side of the ring: androidx `RadioButtonPadding`, 2 dp (`RadioButton.kt`, line 318 and 319). Not
/// in the tokens.
pub const PADDING: Points = Points(2.0);

/// The side of the radio button's box: the ring and the padding on each side of it (`RadioButton.kt`, line 177 to
/// 179). The state layer overflows it, and presses reach past it ([`Radio`]).
pub const FRAME: Points = frame_of(radio_button_tokens::ICON_SIZE, PADDING);

/// The side of a box that holds a ring `ring` across with `padding` on each side of it. A function of its own because
/// with the real padding of 2 points its sum and its product are both 4, so no test of [`FRAME`] can tell a sum from
/// a product; the test of this function gives it a padding where they differ.
const fn frame_of(ring: Points, padding: Points) -> Points {
    Points(ring.0 + padding.0 + padding.0)
}

/// The four looks of a radio button: selected or not, enabled or not, in the order of its color progress. androidx
/// picks the ring's and the dot's color from the same four (`RadioButtonColors.radioColor`, line 289 to 295).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Unselected,
    Selected,
    DisabledUnselected,
    DisabledSelected,
}

impl Look {
    /// Every look, in the order of their [`Look::position`].
    const ALL: [Self; 4] = [
        Self::Unselected,
        Self::Selected,
        Self::DisabledUnselected,
        Self::DisabledSelected,
    ];

    fn of(selected: bool, disabled: bool) -> Self {
        match (selected, disabled) {
            (false, false) => Self::Unselected,
            (true, false) => Self::Selected,
            (false, true) => Self::DisabledUnselected,
            (true, true) => Self::DisabledSelected,
        }
    }

    /// Where the color progress rests for this look: its index in [`Look::ALL`]. The progress springs between the
    /// first two, and is set at once to the others, which are never between two enabled looks.
    fn position(self) -> f32 {
        match self {
            Self::Unselected => 0.0,
            Self::Selected => 1.0,
            Self::DisabledUnselected => 2.0,
            Self::DisabledSelected => 3.0,
        }
    }

    fn is_disabled(self) -> bool {
        matches!(self, Self::DisabledUnselected | Self::DisabledSelected)
    }

    fn is_selected(self) -> bool {
        matches!(self, Self::Selected | Self::DisabledSelected)
    }

    /// The ring's and the dot's color in this look: androidx `RadioButtonDefaults.defaultRadioButtonColors`.
    fn tint(self) -> Tint {
        match self {
            Self::Unselected => Tint::solid(radio_button_tokens::UNSELECTED_ICON_COLOR),
            Self::Selected => Tint::solid(radio_button_tokens::SELECTED_ICON_COLOR),
            Self::DisabledUnselected => Tint::faded(
                radio_button_tokens::DISABLED_UNSELECTED_ICON_COLOR,
                radio_button_tokens::DISABLED_UNSELECTED_ICON_OPACITY,
            ),
            Self::DisabledSelected => Tint::faded(
                radio_button_tokens::DISABLED_SELECTED_ICON_COLOR,
                radio_button_tokens::DISABLED_SELECTED_ICON_OPACITY,
            ),
        }
    }
}

/// The dot's radius when selected: half of [`DOT_SIZE`] (`dotRadius = if (selected) RadioButtonDotSize / 2 else 0.dp`,
/// `RadioButton.kt`, line 91).
fn selected_dot_radius() -> f32 {
    DOT_SIZE.0 / 2.0
}

/// The dot's radius for `selected`: 6 points when selected, else 0.
fn dot_radius(selected: bool) -> f32 {
    if selected { selected_dot_radius() } else { 0.0 }
}

/// The spring of `token`, every field written out, none taken from NSH's defaults: from the speed the value already
/// has, at the screen's own rate (ADR 0010), converted as ADR 0004 says.
fn spring(token: SpringToken) -> Spring {
    let spec = token.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// The spring of the dot's radius: androidx `MotionSchemeKeyTokens.FastSpatial` (`RadioButton.kt`, line 141).
const DOT_SPRING: SpringToken = SpringToken::FastSpatial;

/// The spring of the color: androidx `MotionSchemeKeyTokens.DefaultEffects` (`RadioButton.kt`, line 147).
const COLOR_SPRING: SpringToken = SpringToken::DefaultEffects;

/// How far presses also count around a radio button, so that it has a 48-point target (androidx
/// `minimumInteractiveComponentSize`, `RadioButton.kt`, line 156) without changing its layout (NSH's accessibility
/// guide: extend the touch area with the hit slop, not the size).
fn touch_slop() -> EdgeInsets {
    EdgeInsets::all(px(((MIN_TOUCH_TARGET.0 - FRAME.0) / 2.0).max(0.0)))
}

/// Where the state layer's corner is, relative to the radio button's: the layer is centered on it and wider.
fn layer_offset() -> Points {
    Points((FRAME.0 - radio_button_tokens::STATE_LAYER_SIZE.0) / 2.0)
}

/// The props of a [`Radio`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct RadioProps {
    /// Selected or not. The app owns the value: a press on an unselected radio button reports `true` with
    /// [`RadioEvent::Change`], and the app selects it (and unselects the others of its set).
    #[prop(default)]
    pub checked: Prop<bool>,
    /// What screen readers say: a radio button shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// No presses: the disabled colors, no state layer, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`Radio`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum RadioEvent {
    /// An unselected radio button was pressed, or an arrow key of its `nsh_base::radio_group()` moved the keyboard
    /// focus to it: it asks to be selected, so the value is `true`. It runs before anything changes: the radio button
    /// shows the `checked` the app passes back. A selected radio button stays selected and reports nothing, as a radio
    /// button cannot unselect itself.
    #[args(checked)]
    Change(bool),
}

/// A Material 3 Expressive radio button: a ring, and a dot in it while selected.
///
/// It has the behavior of `nsh-base`'s `Radio` (presses, focus, the keyboard of its `radio_group()`, the disabled
/// state, the radio role and its checked state, named by its label for screen readers) and the look of androidx's
/// `RadioButton`. Hover, focus and press lay a 40-point circle of the radio button's color over the space around it at
/// the opacities of `StateTokens`, and change nothing else, so nothing is laid out again.
///
/// A radio button is 24 points across, and presses count within 48 points of its center.
///
/// ```no_run
/// use nsh_base::radio_group;
/// use nsh_core::prelude::*;
/// use nsh_m3e::radio::radio;
///
/// fn plans() -> impl IntoElement {
///     let plan = signal("basic");
///     let option = move |value: &'static str, label: &'static str| {
///         radio()
///             .label(label)
///             .checked(move || plan.get() == value)
///             .on_change(move |_: &bool| plan.set(value))
///     };
///     // The group gives the arrow keys; the radio buttons give the look.
///     radio_group()
///         .gap_2()
///         .child(option("basic", "Basic"))
///         .child(option("pro", "Pro"))
/// }
/// ```
#[component]
pub fn Radio(props: RadioProps, cx: Scope<RadioEvent>) -> impl IntoElement {
    let initial = untracked(|| Look::of(props.checked.get(), props.disabled.get()));
    // The dot's radius, in points: 0 unselected, `selected_dot_radius()` selected.
    let dot = animated_value(dot_radius(initial.is_selected()));
    // Where the color is among the looks: it springs between the enabled ones and is set at once to the others.
    let progress = animated_value(initial.position());

    // The dot grows or shrinks when `checked` changes, enabled or not (androidx animates `dotRadius` whatever
    // `enabled` is, `RadioButton.kt`, line 137 to 142).
    let selected = Cell::new(untracked(|| props.checked.get()));
    effect(move || {
        let now = props.checked.get();
        if selected.replace(now) == now {
            return;
        }
        let to = dot_radius(now);
        if untracked(reduce_motion) {
            dot.set(to);
        } else {
            let _ = dot.spring(to, spring(DOT_SPRING));
        }
    });

    // The color springs between the enabled looks, and is set at once into, out of and between the disabled ones
    // (`RadioButton.kt`, line 144 to 152).
    let look = Cell::new(initial);
    effect(move || {
        let now = Look::of(props.checked.get(), props.disabled.get());
        let before = look.replace(now);
        if before == now {
            return;
        }
        if now.is_disabled() || before.is_disabled() || untracked(reduce_motion) {
            progress.set(now.position());
        } else {
            let _ = progress.spring(now.position(), spring(COLOR_SPRING));
        }
    });

    // The ring and the dot, in the color of the look the progress is at.
    let indicator = canvas(move || {
        let colors = progress.interpolate_colors(
            &[0.0, 1.0, 2.0, 3.0],
            &Look::ALL.map(|look| tint_color(Some(look.tint()))),
        );
        // The dot is drawn `STROKE_WIDTH / 2` smaller than its radius, and not at all while that is nothing
        // (`RadioButton.kt`, line 188 and 189): from the radius one half stroke wide, a point more makes a point
        // more, and the spring's overshoot goes on.
        let half = STROKE_WIDTH.0 / 2.0;
        let radius = dot.interpolate_with(
            &[half, half + 1.0],
            AnimationOutput::Numbers(vec![0.0, 1.0]),
            Interpolation {
                extrapolate_right: Extrapolate::Extend,
                ..Interpolation::default()
            },
        );
        let middle = radio_button_tokens::ICON_SIZE.0 / 2.0;
        Painter::new()
            .begin_path()
            .arc(middle, middle, middle - half, 0.0, TAU)
            .line_width(STROKE_WIDTH.0)
            .stroke_style(&colors)
            .stroke()
            .begin_path()
            .arc(middle, middle, &radius, 0.0, TAU)
            .fill_style(&colors)
            .fill()
    })
    .w(radio_button_tokens::ICON_SIZE)
    .h(radio_button_tokens::ICON_SIZE);

    // The state layer: a circle in the color of the looks' state layers, shown by the radio button's own states with
    // nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`).
    let layer = canvas(move || {
        let color = progress.interpolate_colors(
            &[0.0, 1.0],
            &[
                tint_color(Some(Tint::solid(
                    radio_button_tokens::UNSELECTED_HOVER_ICON_COLOR,
                ))),
                tint_color(Some(Tint::solid(
                    radio_button_tokens::SELECTED_HOVER_ICON_COLOR,
                ))),
            ],
        );
        let radius = radio_button_tokens::STATE_LAYER_SIZE.0 / 2.0;
        Painter::new()
            .begin_path()
            .arc(radius, radius, radius, 0.0, TAU)
            .fill_style(&color)
            .fill()
    })
    .absolute()
    .left(layer_offset())
    .top(layer_offset())
    .w(radio_button_tokens::STATE_LAYER_SIZE)
    .h(radio_button_tokens::STATE_LAYER_SIZE)
    .opacity(0.0)
    .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
    .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
    .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
    .style_with(move |s| {
        // A disabled radio button takes no press and shows no state.
        if props.disabled.get() { s.hidden() } else { s }
    });

    nsh_base::radio()
        .checked(move || props.checked.get())
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_change(move |checked: &bool| cx.emit(RadioEvent::Change(*checked)))
        .hit_slop(touch_slop())
        .group()
        .items_center()
        .justify_center()
        .w(FRAME)
        .h(FRAME)
        .child(layer)
        .child(indicator)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_base::radio_group;
    use nsh_core::animation::AnimationOp;
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::tailwind::state;
    use nsh_core::{AccessibilityRole, Checked, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::morph_probe::{Step, parameters, paths, replay, set_values, tolerance};

    /// What the probe places: one radio button with these props, a line counting the changes it asks for, and lines
    /// that change what the app passes it.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        checked: Prop<bool>,
        #[prop(default)]
        disabled: Prop<bool>,
        /// An app that does not pass the change back as `checked`.
        #[prop(default)]
        ignore_changes: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let checked = signal(props.checked.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        let ignore = props.ignore_changes.get_untracked();
        let changes = signal(0_u32);
        view()
            .child(
                // A row, so that the radio button keeps its own size, not stretched across a column.
                view().flex_row().child(
                    radio()
                        .label("Plan")
                        .checked(move || checked.get())
                        .disabled(move || disabled.get())
                        .on_change(move |value: &bool| {
                            changes.update(|n| *n += 1);
                            if !ignore {
                                checked.set(*value);
                            }
                        }),
                ),
            )
            .child(text(move || format!("changes: {}", changes.get())))
            .child(text("Select").on_press(move |(): &()| checked.set(true)))
            .child(text("Clear").on_press(move |(): &()| checked.set(false)))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Enable").on_press(move |(): &()| disabled.set(false)))
    }

    /// Four options in a `radio_group()`, the third disabled, and the one selected.
    #[component]
    fn GroupProbe() -> impl IntoElement {
        let plan = signal("basic");
        let option = move |value: &'static str, label: &'static str, disabled: bool| {
            radio()
                .label(label)
                .disabled(disabled)
                .checked(move || plan.get() == value)
                .on_change(move |_: &bool| plan.set(value))
        };
        view()
            .child(
                radio_group()
                    .child(option("basic", "Basic", false))
                    .child(option("pro", "Pro", false))
                    .child(option("max", "Max", true))
                    .child(option("team", "Team", false)),
            )
            .child(text(move || format!("plan: {}", plan.get())))
    }

    /// What the system asks of motion.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Motion {
        #[default]
        Full,
        Reduced,
    }

    /// What the app does with a change the radio button asks for.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Reply {
        /// It passes the change back as `checked`.
        #[default]
        Passes,
        /// It keeps `checked` as it is.
        Ignores,
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        checked: bool,
        disabled: bool,
        dark: bool,
        reply: Reply,
        motion: Motion,
    }

    /// Runs `root` on the headless engine with a manual clock, as the web (which has a mouse to hover with and a
    /// keyboard that tabs to views), until `ready` shows.
    fn launch(root: Root, dark: bool, reduce_motion: bool, ready: &str) -> (Headless, Recording) {
        let environment = Environment {
            os: Os::Web,
            color_scheme: if dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            reduce_motion,
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for(ready).unwrap();
        (app, recording)
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            checked: Prop::new(config.checked),
            disabled: Prop::new(config.disabled),
            ignore_changes: Prop::new(config.reply == Reply::Ignores),
        });
        launch(
            root,
            config.dark,
            config.motion == Motion::Reduced,
            "text:changes: 0",
        )
    }

    /// The radio button's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the radio role: the whole radio button.
        radio: NodeId,
        /// The canvas that draws the state layer.
        layer: NodeId,
        /// The canvas that draws the ring and the dot.
        indicator: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        label: Option<Arc<str>>,
        disabled: Option<bool>,
        checked: Option<Checked>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut radio = None;
            let mut label = None;
            let mut disabled = None;
            let mut checked = None;
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
                        if props.accessibility.role == AccessibilityRole::Radio =>
                    {
                        radio = Some(*id);
                        label = props.accessibility.label.clone();
                        disabled = Some(props.accessibility.state.disabled);
                        checked = props.accessibility.state.checked;
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
            let radio = radio.expect("a view with the radio role");
            let canvases: Vec<NodeId> = children
                .get(&radio)
                .into_iter()
                .flatten()
                .filter(|child| kinds.get(child) == Some(&ViewKind::Canvas))
                .copied()
                .collect();
            // The state layer first, under the ring and the dot; and nothing else in the radio button.
            let [layer, indicator] = canvases[..] else {
                panic!("two canvases in the radio button: {items:?}");
            };
            assert_eq!(children.get(&radio).map(Vec::len), Some(2), "{items:?}");
            Self {
                radio,
                layer,
                indicator,
                styles,
                frames,
                drawings,
                label,
                disabled,
                checked,
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

        /// The radio button's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.radio].size
        }

        /// How many of `items` lay the radio button out again: one of its views' frames.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.radio, self.layer, self.indicator];
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

    /// The 8-bit alpha NSH rounds the token opacity 0.38 to: 0.38 of 255.
    const ALPHA_38: u32 = 97;

    /// Selected or not, disabled or not: the four looks.
    const LOOKS: [(bool, bool); 4] = [(false, false), (true, false), (false, true), (true, true)];

    /// The color the ring and the dot should have, from `RadioButtonTokens` in the baseline schemes: primary selected,
    /// on surface variant not, and on surface at 38 % disabled, selected or not.
    fn expected(selected: bool, disabled: bool, dark: bool) -> Color {
        match (selected, disabled) {
            (true, false) => color(scheme_color(ColorRole::Primary, dark), 255),
            (false, false) => color(scheme_color(ColorRole::OnSurfaceVariant, dark), 255),
            (_, true) => color(scheme_color(ColorRole::OnSurface, dark), ALPHA_38),
        }
    }

    /// The color a paint of `drawing` has in `replay`, as it is now: its constant, or the color its slot's node
    /// computes.
    fn paint_color(drawing: &Drawing, replay: &Replay, stroke: bool) -> Color {
        let paint = drawing
            .ops
            .iter()
            .find_map(|op| match op {
                CanvasOp::FillStyle(paint) if !stroke => Some(paint),
                CanvasOp::StrokeStyle(paint) if stroke => Some(paint),
                _ => None,
            })
            .expect("a paint");
        match paint {
            canvas::Paint::Color(color) => *color,
            canvas::Paint::Slot(slot) => Color(
                replay
                    .color(drawing.slots[usize::from(*slot)])
                    .expect("a node that computes a color"),
            ),
            canvas::Paint::Gradient(_) => panic!("a gradient"),
        }
    }

    /// What the indicator draws in a replay, as it is now.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Drawn {
        /// The ring's radius, and its line's width.
        ring: f32,
        line_width: f32,
        /// The dot's radius.
        dot: f32,
        /// The colors of the ring's stroke and the dot's fill.
        stroke: Color,
        fill: Color,
    }

    fn drawn(mounted: &Mounted, replay: &Replay) -> Drawn {
        let drawing = mounted.drawing(mounted.indicator);
        let arcs = paths(drawing, replay);
        let radius = |path: &[Step]| match path {
            [Step::Arc { x, y, radius, .. }] => {
                // Both are centered in the 20-point canvas.
                assert_eq!((*x, *y), (10.0, 10.0), "{path:?}");
                *radius
            }
            other => panic!("one arc, not {other:?}"),
        };
        let [ring, dot] = &arcs[..] else {
            panic!("a ring and a dot: {arcs:?}");
        };
        let line_width = drawing
            .ops
            .iter()
            .find_map(|op| match op {
                CanvasOp::LineWidth(Num::Const(width)) => Some(*width),
                _ => None,
            })
            .expect("a line width");
        Drawn {
            ring: radius(ring),
            line_width,
            dot: radius(dot),
            stroke: paint_color(drawing, replay, true),
            fill: paint_color(drawing, replay, false),
        }
    }

    /// The color the state layer fills with in a replay, as it is now.
    fn layer_color(mounted: &Mounted, replay: &Replay) -> Color {
        paint_color(mounted.drawing(mounted.layer), replay, false)
    }

    #[test]
    fn each_look_has_its_ring_dot_and_color_light_and_dark() {
        for dark in [false, true] {
            for (checked, disabled) in LOOKS {
                let what = format!("selected {checked}, disabled {disabled}, dark {dark}");
                let (_app, recording) = start(Config {
                    checked,
                    disabled,
                    dark,
                    ..Config::default()
                });
                let mounted = Mounted::of(&recording.items());
                let drawn = drawn(&mounted, &replay(&recording));
                let color = expected(checked, disabled, dark);
                // The ring and the dot have one color.
                assert_eq!(drawn.stroke, color, "{what}");
                assert_eq!(drawn.fill, color, "{what}");
                // The ring is 2 points wide inside the 20-point icon: a circle of radius 10 - 2 / 2.
                assert_eq!(drawn.line_width, 2.0, "{what}");
                assert_eq!(drawn.ring, 9.0, "{what}");
                // The dot is 12 points across when selected, drawn one stroke half smaller; nothing when not.
                assert_eq!(drawn.dot, if checked { 5.0 } else { 0.0 }, "{what}");
            }
        }
        // The colors are the scheme's: the same look differs between light and dark.
        for (checked, disabled) in LOOKS {
            assert_ne!(
                expected(checked, disabled, false),
                expected(checked, disabled, true),
                "selected {checked}, disabled {disabled}"
            );
        }
    }

    #[test]
    fn the_colors_follow_the_appearance_while_the_radio_button_is_mounted() {
        let (mut app, recording) = start(Config {
            checked: true,
            ..Config::default()
        });
        let at = |recording: &Recording| {
            let graph = replay(recording);
            let mounted = Mounted::of(&recording.items());
            (drawn(&mounted, &graph), layer_color(&mounted, &graph))
        };
        let (light, light_layer) = at(&recording);
        assert_eq!(light.stroke, expected(true, false, false));
        assert_eq!(
            light_layer,
            color(scheme_color(ColorRole::Primary, false), 255)
        );
        app.set_environment("dark", |environment| {
            environment.color_scheme = Appearance::Dark;
        })
        .unwrap();
        let (dark, dark_layer) = at(&recording);
        assert_eq!(dark.stroke, expected(true, false, true));
        assert_eq!(dark.fill, expected(true, false, true));
        assert_eq!(
            dark_layer,
            color(scheme_color(ColorRole::Primary, true), 255)
        );
        // And back.
        app.set_environment("light", |environment| {
            environment.color_scheme = Appearance::Light;
        })
        .unwrap();
        assert_eq!(at(&recording).0.stroke, expected(true, false, false));
    }

    /// The roles the state layer takes for hover, focus and press, selected or not.
    fn state_layer_roles() -> [(ColorRole, ColorRole, ColorRole); 2] {
        [
            (
                radio_button_tokens::UNSELECTED_HOVER_ICON_COLOR,
                radio_button_tokens::UNSELECTED_FOCUS_ICON_COLOR,
                radio_button_tokens::UNSELECTED_PRESSED_ICON_COLOR,
            ),
            (
                radio_button_tokens::SELECTED_HOVER_ICON_COLOR,
                radio_button_tokens::SELECTED_FOCUS_ICON_COLOR,
                radio_button_tokens::SELECTED_PRESSED_ICON_COLOR,
            ),
        ]
    }

    #[test]
    fn the_state_layer_has_one_color_for_hover_focus_and_press() {
        // androidx's ripple takes `LocalContentColor`, which NSH does not have, so the layer's color is the tokens':
        // one layer serves the three states only while the tokens do not tell them apart.
        for (hover, focus, pressed) in state_layer_roles() {
            assert_eq!(hover, focus);
            assert_eq!(hover, pressed);
        }
        assert_eq!(
            state_layer_roles().map(|(hover, ..)| hover),
            [ColorRole::OnSurface, ColorRole::Primary]
        );
    }

    #[test]
    fn the_state_layer_is_on_surface_unselected_and_primary_selected_light_and_dark() {
        for dark in [false, true] {
            for (checked, role) in [(false, ColorRole::OnSurface), (true, ColorRole::Primary)] {
                let (_app, recording) = start(Config {
                    checked,
                    dark,
                    ..Config::default()
                });
                let mounted = Mounted::of(&recording.items());
                assert_eq!(
                    layer_color(&mounted, &replay(&recording)),
                    color(scheme_color(role, dark), 255),
                    "selected {checked}, dark {dark}"
                );
            }
        }
    }

    #[test]
    fn it_is_24_points_with_the_ring_centered_and_a_40_point_layer_around_it() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.size(), Size::new(Points(24.0), Points(24.0)));
        // The ring: 20 points, 2 points of padding on each side.
        let ring = mounted.frames[&mounted.indicator];
        assert_eq!(ring.size, Size::new(Points(20.0), Points(20.0)));
        assert_eq!(ring.origin, point(px(2.0), px(2.0)));
        // The state layer: 40 points across, centered on the radio button, so 8 points past it on each side; and
        // out of the layout, which stays 24 points.
        let layer = mounted.frames[&mounted.layer];
        assert_eq!(layer.size, Size::new(Points(40.0), Points(40.0)));
        assert_eq!(layer.origin, point(px(-8.0), px(-8.0)));
        assert_eq!(mounted.size(), Size::new(Points(24.0), Points(24.0)));
    }

    /// The state layer is a circle that fills its 40-point box: centered on it, as wide as it.
    #[test]
    fn the_state_layer_is_a_circle_as_wide_as_its_box() {
        let (_app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let arcs = paths(mounted.drawing(mounted.layer), &replay(&recording));
        let [layer] = &arcs[..] else {
            panic!("one circle: {arcs:?}");
        };
        match layer.as_slice() {
            [Step::Arc { x, y, radius, .. }] => {
                assert_eq!((*x, *y), (20.0, 20.0), "{layer:?}");
                assert_eq!(*radius, 20.0, "{layer:?}");
            }
            other => panic!("one arc, not {other:?}"),
        }
    }

    #[test]
    fn it_has_a_48_point_touch_target_without_growing() {
        let mounted = mount(Config::default());
        let slop = mounted.hit_slop.expect("the radio button's props");
        // 12 points on each side: 24 + 12 + 12 = 48, and no more.
        assert_eq!(slop, EdgeInsets::all(px(12.0)));
        assert_eq!(mounted.size().width.0 + slop.left.0 + slop.right.0, 48.0);
        assert_eq!(mounted.size().height.0 + slop.top.0 + slop.bottom.0, 48.0);
        assert_eq!(mounted.size(), Size::new(Points(24.0), Points(24.0)));
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_radio_button_with_its_state() {
        for (checked, disabled) in LOOKS {
            let mounted = mount(Config {
                checked,
                disabled,
                ..Config::default()
            });
            assert_eq!(mounted.label.as_deref(), Some("Plan"));
            assert_eq!(mounted.disabled, Some(disabled));
            assert_eq!(
                mounted.checked,
                Some(if checked {
                    Checked::True
                } else {
                    Checked::False
                })
            );
        }
    }

    #[test]
    fn a_press_on_an_unselected_radio_button_asks_to_be_selected_once() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        app.press("label:Plan")?;
        assert!(app.shows("changes: 1"));
        assert!(!app.shows("changes: 2"));
        // The app passed the change back: it is selected.
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.checked, Some(Checked::True));
        Ok(())
    }

    #[test]
    fn a_selected_radio_button_stays_selected_and_reports_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            checked: true,
            ..Config::default()
        });
        app.press("label:Plan")?;
        assert!(app.shows("changes: 0"));
        assert_eq!(Mounted::of(&recording.items()).checked, Some(Checked::True));
        Ok(())
    }

    #[test]
    fn a_radio_button_the_app_does_not_select_stays_as_it_was() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            reply: Reply::Ignores,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let before = recording.items().len();
        app.press("label:Plan")?;
        assert!(app.shows("changes: 1"));
        // The press changes nothing it shows: unselected, no spring, no value set, the same drawing.
        let items = recording.items();
        let after = &items[before..];
        assert_eq!(Mounted::of(&items).checked, Some(Checked::False));
        assert_eq!(started_springs(after), Vec::new());
        assert_eq!(set_values(after), 0);
        let drawn = drawn(&mounted, &replay(&recording));
        assert_eq!(drawn.dot, 0.0);
        assert_eq!(drawn.stroke, expected(false, false, false));
        Ok(())
    }

    #[test]
    fn a_disabled_radio_button_takes_no_press_and_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Plan");
        assert!(app.shows("changes: 0"));
        let _ = app.hover("label:Plan");
        let mounted = Mounted::of(&recording.items());
        // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in,
        // hovered or not.
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::default())
        );
        // The radio button keeps its size: only its own views report it, not the hidden state layer.
        assert_eq!(mounted.size(), Size::new(Points(24.0), Points(24.0)));
        // Enabled, its state layer is there.
        let enabled = mount(Config::default());
        assert_eq!(
            enabled.frames.get(&enabled.layer).map(|frame| frame.size),
            Some(Size::new(Points(40.0), Points(40.0)))
        );
    }

    /// The opacities the state layer's style was sent in `items`.
    fn layer_opacities(mounted: &Mounted, items: &[MountItem]) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    /// Nothing but the state layer's style came in `items`: no frame, no drawing, no animation, no other style.
    fn only_the_layer_is_styled(mounted: &Mounted, items: &[MountItem]) {
        assert!(
            items.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } => *id == mounted.layer,
                MountItem::Frame { .. } | MountItem::Canvas { .. } | MountItem::Animation(_) =>
                    false,
                _ => true,
            }),
            "{items:?}"
        );
    }

    #[test]
    fn hover_shows_the_state_layer_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("label:Plan")?;
        let hover = recording.in_step("hover(label:Plan)");
        // `StateTokens.HoverStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &hover), [0.08]);
        only_the_layer_is_styled(&mounted, &hover);
        assert_eq!(mounted.layout_items(&hover), 0);
        Ok(())
    }

    #[test]
    fn press_shows_the_state_layer_and_lays_out_nothing() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        // The headless driver reports hover and focus itself, and not a press's state, which a platform reports
        // (`EngineControl::set_states`): it is reported here as a platform would.
        app.step("pressed");
        app.engine()
            .control()
            .set_states(mounted.radio, state::PRESSED);
        app.settle();
        let down = recording.in_step("pressed");
        // `StateTokens.PressedStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &down), [0.1], "{down:?}");
        only_the_layer_is_styled(&mounted, &down);
        assert_eq!(mounted.layout_items(&down), 0);
        // And it is gone with the press.
        app.step("released");
        app.engine().control().set_states(mounted.radio, 0);
        app.settle();
        let up = recording.in_step("released");
        assert_eq!(layer_opacities(&mounted, &up), [0.0], "{up:?}");
        only_the_layer_is_styled(&mounted, &up);
    }

    #[test]
    fn focus_shows_the_state_layer_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("tab")?;
        let focus = recording.in_step("keyboard.press(tab)");
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &focus), [0.1], "{focus:?}");
        only_the_layer_is_styled(&mounted, &focus);
        assert_eq!(mounted.layout_items(&focus), 0);
        Ok(())
    }

    #[test]
    fn in_a_radio_group_the_arrow_keys_select_the_next_radio_button_past_a_disabled_one() {
        let (mut app, _recording) =
            launch(Root::of::<GroupProbe>(), false, false, "text:plan: basic");
        // Tab enters the group at the selected radio button; Down moves to the next and asks for it.
        app.press_key("tab").unwrap();
        assert!(app.shows("plan: basic"));
        app.press_key("down").unwrap();
        assert!(app.shows("plan: pro"));
        // "Max" is disabled: the focus goes past it.
        app.press_key("down").unwrap();
        assert!(app.shows("plan: team"));
        // And around from the last to the first.
        app.press_key("down").unwrap();
        assert!(app.shows("plan: basic"));
        app.press_key("up").unwrap();
        assert!(app.shows("plan: team"));
    }

    /// The springs of `items` that `token` describes, every parameter as ADR 0004 converts it: from the speed the
    /// value already has, at the screen's rate.
    fn springs_of(items: &[MountItem], token: SpringToken) -> Vec<StartedSpring> {
        let spec = token.spec();
        started_springs(items)
            .into_iter()
            .filter(|spring| spring.stiffness.to_bits() == spec.stiffness.to_bits())
            .inspect(|spring| {
                assert_eq!(
                    spring.damping.to_bits(),
                    spec.damping_coefficient().to_bits(),
                    "{token:?}: {spring:?}"
                );
                assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
                assert_eq!(spring.velocity, None, "{token:?}");
                assert_eq!(spring.frame_rate, None, "{token:?}");
            })
            .collect()
    }

    /// The dot's spring (androidx `MotionSchemeKeyTokens.FastSpatial`) and the color's (`DefaultEffects`) among
    /// `items`, none or one of each, and no other spring.
    fn springs(items: &[MountItem]) -> (Option<StartedSpring>, Option<StartedSpring>) {
        let dot = springs_of(items, SpringToken::FastSpatial);
        let color = springs_of(items, SpringToken::DefaultEffects);
        assert_eq!(
            started_springs(items).len(),
            dot.len() + color.len(),
            "{items:?}"
        );
        assert!(dot.len() <= 1 && color.len() <= 1, "{items:?}");
        (dot.first().copied(), color.first().copied())
    }

    /// The values `items` set at once, as `(node, value)`.
    fn sets(items: &[MountItem]) -> Vec<(NodeId, f32)> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::Animation(AnimationOp::Set { node, value }) => Some((*node, *value)),
                _ => None,
            })
            .collect()
    }

    /// What `selector`'s press sent: the items that came with it.
    fn pressed(app: &mut Headless, recording: &Recording, selector: &str) -> Vec<MountItem> {
        let before = recording.items().len();
        app.press(selector).unwrap();
        recording.items().split_off(before)
    }

    /// Everything a frame of the replay shows.
    #[derive(Debug, Clone, Copy)]
    struct Frame {
        /// The dot's animated value, 0 to 6 points before its overshoot.
        dot: f32,
        /// Where the color is among the looks: 0 unselected, 1 selected.
        progress: f32,
        drawn: Drawn,
    }

    /// Ticks `replay` `frames` times and reads, after each tick, the dot's value, the color's progress and what the
    /// indicator draws.
    fn play(
        replay: &mut Replay,
        mounted: &Mounted,
        (dot, progress): (NodeId, NodeId),
        frames: usize,
    ) -> Vec<Frame> {
        (0..frames)
            .map(|_| {
                let dot = replay.run(dot, 1)[0];
                Frame {
                    dot,
                    progress: replay.value(progress),
                    drawn: drawn(mounted, replay),
                }
            })
            .collect()
    }

    /// Whether every channel of `c` is between those of `a` and `b`.
    fn between(a: Color, b: Color, c: Color) -> bool {
        [24, 16, 8, 0].into_iter().all(|shift| {
            let channel = |color: Color| (color.0 >> shift) & 0xff;
            let (low, high) = (channel(a).min(channel(b)), channel(a).max(channel(b)));
            (low..=high).contains(&channel(c))
        })
    }

    /// What holds in every frame of every motion between the enabled looks: the ring does not move, the ring and the
    /// dot have one color between the unselected one and the selected one (never toward the disabled ones), the
    /// dot's radius is the animated one less a half stroke and never below 0.
    fn check(frame: &Frame, dark: bool, what: &str) {
        let drawn = &frame.drawn;
        assert_eq!(drawn.ring, 9.0, "{what}: {frame:?}");
        assert_eq!(drawn.line_width, 2.0, "{what}: {frame:?}");
        assert!(
            drawn.dot.is_finite() && drawn.dot >= 0.0,
            "{what}: {frame:?}"
        );
        assert!(
            (drawn.dot - (frame.dot - 1.0).max(0.0)).abs() < 1e-3,
            "{what}: {frame:?}"
        );
        assert_eq!(drawn.stroke, drawn.fill, "{what}: {frame:?}");
        assert!(
            between(
                expected(false, false, dark),
                expected(true, false, dark),
                drawn.stroke
            ),
            "{what}: {frame:?}"
        );
        // The progress never goes past the selected look, toward the disabled ones.
        assert!(frame.progress <= 1.0 + 1e-4, "{what}: {frame:?}");
    }

    /// Moves of the dot and the color, `from` to `to` (0 unselected, 1 selected), and what they draw all along.
    fn assert_motion(
        frames: &[Frame],
        (dot, color): (StartedSpring, StartedSpring),
        (from, to): (f64, f64),
        dark: bool,
        what: &str,
    ) {
        for frame in frames {
            check(frame, dark, what);
        }
        // The dot's value is the closed-form `FastSpatial` spring, and the color's progress the `DefaultEffects`
        // one, each within 0.1 % of its travel (ADR 0010).
        let dot_samples: Vec<f32> = frames.iter().map(|frame| frame.dot).collect();
        let color_samples: Vec<f32> = frames.iter().map(|frame| frame.progress).collect();
        let gap = worst_gap(&dot_samples, from * 6.0, &dot);
        assert!(gap < tolerance(from * 6.0, to * 6.0), "{what}: dot {gap}");
        let gap = worst_gap(&color_samples, from, &color);
        assert!(gap < tolerance(from, to), "{what}: color {gap}");
    }

    #[test]
    fn selecting_springs_the_dot_and_the_color_and_the_drawing_follows() {
        for dark in [false, true] {
            let what = format!("dark {dark}");
            let (mut app, recording) = start(Config {
                dark,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let items = pressed(&mut app, &recording, "label:Plan");
            let (dot, color) = springs(&items);
            let (dot, color) = (
                dot.expect("the dot springs"),
                color.expect("the color springs"),
            );
            // The dot grows to its 6-point radius and the color goes to the selected look, springing, setting nothing.
            assert_eq!((dot.to, color.to), (6.0, 1.0), "{what}");
            assert_eq!(set_values(&items), 0, "{what}");
            // Nothing is drawn again or laid out for it.
            assert!(
                !items
                    .iter()
                    .any(|item| matches!(item, MountItem::Canvas { .. })),
                "{what}: {items:?}"
            );
            assert_eq!(mounted.layout_items(&items), 0, "{what}");
            // While they play, the engine sends nothing (ADR 0010).
            app.step("run");
            app.run_for(Duration::from_secs(1)).unwrap();
            assert!(recording.in_step("run").is_empty(), "{what}");

            let mut graph = replay(&recording);
            let nodes = (dot.node, color.node);
            let mut frames = play(&mut graph, &mounted, nodes, 180);
            assert_motion(&frames, (dot, color), (0.0, 1.0), dark, &what);
            // Mid-way the dot is partly grown and the color is partly the selected one.
            let mid = frames[9];
            assert!(
                mid.drawn.dot > 1.0 && mid.drawn.dot < 5.0,
                "{what}: {mid:?}"
            );
            assert!(
                between(
                    expected(false, false, dark),
                    expected(true, false, dark),
                    mid.drawn.stroke
                ) && mid.drawn.stroke != expected(false, false, dark)
                    && mid.drawn.stroke != expected(true, false, dark),
                "{what}: {mid:?}"
            );
            // The dot swells past its size (`FastSpatial` overshoots) and the color does not (`DefaultEffects`).
            let peak = |value: fn(&Frame) -> f32| frames.iter().map(value).fold(f32::MIN, f32::max);
            assert!(peak(|frame| frame.dot) > 6.3, "{what}");
            assert!(peak(|frame| frame.drawn.dot) > 5.3, "{what}");
            assert!(peak(|frame| frame.progress) <= 1.0 + 1e-4, "{what}");
            // They settle exactly on the selected look, and the graph stops.
            frames = play(&mut graph, &mounted, nodes, 600);
            assert!(!graph.is_animating(), "{what}");
            let last = frames[599];
            assert_eq!(last.dot.to_bits(), 6.0_f32.to_bits(), "{what}");
            assert_eq!(last.progress.to_bits(), 1.0_f32.to_bits(), "{what}");
            assert_eq!(last.drawn.dot, 5.0, "{what}");
            assert_eq!(last.drawn.stroke, expected(true, false, dark), "{what}");
        }
    }

    #[test]
    fn deselecting_springs_the_dot_and_the_color_back_and_never_draws_a_negative_dot() {
        let (mut app, recording) = start(Config {
            checked: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let items = pressed(&mut app, &recording, "Clear");
        let (dot, color) = springs(&items);
        let (dot, color) = (
            dot.expect("the dot springs"),
            color.expect("the color springs"),
        );
        assert_eq!((dot.to, color.to), (0.0, 0.0));
        assert_eq!(set_values(&items), 0);
        let mut graph = replay(&recording);
        let nodes = (dot.node, color.node);
        let frames = play(&mut graph, &mounted, nodes, 180);
        assert_motion(&frames, (dot, color), (1.0, 0.0), false, "deselecting");
        // The spring goes below 0 on its way (it overshoots), and the dot is not drawn with a negative radius: the
        // arc would be undefined.
        assert!(frames.iter().any(|frame| frame.dot < -0.1), "{frames:?}");
        assert!(frames.iter().all(|frame| frame.drawn.dot >= 0.0));
        let rest = play(&mut graph, &mounted, nodes, 600);
        assert!(!graph.is_animating());
        let last = rest[599];
        assert_eq!(last.dot.to_bits(), 0.0_f32.to_bits());
        assert_eq!(last.progress.to_bits(), 0.0_f32.to_bits());
        assert_eq!(last.drawn.dot, 0.0);
        assert_eq!(last.drawn.stroke, expected(false, false, false));
    }

    #[test]
    fn a_change_of_mind_springs_back_from_where_and_as_fast_as_the_dot_and_the_color_move() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let items = pressed(&mut app, &recording, "Select");
        let (dot, color) = springs(&items);
        let (dot, color) = (dot.expect("a dot spring"), color.expect("a color spring"));
        let nodes = (dot.node, color.node);
        let mut graph = replay(&recording);
        // 50 ms in, while both still move, the app takes the selection back.
        let before = play(&mut graph, &mounted, nodes, 6);
        let items = pressed(&mut app, &recording, "Clear");
        let (back_dot, back_color) = springs(&items);
        let (back_dot, back_color) = (
            back_dot.expect("the dot springs back"),
            back_color.expect("the color springs back"),
        );
        // The same values, sent to where they were, with no velocity of their own: they go on from what they have.
        assert_eq!((back_dot.node, back_color.node), nodes);
        assert_eq!((back_dot.to, back_color.to), (0.0, 0.0));
        assert_eq!(set_values(&items), 0);
        graph.apply(&items);
        let after = play(&mut graph, &mounted, nodes, 180);
        for frame in before.iter().chain(&after) {
            check(frame, false, "a change of mind");
        }

        // The graph last ticked 5 frames into the first springs: the new ones start there, at that value and that
        // speed.
        let at = 5.0 / REPLAY_HZ;
        let continues = |first: &StartedSpring,
                         back: &StartedSpring,
                         at_frame_5: f32,
                         samples: Vec<f32>,
                         what: &str| {
            let position =
                |t: f64| spring_position(0.0, f64::from(first.to), 0.0, parameters(first), t);
            let from = position(at);
            let h = 1e-6;
            let speed = (position(at + h) - position(at - h)) / (2.0 * h);
            assert!(
                (f64::from(at_frame_5) - from).abs() < 1e-3,
                "{what}: {at_frame_5} {from}"
            );
            assert!(speed > 1.0, "{what}: {speed}");
            let gap = |velocity: f64| {
                samples
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| {
                        let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                        let closed = spring_position(
                            from,
                            f64::from(back.to),
                            velocity,
                            parameters(back),
                            t,
                        );
                        (f64::from(value) - closed).abs()
                    })
                    .fold(0.0, f64::max)
            };
            let tolerance = tolerance(from, f64::from(back.to));
            assert!(gap(speed) < tolerance, "{what}: {}", gap(speed));
            // Starting again from rest, as a new animation would, is another curve.
            assert!(gap(0.0) > 10.0 * tolerance, "{what}: {}", gap(0.0));
        };
        continues(
            &dot,
            &back_dot,
            before[5].dot,
            after.iter().map(|frame| frame.dot).collect(),
            "dot",
        );
        continues(
            &color,
            &back_color,
            before[5].progress,
            after.iter().map(|frame| frame.progress).collect(),
            "color",
        );
        let rest = play(&mut graph, &mounted, nodes, 600);
        assert!(!graph.is_animating());
        assert_eq!(rest[599].dot.to_bits(), 0.0_f32.to_bits());
        assert_eq!(rest[599].progress.to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn spamming_keeps_every_frame_between_the_enabled_looks_and_settles_on_the_last_change() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let mut graph = replay(&recording);
        let mut nodes = None;
        let mut selected = false;
        for round in 0..12 {
            selected = !selected;
            let items = pressed(
                &mut app,
                &recording,
                if selected { "Select" } else { "Clear" },
            );
            let (dot, color) = springs(&items);
            let (dot, color) = (
                dot.expect("the dot springs"),
                color.expect("the color springs"),
            );
            // Always the same two values, sprung from where they are: nothing is set, nothing starts over.
            let both = (dot.node, color.node);
            assert_eq!(*nodes.get_or_insert(both), both, "round {round}");
            assert_eq!(set_values(&items), 0, "round {round}");
            assert_eq!(
                (dot.to, color.to),
                if selected { (6.0, 1.0) } else { (0.0, 0.0) },
                "round {round}"
            );
            graph.apply(&items);
            // One to four frames until the next change, so most are caught mid-spring.
            for frame in play(&mut graph, &mounted, both, 1 + round % 4) {
                check(&frame, false, &format!("round {round}"));
            }
        }
        let nodes = nodes.expect("rounds");
        let rest = play(&mut graph, &mounted, nodes, 600);
        for frame in &rest {
            check(frame, false, "settling");
        }
        assert!(!graph.is_animating());
        let last = rest[599];
        // The last change was to unselected.
        assert!(!selected);
        assert_eq!(last.dot.to_bits(), 0.0_f32.to_bits());
        assert_eq!(last.progress.to_bits(), 0.0_f32.to_bits());
        assert_eq!(last.drawn.stroke, expected(false, false, false));
    }

    #[test]
    fn with_reduce_motion_the_dot_and_the_color_are_set_at_once_and_nothing_springs() {
        // ADR 0010, check 6: no spring, and no jump of anything else either.
        for (checked, step) in [(false, "label:Plan"), (true, "Clear")] {
            let what = format!("selected {checked}");
            let (mut app, recording) = start(Config {
                checked,
                motion: Motion::Reduced,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let items = pressed(&mut app, &recording, step);
            assert_eq!(started_springs(&items), Vec::new(), "{what}");
            // Each value is set once, to where it goes: the dot's 6 points and the color's selected look (1), or 0 and 0.
            let mut values: Vec<f32> = sets(&items).into_iter().map(|(_, value)| value).collect();
            values.sort_by(f32::total_cmp);
            assert_eq!(
                values,
                if checked { [0.0, 0.0] } else { [1.0, 6.0] },
                "{what}"
            );
            // The drawing is at the new look with no frame ticked.
            let graph = replay(&recording);
            let drawn = drawn(&mounted, &graph);
            assert_eq!(drawn.dot, if checked { 0.0 } else { 5.0 }, "{what}");
            assert_eq!(drawn.stroke, expected(!checked, false, false), "{what}");
            assert_eq!(drawn.fill, drawn.stroke, "{what}");
            assert!(!graph.is_animating(), "{what}");
            // Nothing is drawn again or laid out.
            assert!(
                !items
                    .iter()
                    .any(|item| matches!(item, MountItem::Canvas { .. })),
                "{what}"
            );
            assert_eq!(mounted.layout_items(&items), 0, "{what}");
        }
    }

    #[test]
    fn a_disabled_radio_button_snaps_its_color_and_only_the_dot_springs() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());

        // Disabling snaps to the disabled color: no spring, and the progress is set to the disabled unselected look.
        let items = pressed(&mut app, &recording, "Disable");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(
            sets(&items)
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            [2.0]
        );
        let graph = replay(&recording);
        assert_eq!(drawn(&mounted, &graph).stroke, expected(false, true, false));
        assert!(!graph.is_animating());

        // Selecting it from the app, disabled: the dot grows with its spring (androidx animates it whatever
        // `enabled` is), and the color snaps to the disabled selected look.
        let items = pressed(&mut app, &recording, "Select");
        let (dot, color) = springs(&items);
        assert_eq!(dot.map(|spring| spring.to), Some(6.0));
        assert_eq!(color, None);
        assert_eq!(
            sets(&items)
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            [3.0]
        );
        let graph = replay(&recording);
        assert_eq!(drawn(&mounted, &graph).stroke, expected(true, true, false));

        // Enabling it snaps to the selected look at once.
        let items = pressed(&mut app, &recording, "Enable");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(
            sets(&items)
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            [1.0]
        );
        let graph = replay(&recording);
        assert_eq!(drawn(&mounted, &graph).stroke, expected(true, false, false));
    }

    #[test]
    fn enabling_does_not_travel_through_the_selected_color() {
        // A radio button that becomes enabled starts at the color it has, not from the disabled one: from the disabled
        // unselected look (2) to the unselected one (0) would pass the selected one (1) if it sprang.
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let items = pressed(&mut app, &recording, "Enable");
        assert_eq!(started_springs(&items), Vec::new());
        let graph = replay(&recording);
        assert_eq!(
            drawn(&mounted, &graph).stroke,
            expected(false, false, false)
        );
        assert!(!graph.is_animating());
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for dark in [false, true] {
            for (checked, disabled) in LOOKS {
                for motion in [Motion::Full, Motion::Reduced] {
                    let what =
                        format!("selected {checked}, disabled {disabled}, dark {dark}, {motion:?}");
                    let (mut app, recording) = start(Config {
                        checked,
                        disabled,
                        dark,
                        motion,
                        ..Config::default()
                    });
                    let items = recording.items();
                    assert_eq!(started_springs(&items), Vec::new(), "{what}");
                    // The values are created where they rest: nothing is set again.
                    assert_eq!(set_values(&items), 0, "{what}");
                    assert!(!replay(&recording).is_animating(), "{what}");
                    // And the engine sends nothing afterwards.
                    app.step("rest");
                    app.run_for(Duration::from_secs(1)).unwrap();
                    assert!(recording.in_step("rest").is_empty(), "{what}");
                }
            }
        }
    }

    /// The vendored `RadioButton.kt`, at the pinned commit (ADR 0012).
    const RADIO_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/RadioButton.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_androidx_is_still_there() {
        // The sizes `RadioButtonTokens` does not have.
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "private val RadioButtonPadding",
                "get() = 2.dp",
                "private val RadioButtonDotSize",
                "get() = 12.dp",
                "private val RadioStrokeWidth",
                "get() = 2.dp",
            ]
        ));
        assert_eq!(PADDING, Points(2.0));
        assert_eq!(DOT_SIZE, Points(12.0));
        assert_eq!(STROKE_WIDTH, Points(2.0));
        // The sizes it has.
        assert_eq!(radio_button_tokens::ICON_SIZE, Points(20.0));
        assert_eq!(radio_button_tokens::STATE_LAYER_SIZE, Points(40.0));
        assert_eq!(FRAME, Points(24.0));
        // Padding on each side, whatever its size: with 3 points the sum and the product differ.
        assert_eq!(frame_of(Points(20.0), Points(3.0)), Points(26.0));
        assert_eq!(frame_of(Points(20.0), Points(0.0)), Points(20.0));
        // The box: the icon, padded, centered.
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                ".wrapContentSize(Alignment.Center)",
                ".padding(RadioButtonPadding)",
                ".requiredSize(RadioButtonTokens.IconSize)",
            ]
        ));
        // The ring: a circle one half stroke inside the icon's edge, and the dot one half stroke inside its radius, drawn
        // only while it is more than nothing.
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "val strokeWidth = RadioStrokeWidth.toPx()",
                "drawCircle(",
                "animatedColor,",
                "radius = (RadioButtonTokens.IconSize / 2).toPx() - strokeWidth / 2,",
                "style = Stroke(strokeWidth),",
                ")",
                "if (animatedDotRadius > 0.dp) {",
                "drawCircle(animatedColor, animatedDotRadius.toPx() - strokeWidth / 2, style = Fill)",
                "}",
            ]
        ));
        assert!(cites(
            RADIO_BUTTON_KT,
            &["dotRadius = if (selected) RadioButtonDotSize / 2 else 0.dp,"]
        ));
        // The motion: the dot with `FastSpatial`, the color with `DefaultEffects` only while enabled, snapping when not
        // (neither is in the tokens file).
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "val animatedDotRadius by",
                "animateDpAsState(",
                "targetValue = dotRadius,",
                "// TODO Load the motionScheme tokens from the component tokens file",
                "animationSpec = MotionSchemeKeyTokens.FastSpatial.value(),",
                ")",
            ]
        ));
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "if (enabled) {",
                "// TODO Load the motionScheme tokens from the component tokens file",
                "animateColorAsState(color, MotionSchemeKeyTokens.DefaultEffects.value())",
                "} else {",
                "// If not enabled 'snap' to the disabled state, as there should be no animations between",
                "// enabled / disabled.",
                "rememberUpdatedState(color)",
                "}",
            ]
        ));
        // The colors are the tokens', the disabled ones at their opacities.
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "selectedColor = fromToken(RadioButtonTokens.SelectedIconColor),",
                "unselectedColor = fromToken(RadioButtonTokens.UnselectedIconColor),",
                "disabledSelectedColor =",
                "fromToken(RadioButtonTokens.DisabledSelectedIconColor)",
                ".copy(alpha = RadioButtonTokens.DisabledSelectedIconOpacity),",
                "disabledUnselectedColor =",
                "fromToken(RadioButtonTokens.DisabledUnselectedIconColor)",
                ".copy(alpha = RadioButtonTokens.DisabledUnselectedIconOpacity),",
            ]
        ));
        // The ripple is the 40-point state layer, with no color of its own: it takes the content color, which NSH does
        // not have, so the layer takes the tokens' (`the_state_layer_has_one_color_for_hover_focus_and_press`).
        assert!(cites(
            RADIO_BUTTON_KT,
            &[
                "ripple(",
                "bounded = false,",
                "radius = RadioButtonTokens.StateLayerSize / 2,",
                "focusRingShape = CircleShape,",
                "),",
            ]
        ));
        // The touch target.
        assert!(cites(
            RADIO_BUTTON_KT,
            &["Modifier.minimumInteractiveComponentSize()"]
        ));
        // A line that is not there is not found.
        assert!(!cites(
            RADIO_BUTTON_KT,
            &["animationSpec = MotionSchemeKeyTokens.DefaultSpatial.value(),"]
        ));
    }
}
