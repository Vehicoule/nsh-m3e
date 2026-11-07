//! The Material 3 Expressive switch: a track and a round handle that springs from one end to the other, the optional
//! icon in the handle, and the state layer of hover, focus and press around it.
//!
//! The switch follows androidx's `Switch(checked, onCheckedChange, …, thumbContent, …)` in `Switch.kt`, at the pinned
//! commit, as [ADR 0012] says. It reads the tokens that file reads from `SwitchTokens`, and the few values the file
//! writes itself, each cited where it is used.
//!
//! # What androidx does, and what this switch does the same
//!
//! - The handle (androidx's *thumb*) is 16 points across when the switch is unchecked, and 24 when it is checked or
//!   shows an icon: `UncheckedThumbDiameter` and `ThumbDiameter` (`Switch.kt` lines 661-664, chosen at lines 289-290).
//!   Its center is 16 points from the track's start when unchecked and 16 points from its end when checked, whatever
//!   its size (`minBound` and `maxBound`, lines 295-298 and 303-304). The position and the size move with
//!   `MotionSchemeKeyTokens.FastSpatial` (line 188), the Expressive fast spatial spring, converted as [ADR 0004] says.
//! - The handle's colors, the track's, the outline's and the icon's change at once when the switch is checked or
//!   disabled: androidx animates none of them (`SwitchImpl` gets each color as it is, lines 107-111 and 119-200).
//! - A disabled switch has opaque colors: each is a token's color at a token's opacity composited over the scheme's
//!   surface (`compositeOver(surface)`, `SwitchDefaults.defaultSwitchColors`, lines 426-467), so that the handle hides
//!   the track behind it.
//! - The unchecked track has a 2-point outline, `TrackOutlineWidth`, in the color of `UnselectedFocusTrackOutlineColor`;
//!   the checked track has none visible (`checkedBorderColor = Color.Transparent`, lines 176 and 432-437).
//! - The track is `SwitchWidth` by `SwitchHeight`, 52 by 32 points (lines 175 and 666-669), and presses count in at
//!   least 48 points around it (`minimumInteractiveComponentSize`, line 139).
//!
//! # How it is made
//!
//! The switch is built on `nsh-base`'s switch, which owns the behavior (presses, Space and Return, focus, the disabled
//! state, the switch role and its value for screen readers). Its track is `nsh-base`'s track part, styled with the
//! tokens. Its handle, its icon and its state layer are views of this crate over the track, because NSH cannot spring
//! a part's transform from a style: the handle is one view whose position and scale are animated values, and the
//! icon and the state layer follow its center, so every motion is a transform that the platform plays by itself
//! ([ADR 0010]) and nothing is laid out while the handle moves. `nsh-base`'s own handle part is left empty and hidden.
//!
//! The handle's diameter is a scale: the view is 24 points across, `SelectedHandleWidth`, scaled to 16 when
//! unchecked and empty. androidx moves the handle's left edge and its size with two springs of one specification, so
//! the handle's center, which is the left edge plus half the size, moves as one spring too (the springs add): one
//! animated value for the center and one for the scale are the same motion. The icon does not scale, as androidx's
//! does not (`SwitchDefaults.IconSize`, line 473).
//!
//! With reduce motion on, the handle goes to its place at once: no spring, no transform in between. The short fade
//! that [`docs/app-requirements.md`] asks reduce motion to give is not made for the switch: androidx has none, and no
//! token gives its duration (as for the button's morph, [ADR 0011]).
//!
//! # What is left out
//!
//! - **The pressed handle.** androidx grows the handle to 28 points while the switch is pressed
//!   (`SwitchTokens.PressedHandleWidth`; the thumb snaps there with a `SnapSpec` and springs back on release, lines
//!   263-266, 288, 301-302 and 309-315). `nsh-base`'s switch (`crates/nsh-base/src/switch.rs` at the pinned NSH
//!   revision) has no `PressDown` or `PressUp` event: its events are `Change(bool)` only, and its press styling is
//!   the app's `active:` style, which changes at once, so a release would jump instead of spring. Not found: a way to
//!   start the release's spring from the pressed size. The switch is therefore the same at rest as in androidx and
//!   does not grow when pressed; its state layer still shows the press.
//! - **The hover, focus and pressed colors of the token file.** `SwitchTokens` lists a handle, track and icon color
//!   for each of hover, focus and pressed, and `Switch.kt` reads none of them: hover, focus and press only show the
//!   state layer. This switch follows the component (ADR 0012).
//! - **The focus ring.** androidx draws the focus as the state layer's opacity unless the app's
//!   `LocalRippleThemeConfiguration` asks for an inset ring (`Switch.kt` lines 152-168 and 194-198; `Ripple.kt`,
//!   `RippleDefaults.ThemeConfiguration` is `OpacityFocusThemeConfiguration`, line 360, a file this repository does
//!   not vendor); the ring is not made and `SwitchTokens.FocusIndicatorColor` is not read.
//! - **Dragging the handle.** `Switch.kt` has a "TODO: Add Swipeable modifier" (line 136): androidx does not drag the
//!   handle either.
//! - **The color of the state layer.** androidx's ripple takes `LocalContentColor` when it is given no color
//!   (`Ripple.kt` lines 80-84; `Switch.kt` lines 194-198 give it none), the content color of the component around
//!   the switch, which NSH does not have. This switch lays `OnSurface`, the content color a `Surface` provides, over
//!   the handle. `SwitchTokens` has no state layer color: not found.
//!
//! [ADR 0004]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0004-spring-damping-conversion.md
//! [ADR 0010]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0010-measure-animations-in-ci.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [`docs/app-requirements.md`]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/app-requirements.md

use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::icon::Icon;
use crate::motion::{SPRING_MASS, SpringToken};
use crate::tint::{MIN_TOUCH_TARGET, Tint, role_color, tint_color};
use crate::tokens::androidx::{state_tokens, switch_tokens};

/// Which end of the track the handle is at: `Switch.kt` puts it at `minBound` unchecked and `maxBound` checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// Unchecked.
    Start,
    /// Checked.
    End,
}

impl End {
    fn of(checked: bool) -> Self {
        if checked { Self::End } else { Self::Start }
    }
}

/// Whether the handle holds an icon: androidx's `hasContent`, `thumbContent` measuring something (`Switch.kt` lines
/// 283-285).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    Empty,
    Icon,
}

impl Content {
    fn of(icon: bool) -> Self {
        if icon { Self::Icon } else { Self::Empty }
    }
}

/// Where the handle is and how big: its center along the track, and its diameter, both in points.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Handle {
    /// The distance from the track's start to the handle's center.
    center: f32,
    /// The handle's diameter.
    diameter: f32,
}

impl Handle {
    /// The handle at `end` with `content`, as `ThumbNode.measure` places it when the switch is not pressed
    /// (`Switch.kt` lines 281-321).
    fn at(end: End, content: Content) -> Self {
        // `hasContent || checked -> ThumbDiameter`, `else -> UncheckedThumbDiameter` (lines 289-290).
        let diameter = match (end, content) {
            (End::Start, Content::Empty) => switch_tokens::UNSELECTED_HANDLE_WIDTH,
            _ => switch_tokens::SELECTED_HANDLE_WIDTH,
        };
        // `minBound = (SwitchHeight - size) / 2` and `maxBound = (SwitchWidth - ThumbDiameter) - ThumbPadding` are
        // the handle's left edge (lines 295-298 and 303-304); its center is half a diameter further.
        let left = match end {
            End::Start => (switch_tokens::TRACK_HEIGHT.0 - diameter.0) / 2.0,
            End::End => {
                (switch_tokens::TRACK_WIDTH.0 - switch_tokens::SELECTED_HANDLE_WIDTH.0)
                    - thumb_padding()
            }
        };
        Self {
            center: left + diameter.0 / 2.0,
            diameter: diameter.0,
        }
    }

    /// The scale that makes the handle's view, [`BASE_DIAMETER`] across, this diameter.
    fn scale(self) -> f32 {
        self.diameter / BASE_DIAMETER.0
    }
}

/// `ThumbPadding = (SwitchHeight - ThumbDiameter) / 2` (`Switch.kt` line 670): the space between the track's end and
/// the checked handle.
fn thumb_padding() -> f32 {
    (switch_tokens::TRACK_HEIGHT.0 - switch_tokens::SELECTED_HANDLE_WIDTH.0) / 2.0
}

/// The diameter of the handle's view, which is scaled to the handle's diameter: the largest one it has while the
/// switch is not pressed, `ThumbDiameter`.
const BASE_DIAMETER: Points = switch_tokens::SELECTED_HANDLE_WIDTH;

/// The side of the icon: `SwitchDefaults.IconSize`, 16 dp (`Switch.kt` line 473). `SwitchTokens.SelectedIconSize` and
/// `UnselectedIconSize` say the same.
const ICON_SIZE: Points = Points(16.0);

/// The spring that moves the handle: androidx passes `MotionSchemeKeyTokens.FastSpatial.value()` to the thumb (`Switch.kt`
/// line 188), converted as ADR 0004 says. Every field is written out, none taken from NSH's defaults: it starts from the
/// speed the handle already has, at the screen's own rate (ADR 0010).
fn handle_spring() -> Spring {
    let spec = SpringToken::FastSpatial.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// The handle's motion: its center and its scale, two animated values that the handle, the icon and the state layer are
/// bound to, and where each is going. A `Copy` handle, like the values it holds.
#[derive(Clone, Copy)]
struct Motion {
    /// The center's distance from the track's start, in points.
    center: AnimatedValue,
    /// The handle's scale, 1 for [`BASE_DIAMETER`].
    scale: AnimatedValue,
    /// Where the handle is going.
    target: Stored<Handle>,
}

impl Motion {
    /// A handle at rest at `handle`.
    fn new(handle: Handle) -> Self {
        Self {
            center: animated_value(handle.center),
            scale: animated_value(handle.scale()),
            target: stored(handle),
        }
    }

    /// The handle goes to `handle`: each value that has somewhere new to go springs there, from where it is and as fast as
    /// it moves, as androidx's `Animatable`s do when their target changes (`Switch.kt` lines 308-316); with reduce
    /// motion on, it goes there at once. Nothing is sent when the target stays the same.
    fn go_to(self, handle: Handle) {
        untracked(|| {
            let from = self.target.replace(handle);
            let reduced = reduce_motion();
            if from.center.to_bits() != handle.center.to_bits() {
                if reduced {
                    self.center.set(handle.center);
                } else {
                    let _ = self.center.spring(handle.center, handle_spring());
                }
            }
            if from.diameter.to_bits() != handle.diameter.to_bits() {
                if reduced {
                    self.scale.set(handle.scale());
                } else {
                    let _ = self.scale.spring(handle.scale(), handle_spring());
                }
            }
        });
    }
}

/// The four conditions a switch's colors depend on: checked or not, enabled or disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Condition {
    Unchecked,
    Checked,
    DisabledUnchecked,
    DisabledChecked,
}

impl Condition {
    fn of(checked: bool, disabled: bool) -> Self {
        match (checked, disabled) {
            (false, false) => Self::Unchecked,
            (true, false) => Self::Checked,
            (false, true) => Self::DisabledUnchecked,
            (true, true) => Self::DisabledChecked,
        }
    }

    fn is_disabled(self) -> bool {
        matches!(self, Self::DisabledUnchecked | Self::DisabledChecked)
    }
}

/// What a condition gives the switch's parts: androidx's `SwitchColors` for it (`SwitchDefaults.defaultSwitchColors`,
/// `Switch.kt` lines 426-469).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Colors {
    thumb: Tint,
    icon: Tint,
    track: Tint,
    /// The track's outline: none is `Color.Transparent`.
    outline: Option<Tint>,
}

/// The colors of `condition`.
fn colors(condition: Condition) -> Colors {
    match condition {
        Condition::Checked => Colors {
            thumb: Tint::solid(switch_tokens::SELECTED_HANDLE_COLOR),
            icon: Tint::solid(switch_tokens::SELECTED_ICON_COLOR),
            track: Tint::solid(switch_tokens::SELECTED_TRACK_COLOR),
            outline: None,
        },
        Condition::Unchecked => Colors {
            thumb: Tint::solid(switch_tokens::UNSELECTED_HANDLE_COLOR),
            icon: Tint::solid(switch_tokens::UNSELECTED_ICON_COLOR),
            track: Tint::solid(switch_tokens::UNSELECTED_TRACK_COLOR),
            // `uncheckedBorderColor = fromToken(SwitchTokens.UnselectedFocusTrackOutlineColor)` (lines 436-437): the focus
            // token, not `UnselectedTrackOutlineColor`, which names the same role.
            outline: Some(Tint::solid(
                switch_tokens::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR,
            )),
        },
        Condition::DisabledChecked => Colors {
            thumb: Tint::faded(
                switch_tokens::DISABLED_SELECTED_HANDLE_COLOR,
                switch_tokens::DISABLED_SELECTED_HANDLE_OPACITY,
            ),
            icon: Tint::faded(
                switch_tokens::DISABLED_SELECTED_ICON_COLOR,
                switch_tokens::DISABLED_SELECTED_ICON_OPACITY,
            ),
            track: Tint::faded(
                switch_tokens::DISABLED_SELECTED_TRACK_COLOR,
                switch_tokens::DISABLED_TRACK_OPACITY,
            ),
            outline: None,
        },
        Condition::DisabledUnchecked => Colors {
            thumb: Tint::faded(
                switch_tokens::DISABLED_UNSELECTED_HANDLE_COLOR,
                switch_tokens::DISABLED_UNSELECTED_HANDLE_OPACITY,
            ),
            icon: Tint::faded(
                switch_tokens::DISABLED_UNSELECTED_ICON_COLOR,
                switch_tokens::DISABLED_UNSELECTED_ICON_OPACITY,
            ),
            track: Tint::faded(
                switch_tokens::DISABLED_UNSELECTED_TRACK_COLOR,
                switch_tokens::DISABLED_TRACK_OPACITY,
            ),
            // `disabledUncheckedBorderColor` fades the outline color by `DisabledTrackOpacity`, as the track (lines
            // 460-463).
            outline: Some(Tint::faded(
                switch_tokens::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR,
                switch_tokens::DISABLED_TRACK_OPACITY,
            )),
        },
    }
}

/// The color of `tint` in `condition`. A disabled switch's colors are the token's color at its opacity composited over
/// the scheme's surface, so they are opaque: `.copy(alpha = …).compositeOver(surface)` (`Switch.kt` lines 426-467).
/// Composited in each appearance, over that appearance's surface.
fn paint(tint: Option<Tint>, condition: Condition) -> AdaptiveColor {
    match tint {
        None => AdaptiveColor::TRANSPARENT,
        Some(tint) if condition.is_disabled() => {
            let surface = role_color(ColorRole::Surface);
            let color = tint_color(Some(tint));
            AdaptiveColor::new(
                surface.light.blend(color.light),
                surface.dark.blend(color.dark),
            )
        }
        Some(tint) => tint_color(Some(tint)),
    }
}

/// The outline's width, which is the track's border: `TrackOutlineWidth` when the track has an outline, none otherwise.
///
/// androidx draws a transparent outline of the same width when the switch is checked (`Modifier.border(TrackOutlineWidth,
/// borderColor, trackShape)`, line 176), over a background that fills the whole track; the same pixels, with no
/// border for a platform to leave unpainted.
fn outline_width(condition: Condition) -> Points {
    if colors(condition).outline.is_some() {
        switch_tokens::TRACK_OUTLINE_WIDTH
    } else {
        Points(0.0)
    }
}

/// How far presses also count around the track, so that a switch smaller than [`MIN_TOUCH_TARGET`] on a side still has
/// a 48-point target there (androidx `minimumInteractiveComponentSize`, line 139), without changing its M3 size or the
/// layout (NSH's accessibility guide: extend the touch area with the hit slop, not the size).
fn touch_slop() -> EdgeInsets {
    // Half of what is missing on each side, before and after.
    let missing = |side: Points| px(((MIN_TOUCH_TARGET.0 - side.0) / 2.0).max(0.0));
    let (horizontal, vertical) = (
        missing(switch_tokens::TRACK_WIDTH),
        missing(switch_tokens::TRACK_HEIGHT),
    );
    EdgeInsets {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

/// The props of a [`Switch`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SwitchProps {
    /// On or off. The app owns the value: a press reports the next one with [`SwitchEvent::Change`], and the switch
    /// shows the value the app passes back.
    #[prop(default)]
    pub checked: Prop<bool>,
    /// What screen readers say: a switch shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// The icon in the handle (androidx's `thumbContent`), none by default. Give one that depends on `checked`, as
    /// androidx's samples do, for the usual check mark and cross.
    #[prop(default)]
    pub icon: Prop<Option<Icon>>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`Switch`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SwitchEvent {
    /// The switch was pressed (a tap, a click, Space or Return): the value it asks for, the opposite of `checked`.
    #[args(checked)]
    Change(bool),
}

/// A Material 3 Expressive switch: a track and a handle that springs to the end for on and back for off.
///
/// It has the behavior of `nsh-base`'s switch (presses, keyboard activation, focus, the disabled state, the switch role
/// and its value for screen readers, named by its label) and the look of androidx's `Switch`. Hover, focus and press
/// lay a color over the handle at the state layer opacities of `StateTokens`, as androidx's ripple does, and change
/// nothing else, so nothing is laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::switch::switch;
///
/// fn wifi() -> impl IntoElement {
///     let on = signal(false);
///     switch()
///         .label("Wi-Fi")
///         .checked(move || on.get())
///         .on_change(move |checked: &bool| on.set(*checked))
/// }
/// ```
#[component]
pub fn Switch(props: SwitchProps, cx: Scope<SwitchEvent>) -> impl IntoElement {
    let condition = move || Condition::of(props.checked.get(), props.disabled.get());
    let handle = move || {
        Handle::at(
            End::of(props.checked.get()),
            Content::of(props.icon.with(Option::is_some)),
        )
    };
    let motion = untracked(|| Motion::new(handle()));
    effect(move || motion.go_to(handle()));

    // The handle's view is `BASE_DIAMETER` across, and where it sits when its center is `center`.
    let half_base = BASE_DIAMETER.0 / 2.0;
    let track_top = switch_tokens::TRACK_HEIGHT.0 / 2.0;
    nsh_base::switch()
        .checked(props.checked)
        .disabled(props.disabled)
        .accessibility_label(move || Some(props.label.get()))
        .on_change(move |checked: &bool| cx.emit(SwitchEvent::Change(*checked)))
        .hit_slop(touch_slop())
        .group()
        .w(switch_tokens::TRACK_WIDTH)
        .h(switch_tokens::TRACK_HEIGHT)
        .track_style(move |s| {
            s.w(switch_tokens::TRACK_WIDTH)
                .h(switch_tokens::TRACK_HEIGHT)
                .rounded_full()
                .bg(move || paint(Some(colors(condition()).track), condition()))
                .border(move || outline_width(condition()))
                .border_color(move || paint(colors(condition()).outline, condition()))
        })
        // Its handle is the view below: this one is not laid out.
        .thumb_style(Styled::hidden)
        .child(
            // The handle.
            view()
                .absolute()
                .left(px(0.0))
                .top(px(track_top - half_base))
                .size(BASE_DIAMETER)
                .rounded_full()
                .bg(move || paint(Some(colors(condition()).thumb), condition()))
                .bind_translate_x(&motion.center.add(-half_base))
                .bind_scale(&motion.scale),
        )
        .child(
            // The icon, centered on the handle at its own size.
            canvas(move || match props.icon.get() {
                Some(icon) => icon.drawing(ICON_SIZE),
                None => Painter::new(),
            })
            .absolute()
            .left(px(0.0))
            .top(px(track_top - ICON_SIZE.0 / 2.0))
            .w(ICON_SIZE)
            .h(ICON_SIZE)
            .fill_color(move || paint(Some(colors(condition()).icon), condition()))
            .bind_translate_x(&motion.center.add(-ICON_SIZE.0 / 2.0))
            .style_with(move |s| {
                if props.icon.with(Option::is_none) {
                    s.hidden()
                } else {
                    s
                }
            }),
        )
        .child(
            // The state layer, over the handle: androidx's unbounded ripple, 40 points across, centered on the handle,
            // shown by the switch's own states with nothing running.
            view()
                .absolute()
                .left(px(0.0))
                .top(px(track_top - switch_tokens::STATE_LAYER_SIZE.0 / 2.0))
                .size(switch_tokens::STATE_LAYER_SIZE)
                .rounded_full()
                .bg(role_color(ColorRole::OnSurface))
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .bind_translate_x(&motion.center.add(-switch_tokens::STATE_LAYER_SIZE.0 / 2.0))
                .style_with(move |s| {
                    // A disabled switch takes no press and shows no state.
                    if props.disabled.get() { s.hidden() } else { s }
                }),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{AnimationOp, AnimationProp, NodeId as ValueId, Operand};
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Checked, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, spring_position, started_springs, worst_gap,
    };
    use crate::color::{ColorRole, ColorScheme};
    use crate::material_symbols::outlined::{CHECK, CLOSE};
    use crate::morph_probe::{parameters, replay, set_values, tolerance};

    /// What the probe places: one switch, lines reporting what it reports, and lines that change what the app
    /// passes it.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        checked: Prop<bool>,
        #[prop(default)]
        icon: Prop<Icons>,
        #[prop(default)]
        disabled: Prop<bool>,
        #[prop(default)]
        app: Prop<App>,
    }

    /// Whether the probe's switch has an icon in its handle.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
    enum Icons {
        #[default]
        Without,
        With,
    }

    impl Icons {
        fn of(icon: bool) -> Self {
            if icon { Self::With } else { Self::Without }
        }
    }

    /// What the probe's app does with the value a press asks for.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
    enum App {
        /// It takes the value: the usual controlled switch.
        #[default]
        Takes,
        /// It keeps the value it has: a change is reported and not taken.
        Keeps,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let on = signal(props.checked.get_untracked());
        let icon = signal(props.icon.get_untracked() == Icons::With);
        let disabled = signal(props.disabled.get_untracked());
        let refuse = props.app.get_untracked() == App::Keeps;
        let changes = signal(0_u32);
        let reported = signal(String::from("none"));
        // A row, so that the switch keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    switch()
                        .label("Wi-Fi")
                        .checked(move || on.get())
                        .icon(move || icon.get().then(|| if on.get() { CHECK } else { CLOSE }))
                        .disabled(move || disabled.get())
                        .on_change(move |checked: &bool| {
                            changes.update(|n| *n += 1);
                            reported.set(format!("{checked}"));
                            if !refuse {
                                on.set(*checked);
                            }
                        }),
                ),
            )
            .child(text(move || format!("changes: {}", changes.get())))
            .child(text(move || format!("reported: {}", reported.get())))
            .child(text("Icon on").on_press(move |(): &()| icon.set(true)))
            .child(text("Icon off").on_press(move |(): &()| icon.set(false)))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Turn on").on_press(move |(): &()| on.set(true)))
            .child(text("Turn off").on_press(move |(): &()| on.set(false)))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        checked: bool,
        icon: Icons,
        disabled: bool,
        app: App,
        dark: Appearance,
        reduce_motion: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            checked: Prop::new(config.checked),
            icon: Prop::new(config.icon),
            disabled: Prop::new(config.disabled),
            app: Prop::new(config.app),
        });
        // The web has a mouse to hover with, whatever its idiom, and a keyboard that tabs to views.
        let environment = Environment {
            os: Os::Web,
            color_scheme: config.dark,
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

    /// The switch's views, and the last style, frame, drawing and animation bindings each was sent.
    struct Mounted {
        /// The press target, with the switch role: the whole switch.
        root: NodeId,
        /// `nsh-base`'s track part, which this module styles as the track.
        track: NodeId,
        /// `nsh-base`'s handle part, left empty and hidden.
        thumb: NodeId,
        /// The handle's view.
        handle: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        /// The state layer.
        layer: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        drawings: HashMap<NodeId, Arc<Drawing>>,
        bindings: HashMap<NodeId, Vec<(AnimationProp, ValueId)>>,
        /// What the animated values are made of: each math node's left operand and constant.
        sums: HashMap<ValueId, (ValueId, f32)>,
        label: Option<Arc<str>>,
        disabled: Option<bool>,
        checked: Option<Checked>,
        hit_slop: Option<EdgeInsets>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut root = None;
            let mut label = None;
            let mut disabled = None;
            let mut checked = None;
            let mut hit_slop = None;
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut drawings = HashMap::new();
            let mut bindings = HashMap::new();
            let mut sums = HashMap::new();
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
                        if props.accessibility.role == AccessibilityRole::Switch =>
                    {
                        root = Some(*id);
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
                    MountItem::AnimationBindings { id, bindings: set } => {
                        bindings.insert(*id, set.clone());
                    }
                    MountItem::Animation(AnimationOp::CreateMath {
                        node,
                        a: Operand::Node(a),
                        b: Operand::Const(b),
                        ..
                    }) => {
                        sums.insert(*node, (*a, *b));
                    }
                    _ => {}
                }
            }
            let root = root.expect("a view with the switch role");
            let ids = children.get(&root).cloned().unwrap_or_default();
            // The track first, under the handle, under the icon, under the state layer; and nothing else.
            let [track, handle, icon, layer] = ids[..] else {
                panic!("four views in the switch: {items:?}");
            };
            let kind = |id| kinds.get(&id).copied();
            assert_eq!(
                [kind(track), kind(handle), kind(icon), kind(layer)],
                [
                    Some(ViewKind::View),
                    Some(ViewKind::View),
                    Some(ViewKind::Canvas),
                    Some(ViewKind::View)
                ],
                "{items:?}"
            );
            let [thumb] = children.get(&track).cloned().unwrap_or_default()[..] else {
                panic!("the track holds nsh-base's handle part and nothing else: {items:?}");
            };
            Self {
                root,
                track,
                thumb,
                handle,
                icon,
                layer,
                styles,
                frames,
                drawings,
                bindings,
                sums,
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

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }

        /// The switch's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frame(self.root).size
        }

        /// How many of `items` lay one of the switch's views out again.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [
                self.root,
                self.track,
                self.thumb,
                self.handle,
                self.icon,
                self.layer,
            ];
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Frame { id, .. } if ours.contains(id)))
                .count()
        }

        /// The animated value that binds `prop` of view `id`.
        fn bound(&self, id: NodeId, prop: AnimationProp) -> Option<ValueId> {
            self.bindings
                .get(&id)?
                .iter()
                .find(|(bound, _)| *bound == prop)
                .map(|(_, node)| *node)
        }

        /// How far `id` is moved along the track now (its translation) in `replay`; none if it is not.
        fn translation(&self, id: NodeId, replay: &Replay) -> f32 {
            self.bound(id, AnimationProp::Transform(0))
                .map_or(0.0, |node| replay.value(node))
        }

        /// The center of `id` along the track now: where its frame is, and its translation.
        fn center_of(&self, id: NodeId, replay: &Replay) -> f32 {
            let frame = self.frame(id);
            frame.origin.x.0 + frame.size.width.0 / 2.0 + self.translation(id, replay)
        }

        /// The handle's center along the track and its diameter now in `replay`.
        fn handle(&self, replay: &Replay) -> (f32, f32) {
            let scale = self
                .bound(self.handle, AnimationProp::Transform(1))
                .map_or(1.0, |node| replay.value(node));
            (
                self.center_of(self.handle, replay),
                self.frame(self.handle).size.width.0 * scale,
            )
        }

        /// The animated values the handle moves with: its center's and its scale's.
        fn values(&self) -> (ValueId, ValueId) {
            let translate = self
                .bound(self.handle, AnimationProp::Transform(0))
                .expect("the handle's translation is bound");
            let scale = self
                .bound(self.handle, AnimationProp::Transform(1))
                .expect("the handle's scale is bound");
            let (center, _) = self.sums[&translate];
            (center, scale)
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

    fn scheme_hex(role: ColorRole, dark: bool) -> u32 {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        scheme.get(role).to_hex()
    }

    /// `role` at `percent` % opacity composited over the scheme's surface: androidx's `.copy(alpha = …)
    /// .compositeOver(surface)`, rounded to a whole 8-bit channel. It is opaque.
    fn over_surface(role: ColorRole, percent: u32, dark: bool) -> Color {
        let surface = scheme_hex(ColorRole::Surface, dark);
        let top = scheme_hex(role, dark);
        let channel = |shift: u32| {
            let (below, above) = ((surface >> shift) & 0xff, (top >> shift) & 0xff);
            (below * (100 - percent) + above * percent + 50) / 100
        };
        color((channel(16) << 16) | (channel(8) << 8) | channel(0), 255)
    }

    /// Whether `actual` is `expected`, each of red, green and blue within `tolerance`, and the alpha the same.
    fn close(actual: Color, expected: Color, tolerance: u32) -> bool {
        let channels = |c: Color| [(c.0 >> 24) & 0xff, (c.0 >> 16) & 0xff, (c.0 >> 8) & 0xff];
        actual.0 & 0xff == expected.0 & 0xff
            && channels(actual)
                .into_iter()
                .zip(channels(expected))
                .all(|(a, b)| a.abs_diff(b) <= tolerance)
    }

    /// The color `ops` fill with, if any.
    fn fill(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    /// The fills `ops` make, to compare two drawings of one path.
    fn fills(ops: &[CanvasOp]) -> Vec<CanvasOp> {
        ops.iter()
            .filter(|op| matches!(op, CanvasOp::Fill(..)))
            .cloned()
            .collect()
    }

    /// What the switch's parts should look like, written from androidx's `SwitchDefaults.colors` and
    /// `SwitchTokens`, not from this module's tables: the handle, the icon, the track, the outline's width and color.
    struct Expected {
        handle: Color,
        icon: Color,
        track: Color,
        outline_width: f32,
        outline: Color,
        /// Each channel may differ by this much: composited colors are rounded.
        tolerance: u32,
    }

    fn expected(checked: bool, disabled: bool, dark: bool) -> Expected {
        let solid = |role| color(scheme_hex(role, dark), 255);
        let none = Color(0);
        match (checked, disabled) {
            // `SwitchTokens`: the selected handle is on primary, the track primary, the icon on primary container;
            // `checkedBorderColor = Color.Transparent` (`Switch.kt` line 432).
            (true, false) => Expected {
                handle: solid(ColorRole::OnPrimary),
                icon: solid(ColorRole::OnPrimaryContainer),
                track: solid(ColorRole::Primary),
                outline_width: 0.0,
                outline: none,
                tolerance: 0,
            },
            // The unselected handle is outline, the track and the icon surface container highest, and the outline
            // is outline (`UnselectedFocusTrackOutlineColor`, `Switch.kt` lines 436-437).
            (false, false) => Expected {
                handle: solid(ColorRole::Outline),
                icon: solid(ColorRole::SurfaceContainerHighest),
                track: solid(ColorRole::SurfaceContainerHighest),
                outline_width: 2.0,
                outline: solid(ColorRole::Outline),
                tolerance: 0,
            },
            // `SwitchTokens.kt` lines 27-35: disabled selected handle surface at 100 %, icon on surface at 38 %,
            // track on surface at 12 %; all over the surface (`Switch.kt` lines 440-452).
            (true, true) => Expected {
                handle: over_surface(ColorRole::Surface, 100, dark),
                icon: over_surface(ColorRole::OnSurface, 38, dark),
                track: over_surface(ColorRole::OnSurface, 12, dark),
                outline_width: 0.0,
                outline: none,
                tolerance: 1,
            },
            // Disabled unselected: handle on surface at 38 %, icon and track surface container highest at 38 % and
            // 12 %, outline on surface at 12 % (`SwitchTokens.kt` lines 35-43, `Switch.kt` lines 453-467).
            (false, true) => Expected {
                handle: over_surface(ColorRole::OnSurface, 38, dark),
                icon: over_surface(ColorRole::SurfaceContainerHighest, 38, dark),
                track: over_surface(ColorRole::SurfaceContainerHighest, 12, dark),
                outline_width: 2.0,
                outline: over_surface(ColorRole::OnSurface, 12, dark),
                tolerance: 1,
            },
        }
    }

    #[test]
    fn each_condition_has_its_colors_light_and_dark() {
        for checked in [false, true] {
            for disabled in [false, true] {
                for dark in [false, true] {
                    let what = format!("checked: {checked} disabled: {disabled} dark: {dark}");
                    let mounted = mount(Config {
                        checked,
                        disabled,
                        dark: if dark {
                            Appearance::Dark
                        } else {
                            Appearance::Light
                        },
                        icon: Icons::With,
                        ..Config::default()
                    });
                    let want = expected(checked, disabled, dark);
                    let tolerance = want.tolerance;
                    let handle = mounted.style(mounted.handle).background_color;
                    assert!(
                        close(handle, want.handle, tolerance),
                        "{what}: handle {handle:?}"
                    );
                    let track = mounted.style(mounted.track);
                    assert!(
                        close(track.background_color, want.track, tolerance),
                        "{what}: track {:?}",
                        track.background_color
                    );
                    assert_eq!(track.borders.widths.top.0, want.outline_width, "{what}");
                    assert_eq!(track.borders.widths.left.0, want.outline_width, "{what}");
                    assert!(
                        close(track.borders.colors[0], want.outline, tolerance),
                        "{what}: outline {:?}",
                        track.borders.colors[0]
                    );
                    let icon = fill(&mounted.drawings[&mounted.icon].ops).expect("the icon's fill");
                    assert!(close(icon, want.icon, tolerance), "{what}: icon {icon:?}");
                    // A disabled switch's colors are opaque: none is the token color made transparent.
                    if disabled {
                        for color in [handle, track.background_color, icon] {
                            assert_eq!(color.0 & 0xff, 255, "{what}: {color:?}");
                        }
                    }
                    // The switch itself and the handle part of `nsh-base` paint nothing.
                    for id in [mounted.root, mounted.thumb] {
                        let style = mounted.style(id);
                        assert_eq!(style.background_color, Color::TRANSPARENT, "{what}");
                        assert_eq!(style.borders.widths.top, px(0.0), "{what}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_disabled_switch_is_not_the_token_color_at_its_opacity() {
        // `Switch.kt` composites over the surface: the unchecked handle is on surface at 38 % on the surface, which
        // is not the color the token names with an alpha, as most tokens' disabled states are.
        let mounted = mount(Config {
            disabled: true,
            ..Config::default()
        });
        let handle = mounted.style(mounted.handle).background_color;
        let translucent = color(scheme_hex(ColorRole::OnSurface, false), 97);
        assert_ne!(handle, translucent);
        assert!(!close(handle, translucent, 1));
        assert_eq!(handle.0 & 0xff, 255);
    }

    #[test]
    fn the_track_the_handle_the_icon_and_the_state_layer_have_the_token_sizes_and_shapes() {
        let mounted = mount(Config {
            icon: Icons::With,
            ..Config::default()
        });
        // `SwitchTokens.TrackWidth` and `TrackHeight`: 52 by 32 points, as the switch and as its track.
        assert_eq!(mounted.size(), Size::new(Points(52.0), Points(32.0)));
        assert_eq!(mounted.frame(mounted.track).size, mounted.size());
        assert_eq!(mounted.frame(mounted.track).origin, Point::default());
        // The handle is `SelectedHandleWidth` across, centered on the track's height; the icon `IconSize`; the state
        // layer `StateLayerSize`, which is taller than the track and so starts above it.
        let handle = mounted.frame(mounted.handle);
        assert_eq!(handle.size, Size::new(Points(24.0), Points(24.0)));
        assert_eq!(handle.origin.y, Points(4.0));
        let icon = mounted.frame(mounted.icon);
        assert_eq!(icon.size, Size::new(Points(16.0), Points(16.0)));
        assert_eq!(icon.origin.y, Points(8.0));
        let layer = mounted.frame(mounted.layer);
        assert_eq!(layer.size, Size::new(Points(40.0), Points(40.0)));
        assert_eq!(layer.origin.y, Points(-4.0));
        // Every shape is `CornerFull`: a radius of at least half the side.
        for (id, side) in [
            (mounted.track, 32.0),
            (mounted.handle, 24.0),
            (mounted.layer, 40.0),
        ] {
            assert!(mounted.style(id).border_radius.0 >= side / 2.0, "{id}");
        }
        // The handle part of `nsh-base` is not laid out and not shown.
        assert_eq!(mounted.frame(mounted.thumb).size, Size::default());
    }

    #[test]
    fn the_handle_rests_where_and_as_big_as_androidx_puts_it() {
        // (checked, icon, left edge, diameter): `Switch.kt` lines 283-304 and 661-671. Unchecked, the left edge is
        // `(SwitchHeight - size) / 2`, 8 for the 16-point handle and 4 for the 24-point one with an icon; checked, it
        // is `(SwitchWidth - ThumbDiameter) - ThumbPadding` = (52 - 24) - 4 = 24, and the size is 24 either way.
        for (checked, icon, left, diameter) in [
            (false, false, 8.0, 16.0),
            (false, true, 4.0, 24.0),
            (true, false, 24.0, 24.0),
            (true, true, 24.0, 24.0),
        ] {
            let what = format!("checked: {checked} icon: {icon}");
            let (_app, recording) = start(Config {
                checked,
                icon: Icons::of(icon),
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let replay = replay(&recording);
            let (center, size) = mounted.handle(&replay);
            assert!((size - diameter).abs() < 1e-4, "{what}: {size}");
            assert!(
                (center - size / 2.0 - left).abs() < 1e-4,
                "{what}: {center}"
            );
            // The icon and the state layer are centered on the handle, which is centered on the track's height.
            // Without an icon its canvas is not laid out, and has no center.
            let mut centered = vec![mounted.handle, mounted.layer];
            if icon {
                centered.push(mounted.icon);
            }
            for id in centered {
                assert!(
                    (mounted.center_of(id, &replay) - center).abs() < 1e-4,
                    "{what}: {id}"
                );
                let frame = mounted.frame(id);
                assert_eq!(frame.origin.y.0 + frame.size.height.0 / 2.0, 16.0, "{what}");
            }
        }
    }

    #[test]
    fn the_handle_geometry_is_the_arithmetic_of_switch_kt() {
        assert_eq!(thumb_padding(), 4.0);
        let handle = |checked, icon| Handle::at(End::of(checked), Content::of(icon));
        assert_eq!(
            handle(false, false),
            Handle {
                center: 16.0,
                diameter: 16.0
            }
        );
        assert_eq!(
            handle(false, true),
            Handle {
                center: 16.0,
                diameter: 24.0
            }
        );
        assert_eq!(
            handle(true, false),
            Handle {
                center: 36.0,
                diameter: 24.0
            }
        );
        assert_eq!(
            handle(true, true),
            Handle {
                center: 36.0,
                diameter: 24.0
            }
        );
        // The handle's view is the largest size, so the scale never exceeds 1.
        assert_eq!(handle(true, false).scale(), 1.0);
        assert!((handle(false, false).scale() - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn the_icon_is_its_path_at_16_points_in_its_color_and_follows_the_state() {
        let (mut app, recording) = start(Config {
            icon: Icons::With,
            ..Config::default()
        });
        // Unchecked, the probe gives a cross; the path is the one at the token size, 16 points, 960-point view box
        // scaled to it and moved down by its height.
        let mounted = Mounted::of(&recording.items());
        let ops = &mounted.drawings[&mounted.icon].ops;
        let scale = 16.0 / 960.0;
        let constant = |n: &Num| match n {
            Num::Const(v) => Some(*v),
            Num::Slot(_) => None,
        };
        assert!(
            ops.iter().any(|op| matches!(op, CanvasOp::Scale(x, y)
                if constant(x) == Some(scale) && constant(y) == Some(scale))),
            "{ops:?}"
        );
        let cross = CLOSE.drawing(Points(16.0)).finish();
        assert_eq!(fills(ops), fills(&cross.ops));
        assert_eq!(fills(ops).len(), 1);
        // The icon does not scale with the handle: it is moved along the track and nothing else.
        assert!(
            mounted
                .bound(mounted.icon, AnimationProp::Transform(0))
                .is_some()
        );
        assert!(
            mounted
                .bound(mounted.icon, AnimationProp::Transform(1))
                .is_none()
        );
        assert!(
            mounted
                .bound(mounted.layer, AnimationProp::Transform(1))
                .is_none()
        );
        // Checked, it is the check mark.
        app.press("label:Wi-Fi").unwrap();
        let mounted = Mounted::of(&recording.items());
        let mark = CHECK.drawing(Points(16.0)).finish();
        assert_eq!(
            fills(&mounted.drawings[&mounted.icon].ops),
            fills(&mark.ops)
        );
        assert_ne!(fills(&mark.ops), fills(&cross.ops));
    }

    #[test]
    fn without_an_icon_nothing_is_drawn_or_laid_out_for_it() {
        let mounted = mount(Config::default());
        assert!(
            mounted
                .drawings
                .get(&mounted.icon)
                .is_none_or(|drawing| fills(&drawing.ops).is_empty())
        );
        assert_eq!(mounted.frame(mounted.icon).size, Size::default());
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_switch_with_its_value() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.label.as_deref(), Some("Wi-Fi"));
        assert_eq!(mounted.disabled, Some(false));
        assert_eq!(mounted.checked, Some(Checked::False));
        assert_eq!(
            mount(Config {
                checked: true,
                ..Config::default()
            })
            .checked,
            Some(Checked::True)
        );
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
    fn it_has_a_48_point_touch_target_without_growing() {
        let mounted = mount(Config::default());
        let slop = mounted.hit_slop.expect("the switch's props");
        // Each side reaches 48 points with the slop, and a side already 48 or more gets none: the track is 52 wide
        // and 32 high.
        let reach = |side: f32| (48.0 - side).max(0.0) / 2.0;
        assert_eq!(slop.left.0, reach(52.0));
        assert_eq!(slop.right.0, reach(52.0));
        assert_eq!(slop.top.0, reach(32.0));
        assert_eq!(slop.bottom.0, reach(32.0));
        assert_eq!(slop.top.0, 8.0);
        assert_eq!(slop.left.0, 0.0);
        assert!(52.0 + slop.left.0 + slop.right.0 >= 48.0);
        assert!(32.0 + slop.top.0 + slop.bottom.0 >= 48.0);
        assert_eq!(mounted.size(), Size::new(Points(52.0), Points(32.0)));
    }

    #[test]
    fn a_press_reports_the_value_it_asks_for_once_and_the_app_owns_the_value() -> Result<(), String>
    {
        let (mut app, recording) = start(Config::default());
        app.press("label:Wi-Fi")?;
        assert!(app.shows("changes: 1"));
        assert!(!app.shows("changes: 2"));
        assert!(app.shows("reported: true"));
        assert_eq!(Mounted::of(&recording.items()).checked, Some(Checked::True));
        app.press("label:Wi-Fi")?;
        assert!(app.shows("changes: 2"));
        assert!(app.shows("reported: false"));
        assert_eq!(
            Mounted::of(&recording.items()).checked,
            Some(Checked::False)
        );

        // An app that keeps its value is obeyed: the press is reported, and the switch neither says it is on nor
        // moves.
        let (mut app, recording) = start(Config {
            app: App::Keeps,
            ..Config::default()
        });
        app.press("label:Wi-Fi")?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("reported: true"));
        let items = recording.in_step("press(label:Wi-Fi)");
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 0);
        assert_eq!(
            Mounted::of(&recording.items()).checked,
            Some(Checked::False)
        );

        // A value the app sets is not a change the switch reports.
        let (mut app, recording) = start(Config::default());
        app.press("Turn on")?;
        assert!(app.shows("changes: 0"));
        assert_eq!(Mounted::of(&recording.items()).checked, Some(Checked::True));
        Ok(())
    }

    #[test]
    fn space_and_return_press_a_focused_switch() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press_key("tab")?;
        app.press_key("space")?;
        assert!(app.shows("changes: 1"));
        assert!(app.shows("reported: true"));
        app.press_key("enter")?;
        assert!(app.shows("changes: 2"));
        assert!(app.shows("reported: false"));
        Ok(())
    }

    #[test]
    fn a_disabled_switch_takes_no_press_shows_no_state_and_stays_where_it_is() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Wi-Fi");
        let _ = app.hover("label:Wi-Fi");
        assert!(app.shows("changes: 0"));
        let items = recording.items();
        assert_eq!(started_springs(&items), Vec::new());
        let mounted = Mounted::of(&items);
        assert_eq!(mounted.checked, Some(Checked::False));
        // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
        assert_eq!(mounted.frame(mounted.layer).size, Size::default());
        // The switch keeps its size and its track its own.
        assert_eq!(mounted.size(), Size::new(Points(52.0), Points(32.0)));
        assert_eq!(mounted.frame(mounted.track).size, mounted.size());
        // Enabled, its state layer is there.
        let enabled = mount(Config::default());
        assert_eq!(
            enabled.frame(enabled.layer).size,
            Size::new(Points(40.0), Points(40.0))
        );
    }

    #[test]
    fn a_switch_disabled_while_on_keeps_its_place_and_takes_the_disabled_colors() {
        let (mut app, recording) = start(Config {
            checked: true,
            ..Config::default()
        });
        let before = recording.items().len();
        app.press("Disable").unwrap();
        let items = recording.items()[before..].to_vec();
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 0);
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.disabled, Some(true));
        assert_eq!(mounted.checked, Some(Checked::True));
        let want = expected(true, true, false);
        let track = mounted.style(mounted.track).background_color;
        assert!(close(track, want.track, want.tolerance), "{track:?}");
        let (center, diameter) = mounted.handle(&replay(&recording));
        assert_eq!((center, diameter), (36.0, 24.0));
    }

    /// The opacity of the state layer in each style it was sent in `items`.
    fn layer_opacities(mounted: &Mounted, items: &[MountItem]) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    /// Whether `items` are the state layer's styles and nothing else: no other view's style, no frame, no drawing, no
    /// animation.
    fn only_the_layer(mounted: &Mounted, items: &[MountItem]) -> bool {
        items
            .iter()
            .all(|item| matches!(item, MountItem::ViewStyle { id, .. } if *id == mounted.layer))
    }

    /// Reports `states` (bits of `nsh_core::tailwind::state`) of view `id` in the step `name`, as a platform reports
    /// them when a finger lands on the view or leaves it: headless input hovers and focuses but never presses (its
    /// mouse "never presses here"), so the pressed state is reported to the engine as a platform would.
    fn report_states(app: &mut Headless, name: &str, id: NodeId, states: u16) {
        app.step(name);
        app.engine().control().set_states(id, states);
        app.next_change(Duration::from_secs(5)).unwrap();
        app.catch_up();
    }

    /// The pressed state of the switch while a finger is down on it: pressed, and so hovered.
    const FINGER_DOWN: u16 =
        nsh_core::tailwind::state::PRESSED | nsh_core::tailwind::state::HOVERED;

    #[test]
    fn hover_focus_and_press_show_the_state_layer_and_lay_out_nothing() -> Result<(), String> {
        for checked in [false, true] {
            let what = format!("checked: {checked}");
            let config = Config {
                checked,
                ..Config::default()
            };
            let (mut app, recording) = start(config);
            let mounted = Mounted::of(&recording.items());
            // At rest the state layer is there and clear, in the content color of a surface.
            assert_eq!(mounted.style(mounted.layer).opacity, 0.0, "{what}");
            assert_eq!(
                mounted.style(mounted.layer).background_color,
                color(scheme_hex(ColorRole::OnSurface, false), 255),
                "{what}"
            );

            // Each state sets the layer's opacity (`StateTokens.kt` lines 23-25) and changes nothing else.
            app.hover("label:Wi-Fi")?;
            let hover = recording.in_step("hover(label:Wi-Fi)");
            assert_eq!(layer_opacities(&mounted, &hover), [0.08], "{what}");
            assert!(only_the_layer(&mounted, &hover), "{what}: {hover:?}");
            // It goes away when the pointer does.
            app.hover("Icon on")?;
            let away = recording.in_step("hover(Icon on)");
            assert_eq!(layer_opacities(&mounted, &away), [0.0], "{what}");
            assert!(only_the_layer(&mounted, &away), "{what}: {away:?}");

            let (mut app, recording) = start(config);
            app.press_key("tab")?;
            let focus = recording.in_step("keyboard.press(tab)");
            assert_eq!(layer_opacities(&mounted, &focus), [0.1], "{what}");
            assert!(only_the_layer(&mounted, &focus), "{what}: {focus:?}");

            // A finger down: the layer shows it; the handle does not grow and nothing springs (the pressed handle is
            // left out, the module's docs say why); the press itself, and so the change, comes when it lifts.
            let (mut app, recording) = start(config);
            report_states(&mut app, "down", mounted.root, FINGER_DOWN);
            let down = recording.in_step("down");
            assert_eq!(layer_opacities(&mounted, &down), [0.1], "{what}");
            assert!(only_the_layer(&mounted, &down), "{what}: {down:?}");
            assert!(app.shows("changes: 0"), "{what}");
            let (_, diameter) = mounted.handle(&replay(&recording));
            assert_eq!(diameter, if checked { 24.0 } else { 16.0 }, "{what}");
            report_states(&mut app, "up", mounted.root, 0);
            let up = recording.in_step("up");
            assert_eq!(layer_opacities(&mounted, &up), [0.0], "{what}");
            assert!(only_the_layer(&mounted, &up), "{what}: {up:?}");
        }
        Ok(())
    }

    #[test]
    fn hover_focus_and_press_do_not_recolor_the_handle_the_track_or_the_icon() -> Result<(), String>
    {
        let (mut app, recording) = start(Config {
            icon: Icons::With,
            ..Config::default()
        });
        let before = Mounted::of(&recording.items());
        let colors = |mounted: &Mounted| {
            (
                mounted.style(mounted.handle).background_color,
                mounted.style(mounted.track).background_color,
                fill(&mounted.drawings[&mounted.icon].ops),
            )
        };
        let rest = colors(&before);
        let n = recording.items().len();
        app.hover("label:Wi-Fi")?;
        app.press_key("tab")?;
        report_states(&mut app, "down", before.root, FINGER_DOWN);
        let items = since(&recording, n);
        // Neither the handle's, the track's nor the icon's style or drawing is sent again: they stay as they are.
        assert!(
            !items.iter().any(|item| match item {
                MountItem::ViewStyle { id, .. } =>
                    [before.handle, before.track, before.icon].contains(id),
                MountItem::Canvas { .. } => true,
                _ => false,
            }),
            "{items:?}"
        );
        assert_eq!(colors(&Mounted::of(&recording.items())), rest);
        // The token file has such colors, different from the rest ones, and `Switch.kt` reads none of them
        // (ADR 0012: the component wins).
        assert_ne!(
            switch_tokens::SELECTED_HOVER_HANDLE_COLOR,
            switch_tokens::SELECTED_HANDLE_COLOR
        );
        assert_ne!(
            switch_tokens::UNSELECTED_PRESSED_HANDLE_COLOR,
            switch_tokens::UNSELECTED_HANDLE_COLOR
        );
        for state in ["Hover", "Focus", "Pressed"] {
            for part in ["Handle", "Track", "Icon"] {
                let name = format!("{state}{part}Color");
                assert!(!SWITCH_KT.contains(&name), "{name}");
            }
        }
        Ok(())
    }

    /// The springs `items` start, for the handle's center and its scale, as `mounted` binds them. Each is a spring of
    /// androidx's `FastSpatial`, converted as ADR 0004 says, from the speed the handle has, at the screen's rate;
    /// nothing else is started and nothing is set.
    fn handle_springs(
        mounted: &Mounted,
        items: &[MountItem],
        step: &str,
    ) -> (Option<StartedSpring>, Option<StartedSpring>) {
        let (center, scale) = mounted.values();
        let springs = started_springs(items);
        let on = |node: ValueId| springs.iter().copied().find(|spring| spring.node == node);
        assert_eq!(
            springs.len(),
            usize::from(on(center).is_some()) + usize::from(on(scale).is_some()),
            "{step}: {springs:?}"
        );
        for spring in &springs {
            // `ExpressiveMotionTokens.kt` lines 26-27: damping ratio 0.6, stiffness 800; the coefficient is
            // `2 · ζ · √(k · m)`, with the mass of 1 that NSH's springs take.
            assert_eq!(spring.stiffness, 800.0, "{step}");
            assert!(
                (spring.damping - 2.0 * 0.6 * 800.0_f32.sqrt()).abs() < 1e-4,
                "{step}: {spring:?}"
            );
            assert_eq!(spring.mass, 1.0, "{step}");
            assert_eq!(spring.velocity, None, "{step}");
            assert_eq!(spring.frame_rate, None, "{step}");
        }
        assert_eq!(set_values(items), 0, "{step}: {items:?}");
        (on(center), on(scale))
    }

    /// The items sent after the first `n`.
    fn since(recording: &Recording, n: usize) -> Vec<MountItem> {
        recording.items()[n..].to_vec()
    }

    #[test]
    fn a_press_springs_the_handle_along_the_track_and_up_in_size_as_fast_spatial() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let items = since(&recording, n);
        let (center, scale) = handle_springs(&mounted, &items, "press");
        let center = center.expect("a spring for the center");
        let scale = scale.expect("a spring for the scale");
        assert_eq!((center.to, scale.to), (36.0, 1.0));
        // Nothing is laid out: the handle moves with transforms.
        assert_eq!(mounted.layout_items(&items), 0, "{items:?}");
        // While it plays, the engine sends nothing (ADR 0010).
        app.step("run");
        app.run_for(Duration::from_secs(1)).unwrap();
        assert!(recording.in_step("run").is_empty());

        // Mid-way, the handle is between its two places, its size the same fraction of the way: androidx's two
        // springs have one specification, so the left edge, the size and so the center all move in step. Its icon
        // and its state layer are on its center.
        let mut replay = replay(&recording);
        let mut centers = replay.run(center.node, 9);
        let (now, diameter) = mounted.handle(&replay);
        assert!(now > 17.0 && now < 35.0, "{now}");
        assert!(
            (diameter - (16.0 + 8.0 * (now - 16.0) / 20.0)).abs() < 1e-3,
            "{diameter}"
        );
        assert!((mounted.center_of(mounted.layer, &replay) - now).abs() < 1e-3);
        // The whole curves are the closed-form spring from the rest, within 0.1 % of their travel.
        centers.extend(replay.run(center.node, 171));
        let gap = worst_gap(&centers, 16.0, &center);
        assert!(gap < tolerance(16.0, 36.0), "center: {gap}");
        let mut again = crate::morph_probe::replay(&recording);
        let scales = again.run(scale.node, 180);
        let from = 2.0_f64 / 3.0;
        let gap = worst_gap(&scales, from, &scale);
        assert!(gap < tolerance(from, 1.0), "scale: {gap}");
        // Both settle on their ends exactly, and the graph stops.
        let settled = replay.run(center.node, 600);
        assert!(!replay.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(36.0_f32.to_bits())
        );
        assert_eq!(mounted.handle(&replay), (36.0, 24.0));
        // And back: the same spring the other way.
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let (center, scale) = handle_springs(&mounted, &since(&recording, n), "press back");
        assert_eq!(
            (center.map(|s| s.to), scale.map(|s| s.to)),
            (Some(16.0), Some(2.0 / 3.0))
        );
    }

    #[test]
    fn only_what_changes_springs() {
        // Unchecked with an icon, the handle is 24 across already: checking moves it and does not resize it.
        let (mut app, recording) = start(Config {
            icon: Icons::With,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let (center, scale) = handle_springs(&mounted, &since(&recording, n), "check with an icon");
        assert_eq!(center.map(|s| s.to), Some(36.0));
        assert_eq!(scale, None);

        // Checked, an icon coming or going changes nothing about the handle: it is 24 across and at the end.
        let (mut app, recording) = start(Config {
            checked: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        for step in ["Icon on", "Icon off"] {
            let n = recording.items().len();
            app.press(step).unwrap();
            let items = since(&recording, n);
            assert_eq!(handle_springs(&mounted, &items, step), (None, None));
        }

        // Unchecked, an icon makes the handle grow and go back, in place.
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("Icon on").unwrap();
        let (center, scale) = handle_springs(&mounted, &since(&recording, n), "icon on");
        assert_eq!(center, None);
        assert_eq!(scale.map(|s| s.to), Some(1.0));
        let n = recording.items().len();
        app.press("Icon off").unwrap();
        let (center, scale) = handle_springs(&mounted, &since(&recording, n), "icon off");
        assert_eq!(center, None);
        assert_eq!(scale.map(|s| s.to), Some(2.0 / 3.0));
    }

    #[test]
    fn a_value_the_app_sets_moves_the_handle_the_same_way() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("Turn on").unwrap();
        let (center, scale) = handle_springs(&mounted, &since(&recording, n), "turn on");
        assert_eq!(
            (center.map(|s| s.to), scale.map(|s| s.to)),
            (Some(36.0), Some(1.0))
        );
        // Setting the value it already has moves nothing.
        let n = recording.items().len();
        app.press("Turn on").unwrap();
        assert_eq!(
            handle_springs(&mounted, &since(&recording, n), "turn on again"),
            (None, None)
        );
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for checked in [false, true] {
            for icon in [false, true] {
                let what = format!("checked: {checked} icon: {icon}");
                let (mut app, recording) = start(Config {
                    checked,
                    icon: Icons::of(icon),
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{what}");
                // The values are created where the handle rests: nothing is set again after the layout.
                assert_eq!(set_values(&items), 0, "{what}");
                assert!(!replay(&recording).is_animating(), "{what}");
                // At rest, with time passing, nothing is sent.
                app.step("later");
                app.run_for(Duration::from_secs(2)).unwrap();
                assert!(recording.in_step("later").is_empty(), "{what}");
            }
        }
    }

    #[test]
    fn a_press_during_a_press_starts_from_where_and_as_fast_as_the_handle_moves() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let (center, _) = handle_springs(&mounted, &since(&recording, n), "check");
        let down = center.expect("a spring for the center");
        let mut replay = replay(&recording);
        // The handle moves for 50 ms, then the switch is pressed again.
        let before = replay.run(down.node, 6);
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let items = since(&recording, n);
        let (up, scale) = handle_springs(&mounted, &items, "uncheck");
        let up = up.expect("a spring for the center");
        assert_eq!((up.to, scale.map(|s| s.to)), (16.0, Some(2.0 / 3.0)));
        assert_eq!(up.node, down.node);
        replay.apply(&items);
        let after = replay.run(up.node, 180);

        // The graph last ticked 5 frames into the first spring: the second starts there, at that value and that
        // speed, as androidx's `Animatable` does when its target changes.
        let first = parameters(&down);
        let at = 5.0 / REPLAY_HZ;
        let position = |t: f64| spring_position(16.0, 36.0, 0.0, first, t);
        let from = position(at);
        let h = 1e-6;
        let speed = (position(at + h) - position(at - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed > 10.0, "{speed}");
        let gap = |velocity: f64| {
            after
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let t = f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                    let closed = spring_position(from, 16.0, velocity, parameters(&up), t);
                    (f64::from(value) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        let tolerance = tolerance(from, 16.0);
        assert!(gap(speed) < tolerance, "{}", gap(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(gap(0.0) > 10.0 * tolerance, "{}", gap(0.0));
        let settled = replay.run(up.node, 600);
        assert!(!replay.is_animating());
        assert_eq!(
            settled.last().map(|v| v.to_bits()),
            Some(16.0_f32.to_bits())
        );
        assert_eq!(mounted.handle(&replay), (16.0, 16.0));
    }

    #[test]
    fn spamming_the_switch_ends_on_the_last_value_with_nothing_left_running() {
        for presses in [1_u32, 2, 5, 8] {
            let (mut app, recording) = start(Config::default());
            let mounted = Mounted::of(&recording.items());
            let (center, _) = mounted.values();
            let mut replay = replay(&recording);
            let mounted_at = recording.items().len();
            let mut seen = mounted_at;
            for _ in 0..presses {
                app.press("label:Wi-Fi").unwrap();
                let items = since(&recording, seen);
                seen = recording.items().len();
                replay.apply(&items);
                // A few frames between presses, as a thumb that taps quickly leaves.
                for sample in replay.run(center, 3) {
                    assert!(sample.is_finite(), "{presses}: {sample}");
                }
            }
            let checked = presses % 2 == 1;
            assert_eq!(
                Mounted::of(&recording.items()).checked,
                Some(if checked {
                    Checked::True
                } else {
                    Checked::False
                }),
                "{presses}"
            );
            let settled = replay.run(center, 900);
            assert!(!replay.is_animating(), "{presses}");
            let (place, diameter) = if checked { (36.0, 24.0) } else { (16.0, 16.0) };
            assert_eq!(
                settled.last().map(|v| v.to_bits()),
                Some(f32::to_bits(place)),
                "{presses}"
            );
            assert_eq!(mounted.handle(&replay), (place, diameter), "{presses}");
            // Only springs and values: nothing was set and nothing was laid out.
            assert_eq!(set_values(&recording.items()), 0, "{presses}");
            assert_eq!(
                mounted.layout_items(&since(&recording, mounted_at)),
                0,
                "{presses}"
            );
        }
    }

    #[test]
    fn with_reduce_motion_the_handle_goes_to_its_place_at_once() {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let items = since(&recording, n);
        // ADR 0010, check 6: no spring, and the values are set, not eased.
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 2);
        let mut replay = replay(&recording);
        assert_eq!(mounted.handle(&replay), (36.0, 24.0));
        assert!(!replay.is_animating());
        let (center, _) = mounted.values();
        assert_eq!(replay.run(center, 2), [36.0, 36.0]);
        // Only the value that changes is set: checked with an icon, the handle is 24 across already.
        let (mut app, recording) = start(Config {
            icon: Icons::With,
            reduce_motion: true,
            ..Config::default()
        });
        let n = recording.items().len();
        app.press("label:Wi-Fi").unwrap();
        let items = since(&recording, n);
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 1);
        // And when the app sets the value, as when it does with motion.
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::default()
        });
        let n = recording.items().len();
        app.press("Turn on").unwrap();
        let items = since(&recording, n);
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(set_values(&items), 2);
        assert!(!crate::morph_probe::replay(&recording).is_animating());
    }

    #[test]
    fn the_handle_spring_is_fast_spatial_at_the_screens_rate() {
        let spring = handle_spring();
        assert_eq!(spring.stiffness, 800.0);
        assert!((spring.damping - 2.0 * 0.6 * 800.0_f32.sqrt()).abs() < 1e-4);
        assert_eq!(spring.mass, 1.0);
        assert_eq!(spring.velocity, None);
        assert_eq!(spring.frame_rate, None);
        // It is not the ratio taken as the coefficient (ADR 0004's bug), and not the standard scheme's spring.
        assert_ne!(spring.damping, 0.6);
        assert_ne!(spring.stiffness, 1400.0);
    }

    /// The vendored `Switch.kt` and `SwitchTokens.kt`, at the pinned commit (ADR 0012).
    const SWITCH_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Switch.kt");
    const SWITCH_TOKENS_KT: &str =
        include_str!("../../../reference/androidx-material3-tokens/SwitchTokens.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_outline_and_the_disabled_colors_are_the_components() {
        // The unchecked outline is the focus token's, which names the same role as the plain one.
        assert!(cites(
            SWITCH_KT,
            &[
                "uncheckedBorderColor =",
                "fromToken(SwitchTokens.UnselectedFocusTrackOutlineColor),",
            ]
        ));
        assert_eq!(
            switch_tokens::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR,
            switch_tokens::UNSELECTED_TRACK_OUTLINE_COLOR
        );
        assert_eq!(
            colors(Condition::Unchecked).outline,
            Some(Tint::solid(
                switch_tokens::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR
            ))
        );
        // The checked outline is transparent, and the outline is `TrackOutlineWidth` wide.
        assert!(cites(
            SWITCH_KT,
            &["checkedBorderColor = Color.Transparent,"]
        ));
        assert!(cites(
            SWITCH_KT,
            &[".border(TrackOutlineWidth, borderColor, trackShape)"]
        ));
        assert_eq!(colors(Condition::Checked).outline, None);
        assert_eq!(outline_width(Condition::Checked), Points(0.0));
        assert_eq!(
            outline_width(Condition::Unchecked),
            switch_tokens::TRACK_OUTLINE_WIDTH
        );
        assert_eq!(
            outline_width(Condition::DisabledUnchecked),
            switch_tokens::TRACK_OUTLINE_WIDTH
        );
        // Every disabled color is composited over the surface, the outline's included.
        assert!(cites(
            SWITCH_KT,
            &[
                "disabledUncheckedBorderColor =",
                "fromToken(SwitchTokens.DisabledUnselectedTrackOutlineColor)",
                ".copy(alpha = SwitchTokens.DisabledTrackOpacity)",
                ".compositeOver(surface),",
            ]
        ));
        assert!(cites(
            SWITCH_KT,
            &[
                "disabledCheckedThumbColor =",
                "fromToken(SwitchTokens.DisabledSelectedHandleColor)",
                ".copy(alpha = SwitchTokens.DisabledSelectedHandleOpacity)",
                ".compositeOver(surface),",
            ]
        ));
    }

    #[test]
    fn the_icon_and_handle_sizes_are_the_components() {
        // The icon's size is `SwitchDefaults.IconSize`, which the two tokens say too.
        assert!(cites(SWITCH_KT, &["public val IconSize: Dp = 16.dp"]));
        assert_eq!(ICON_SIZE, switch_tokens::SELECTED_ICON_SIZE);
        assert_eq!(ICON_SIZE, switch_tokens::UNSELECTED_ICON_SIZE);
        // The handle's sizes: 24 with an icon or checked (`SelectedHandleWidth`, not `IconHandleWidth`), 16 else.
        assert!(cites(
            SWITCH_KT,
            &[
                "isPressed -> SwitchTokens.PressedHandleWidth",
                "hasContent || checked -> ThumbDiameter",
                "else -> UncheckedThumbDiameter",
            ]
        ));
        assert!(cites(
            SWITCH_KT,
            &[
                "internal val ThumbDiameter",
                "get() = SwitchTokens.SelectedHandleWidth"
            ]
        ));
        assert!(cites(
            SWITCH_KT,
            &[
                "internal val UncheckedThumbDiameter",
                "get() = SwitchTokens.UnselectedHandleWidth"
            ]
        ));
        assert_eq!(BASE_DIAMETER, switch_tokens::SELECTED_HANDLE_WIDTH);
        assert_eq!(
            switch_tokens::ICON_HANDLE_WIDTH,
            switch_tokens::SELECTED_HANDLE_WIDTH
        );
    }

    #[test]
    fn the_motion_the_state_layer_and_the_touch_target_are_the_components() {
        // The handle moves with the Expressive fast spatial spring, and snaps to the pressed size (left out).
        assert!(cites(
            SWITCH_KT,
            &["animationSpec = MotionSchemeKeyTokens.FastSpatial.value(),"]
        ));
        assert!(cites(
            SWITCH_KT,
            &["sizeAnim?.animateTo(size, if (isPressed) SnapSpec else animationSpec)"]
        ));
        assert_eq!(switch_tokens::PRESSED_HANDLE_WIDTH, Points(28.0));
        // The state layer is a ripple of the handle, `StateLayerSize` across, drawn by the app's ripple settings.
        assert!(cites(
            SWITCH_KT,
            &["radius = SwitchTokens.StateLayerSize / 2,"]
        ));
        assert_eq!(switch_tokens::STATE_LAYER_SIZE, Points(40.0));
        // Presses reach 48 points, and nothing drags the handle.
        assert!(cites(
            SWITCH_KT,
            &["Modifier.minimumInteractiveComponentSize()"]
        ));
        assert!(cites(
            SWITCH_KT,
            &["// TODO: Add Swipeable modifier b/223797571"]
        ));
        // The token file's own numbers, read where the opacities are written.
        assert!(SWITCH_TOKENS_KT.contains("const val DisabledTrackOpacity = 0.12f"));
        assert!(SWITCH_TOKENS_KT.contains("const val DisabledUnselectedHandleOpacity = 0.38f"));
        assert!((switch_tokens::DISABLED_TRACK_OPACITY - 0.12).abs() < 1e-6);
        assert!((switch_tokens::DISABLED_UNSELECTED_HANDLE_OPACITY - 0.38).abs() < 1e-6);
        // A line that is not there is not found.
        assert!(!cites(
            SWITCH_KT,
            &["uncheckedBorderColor = fromToken(SwitchTokens.UnselectedTrackOutlineColor),"]
        ));
    }
}
