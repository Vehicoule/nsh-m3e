//! The Material 3 Expressive checkbox: a box with a rounded corner that fills with the primary color and shows a
//! check (or, for a partial selection, a dash) when it is not unchecked, and the state layer of hover, focus and
//! press around it.
//!
//! The checkbox follows androidx's `Checkbox` and `TriStateCheckbox` in `Checkbox.kt`, at the pinned commit, as
//! [ADR 0012] says. Its box and its mark are drawn by one canvas whose colors, check and dash are animated values the
//! platform moves frame by frame ([ADR 0011]'s drawn container, with the mark drawn from path data as [ADR 0017]
//! draws icons), and its state follows `nsh-base`'s `Checkbox`: presses, keyboard activation, focus, the disabled
//! state and the checkbox role with its value for screen readers.
//!
//! # Which androidx path
//!
//! `Checkbox.kt` has two looks, chosen by `ComposeMaterial3Flags.isCheckboxStylingFixEnabled` (`false` at line 49 of
//! the vendored `ComposeMaterial3Flags.kt`): the shipped default is a 20-point box inside 2 points of padding with the
//! check's points at 0.2, 0.7 and 0.8 of its side, and the other reads `CheckboxTokens`, an 18-point box, the check at
//! 0.25, 0.65 and 0.75, and colors for the state layer (`Checkbox.kt` lines 497 to 534 and 711 to 724). This
//! checkbox follows the one that reads the tokens, the look the M3 specification gives. A flag file is vendored beside
//! `Checkbox.kt` only so that this choice can be read and checked.
//!
//! # Where this checkbox follows `Checkbox.kt` over the tokens (ADR 0012)
//!
//! - **The outline is 2 points in every state.** `CheckboxTokens.SelectedOutlineWidth` is 0, but `drawBox` strokes the
//!   border with `CheckboxDefaults.StrokeWidth`, 2 points (`Checkbox.kt` lines 482, 651 to 663 and 675 to 701), in the
//!   fill's own color when checked, so the box is the same filled square.
//! - **The mark is 2 points wide with square ends**, `CheckboxDefaults.StrokeWidth` and `StrokeCap.Square` (lines 482
//!   and 669), which no token holds. `CheckboxTokens.IconSize` (18) is the box's side here, as androidx's
//!   `containerSize` takes it (lines 518 to 523).
//! - **The mark's geometry** is `Checkbox.kt` lines 713 to 724 (the fractions of the box's side), and its draw
//!   fraction and gravitation are lines 595 to 631.
//! - **The colors** are `CheckboxColors` as `defaultCheckboxColors` builds it (lines 446 to 476) and the functions
//!   that pick among them (lines 918 to 997). The disabled checkmark is `SelectedDisabledIconColor` with no opacity
//!   (lines 452 and 453), where the container and the outline are faded to 38 %.
//! - **The state layer's size and shape** are `StateLayerSize` and a circle (lines 554 to 560).
//! - **The motion** is `MotionSchemeKeyTokens.DefaultSpatial` for the check and the dash, `DefaultEffects` for the
//!   colors going to checked and `FastEffects` for the colors going to unchecked (lines 595 to 650 and 1038 to 1049),
//!   and a 100 ms delay before the check disappears (line 1061).
//!
//! # Where it differs from `Checkbox.kt`
//!
//! - **An unchecked checkbox has a state layer.** With the flag on, androidx colors the layer with `indicatorColor`,
//!   which is the unchecked fill, transparent (lines 510 to 515 and 918 to 923): nothing shows on hover, focus or
//!   press. The tokens name `on-surface` for those states (`UnselectedHoverOutlineColor` and its siblings), as the
//!   specification draws them, so this checkbox shows an `on-surface` layer. That is a decision for the user.
//! - **The layout is 40 by 40 points**, the state layer's size. androidx lays out 48 by 48
//!   (`minimumInteractiveComponentSize`, line 566); here the 48 is the hit slop, so that the touch target is as large
//!   with no change to the layout (NSH's accessibility guide).
//! - **The layer's color does not change on hover, focus or press of the outline.** The tokens give the unselected
//!   outline a color for each of these states (all `on-surface`), which androidx does not use and nor does this
//!   checkbox: the outline keeps `UnselectedOutlineColor`.
//! - **A disabled mark fades with the state.** androidx draws it in one color in every state and hides it by its
//!   length (line 940). This checkbox fades it from transparent, as the enabled one, because a path of no length
//!   drawn with square ends is a dot in Canvas 2D implementations as in Skia, and the draw fraction of an unchecked
//!   checkbox is 0. A disabled checkbox also moves its colors at once, where androidx moves the mark's (lines 633 to
//!   650): a disabled mark that is cleared is therefore gone at once, where androidx's stays for the 100 ms of its
//!   delay, and one that is checked is drawn in its final color while it grows.
//! - **Colors move in sRGB**, as NSH interpolates colors (`AnimatedNode::interpolate_colors`); androidx moves them in
//!   Oklab, a difference of the shades between the two ends only.
//! - **The mark's length is read from a table.** androidx measures the path every frame (`PathMeasure`, line 732); a
//!   graph of animated values has no square root, so the lengths of the check's two strokes are tabulated over the
//!   gravitation (17 points from -0.5 to 1.5, wide enough for the spring's overshoot) and interpolated, and the draw
//!   fraction is cut from them as androidx cuts the path. The mark is within 0.01 point of the one androidx's cut
//!   gives, which `a_check_changed_while_it_draws_is_cut_where_androidx_cuts_it` bounds (measured at most 0.005 point,
//!   over 16 changes of state in the middle of the drawing, in a debug build, in a box of 18 points). A change of
//!   state therefore makes no new drawing.
//! - **Reduce motion** sends no spring and no delay; the checkbox goes to its new state at once. ADR 0010 asks for a
//!   short fade with the duration its spec gives, and no duration for it is found in the tokens: ADR 0011 leaves it
//!   to be decided with the fade of every component.
//!
//! # Left out
//!
//! - **The error state.** `CheckboxTokens` names error colors, but `Checkbox.kt` has no error state and no parameter
//!   to ask for one.
//! - **The focus indicator.** `CheckboxTokens.FocusIndicatorColor` is the ring of keyboard focus; androidx draws it
//!   with the ripple's `focusRingShape` (line 559) and NSH has no focus ring. Focus shows the state layer only.
//! - **A label.** androidx's `Checkbox` has none: the app places a text beside it. The caller names the checkbox for
//!   screen readers with `label`.
//! - **The flooring of the stroke width to device pixels** (line 653). NSH draws in points.
//! - **Elevated variants.** A checkbox has no elevation.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use std::time::Duration;

use nsh_core::canvas::LineCap;
use nsh_core::geometry::{Bounds, Points};
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::morph;
use crate::motion::{SPRING_MASS, SpringToken};
use crate::tint::{MIN_TOUCH_TARGET, Tint, tint_color};
use crate::tokens::androidx::{checkbox_tokens, state_tokens};

/// What a [`Checkbox`] shows: androidx's `ToggleableState` (`Off`, `On`, `Indeterminate`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum CheckboxState {
    /// An empty box: androidx `ToggleableState.Off`.
    #[default]
    Unchecked,
    /// A filled box with a check: androidx `ToggleableState.On`.
    Checked,
    /// A filled box with a dash, for a selection that is partly checked ("select all" over a few items): androidx
    /// `ToggleableState.Indeterminate`.
    Indeterminate,
}

/// The props of a [`Checkbox`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct CheckboxProps {
    /// What it shows. The app owns it: a press reports the state it asks for with [`CheckboxEvent::Change`], and the
    /// checkbox shows the state the app passes back.
    #[prop(default)]
    pub state: Prop<CheckboxState>,
    /// What screen readers say: a checkbox shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// No presses: the disabled colors, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
}

/// The events of a [`Checkbox`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum CheckboxEvent {
    /// The checkbox was pressed (a tap, a click, Space or Return): the state it asks for, `true` (checked) from
    /// unchecked and from indeterminate, `false` (unchecked) from checked, as `nsh-base`'s `Checkbox` reports it.
    #[args(checked)]
    Change(bool),
}

/// How an animated value goes to its target when the state changes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Move {
    /// It is where it should be, or going there: nothing starts.
    Hold,
    /// A spring from where it is and as fast as it moves.
    Spring(SpringToken),
    /// A jump now (androidx `snap()`).
    Snap,
    /// A jump after a delay, the value staying where it is until then (androidx `snap(delayMillis = …)`).
    SnapAfter(Duration),
}

/// How each of the three animated values moves for one change of state.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Moves {
    /// The colors' progress from the unchecked colors (0) to the checked ones (1).
    color: Move,
    /// How much of the mark is drawn, from nothing (0) to all of it (1).
    draw: Move,
    /// How far the check has gone to the dash, from the check (0) to the dash (1).
    gravity: Move,
}

/// The delay before the mark of a checkbox going to unchecked disappears, while its colors fade out: androidx
/// `SnapAnimationDelay` (`Checkbox.kt` line 1061).
const SNAP_DELAY: Duration = Duration::from_millis(100);

/// How the values move when the state goes from `from` to `to`, as `Checkbox.kt` gives them: the draw fraction at
/// lines 597 to 613, the gravitation at lines 615 to 631 and the colors at lines 1040 to 1048.
///
/// androidx springs a value that is already at its target when both states are checked (`else ->
/// defaultAnimationSpec` for the draw fraction going from checked to indeterminate stays at 1); that is no motion,
/// so nothing starts and a spring still running to the same target runs on.
fn moves(from: CheckboxState, to: CheckboxState) -> Moves {
    if from == to {
        return Moves {
            color: Move::Hold,
            draw: Move::Hold,
            gravity: Move::Hold,
        };
    }
    let from_unchecked = from == CheckboxState::Unchecked;
    let to_unchecked = to == CheckboxState::Unchecked;
    // The colors take the speed of the state they go to: `FastEffects` out of the box, `DefaultEffects` into it. Going
    // between checked and indeterminate they stay (androidx's target colors are the same).
    let color = if from_unchecked || to_unchecked {
        Move::Spring(if to_unchecked {
            SpringToken::FastEffects
        } else {
            SpringToken::DefaultEffects
        })
    } else {
        Move::Hold
    };
    let draw = if from_unchecked {
        Move::Spring(SpringToken::DefaultSpatial)
    } else if to_unchecked {
        Move::SnapAfter(SNAP_DELAY)
    } else {
        Move::Hold
    };
    let gravity = if from_unchecked {
        Move::Snap
    } else if to_unchecked {
        Move::SnapAfter(SNAP_DELAY)
    } else {
        Move::Spring(SpringToken::DefaultSpatial)
    };
    Moves {
        color,
        draw,
        gravity,
    }
}

/// Where the three animated values are for a state.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Targets {
    color: f32,
    draw: f32,
    gravity: f32,
}

/// The targets of `state`: `Checkbox.kt` lines 608 to 612 (the draw fraction) and 626 to 630 (the gravitation); the
/// colors are the unchecked ones at 0 and the checked ones, for checked and indeterminate alike, at 1.
fn targets(state: CheckboxState) -> Targets {
    match state {
        CheckboxState::Unchecked => Targets {
            color: 0.0,
            draw: 0.0,
            gravity: 0.0,
        },
        CheckboxState::Checked => Targets {
            color: 1.0,
            draw: 1.0,
            gravity: 0.0,
        },
        CheckboxState::Indeterminate => Targets {
            color: 1.0,
            draw: 1.0,
            gravity: 1.0,
        },
    }
}

/// The spring of `token`, converted as ADR 0004 says and written out in every field: it starts from the speed the
/// value already has, at the screen's own rate (ADR 0010).
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

/// The animated values that draw the box, and the state they show. A `Copy` handle, like the signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Values {
    color: AnimatedValue,
    draw: AnimatedValue,
    gravity: AnimatedValue,
    /// The state they are at, or going to.
    state: Signal<CheckboxState>,
    /// Whether the checkbox is disabled: its colors then move at once.
    disabled: Signal<bool>,
}

impl Values {
    /// Values at rest for `state`, with nothing started.
    fn new(state: CheckboxState, disabled: bool) -> Self {
        let at = targets(state);
        Self {
            color: animated_value(at.color),
            draw: animated_value(at.draw),
            gravity: animated_value(at.gravity),
            state: signal(state),
            disabled: signal(disabled),
        }
    }

    /// The state changes to `to`: each value goes where `Checkbox.kt` sends it. With reduce motion on, nothing is
    /// sprung or delayed: they go there at once.
    fn go(self, to: CheckboxState) {
        untracked(|| {
            let from = self.state.get();
            if from == to {
                return;
            }
            self.state.set(to);
            let (moves, at) = (moves(from, to), targets(to));
            let reduced = reduce_motion();
            // androidx moves the colors of a disabled checkbox at once: "there should be no animations between
            // enabled / disabled" (`Checkbox.kt` lines 639 and 647).
            let color_at_once = reduced || self.disabled.get();
            start(self.color, at.color, moves.color, color_at_once);
            start(self.draw, at.draw, moves.draw, reduced);
            start(self.gravity, at.gravity, moves.gravity, reduced);
        });
    }

    /// The checkbox becomes disabled (or enabled again). Disabled, its colors jump to where they are going to, as
    /// androidx's disabled colors are not animated.
    fn disable(self, disabled: bool) {
        untracked(|| {
            if self.disabled.get() == disabled {
                return;
            }
            self.disabled.set(disabled);
            if disabled {
                self.color.set(targets(self.state.get()).color);
            }
        });
    }
}

/// Moves `value` to `to` as `mv` says, or jumps there when `at_once` is set.
fn start(value: AnimatedValue, to: f32, mv: Move, at_once: bool) {
    match mv {
        Move::Hold => {}
        _ if at_once => value.set(to),
        Move::Snap => value.set(to),
        Move::Spring(token) => {
            let _ = value.spring(to, spring(token));
        }
        Move::SnapAfter(delay) => {
            // A timing that takes no time, after the delay: the value stays where it is until then, and a new target
            // replaces it, as a new target replaces androidx's transition.
            let timing = Timing {
                delay_ms: delay.as_secs_f32() * 1000.0,
                ..Timing::new(0.0, Easing::Linear)
            };
            let _ = value.timing_with(to, timing);
        }
    }
}

/// The three colors the box is painted with in one state: `CheckboxColors`' `boxColor`, `borderColor` and
/// `checkmarkColor` (`Checkbox.kt` lines 932 to 997).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Look {
    fill: Tint,
    border: Tint,
    mark: Tint,
}

/// The look of the box for `checked` (checked or indeterminate: androidx colors them alike) and `disabled`, as
/// `defaultCheckboxColors` builds it (`Checkbox.kt` lines 446 to 476).
///
/// androidx's "transparent" colors are `Color.Transparent` (black, no alpha). A color moving to or from one keeps
/// the other end's hue in androidx's interpolation, which weights a color by its alpha; NSH interpolates colors with
/// no such weight, so each transparent color here is its neighbor's role at no opacity, which gives the same colors
/// in between.
fn look(checked: bool, disabled: bool) -> Look {
    use checkbox_tokens as t;
    match (checked, disabled) {
        (false, false) => Look {
            fill: Tint::faded(t::SELECTED_CONTAINER_COLOR, 0.0),
            border: Tint::solid(t::UNSELECTED_OUTLINE_COLOR),
            mark: Tint::faded(t::SELECTED_ICON_COLOR, 0.0),
        },
        (true, false) => Look {
            fill: Tint::solid(t::SELECTED_CONTAINER_COLOR),
            border: Tint::solid(t::SELECTED_CONTAINER_COLOR),
            mark: Tint::solid(t::SELECTED_ICON_COLOR),
        },
        (false, true) => Look {
            fill: Tint::faded(t::SELECTED_DISABLED_CONTAINER_COLOR, 0.0),
            border: Tint::faded(
                t::UNSELECTED_DISABLED_OUTLINE_COLOR,
                t::UNSELECTED_DISABLED_CONTAINER_OPACITY,
            ),
            mark: Tint::faded(t::SELECTED_DISABLED_ICON_COLOR, 0.0),
        },
        (true, true) => {
            let faded = Tint::faded(
                t::SELECTED_DISABLED_CONTAINER_COLOR,
                t::SELECTED_DISABLED_CONTAINER_OPACITY,
            );
            Look {
                fill: faded,
                border: faded,
                mark: Tint::solid(t::SELECTED_DISABLED_ICON_COLOR),
            }
        }
    }
}

/// The color of the state layer for `state`: the layer of a checked or indeterminate checkbox is the fill's color
/// (`SelectedHoverContainerColor`, `SelectedFocusContainerColor`, `SelectedPressedContainerColor`), and the layer of
/// an unchecked one the outline's role of the same states (`UnselectedHoverOutlineColor` and its siblings), which
/// androidx does not draw (module docs). The three states of each have one color in the tokens.
fn layer_role(state: CheckboxState) -> ColorRole {
    match state {
        CheckboxState::Unchecked => checkbox_tokens::UNSELECTED_HOVER_OUTLINE_COLOR,
        CheckboxState::Checked | CheckboxState::Indeterminate => {
            checkbox_tokens::SELECTED_HOVER_CONTAINER_COLOR
        }
    }
}

/// The box's side: `CheckboxTokens.ContainerSize`, as the branch of the flag that reads the tokens takes it
/// (`Checkbox.kt` lines 518 to 523).
const BOX: Points = checkbox_tokens::CONTAINER_SIZE;

/// The width of the outline and of the mark: androidx `CheckboxDefaults.StrokeWidth` (`Checkbox.kt` line 482), which
/// `CheckboxTokens.UnselectedOutlineWidth` equals (`the_outline_and_the_mark_are_as_wide_as_the_token_says`).
const STROKE: Points = checkbox_tokens::UNSELECTED_OUTLINE_WIDTH;

/// The points of the check as fractions of the box's side, and the point they gravitate to: `Checkbox.kt` lines 713
/// to 724 on the branch the tokens take (0.25, 0.65 and 0.75; the default branch's are 0.2, 0.7 and 0.8). The cross
/// is where the two strokes meet.
const CROSS: (f32, f32) = (0.4, 0.65);
const LEFT: (f32, f32) = (0.25, 0.5);
const RIGHT: (f32, f32) = (0.75, 0.3);
/// Where every point's height gravitates to when the check becomes a dash, and the cross's width (line 720).
const CENTER: f32 = 0.5;

/// The lengths, in points, of the mark's first stroke (from its left end to the cross) and its second (from the cross
/// to its right end), when the check has gone `gravity` of the way to the dash (0 the check, 1 the dash).
fn stroke_lengths(gravity: f32) -> (f32, f32) {
    let toward_center = |from: f32| from + (CENTER - from) * gravity;
    let cross = (toward_center(CROSS.0), toward_center(CROSS.1));
    let right_y = toward_center(RIGHT.1);
    let first = (cross.0 - LEFT.0).hypot(cross.1 - LEFT.1);
    let second = (RIGHT.0 - cross.0).hypot(right_y - cross.1);
    (first * BOX.0, second * BOX.0)
}

/// The gravitations at which the strokes' lengths are tabulated: from `TABLE_FROM` in `TABLE_STEPS` equal steps of
/// `TABLE_STEP`, a range around the check (0) and the dash (1) wide enough for the spring that moves between them to
/// overshoot (`the_table_covers_the_springs_overshoot`). The graph of animated values has no square root, so the
/// lengths a path is measured by are read from this table, interpolated between its points.
const TABLE_FROM: f32 = -0.5;
const TABLE_STEP: f32 = 0.125;
const TABLE_STEPS: u8 = 16;

/// The gravitations of the table and, at each, the two strokes' lengths.
fn length_table() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let at: Vec<f32> = (0..=TABLE_STEPS)
        .map(|step| TABLE_FROM + f32::from(step) * TABLE_STEP)
        .collect();
    let (first, second) = at.iter().map(|gravity| stroke_lengths(*gravity)).unzip();
    (at, first, second)
}

/// The corner radius of the box: `CheckboxTokens.ContainerShape`, which androidx's `RadiusSize` (2 points, line 1068)
/// equals.
fn corner_radius() -> f32 {
    radius_of(morph::corners(checkbox_tokens::CONTAINER_SHAPE))
}

/// The radius `corners` give the box: a radius in points as it is, a full one half the box (a token's `ContainerShape`
/// is the first, so the second is only what a token with another shape would give).
fn radius_of(corners: morph::Corners) -> f32 {
    match corners {
        morph::Corners::Radius(radius) => radius.0,
        morph::Corners::Full => BOX.0 / 2.0,
    }
}

/// `Bounds` of a square `inset` points inside the box.
fn inside(inset: f32) -> Bounds<Points> {
    let side = (BOX.0 - 2.0 * inset).max(0.0);
    Bounds::new(point(px(inset), px(inset)), size(px(side), px(side)))
}

/// The drawing of the box and its mark, for a checkbox that is `disabled`. Its colors and the mark follow the animated
/// values the platform moves; the drawing is made again only when the colors it goes between change, so a change of
/// state makes no new drawing.
fn drawing(values: Values, disabled: bool) -> Painter {
    let (off, on) = (look(false, disabled), look(true, disabled));
    let color = |from: Tint, to: Tint| {
        values
            .color
            .interpolate_colors(&[0.0, 1.0], &[tint_color(Some(from)), tint_color(Some(to))])
    };
    let (fill, border, mark) = (
        color(off.fill, on.fill),
        color(off.border, on.border),
        color(off.mark, on.mark),
    );
    let radius = corner_radius();
    let width = STROKE.0;
    let at = |fraction: f32| fraction * BOX.0;
    // The points, from `Checkbox.kt` lines 720 to 724: only the points off the middle line move to it.
    let gravitate = |from: f32| values.gravity.mul(at(CENTER - from)).add(at(from));
    let (cross_x, cross_y, right_y) = (gravitate(CROSS.0), gravitate(CROSS.1), gravitate(RIGHT.1));
    let (left_x, left_y, right_x) = (at(LEFT.0), at(LEFT.1), at(RIGHT.0));
    // The stroke drawn so far: `getSegment(0, length * fraction)` of the path (`Checkbox.kt` line 734), which androidx
    // measures each frame. The lengths of the two strokes are those of the gravitation now, from the table; the drawn
    // length is the fraction of both, the first stroke is drawn up to its own length, and the second with what is left.
    // Each share stops at its ends, as the draw fraction goes past 1 while its spring overshoots.
    let (gravities, first_lengths, second_lengths) = length_table();
    let first_length = values.gravity.interpolate(&gravities, &first_lengths);
    let second_length = values.gravity.interpolate(&gravities, &second_lengths);
    let drawn = values.draw.mul(&first_length.add(&second_length));
    let share = |node: AnimatedNode| node.interpolate(&[0.0, 1.0], &[0.0, 1.0]);
    let first = share(drawn.div(&first_length));
    let second = share(drawn.sub(&first_length).div(&second_length));
    let first_step = (cross_x.sub(left_x), cross_y.sub(left_y));
    let bend = (
        first.mul(&first_step.0).add(left_x),
        first.mul(&first_step.1).add(left_y),
    );
    let second_step = (cross_x.mul(-1.0).add(right_x), right_y.sub(&cross_y));
    let end = (
        second.mul(&second_step.0).add(&bend.0),
        second.mul(&second_step.1).add(&bend.1),
    );
    Painter::new()
        // `drawBox`: the fill inside the border, and the border inside the box's edge (lines 686 to 699).
        .fill_style(&fill)
        .begin_path()
        .round_rect(inside(width), px((radius - width).max(0.0)))
        .fill()
        .stroke_style(&border)
        .line_width(STROKE)
        .begin_path()
        .round_rect(inside(width / 2.0), px((radius - width / 2.0).max(0.0)))
        .stroke()
        // `drawCheck`: the mark, a square-ended stroke (lines 728 to 736 and 669).
        .stroke_style(&mark)
        .line_width(STROKE)
        .line_cap(LineCap::Square)
        .begin_path()
        .move_to(left_x, left_y)
        .line_to(&bend.0, &bend.1)
        .line_to(&end.0, &end.1)
        .stroke()
}

/// How far presses also count around the layout, so that a checkbox has a 48-point target on each side
/// (androidx `minimumInteractiveComponentSize`, `Checkbox.kt` line 566), without changing its layout (NSH's
/// accessibility guide: extend the touch area with the hit slop, not the size).
fn touch_slop() -> EdgeInsets {
    EdgeInsets::all(px(((MIN_TOUCH_TARGET.0
        - checkbox_tokens::STATE_LAYER_SIZE.0)
        / 2.0)
        .max(0.0)))
}

/// A Material 3 Expressive checkbox: a box that is empty, filled with a check, or filled with a dash.
///
/// It has the behavior of `nsh-base`'s `Checkbox` (presses, Space and Return, focus, the disabled state, the
/// checkbox role with its value for screen readers, named by its label) and the look of androidx's `Checkbox`. The
/// state it shows is the app's: a press only reports the state it asks for. Hover, focus and press lay a circle of the
/// box's color over it at the opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so
/// nothing is laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::checkbox::{CheckboxState, checkbox};
///
/// fn remember() -> impl IntoElement {
///     let state = signal(CheckboxState::Unchecked);
///     checkbox()
///         .label("Remember me")
///         .state(move || state.get())
///         .on_change(move |checked: &bool| {
///             state.set(if *checked { CheckboxState::Checked } else { CheckboxState::Unchecked });
///         })
/// }
/// ```
#[component]
pub fn Checkbox(props: CheckboxProps, cx: Scope<CheckboxEvent>) -> impl IntoElement {
    let state = props.state;
    let values = untracked(|| Values::new(state.get(), props.disabled.get()));
    effect(move || values.go(state.get()));
    effect(move || values.disable(props.disabled.get()));
    nsh_base::checkbox()
        .checked(move || state.get() == CheckboxState::Checked)
        .indeterminate(move || state.get() == CheckboxState::Indeterminate)
        .disabled(move || props.disabled.get())
        .accessibility_label(move || Some(props.label.get()))
        .on_change(move |checked: &bool| cx.emit(CheckboxEvent::Change(*checked)))
        .hit_slop(touch_slop())
        .group()
        .items_center()
        .justify_center()
        .w(checkbox_tokens::STATE_LAYER_SIZE)
        .h(checkbox_tokens::STATE_LAYER_SIZE)
        .child(
            // The box and its mark.
            canvas(move || drawing(values, props.disabled.get()))
                .w(BOX)
                .h(BOX),
        )
        .child(
            // The state layer: a circle of the state layer's size over the box, in the box's color, shown by the
            // checkbox's own states with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`).
            view()
                .absolute()
                .inset(px(0.0))
                .rounded_full()
                .bg(move || tint_color(Some(Tint::solid(layer_role(state.get())))))
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| {
                    // A disabled checkbox takes no press and shows no state.
                    if props.disabled.get() { s.hidden() } else { s }
                }),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{AnimationOp, Driver, NodeId as AnimationNode};
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num, PathOp};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, Checked, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, layout_items, spring_position,
        started_springs, worst_gap,
    };
    use crate::color::ColorScheme;
    use crate::morph_probe::{parameters, replay, set_values, tolerance};
    use crate::tokens::Shape;

    /// What the probe places: one checkbox with these props, a line of what presses asked for (`T` for checked, `F`
    /// for unchecked), and lines that change its state and disable it, as an app that owns the state would.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        state: Prop<CheckboxState>,
        #[prop(default)]
        disabled: Prop<bool>,
        /// Whether the app ignores what a press asks for, as an app that refuses the change would.
        #[prop(default)]
        refuses: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let state = signal(props.state.get_untracked());
        let disabled = signal(props.disabled.get_untracked());
        let asked = signal(String::new());
        let refuses = props.refuses.get_untracked();
        // A row, so that the checkbox keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    checkbox()
                        .label("Agree")
                        .state(move || state.get())
                        .disabled(move || disabled.get())
                        .on_change(move |checked: &bool| {
                            asked.update(|asked| asked.push(if *checked { 'T' } else { 'F' }));
                            if !refuses {
                                state.set(if *checked {
                                    CheckboxState::Checked
                                } else {
                                    CheckboxState::Unchecked
                                });
                            }
                        }),
                ),
            )
            .child(text(move || format!("asked [{}]", asked.get())))
            .child(text("Check").on_press(move |(): &()| state.set(CheckboxState::Checked)))
            .child(text("Clear").on_press(move |(): &()| state.set(CheckboxState::Unchecked)))
            .child(text("Mix").on_press(move |(): &()| state.set(CheckboxState::Indeterminate)))
            .child(text("Disable").on_press(move |(): &()| disabled.set(true)))
            .child(text("Enable").on_press(move |(): &()| disabled.set(false)))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        state: CheckboxState,
        disabled: bool,
        refuses: bool,
        dark: bool,
        motion: Motion,
    }

    /// What the system says about motion.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Motion {
        #[default]
        Full,
        /// "Reduce motion" is on.
        Reduced,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            state: Prop::new(config.state),
            disabled: Prop::new(config.disabled),
            refuses: Prop::new(config.refuses),
        });
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            reduce_motion: config.motion == Motion::Reduced,
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:asked []").unwrap();
        (app, recording)
    }

    /// The checkbox's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the checkbox role: the whole checkbox.
        root: NodeId,
        /// `nsh-base`'s indicator, which holds nothing here.
        indicator: NodeId,
        /// The canvas that draws the box and its mark.
        canvas: NodeId,
        /// The view that is the state layer.
        layer: NodeId,
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
            let mut root = None;
            let (mut label, mut disabled, mut checked, mut hit_slop) = (None, None, None, None);
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
                        if props.accessibility.role == AccessibilityRole::Checkbox =>
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
                    _ => {}
                }
            }
            let root = root.expect("a view with the checkbox role");
            // The indicator `nsh-base` always makes, empty; then the box's canvas, under the state layer; and
            // nothing else in the checkbox.
            let own: Vec<NodeId> = children.get(&root).cloned().unwrap_or_default();
            let [indicator, canvas, layer] = own[..] else {
                panic!("three children in the checkbox: {items:?}");
            };
            assert_eq!(
                own.iter().map(|id| kinds.get(id)).collect::<Vec<_>>(),
                [
                    Some(&ViewKind::View),
                    Some(&ViewKind::Canvas),
                    Some(&ViewKind::View)
                ],
                "{items:?}"
            );
            assert_eq!(children.get(&indicator).map(Vec::len), None, "{items:?}");
            Self {
                root,
                indicator,
                canvas,
                layer,
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

        fn drawing(&self) -> &Drawing {
            &self.drawings[&self.canvas]
        }

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }

        /// How many of `items` lay the checkbox out again: one of its views' frames.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.root, self.indicator, self.canvas, self.layer];
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Frame { id, .. } if ours.contains(id)))
                .count()
        }

        /// How many of `items` draw the box again.
        fn canvas_items(&self, items: &[MountItem]) -> usize {
            items
                .iter()
                .filter(|item| matches!(item, MountItem::Canvas { id, .. } if *id == self.canvas))
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

    /// The 8-bit alpha NSH rounds 0.38 of 255 to.
    const ALPHA_38: u32 = 97;

    /// A number of a drawing that no animated value moves.
    fn constant(num: Num) -> f32 {
        match num {
            Num::Const(value) => value,
            Num::Slot(slot) => panic!("slot {slot} where a constant is drawn"),
        }
    }

    /// The color of `paint` in `replay`.
    fn paint_color(drawing: &Drawing, paint: &canvas::Paint, replay: &Replay) -> Color {
        match paint {
            canvas::Paint::Color(color) => *color,
            canvas::Paint::Slot(slot) => Color(
                replay
                    .color(drawing.slots[usize::from(*slot)])
                    .expect("a node of colors"),
            ),
            canvas::Paint::Gradient(_) => panic!("no gradient in a checkbox"),
        }
    }

    /// The colors the box is filled, bordered and marked with, in `replay`.
    fn paints(drawing: &Drawing, replay: &Replay) -> [Color; 3] {
        let found: Vec<Color> = drawing
            .ops
            .iter()
            .filter_map(|op| match op {
                CanvasOp::FillStyle(paint) | CanvasOp::StrokeStyle(paint) => {
                    Some(paint_color(drawing, paint, replay))
                }
                _ => None,
            })
            .collect();
        let [fill, border, mark] = found[..] else {
            panic!("a fill, a border and a mark: {found:?}");
        };
        [fill, border, mark]
    }

    /// The rounded rectangles the drawing traces, each as its `x`, `y`, `width`, `height` and the radius of its
    /// corners.
    fn rectangles(drawing: &Drawing) -> Vec<([f32; 4], [f32; 4])> {
        drawing
            .ops
            .iter()
            .filter_map(|op| match op {
                CanvasOp::Path(PathOp::RoundRect(rect, radii)) => {
                    Some((rect.map(constant), radii.map(constant)))
                }
                _ => None,
            })
            .collect()
    }

    /// The three points of the mark's path in `replay`: where it starts, bends and ends.
    fn mark(drawing: &Drawing, replay: &Replay) -> [(f32, f32); 3] {
        let slot = |slot: u16| replay.value(drawing.slots[usize::from(slot)]);
        let points: Vec<(f32, f32)> = drawing
            .ops
            .iter()
            .filter_map(|op| match op {
                CanvasOp::Path(PathOp::MoveTo(x, y) | PathOp::LineTo(x, y)) => {
                    Some((x.get(slot), y.get(slot)))
                }
                _ => None,
            })
            .collect();
        let [start, bend, end] = points[..] else {
            panic!("a start and two lines: {points:?}");
        };
        [start, bend, end]
    }

    /// Every point of `found` is within a thousandth of a point of `expected`.
    fn assert_points(found: [(f32, f32); 3], expected: [(f32, f32); 3], what: &str) {
        let same = found
            .iter()
            .zip(&expected)
            .all(|(a, b)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3);
        assert!(same, "{what}: {found:?} is not {expected:?}");
    }

    const STATES: [CheckboxState; 3] = [
        CheckboxState::Unchecked,
        CheckboxState::Checked,
        CheckboxState::Indeterminate,
    ];

    /// The fill, border and mark each state should paint at rest, in the baseline schemes: `Checkbox.kt`'s
    /// `defaultCheckboxColors` (lines 446 to 476) with `CheckboxTokens`' roles. A transparent color is its
    /// neighbor's role at no opacity.
    fn expected_colors(state: CheckboxState, disabled: bool, dark: bool) -> [Color; 3] {
        let solid = |role| color(scheme_color(role, dark), 255);
        let faded = |role, alpha| color(scheme_color(role, dark), alpha);
        let checked = state != CheckboxState::Unchecked;
        match (checked, disabled) {
            // `uncheckedBoxColor` and `uncheckedCheckmarkColor` are transparent; `UnselectedOutlineColor`.
            (false, false) => [
                faded(ColorRole::Primary, 0),
                solid(ColorRole::OnSurfaceVariant),
                faded(ColorRole::OnPrimary, 0),
            ],
            // `SelectedContainerColor` for the box and its border, `SelectedIconColor`.
            (true, false) => [
                solid(ColorRole::Primary),
                solid(ColorRole::Primary),
                solid(ColorRole::OnPrimary),
            ],
            // Transparent box; `UnselectedDisabledOutlineColor` at `UnselectedDisabledContainerOpacity`.
            (false, true) => [
                faded(ColorRole::OnSurface, 0),
                faded(ColorRole::OnSurface, ALPHA_38),
                faded(ColorRole::Surface, 0),
            ],
            // `SelectedDisabledContainerColor` at `SelectedDisabledContainerOpacity` for the box and its border, and
            // `SelectedDisabledIconColor` with no opacity for the mark.
            (true, true) => [
                faded(ColorRole::OnSurface, ALPHA_38),
                faded(ColorRole::OnSurface, ALPHA_38),
                solid(ColorRole::Surface),
            ],
        }
    }

    #[test]
    fn each_state_has_its_colors_enabled_and_disabled_light_and_dark() {
        for state in STATES {
            for disabled in [false, true] {
                for dark in [false, true] {
                    let what = format!("{state:?} disabled: {disabled} dark: {dark}");
                    let (_app, recording) = start(Config {
                        state,
                        disabled,
                        dark,
                        ..Config::default()
                    });
                    let mounted = Mounted::of(&recording.items());
                    assert_eq!(
                        paints(mounted.drawing(), &replay(&recording)),
                        expected_colors(state, disabled, dark),
                        "{what}"
                    );
                    // No view paints a background or a border of its own under the box.
                    for id in [mounted.root, mounted.indicator, mounted.canvas] {
                        let style = mounted.style(id);
                        assert_eq!(style.background_color, Color::TRANSPARENT, "{what}");
                        assert_eq!(style.borders.widths.top, px(0.0), "{what}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_box_is_18_points_in_the_middle_of_a_40_point_state_layer() {
        for state in STATES {
            let mounted = mount(Config {
                state,
                ..Config::default()
            });
            // `StateLayerSize` is the layout; `ContainerSize` the box, in the middle of it.
            assert_eq!(
                mounted.frame(mounted.root).size,
                Size::new(Points(40.0), Points(40.0))
            );
            let canvas = mounted.frame(mounted.canvas);
            assert_eq!(canvas.size, Size::new(Points(18.0), Points(18.0)));
            assert_eq!((canvas.origin.x.0, canvas.origin.y.0), (11.0, 11.0));
            let layer = mounted.frame(mounted.layer);
            assert_eq!(layer.size, mounted.frame(mounted.root).size);
            assert_eq!((layer.origin.x.0, layer.origin.y.0), (0.0, 0.0));
            // The indicator `nsh-base` makes takes no room.
            assert_eq!(
                mounted
                    .frames
                    .get(&mounted.indicator)
                    .map(|frame| frame.size),
                Some(Size::default())
            );
        }
        assert_eq!(BOX, Points(18.0));
        assert_eq!(checkbox_tokens::STATE_LAYER_SIZE, Points(40.0));
    }

    #[test]
    fn the_box_draws_its_fill_inside_its_border_with_2_point_corners() {
        for state in STATES {
            for disabled in [false, true] {
                let what = format!("{state:?} disabled: {disabled}");
                let mounted = mount(Config {
                    state,
                    disabled,
                    ..Config::default()
                });
                let drawing = mounted.drawing();
                // The fill is inset by the stroke's width and rounded by the radius less that width (none left:
                // `max(0, radius - width)`, `Checkbox.kt` line 690); the border is stroked half of its width inside
                // the edge, rounded by the radius less half the width (line 697).
                assert_eq!(
                    rectangles(drawing),
                    vec![
                        ([2.0, 2.0, 14.0, 14.0], [0.0; 4]),
                        ([1.0, 1.0, 16.0, 16.0], [1.0; 4])
                    ],
                    "{what}"
                );
                // Fill, then stroke, then the mark's stroke; the border and the mark 2 points wide, the mark with
                // square ends and the border with the default butt ones.
                let widths: Vec<f32> = drawing
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        CanvasOp::LineWidth(width) => Some(constant(*width)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(widths, [2.0, 2.0], "{what}");
                let caps: Vec<&CanvasOp> = drawing
                    .ops
                    .iter()
                    .filter(|op| matches!(op, CanvasOp::LineCap(_)))
                    .collect();
                assert_eq!(caps, [&CanvasOp::LineCap(LineCap::Square)], "{what}");
                let kinds: Vec<&str> = drawing
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        CanvasOp::Fill(..) => Some("fill"),
                        CanvasOp::Stroke(None) => Some("stroke"),
                        _ => None,
                    })
                    .collect();
                assert_eq!(kinds, ["fill", "stroke", "stroke"], "{what}");
            }
        }
    }

    #[test]
    fn the_outline_and_the_mark_are_as_wide_as_the_token_says() {
        // androidx strokes every state's border and the mark at `CheckboxDefaults.StrokeWidth`, 2 points (lines 482
        // and 651 to 669), where the selected tokens say 0: the border has the fill's own color, so the box is as
        // filled as the token draws it (ADR 0012).
        assert_eq!(STROKE, Points(2.0));
        for width in [
            checkbox_tokens::UNSELECTED_OUTLINE_WIDTH,
            checkbox_tokens::UNSELECTED_DISABLED_OUTLINE_WIDTH,
            checkbox_tokens::UNSELECTED_HOVER_OUTLINE_WIDTH,
            checkbox_tokens::UNSELECTED_FOCUS_OUTLINE_WIDTH,
            checkbox_tokens::UNSELECTED_PRESSED_OUTLINE_WIDTH,
        ] {
            assert_eq!(width, STROKE);
        }
        assert_eq!(checkbox_tokens::SELECTED_OUTLINE_WIDTH, Points(0.0));
        assert_ne!(checkbox_tokens::SELECTED_OUTLINE_WIDTH, STROKE);
        assert_eq!(corner_radius().to_bits(), 2.0_f32.to_bits());
    }

    /// A radius in points is the box's radius as it is, and a full one is half the box: the second is not what the
    /// token gives, so only this says what it would.
    #[test]
    fn the_box_takes_its_radius_from_its_corners() {
        assert_eq!(
            radius_of(morph::Corners::Radius(px(3.0))).to_bits(),
            3.0_f32.to_bits()
        );
        assert_eq!(
            radius_of(morph::Corners::Full).to_bits(),
            (BOX.0 / 2.0).to_bits()
        );
        assert_eq!(BOX, Points(18.0));
        assert_eq!(radius_of(morph::Corners::Full).to_bits(), 9.0_f32.to_bits());
    }

    #[test]
    fn the_mark_at_rest_is_nothing_a_check_or_a_dash() {
        // 18 points: the check's points at 0.25 / 0.5, 0.4 / 0.65 and 0.75 / 0.3 of the side, the dash's on the
        // middle line (`Checkbox.kt` lines 713 to 724).
        let start = (4.5, 9.0);
        let expected = [
            // No length: the draw fraction is 0.
            (CheckboxState::Unchecked, [start, start, start]),
            (CheckboxState::Checked, [start, (7.2, 11.7), (13.5, 5.4)]),
            (
                CheckboxState::Indeterminate,
                [start, (9.0, 9.0), (13.5, 9.0)],
            ),
        ];
        for (state, points) in expected {
            let (_app, recording) = start_with(state);
            let mounted = Mounted::of(&recording.items());
            assert_points(
                mark(mounted.drawing(), &replay(&recording)),
                points,
                &format!("{state:?}"),
            );
        }
    }

    fn start_with(state: CheckboxState) -> (Headless, Recording) {
        start(Config {
            state,
            ..Config::default()
        })
    }

    #[test]
    fn screen_readers_name_it_by_its_label_with_its_state() {
        let expected = [
            (CheckboxState::Unchecked, Checked::False),
            (CheckboxState::Checked, Checked::True),
            (CheckboxState::Indeterminate, Checked::Mixed),
        ];
        for (state, checked) in expected {
            let mounted = mount(Config {
                state,
                ..Config::default()
            });
            assert_eq!(mounted.label.as_deref(), Some("Agree"), "{state:?}");
            assert_eq!(mounted.checked, Some(checked), "{state:?}");
            assert_eq!(mounted.disabled, Some(false), "{state:?}");
        }
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
    fn a_press_asks_for_the_next_state_once() -> Result<(), String> {
        // `nsh-base`: checked from unchecked and from indeterminate, unchecked from checked.
        let expected = [
            (CheckboxState::Unchecked, "asked [T]"),
            (CheckboxState::Checked, "asked [F]"),
            (CheckboxState::Indeterminate, "asked [T]"),
        ];
        for (state, asked) in expected {
            let (mut app, _recording) = start_with(state);
            app.press("label:Agree")?;
            assert!(app.shows(asked), "{state:?}");
            // Nothing else was asked.
            assert!(
                !app.shows("asked [TT]") && !app.shows("asked [FF]"),
                "{state:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn it_shows_the_state_the_app_gives_back_and_not_the_one_it_asked_for() -> Result<(), String> {
        let (mut app, recording) = start(Config {
            refuses: true,
            ..Config::default()
        });
        app.press("label:Agree")?;
        assert!(app.shows("asked [T]"));
        // The app refused: the box is still empty.
        assert_eq!(
            Mounted::of(&recording.items()).checked,
            Some(Checked::False)
        );
        app.press("Check")?;
        assert_eq!(Mounted::of(&recording.items()).checked, Some(Checked::True));
        Ok(())
    }

    #[test]
    fn space_asks_for_the_next_state_of_the_focused_checkbox() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press_key("tab")?;
        app.press_key("space")?;
        assert!(app.shows("asked [T]"));
        assert!(!app.shows("asked [TT]"));
        Ok(())
    }

    #[test]
    fn every_checkbox_has_a_48_point_touch_target_without_growing() {
        let mounted = mount(Config::default());
        let slop = mounted.hit_slop.expect("the checkbox's props");
        // 40 points of layout, 4 of slop on each side: 48. The layout stays the state layer's.
        let side = mounted.frame(mounted.root).size;
        assert_eq!(side, Size::new(Points(40.0), Points(40.0)));
        assert_eq!(slop, EdgeInsets::all(px(4.0)));
        assert!(side.width.0 + slop.left.0 + slop.right.0 >= 48.0);
        assert!(side.height.0 + slop.top.0 + slop.bottom.0 >= 48.0);
        assert_eq!(MIN_TOUCH_TARGET, Points(48.0));
    }

    #[test]
    fn a_disabled_checkbox_takes_no_press_and_shows_no_state() {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("label:Agree");
        let _ = app.press_key("tab");
        let _ = app.press_key("space");
        let _ = app.hover("label:Agree");
        let mounted = Mounted::of(&recording.items());
        // Even with a press told to the engine, as a platform would, nothing is asked and nothing shows.
        press_state(&mut app, &recording, &mounted, true).unwrap();
        assert!(app.shows("asked []"));
        let mounted = Mounted::of(&recording.items());
        // The state layer is not laid out (`hidden`, CSS's `display: none`): it has no frame to show a state in.
        assert_eq!(
            mounted.frames.get(&mounted.layer).map(|frame| frame.size),
            Some(Size::default())
        );
        // The checkbox keeps its size, and says it is disabled.
        assert_eq!(mounted.frame(mounted.root).size.width, Points(40.0));
        assert_eq!(mounted.disabled, Some(true));
        // Enabled, its state layer is there.
        let enabled = mount(Config::default());
        assert_eq!(
            enabled.frames.get(&enabled.layer).map(|frame| frame.size),
            Some(enabled.frame(enabled.root).size)
        );
    }

    #[test]
    fn the_state_layer_is_a_circle_in_the_boxs_color_and_shows_nothing_at_rest() {
        let roles = [
            // The tokens' `UnselectedHoverOutlineColor`, which androidx does not draw (module docs).
            (CheckboxState::Unchecked, ColorRole::OnSurface),
            (CheckboxState::Checked, ColorRole::Primary),
            (CheckboxState::Indeterminate, ColorRole::Primary),
        ];
        for (state, role) in roles {
            for dark in [false, true] {
                let what = format!("{state:?} dark: {dark}");
                let mounted = mount(Config {
                    state,
                    dark,
                    ..Config::default()
                });
                let style = mounted.style(mounted.layer);
                assert_eq!(style.opacity, 0.0, "{what}");
                assert_eq!(
                    style.background_color,
                    color(scheme_color(role, dark), 255),
                    "{what}"
                );
                // `rounded_full` (9999 points) is a circle on a square: the radius clamps to half of the side.
                assert!(style.border_radius.0 >= 20.0, "{what}");
                assert_eq!(
                    mounted.frame(mounted.layer).size,
                    Size::new(Points(40.0), Points(40.0)),
                    "{what}"
                );
            }
        }
        assert_eq!(checkbox_tokens::STATE_LAYER_SHAPE, Shape::Circle);
        assert_eq!(layer_role(CheckboxState::Unchecked), ColorRole::OnSurface);
    }

    #[test]
    fn the_layers_color_is_the_same_in_hover_focus_and_press() {
        // One color per state of the box serves the three states of the layer only while the tokens agree.
        use checkbox_tokens as t;
        assert_eq!(
            t::SELECTED_HOVER_CONTAINER_COLOR,
            t::SELECTED_FOCUS_CONTAINER_COLOR
        );
        assert_eq!(
            t::SELECTED_HOVER_CONTAINER_COLOR,
            t::SELECTED_PRESSED_CONTAINER_COLOR
        );
        assert_eq!(
            t::UNSELECTED_HOVER_OUTLINE_COLOR,
            t::UNSELECTED_FOCUS_OUTLINE_COLOR
        );
        assert_eq!(
            t::UNSELECTED_HOVER_OUTLINE_COLOR,
            t::UNSELECTED_PRESSED_OUTLINE_COLOR
        );
        // And the layer's color is the fill's role when checked.
        assert_eq!(
            t::SELECTED_HOVER_CONTAINER_COLOR,
            t::SELECTED_CONTAINER_COLOR
        );
    }

    /// Tells the engine that the checkbox is pressed (`down`) or not, as a platform does when a finger lands on it or
    /// lifts (`PRESSED` in the view's states, which `Headless::drag_and_hold` does not set), and returns the items
    /// that came of it.
    fn press_state(
        app: &mut Headless,
        recording: &Recording,
        mounted: &Mounted,
        down: bool,
    ) -> Result<Vec<MountItem>, String> {
        use nsh_core::tailwind::state::PRESSED;
        let before = recording.items().len();
        app.engine()
            .control()
            .set_states(mounted.root, if down { PRESSED } else { 0 });
        app.run_for(Duration::from_millis(1))?;
        Ok(recording.items().split_off(before))
    }

    /// The opacity of every style the layer was sent in `items`, and whether anything else was styled or drawn.
    fn layer_in(mounted: &Mounted, items: &[MountItem]) -> (Vec<f32>, bool) {
        let layer: Vec<f32> = items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect();
        let only_the_layer = items.iter().all(|item| match item {
            MountItem::ViewStyle { id, .. } | MountItem::Canvas { id, .. } => *id == mounted.layer,
            _ => true,
        });
        (layer, only_the_layer)
    }

    #[test]
    fn hover_focus_and_press_show_the_state_layer_and_lay_out_nothing() -> Result<(), String> {
        for state in STATES {
            let (mut app, recording) = start_with(state);
            let mounted = Mounted::of(&recording.items());
            app.hover("label:Agree")?;
            let hover = recording.in_step("hover(label:Agree)");
            // `StateTokens.HoverStateLayerOpacity`; nothing else styled, drawn, laid out or animated.
            assert_eq!(layer_in(&mounted, &hover), (vec![0.08], true), "{state:?}");
            assert_eq!(layout_items(&hover), 0, "{state:?}");
            assert!(
                hover
                    .iter()
                    .all(|item| !matches!(item, MountItem::Animation(_))),
                "{state:?}"
            );
            // `nsh-testing`'s pointer never presses a view's state (the platforms do), so the press is told to the
            // engine as a platform tells it.
            let press = press_state(&mut app, &recording, &mounted, true)?;
            // `StateTokens.PressedStateLayerOpacity`, which is the focus one too; the same restraint.
            let (opacities, only_the_layer) = layer_in(&mounted, &press);
            assert_eq!(opacities.last(), Some(&0.1), "{state:?}");
            assert!(only_the_layer, "{state:?}");
            assert_eq!(layout_items(&press), 0, "{state:?}");
            assert!(
                press
                    .iter()
                    .all(|item| !matches!(item, MountItem::Animation(_))),
                "{state:?}"
            );
            // The press ends: the states are none again, and the layer is at rest, with nothing else.
            let lifted = press_state(&mut app, &recording, &mounted, false)?;
            let (opacities, only_the_layer) = layer_in(&mounted, &lifted);
            assert_eq!(opacities.last(), Some(&0.0), "{state:?}");
            assert!(only_the_layer, "{state:?}");
        }
        Ok(())
    }

    #[test]
    fn focus_shows_the_state_layer() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("tab")?;
        let focus = recording.in_step("keyboard.press(tab)");
        let (opacities, only_the_layer) = layer_in(&mounted, &focus);
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(opacities.last(), Some(&0.1));
        assert!(only_the_layer);
        assert_eq!(layout_items(&focus), 0);
        assert_eq!(
            state_tokens::FOCUS_STATE_LAYER_OPACITY.to_bits(),
            0.1_f32.to_bits()
        );
        Ok(())
    }

    /// The animated values' nodes, in the order the component creates them: the colors' progress, the draw fraction
    /// and the gravitation.
    fn value_nodes(recording: &Recording) -> [AnimationNode; 3] {
        let created: Vec<AnimationNode> = recording
            .items()
            .iter()
            .filter_map(|item| match item {
                MountItem::Animation(AnimationOp::CreateValue { node, .. }) => Some(*node),
                _ => None,
            })
            .collect();
        let [color, draw, gravity] = created[..] else {
            panic!("three animated values: {created:?}");
        };
        [color, draw, gravity]
    }

    /// A timing an animation operation started, as the engine sent it.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct StartedTiming {
        node: AnimationNode,
        to: f32,
        duration_ms: f32,
        delay_ms: f32,
        frame_rate: Option<f32>,
    }

    fn started_timings(items: &[MountItem]) -> Vec<StartedTiming> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::Animation(AnimationOp::Start {
                    node,
                    driver:
                        Driver::Timing {
                            to,
                            duration_ms,
                            delay_ms,
                            frame_rate,
                            ..
                        },
                    ..
                }) => Some(StartedTiming {
                    node: *node,
                    to: *to,
                    duration_ms: *duration_ms,
                    delay_ms: *delay_ms,
                    frame_rate: *frame_rate,
                }),
                _ => None,
            })
            .collect()
    }

    /// The jumps `items` make, in order: the value and where it goes.
    fn sets(items: &[MountItem]) -> Vec<(AnimationNode, f32)> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::Animation(AnimationOp::Set { node, value }) => Some((*node, *value)),
                _ => None,
            })
            .collect()
    }

    /// The timing that takes `node` to `to` after androidx's delay, taking no time of its own.
    fn delayed_jump(node: AnimationNode, to: f32) -> StartedTiming {
        StartedTiming {
            node,
            to,
            duration_ms: 0.0,
            delay_ms: 100.0,
            frame_rate: None,
        }
    }

    /// The one spring `items` start on `node`.
    fn spring_on(items: &[MountItem], node: AnimationNode) -> StartedSpring {
        let on: Vec<StartedSpring> = started_springs(items)
            .into_iter()
            .filter(|spring| spring.node == node)
            .collect();
        let [spring] = on[..] else {
            panic!("one spring on {node}: {on:?} in {items:?}");
        };
        spring
    }

    /// `spring` is `token`'s spring, converted as ADR 0004 says, going to `to` from where the value is and as fast as
    /// it moves, at the screen's rate (ADR 0010 check 1 and 4).
    fn assert_spring(spring: &StartedSpring, token: SpringToken, to: f32, what: &str) {
        let spec = token.spec();
        assert_eq!(spring.to.to_bits(), to.to_bits(), "{what}: {spring:?}");
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

    /// `nodes` after each of `frames` frames of `replay`.
    fn sample(replay: &mut Replay, nodes: &[AnimationNode], frames: usize) -> Vec<Vec<f32>> {
        (0..frames)
            .map(|_| {
                replay.run(nodes[0], 1);
                nodes.iter().map(|&node| replay.value(node)).collect()
            })
            .collect()
    }

    /// The mark's three points when `fraction` of its length is drawn and the check has gone `gravity` of the way to
    /// the dash, cut as `PathMeasure.getSegment(0, length * fraction)` cuts it (`Checkbox.kt` line 734), worked in
    /// lengths as androidx does and not as the component's nodes do, from the fractions of lines 713 to 724.
    fn cut(fraction: f32, gravity: f32) -> [(f32, f32); 3] {
        let side = 18.0;
        let toward_middle = |from: f32| (from + (0.5 - from) * gravity) * side;
        let start = (0.25 * side, 0.5 * side);
        let cross = (toward_middle(0.4), toward_middle(0.65));
        let end = (0.75 * side, toward_middle(0.3));
        let length = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0).hypot(b.1 - a.1);
        let (first, second) = (length(start, cross), length(cross, end));
        let drawn = fraction.clamp(0.0, 1.0) * (first + second);
        let along = |a: (f32, f32), b: (f32, f32), share: f32| {
            (a.0 + (b.0 - a.0) * share, a.1 + (b.1 - a.1) * share)
        };
        let bend = along(start, cross, (drawn / first).min(1.0));
        let tip = if drawn <= first {
            bend
        } else {
            along(cross, end, (drawn - first) / second)
        };
        [start, bend, tip]
    }

    #[test]
    fn the_cut_helper_is_the_rest_marks_at_the_ends() {
        let start = (4.5, 9.0);
        assert_points(cut(0.0, 0.0), [start; 3], "nothing");
        assert_points(cut(1.0, 0.0), [start, (7.2, 11.7), (13.5, 5.4)], "check");
        assert_points(cut(1.0, 1.0), [start, (9.0, 9.0), (13.5, 9.0)], "dash");
        // Past the end of its length it stops; before its start it is nothing.
        assert_points(cut(1.3, 0.0), cut(1.0, 0.0), "overshoot");
        assert_points(cut(-0.2, 0.0), cut(0.0, 0.0), "undershoot");
        // Half of the check's length is past its first stroke (30 % of it).
        let half = cut(0.5, 0.0);
        assert_points(
            [half[0], half[1], half[0]],
            [start, (7.2, 11.7), start],
            "bend",
        );
        assert!(half[2].0 > 7.2 && half[2].0 < 13.5);
    }

    #[test]
    fn every_change_of_state_moves_the_values_as_androidx_says() {
        use CheckboxState::{Checked, Indeterminate, Unchecked};
        let default_effects = Move::Spring(SpringToken::DefaultEffects);
        let fast_effects = Move::Spring(SpringToken::FastEffects);
        let spatial = Move::Spring(SpringToken::DefaultSpatial);
        let after_delay = Move::SnapAfter(Duration::from_millis(100));
        let moves_of = |color, draw, gravity| Moves {
            color,
            draw,
            gravity,
        };
        // `Checkbox.kt`: the draw fraction at lines 597 to 613, the gravitation at 615 to 631, the colors at 1040 to
        // 1048. Going out of the box the colors are fast and the mark waits for them; going into it the colors take
        // the default speed and the check draws; from checked to indeterminate only the check bends.
        let expected = [
            (
                Unchecked,
                Checked,
                moves_of(default_effects, spatial, Move::Snap),
            ),
            (
                Unchecked,
                Indeterminate,
                moves_of(default_effects, spatial, Move::Snap),
            ),
            (
                Checked,
                Unchecked,
                moves_of(fast_effects, after_delay, after_delay),
            ),
            (
                Indeterminate,
                Unchecked,
                moves_of(fast_effects, after_delay, after_delay),
            ),
            (
                Checked,
                Indeterminate,
                moves_of(Move::Hold, Move::Hold, spatial),
            ),
            (
                Indeterminate,
                Checked,
                moves_of(Move::Hold, Move::Hold, spatial),
            ),
        ];
        for (from, to, moves_expected) in expected {
            assert_eq!(moves(from, to), moves_expected, "{from:?} to {to:?}");
        }
        for state in STATES {
            assert_eq!(
                moves(state, state),
                moves_of(Move::Hold, Move::Hold, Move::Hold),
                "{state:?}"
            );
        }
        // Where each state leaves the values (lines 608 to 612 and 626 to 630).
        let target = |color, draw, gravity| Targets {
            color,
            draw,
            gravity,
        };
        assert_eq!(targets(Unchecked), target(0.0, 0.0, 0.0));
        assert_eq!(targets(Checked), target(1.0, 1.0, 0.0));
        assert_eq!(targets(Indeterminate), target(1.0, 1.0, 1.0));
    }

    #[test]
    fn the_springs_are_three_different_springs_and_the_delay_is_100_ms() {
        let (a, b, c) = (
            SpringToken::DefaultSpatial.spec(),
            SpringToken::DefaultEffects.spec(),
            SpringToken::FastEffects.spec(),
        );
        let key = |spec: crate::motion::SpringSpec| {
            (spec.stiffness.to_bits(), spec.damping_ratio.to_bits())
        };
        assert!(key(a) != key(b) && key(b) != key(c) && key(a) != key(c));
        assert_eq!(SNAP_DELAY, Duration::from_millis(100));
        // What `spring` builds: every field written out, none left to NSH's defaults.
        let spring = spring(SpringToken::FastEffects);
        assert_eq!(spring.stiffness.to_bits(), c.stiffness.to_bits());
        assert_eq!(spring.damping.to_bits(), c.damping_coefficient().to_bits());
        assert_ne!(spring.damping.to_bits(), c.damping_ratio.to_bits());
        assert_eq!(spring.mass.to_bits(), SPRING_MASS.to_bits());
        assert_eq!((spring.velocity, spring.frame_rate), (None, None));
    }

    #[test]
    fn mounting_starts_nothing_and_leaves_the_graph_still() {
        for state in STATES {
            for disabled in [false, true] {
                let what = format!("{state:?} disabled: {disabled}");
                let (_app, recording) = start(Config {
                    state,
                    disabled,
                    ..Config::default()
                });
                let items = recording.items();
                assert_eq!(started_springs(&items), Vec::new(), "{what}");
                assert_eq!(started_timings(&items), Vec::new(), "{what}");
                // The values are created where they rest: nothing sets them again.
                assert_eq!(set_values(&items), 0, "{what}");
                assert!(!replay(&recording).is_animating(), "{what}");
                // And they are where the state puts them.
                let replayed = replay(&recording);
                let at = targets(state);
                let values = value_nodes(&recording).map(|node| replayed.value(node));
                assert_eq!(
                    values.map(f32::to_bits),
                    [at.color, at.draw, at.gravity].map(f32::to_bits),
                    "{what}"
                );
            }
        }
    }

    /// The `hex` color's channels, and `color`'s, as floats: red, green, blue, alpha.
    fn channels(color: Color) -> [f32; 4] {
        let [r, g, b, a] = color.0.to_be_bytes();
        [r, g, b, a].map(f32::from)
    }

    /// `from` moved `by` of the way to `to` in each channel, as the graph interpolates colors (8 bits of each
    /// channel, rounded): the channels a color should have within a unit of.
    fn between(from: Color, to: Color, by: f32) -> [f32; 4] {
        let (from, to) = (channels(from), channels(to));
        [0, 1, 2, 3].map(|i| from[i] + (to[i] - from[i]) * by)
    }

    fn assert_channels(found: Color, expected: [f32; 4], what: &str) {
        let found_channels = channels(found);
        let same = found_channels
            .iter()
            .zip(&expected)
            .all(|(a, b)| (a - b).abs() <= 1.0);
        assert!(same, "{what}: {found_channels:?} is not {expected:?}");
    }

    #[test]
    fn checking_draws_the_check_with_the_spatial_spring_and_fills_with_the_effects_one()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        app.press("Check")?;
        let items = recording.in_step("press(Check)");
        // The colors and the draw fraction spring; the gravitation, already at the check, jumps there (androidx
        // `snap()`); nothing else moves, is drawn or is laid out again.
        assert_eq!(started_springs(&items).len(), 2, "{items:?}");
        let color_spring = spring_on(&items, color_node);
        let draw_spring = spring_on(&items, draw_node);
        assert_spring(&color_spring, SpringToken::DefaultEffects, 1.0, "colors");
        assert_spring(
            &draw_spring,
            SpringToken::DefaultSpatial,
            1.0,
            "draw fraction",
        );
        assert_eq!(sets(&items), vec![(gravity_node, 0.0)]);
        assert_eq!(started_timings(&items), Vec::new());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.layout_items(&items), 0);
        assert_eq!(mounted.canvas_items(&items), 0);
        assert_eq!(mounted.checked, Some(Checked::True));
        // While it plays, the engine sends nothing (ADR 0010 check 3).
        app.step("run");
        app.run_for(Duration::from_secs(1))?;
        assert!(recording.in_step("run").is_empty());

        let mut replayed = replay(&recording);
        let mut frames = Vec::new();
        for frame in 0..180 {
            let at = sample(&mut replayed, &[draw_node, color_node], 1).remove(0);
            // Every 15 frames the mark and the three colors are what the values say.
            if frame % 15 == 0 {
                assert_points(
                    mark(mounted.drawing(), &replayed),
                    cut(at[0], 0.0),
                    &format!("frame {frame}"),
                );
                let [fill, border, mark_color] = paints(mounted.drawing(), &replayed);
                let primary = color(scheme_color(ColorRole::Primary, false), 255);
                let on_primary = color(scheme_color(ColorRole::OnPrimary, false), 255);
                let outline = color(scheme_color(ColorRole::OnSurfaceVariant, false), 255);
                let hue = |role: ColorRole| color(scheme_color(role, false), 0);
                assert_channels(
                    fill,
                    between(hue(ColorRole::Primary), primary, at[1]),
                    "fill",
                );
                assert_channels(border, between(outline, primary, at[1]), "border");
                assert_channels(
                    mark_color,
                    between(hue(ColorRole::OnPrimary), on_primary, at[1]),
                    "mark",
                );
            }
            frames.push(at);
        }
        // Each curve is the closed-form spring from rest, within 0.1 % of its travel at every frame (ADR 0010 check
        // 2); the check's spring overshoots (`DefaultSpatial` is underdamped), the colors' does not.
        let drawn: Vec<f32> = frames.iter().map(|at| at[0]).collect();
        let colors: Vec<f32> = frames.iter().map(|at| at[1]).collect();
        assert!(worst_gap(&drawn, 0.0, &draw_spring) < tolerance(0.0, 1.0));
        assert!(worst_gap(&colors, 0.0, &color_spring) < tolerance(0.0, 1.0));
        assert!(drawn.iter().any(|v| *v > 1.001), "{drawn:?}");
        assert!(colors.iter().all(|v| *v <= 1.0));
        // The mark stops at its end while the fraction overshoots.
        let end = drawn.iter().position(|v| *v > 1.001).unwrap_or(0);
        let mut again = replay(&recording);
        sample(&mut again, &[draw_node], end + 1);
        assert_points(mark(mounted.drawing(), &again), cut(1.0, 0.0), "overshoot");
        // Both settle exactly on their targets, and the graph stops.
        let settled = sample(&mut replayed, &[draw_node, color_node, gravity_node], 600);
        assert!(!replayed.is_animating());
        let last = settled.last().expect("frames");
        assert_eq!(
            last.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            [1.0_f32, 1.0, 0.0].map(f32::to_bits)
        );
        Ok(())
    }

    #[test]
    fn unchecking_fades_the_colors_out_fast_and_takes_the_mark_away_after_100_ms()
    -> Result<(), String> {
        let (mut app, recording) = start_with(CheckboxState::Checked);
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        app.press("Clear")?;
        let items = recording.in_step("press(Clear)");
        assert_eq!(started_springs(&items).len(), 1, "{items:?}");
        let color_spring = spring_on(&items, color_node);
        assert_spring(&color_spring, SpringToken::FastEffects, 0.0, "colors");
        // The mark and the gravitation jump after the delay, taking no time of their own, and ask for the screen's
        // rate; nothing jumps now.
        assert_eq!(
            started_timings(&items),
            vec![
                delayed_jump(draw_node, 0.0),
                delayed_jump(gravity_node, 0.0)
            ]
        );
        assert_eq!(sets(&items), Vec::new());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.layout_items(&items), 0);
        assert_eq!(mounted.canvas_items(&items), 0);
        assert_eq!(mounted.checked, Some(Checked::False));

        let mut replayed = replay(&recording);
        let first = sample(&mut replayed, &[draw_node, color_node], 6);
        // 50 ms in, the whole check is still there and its colors have started to go.
        assert!(first.iter().all(|at| at[0].to_bits() == 1.0_f32.to_bits()));
        assert_points(mark(mounted.drawing(), &replayed), cut(1.0, 0.0), "waiting");
        let now = replayed.value(color_node);
        assert!(now < 0.9 && now > 0.1, "{now}");
        let rest = sample(&mut replayed, &[draw_node, color_node], 34);
        let drawn: Vec<f32> = first.iter().chain(&rest).map(|at| at[0]).collect();
        // It is whole, then gone at once, at 100 ms: frame 12 of a 120 Hz replay, or the next.
        let gone = drawn.iter().position(|v| *v == 0.0).expect("the mark goes");
        assert!((12..=13).contains(&gone), "{gone}");
        assert!(
            drawn[..gone]
                .iter()
                .all(|v| v.to_bits() == 1.0_f32.to_bits())
        );
        assert!(drawn[gone..].iter().all(|v| *v == 0.0));
        // By then the colors are nearly gone (a critically damped spring of 3800 is 2 % from its end at 92 ms):
        // `FastEffects` is the fast one.
        let at_gone = rest.get(gone - 6).map_or(1.0, |at| at[1]);
        assert!(at_gone < 0.05, "{at_gone}");
        assert_points(mark(mounted.drawing(), &replayed), cut(0.0, 0.0), "gone");
        sample(&mut replayed, &[color_node], 600);
        assert!(!replayed.is_animating());
        assert_eq!(replayed.value(color_node).to_bits(), 0.0_f32.to_bits());
        assert_eq!(replayed.value(gravity_node).to_bits(), 0.0_f32.to_bits());
        Ok(())
    }

    #[test]
    fn from_the_check_to_the_dash_only_the_gravitation_springs_and_the_check_bends()
    -> Result<(), String> {
        let (mut app, recording) = start_with(CheckboxState::Checked);
        let [_, draw_node, gravity_node] = value_nodes(&recording);
        app.press("Mix")?;
        let items = recording.in_step("press(Mix)");
        // The colors stay (both states are filled) and the check stays whole: one spring, and no new drawing, as the
        // strokes' lengths follow the gravitation.
        assert_eq!(started_springs(&items).len(), 1, "{items:?}");
        let spring = spring_on(&items, gravity_node);
        assert_spring(&spring, SpringToken::DefaultSpatial, 1.0, "gravitation");
        assert_eq!(
            (sets(&items), started_timings(&items)),
            (Vec::new(), Vec::new())
        );
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.canvas_items(&items), 0);
        assert_eq!(mounted.layout_items(&items), 0);
        assert_eq!(mounted.checked, Some(Checked::Mixed));

        let mut replayed = replay(&recording);
        let mut samples = Vec::new();
        for frame in 0..180 {
            let at = sample(&mut replayed, &[gravity_node, draw_node], 1).remove(0);
            if frame % 10 == 0 {
                // The whole mark, with every point gravitating by the value (past 1 while the spring overshoots).
                assert_points(
                    mark(mounted.drawing(), &replayed),
                    cut(1.0, at[0]),
                    &format!("frame {frame}"),
                );
            }
            assert_eq!(at[1].to_bits(), 1.0_f32.to_bits(), "frame {frame}");
            samples.push(at[0]);
        }
        assert!(worst_gap(&samples, 0.0, &spring) < tolerance(0.0, 1.0));
        assert!(samples.iter().any(|v| *v > 1.001));
        let settled = sample(&mut replayed, &[gravity_node], 600);
        assert_eq!(
            settled.last().map(|at| at[0].to_bits()),
            Some(1.0_f32.to_bits())
        );
        assert!(!replayed.is_animating());
        assert_points(mark(mounted.drawing(), &replayed), cut(1.0, 1.0), "dash");

        // And back: the dash straightens into the check.
        app.press("Check")?;
        let items = recording.in_step("press(Check)");
        let spring = spring_on(&items, gravity_node);
        assert_spring(
            &spring,
            SpringToken::DefaultSpatial,
            0.0,
            "gravitation back",
        );
        assert_eq!(started_springs(&items).len(), 1);
        assert_eq!(Mounted::of(&recording.items()).canvas_items(&items), 0);
        Ok(())
    }

    #[test]
    fn from_unchecked_to_indeterminate_the_dash_is_there_at_once_and_draws_straight()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        app.press("Mix")?;
        let items = recording.in_step("press(Mix)");
        // The check's gravitation jumps to the dash (androidx `snap()`, `Checkbox.kt` line 620) while the draw
        // fraction springs, so the dash grows straight.
        assert_eq!(sets(&items), vec![(gravity_node, 1.0)]);
        assert_spring(
            &spring_on(&items, draw_node),
            SpringToken::DefaultSpatial,
            1.0,
            "draw",
        );
        assert_spring(
            &spring_on(&items, color_node),
            SpringToken::DefaultEffects,
            1.0,
            "colors",
        );
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.canvas_items(&items), 0);
        let mut replayed = replay(&recording);
        for frame in 0..60 {
            let at = sample(&mut replayed, &[draw_node], 1).remove(0);
            assert_points(
                mark(mounted.drawing(), &replayed),
                cut(at[0], 1.0),
                &format!("frame {frame}"),
            );
        }
        // Straight: every point of it is on the middle line.
        let points = mark(mounted.drawing(), &replayed);
        assert!(points.iter().all(|point| (point.1 - 9.0).abs() < 1e-3));
        assert!(points[2].0 > 4.5);
        Ok(())
    }

    #[test]
    fn no_change_of_state_draws_the_box_again_or_lays_it_out() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        // The strokes' lengths are nodes of the gravitation, so even the check and the dash trading places leave the
        // drawing as it is.
        let steps = [
            "Check", "Mix", "Check", "Clear", "Mix", "Clear", "Check", "Clear",
        ];
        for (index, name) in steps.into_iter().enumerate() {
            let before = recording.items().len();
            app.press(name)?;
            let items = recording.items().split_off(before);
            assert_eq!(mounted.canvas_items(&items), 0, "{index} {name}");
            assert_eq!(mounted.layout_items(&items), 0, "{index} {name}");
            assert_eq!(layout_items(&items), 0, "{index} {name}");
        }
        Ok(())
    }

    #[test]
    fn the_strokes_are_as_long_as_the_fractions_of_the_check_and_the_dash_say() {
        // The lengths of the strokes from `Checkbox.kt` lines 713 to 724, in an 18-point box: the check's strokes go
        // 0.15 and 0.35 of the side along each axis, the dash's 0.25 and 0.25 along one.
        let (first, second) = stroke_lengths(0.0);
        assert!(
            (first - 18.0 * 0.15 * 2.0_f32.sqrt()).abs() < 1e-4,
            "{first}"
        );
        assert!(
            (second - 18.0 * 0.35 * 2.0_f32.sqrt()).abs() < 1e-4,
            "{second}"
        );
        let (first, second) = stroke_lengths(1.0);
        assert!(
            (first - 4.5).abs() < 1e-4 && (second - 4.5).abs() < 1e-4,
            "{first} {second}"
        );
        // The table holds those lengths at its points and covers a wide range of gravitations.
        let (at, first_lengths, second_lengths) = length_table();
        assert_eq!(at.len(), usize::from(TABLE_STEPS) + 1);
        for (index, gravity) in at.iter().enumerate() {
            assert_eq!(
                (first_lengths[index], second_lengths[index]),
                stroke_lengths(*gravity)
            );
        }
        assert_eq!(at.first().copied(), Some(-0.5));
        assert_eq!(at.last().copied(), Some(1.5));
        assert!(at.contains(&0.0) && at.contains(&1.0));
        // Every stroke has a length at every point of the table: a share is never divided by nothing.
        assert!(
            first_lengths
                .iter()
                .chain(&second_lengths)
                .all(|length| *length > 1.0)
        );
    }

    #[test]
    fn the_table_covers_the_springs_overshoot() {
        // The gravitation goes from one end to the other with `DefaultSpatial`, which overshoots a little; the table
        // reaches much further, so the lengths are never held at its ends.
        let spec = SpringToken::DefaultSpatial.spec();
        let parameters = (
            f64::from(spec.stiffness),
            f64::from(spec.damping_coefficient()),
            f64::from(SPRING_MASS),
        );
        let (mut highest, mut lowest) = (1.0_f64, 0.0_f64);
        for frame in 0..600_u32 {
            let at = spring_position(0.0, 1.0, 0.0, parameters, f64::from(frame) / REPLAY_HZ);
            highest = highest.max(at);
            lowest = lowest.min(at);
            let back = spring_position(1.0, 0.0, 0.0, parameters, f64::from(frame) / REPLAY_HZ);
            highest = highest.max(back);
            lowest = lowest.min(back);
        }
        assert!(highest > 1.0, "the spring overshoots: {highest}");
        assert!(
            highest < f64::from(TABLE_FROM + f32::from(TABLE_STEPS) * TABLE_STEP) - 0.25,
            "{highest}"
        );
        assert!(lowest > f64::from(TABLE_FROM) + 0.25, "{lowest}");
    }

    /// How far `spring`'s value moves in its largest 120 Hz frame, from rest over a unit travel: no frame of a
    /// spring interrupted in the middle moves more (ADR 0010 check 7).
    fn largest_step(token: SpringToken) -> f64 {
        let spec = token.spec();
        let parameters = (
            f64::from(spec.stiffness),
            f64::from(spec.damping_coefficient()),
            f64::from(SPRING_MASS),
        );
        let at =
            |frame: u32| spring_position(0.0, 1.0, 0.0, parameters, f64::from(frame) / REPLAY_HZ);
        (0..240)
            .map(|frame| (at(frame + 1) - at(frame)).abs())
            .fold(0.0, f64::max)
    }

    #[test]
    fn a_new_state_in_the_middle_of_a_fade_goes_back_from_where_and_as_fast_as_the_colors_move()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let [color_node, ..] = value_nodes(&recording);
        app.press("Check")?;
        let down = spring_on(&recording.in_step("press(Check)"), color_node);
        let mut replayed = replay(&recording);
        // 50 ms in, the check is cleared while the colors still move.
        let before = sample(&mut replayed, &[color_node], 6);
        app.press("Clear")?;
        let items = recording.in_step("press(Clear)");
        let up = spring_on(&items, color_node);
        assert_spring(&up, SpringToken::FastEffects, 0.0, "back");
        replayed.apply(&items);
        let after = sample(&mut replayed, &[color_node], 180);

        // The graph last ticked 5 frames into the first fade: the second starts there, at that value and speed.
        let press = parameters(&down);
        let position = |t: f64| spring_position(0.0, 1.0, 0.0, press, t);
        let from = position(5.0 / REPLAY_HZ);
        let h = 1e-6;
        let speed = (position(5.0 / REPLAY_HZ + h) - position(5.0 / REPLAY_HZ - h)) / (2.0 * h);
        assert!(
            (f64::from(before[5][0]) - from).abs() < 1e-3,
            "{before:?} {from}"
        );
        assert!(speed > 5.0, "{speed}");
        let gap = |velocity: f64| {
            after
                .iter()
                .zip(0_u32..)
                .map(|(at, frame)| {
                    let t = f64::from(frame) / REPLAY_HZ;
                    let closed = spring_position(from, 0.0, velocity, parameters(&up), t);
                    (f64::from(at[0]) - closed).abs()
                })
                .fold(0.0, f64::max)
        };
        let allowed = tolerance(from, 0.0);
        assert!(gap(speed) < allowed, "{}", gap(speed));
        // Starting again from rest, as a new animation would, is another curve.
        assert!(gap(0.0) > 10.0 * allowed, "{}", gap(0.0));
        // No frame jumps more than the faster spring's own largest step.
        let track: Vec<f32> = before.iter().chain(&after).map(|at| at[0]).collect();
        let worst = track
            .windows(2)
            .map(|pair| f64::from((pair[1] - pair[0]).abs()))
            .fold(0.0, f64::max);
        assert!(
            worst <= largest_step(SpringToken::FastEffects) + 1e-6,
            "{worst}"
        );
        assert_eq!(
            after.last().map(|at| at[0].to_bits()),
            Some(0.0_f32.to_bits())
        );
        Ok(())
    }

    #[test]
    fn a_check_cleared_while_it_draws_stays_as_it_is_for_the_delay_and_then_goes()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let [_, draw_node, _] = value_nodes(&recording);
        app.press("Check")?;
        let mut replayed = replay(&recording);
        let before = sample(&mut replayed, &[draw_node], 6);
        let drawn = before[5][0];
        assert!(drawn > 0.05 && drawn < 0.95, "{drawn}");
        app.press("Clear")?;
        let items = recording.in_step("press(Clear)");
        // The spring is replaced by the jump, which holds the value for 100 ms (androidx's transition does the same).
        assert_eq!(
            started_timings(&items)
                .iter()
                .filter(|timing| timing.node == draw_node)
                .collect::<Vec<_>>(),
            [&delayed_jump(draw_node, 0.0)]
        );
        replayed.apply(&items);
        let after = sample(&mut replayed, &[draw_node], 30);
        let gone = after.iter().position(|at| at[0] == 0.0).expect("it goes");
        assert!((12..=13).contains(&gone), "{gone}");
        assert!(
            after[..gone]
                .iter()
                .all(|at| at[0].to_bits() == drawn.to_bits())
        );
        assert!(!replayed.is_animating());
        Ok(())
    }

    #[test]
    fn however_often_it_is_pressed_it_ends_in_the_state_the_presses_mean() -> Result<(), String> {
        for presses in 1..=7_u32 {
            let (mut app, recording) = start(Config::default());
            let nodes = value_nodes(&recording);
            let mut replayed = replay(&recording);
            let mut colors = Vec::new();
            for _ in 0..presses {
                let before = recording.items().len();
                app.press("label:Agree")?;
                replayed.apply(&recording.items().split_off(before));
                app.run_for(Duration::from_millis(30))?;
                // 30 ms of the replay's clock, 4 frames, before the next press.
                colors.extend(sample(&mut replayed, &nodes, 4).iter().map(|at| at[0]));
            }
            let expected = if presses % 2 == 1 {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            };
            // The checkbox shows what the app made of the presses, and the values end where that state puts them,
            // exactly, with nothing left running.
            let mounted = Mounted::of(&recording.items());
            assert_eq!(
                mounted.checked,
                Some(if presses % 2 == 1 {
                    Checked::True
                } else {
                    Checked::False
                }),
                "{presses}"
            );
            let settled = sample(&mut replayed, &nodes, 1200);
            let at = targets(expected);
            assert_eq!(
                settled
                    .last()
                    .map(|last| last.iter().map(|v| v.to_bits()).collect::<Vec<_>>()),
                Some([at.color, at.draw, at.gravity].map(f32::to_bits).to_vec()),
                "{presses}"
            );
            assert!(!replayed.is_animating(), "{presses}");
            // However the presses came, the colors never jumped further than the fast spring's largest frame.
            let worst = colors
                .windows(2)
                .map(|pair| f64::from((pair[1] - pair[0]).abs()))
                .fold(0.0, f64::max);
            assert!(
                worst <= largest_step(SpringToken::FastEffects) + 1e-6,
                "{presses}: {worst}"
            );
        }
        Ok(())
    }

    /// The largest distance, in points, between the mark the checkbox draws and the mark `cut` says (the one androidx
    /// measures every frame), over the frames of a checkbox that is sent to `first` and, `after` frames later, to
    /// `second`.
    fn worst_mark_error(first: &str, after: usize, second: &str) -> Result<f32, String> {
        let (mut app, recording) = start(Config::default());
        let [_, draw_node, gravity_node] = value_nodes(&recording);
        app.press(first)?;
        let mut replayed = replay(&recording);
        sample(&mut replayed, &[draw_node], after);
        let before = recording.items().len();
        app.press(second)?;
        replayed.apply(&recording.items().split_off(before));
        let mounted = Mounted::of(&recording.items());
        let mut worst = 0.0_f32;
        for _ in 0..240 {
            let at = sample(&mut replayed, &[draw_node, gravity_node], 1).remove(0);
            let found = mark(mounted.drawing(), &replayed);
            let expected = cut(at[0], at[1]);
            for (a, b) in found.iter().zip(&expected) {
                worst = worst.max((a.0 - b.0).hypot(a.1 - b.1));
            }
        }
        Ok(worst)
    }

    #[test]
    fn a_check_changed_while_it_draws_is_cut_where_androidx_cuts_it() -> Result<(), String> {
        // The check is sent to the dash (and the dash to the check) after 1 to 24 frames of its drawing, while the
        // draw fraction and the gravitation both move: the mark must stay on `PathMeasure`'s cut. The strokes' lengths
        // are read from a table (the graph has no square root): measured at most 0.005 point off over these 16
        // changes, in a debug build, in a box of 18; the bound is twice that. Reading the share of the shape the check
        // goes to, as an earlier version did, was off by up to 2.3 points.
        let mut worst = 0.0_f32;
        for (first, second) in [("Check", "Mix"), ("Mix", "Check")] {
            for after in [1, 2, 3, 4, 6, 8, 12, 24] {
                worst = worst.max(worst_mark_error(first, after, second)?);
            }
        }
        assert!(worst < 0.01, "{worst}");
        Ok(())
    }

    /// The vendored `Checkbox.kt` and the flags file it reads, at the pinned commit (ADR 0012).
    const CHECKBOX_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Checkbox.kt");
    const FLAGS_KT: &str =
        include_str!("../../../reference/androidx-material3-components/ComposeMaterial3Flags.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    /// Whether line `number` of `file` (the first is 1), trimmed, is `text`: the lines this file's documentation cites.
    fn line_is(file: &str, number: usize, text: &str) -> bool {
        file.lines().nth(number.saturating_sub(1)).map(str::trim) == Some(text)
    }

    #[test]
    fn the_flag_and_the_strokes_the_documentation_cites_are_there_in_checkbox_kt() {
        // The flag the checkbox's choice of look rests on, and its default.
        assert!(line_is(
            FLAGS_KT,
            49,
            "public var isCheckboxStylingFixEnabled: Boolean = false"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            497,
            "val isCheckboxStylingFixEnabled = ComposeMaterial3Flags.isCheckboxStylingFixEnabled"
        ));
        // The stroke of the outline and of the mark, which the tokens do not give (ADR 0012).
        assert!(line_is(
            CHECKBOX_KT,
            482,
            "public val StrokeWidth: Dp = 2.dp"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            653,
            "with(LocalDensity.current) { floor(CheckboxDefaults.StrokeWidth.toPx()) }"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            669,
            "stroke = checkmarkStroke ?: Stroke(width = strokeWidthPx, cap = StrokeCap.Square),"
        ));
        assert_eq!(STROKE, Points(2.0));
    }

    #[test]
    fn the_box_and_the_state_layer_the_documentation_cites_are_there_in_checkbox_kt() {
        // The side of the box on the branch of the flag that reads the tokens, the corner and the state layer.
        assert!(cites(
            CHECKBOX_KT,
            &[
                "if (isCheckboxStylingFixEnabled) {",
                "CheckboxTokens.ContainerSize",
                "} else {",
                "CheckboxSize",
                "},",
                "cornerSize = RadiusSize,"
            ]
        ));
        assert!(line_is(CHECKBOX_KT, 520, "CheckboxTokens.ContainerSize"));
        assert!(line_is(CHECKBOX_KT, 1068, "private val RadiusSize"));
        assert!(line_is(CHECKBOX_KT, 1069, "get() = 2.dp"));
        assert_eq!(BOX, checkbox_tokens::ICON_SIZE);
        assert_eq!(corner_radius().to_bits(), 2.0_f32.to_bits());
        assert!(line_is(
            CHECKBOX_KT,
            557,
            "radius = CheckboxTokens.StateLayerSize / 2,"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            566,
            "Modifier.minimumInteractiveComponentSize()"
        ));
    }

    #[test]
    fn the_check_the_documentation_cites_is_there_in_checkbox_kt() {
        // The points of the check, on the branch of the flag that is on, and where they gravitate to.
        assert!(line_is(CHECKBOX_KT, 713, "val checkCrossX = 0.4f"));
        assert!(line_is(
            CHECKBOX_KT,
            714,
            "val checkCrossY = if (isCheckboxStylingFixEnabled) 0.65f else 0.7f"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            715,
            "val leftX = if (isCheckboxStylingFixEnabled) 0.25f else 0.2f"
        ));
        assert!(line_is(CHECKBOX_KT, 716, "val leftY = 0.5f"));
        assert!(line_is(
            CHECKBOX_KT,
            717,
            "val rightX = if (isCheckboxStylingFixEnabled) 0.75f else 0.8f"
        ));
        assert!(line_is(CHECKBOX_KT, 718, "val rightY = 0.3f"));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "val gravitatedCrossX = lerp(checkCrossX, 0.5f, crossCenterGravitation)",
                "val gravitatedCrossY = lerp(checkCrossY, 0.5f, crossCenterGravitation)",
                "// gravitate only Y for end to achieve center line",
                "val gravitatedLeftY = lerp(leftY, 0.5f, crossCenterGravitation)",
                "val gravitatedRightY = lerp(rightY, 0.5f, crossCenterGravitation)",
            ]
        ));
        assert_eq!(
            (CROSS, LEFT, RIGHT, CENTER),
            ((0.4, 0.65), (0.25, 0.5), (0.75, 0.3), 0.5)
        );
        assert!(line_is(
            CHECKBOX_KT,
            734,
            "pathMeasure.getSegment(0f, pathMeasure.length * checkFraction, pathToDraw, true)"
        ));
    }

    #[test]
    fn the_motion_the_documentation_cites_is_there_in_checkbox_kt() {
        // Where the draw fraction and the gravitation go in each state.
        assert!(cites(
            CHECKBOX_KT,
            &[
                "ToggleableState.On -> 1f",
                "ToggleableState.Off -> 0f",
                "ToggleableState.Indeterminate -> 1f",
            ]
        ));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "ToggleableState.On -> 0f",
                "ToggleableState.Off -> 0f",
                "ToggleableState.Indeterminate -> 1f",
            ]
        ));
        // How each moves: springs and jumps (the draw fraction, then the gravitation), and the delay.
        assert!(line_is(
            CHECKBOX_KT,
            596,
            "val defaultAnimationSpec = MotionSchemeKeyTokens.DefaultSpatial.value<Float>()"
        ));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "initialState == ToggleableState.Off -> defaultAnimationSpec",
                "targetState == ToggleableState.Off -> snap(delayMillis = SnapAnimationDelay)",
                "else -> defaultAnimationSpec",
            ]
        ));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "initialState == ToggleableState.Off -> snap()",
                "targetState == ToggleableState.Off -> snap(delayMillis = SnapAnimationDelay)",
                "else -> defaultAnimationSpec",
            ]
        ));
        assert!(line_is(
            CHECKBOX_KT,
            1061,
            "private const val SnapAnimationDelay = 100"
        ));
        assert_eq!(SNAP_DELAY, Duration::from_millis(100));
        // The colors' springs: fast out of the box, the default one into it.
        assert!(cites(
            CHECKBOX_KT,
            &[
                "return if (state == ToggleableState.Off) {",
                "// Box out",
                "MotionSchemeKeyTokens.FastEffects.value()",
                "} else {",
                "// Box in",
                "MotionSchemeKeyTokens.DefaultEffects.value()",
                "}",
            ]
        ));
        // A disabled checkbox's box and border jump (the mark's color does not).
        assert!(cites(
            CHECKBOX_KT,
            &[
                "if (enabled) {",
                "animateColorAsState(boxColor, colorAnimationSpec)",
                "} else {",
                "// If not enabled 'snap' to the disabled state, as there should be no animations between",
                "// enabled / disabled.",
                "rememberUpdatedState(boxColor)",
                "}",
            ]
        ));
        assert!(line_is(
            CHECKBOX_KT,
            634,
            "val animatedCheckboxColor by animateColorAsState(checkmarkColor, colorAnimationSpec)"
        ));
    }

    #[test]
    fn every_color_the_documentation_takes_from_checkbox_kt_is_still_there() {
        // `defaultCheckboxColors` (lines 446 to 476): the transparent colors, and the faded disabled ones.
        assert!(line_is(
            CHECKBOX_KT,
            451,
            "uncheckedCheckmarkColor = Color.Transparent,"
        ));
        assert!(line_is(CHECKBOX_KT, 452, "disabledCheckmarkColor ="));
        assert!(line_is(
            CHECKBOX_KT,
            453,
            "fromToken(CheckboxTokens.SelectedDisabledIconColor),"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            455,
            "uncheckedBoxColor = Color.Transparent,"
        ));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "disabledCheckedBoxColor =",
                "fromToken(CheckboxTokens.SelectedDisabledContainerColor)",
                ".copy(alpha = CheckboxTokens.SelectedDisabledContainerOpacity),",
            ]
        ));
        assert!(cites(
            CHECKBOX_KT,
            &[
                "disabledUncheckedBorderColor =",
                "fromToken(CheckboxTokens.UnselectedDisabledOutlineColor)",
                ".copy(alpha = CheckboxTokens.UnselectedDisabledContainerOpacity),",
            ]
        ));
        assert!(line_is(
            CHECKBOX_KT,
            464,
            "uncheckedBorderColor = fromToken(CheckboxTokens.UnselectedOutlineColor),"
        ));
        assert!(line_is(
            CHECKBOX_KT,
            463,
            "checkedBorderColor = fromToken(CheckboxTokens.SelectedContainerColor),"
        ));
        // The disabled mark has one color in every state (line 940), where the box's two are faded.
        assert!(line_is(CHECKBOX_KT, 940, "disabledCheckmarkColor"));
        // The state layer's color, which is transparent for an unchecked box (the decision in the module docs).
        assert!(cites(
            CHECKBOX_KT,
            &[
                "internal fun indicatorColor(state: ToggleableState): Color =",
                "if (state == ToggleableState.Off) {",
                "uncheckedBoxColor",
                "} else {",
                "checkedBoxColor",
                "}",
            ]
        ));
        // A line that is not there is not found.
        assert!(!cites(CHECKBOX_KT, &["val checkCrossY = 0.65f"]));
        assert!(!line_is(
            FLAGS_KT,
            49,
            "public var isCheckboxStylingFixEnabled: Boolean = true"
        ));
    }

    #[test]
    fn with_reduce_motion_every_change_of_state_goes_there_at_once_and_in_order()
    -> Result<(), String> {
        let (mut app, recording) = start(Config {
            motion: Motion::Reduced,
            ..Config::default()
        });
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        let all = |state| {
            let at = targets(state);
            vec![
                (color_node, at.color),
                (draw_node, at.draw),
                (gravity_node, at.gravity),
            ]
        };
        // Each press, what it jumps (the colors, the draw fraction, the gravitation, in this order, and only what
        // moves: nothing for a value androidx would leave where it is), and the state it ends in.
        let steps = [
            ("Check", all(CheckboxState::Checked), CheckboxState::Checked),
            (
                "Mix",
                vec![(gravity_node, 1.0)],
                CheckboxState::Indeterminate,
            ),
            (
                "Clear",
                all(CheckboxState::Unchecked),
                CheckboxState::Unchecked,
            ),
            (
                "Mix",
                all(CheckboxState::Indeterminate),
                CheckboxState::Indeterminate,
            ),
            ("Check", vec![(gravity_node, 0.0)], CheckboxState::Checked),
            (
                "Clear",
                all(CheckboxState::Unchecked),
                CheckboxState::Unchecked,
            ),
        ];
        let mut replayed = replay(&recording);
        let mounted = Mounted::of(&recording.items());
        for (index, (name, jumps, state)) in steps.into_iter().enumerate() {
            let before = recording.items().len();
            app.press(name)?;
            let items = recording.items().split_off(before);
            let what = format!("{index} {name}");
            // No spring, and no delayed jump: a delay is motion too.
            assert_eq!(started_springs(&items), Vec::new(), "{what}");
            assert_eq!(started_timings(&items), Vec::new(), "{what}");
            assert_eq!(sets(&items), jumps, "{what}");
            // The values are where the state puts them before the graph ticks once, and nothing is left running.
            replayed.apply(&items);
            let at = targets(state);
            let found = value_nodes(&recording).map(|node| replayed.value(node));
            assert_eq!(
                found.map(f32::to_bits),
                [at.color, at.draw, at.gravity].map(f32::to_bits),
                "{what}"
            );
            assert!(!replayed.is_animating(), "{what}");
            assert_points(
                mark(mounted.drawing(), &replayed),
                cut(at.draw, at.gravity),
                &what,
            );
            assert_eq!(
                paints(mounted.drawing(), &replayed),
                expected_colors(state, false, false),
                "{what}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_disabled_checkbox_moves_its_colors_at_once_and_still_draws_its_mark() -> Result<(), String>
    {
        let (mut app, recording) = start(Config {
            disabled: true,
            ..Config::default()
        });
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        let mounted = Mounted::of(&recording.items());
        let mut replayed = replay(&recording);
        // Into the box: androidx's disabled box and border jump (lines 633 to 650), and the mark draws with the
        // spatial spring.
        app.press("Check")?;
        let items = recording.in_step("press(Check)");
        assert_eq!(sets(&items), vec![(color_node, 1.0), (gravity_node, 0.0)]);
        assert_spring(
            &spring_on(&items, draw_node),
            SpringToken::DefaultSpatial,
            1.0,
            "draw",
        );
        assert_eq!(started_springs(&items).len(), 1);
        assert_eq!(started_timings(&items), Vec::new());
        replayed.apply(&items);
        assert_eq!(
            paints(mounted.drawing(), &replayed),
            expected_colors(CheckboxState::Checked, true, false)
        );
        assert!(replayed.is_animating());
        sample(&mut replayed, &[draw_node], 600);
        assert_points(mark(mounted.drawing(), &replayed), cut(1.0, 0.0), "checked");
        assert!(!replayed.is_animating());
        // Out of it: the colors jump, and the mark, which has no color left to show, goes after its delay as an
        // enabled one does.
        app.press("Clear")?;
        let items = recording.in_step("press(Clear)");
        assert_eq!(sets(&items), vec![(color_node, 0.0)]);
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(
            started_timings(&items),
            vec![
                delayed_jump(draw_node, 0.0),
                delayed_jump(gravity_node, 0.0)
            ]
        );
        replayed.apply(&items);
        assert_eq!(
            paints(mounted.drawing(), &replayed),
            expected_colors(CheckboxState::Unchecked, true, false)
        );
        assert_points(mark(mounted.drawing(), &replayed), cut(1.0, 0.0), "waiting");
        sample(&mut replayed, &[draw_node], 30);
        assert_points(mark(mounted.drawing(), &replayed), cut(0.0, 0.0), "gone");
        assert!(!replayed.is_animating());
        Ok(())
    }

    #[test]
    fn disabling_in_the_middle_of_a_fade_takes_the_colors_to_where_they_go_at_once()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let [color_node, draw_node, gravity_node] = value_nodes(&recording);
        app.press("Check")?;
        let mut replayed = replay(&recording);
        sample(&mut replayed, &[color_node], 5);
        let fading = replayed.value(color_node);
        assert!(fading > 0.05 && fading < 0.95, "{fading}");
        let drawn = replayed.value(draw_node);

        let before = recording.items().len();
        app.press("Disable")?;
        let items = recording.items().split_off(before);
        // The colors jump to the checked ones, and nothing else is started or stopped: the mark goes on drawing.
        assert_eq!(sets(&items), vec![(color_node, 1.0)]);
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(started_timings(&items), Vec::new());
        let mounted = Mounted::of(&recording.items());
        // The box is drawn again once, with the disabled colors.
        assert_eq!(mounted.canvas_items(&items), 1);
        // The state layer is taken out of the layout (it shows nothing on a disabled checkbox) and nothing else is.
        let framed: Vec<NodeId> = items
            .iter()
            .filter_map(|item| match item {
                MountItem::Frame { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(framed, vec![mounted.layer]);
        assert_eq!(mounted.frame(mounted.layer).size.width, px(0.0));
        replayed.apply(&items);
        assert_eq!(replayed.value(color_node).to_bits(), 1.0_f32.to_bits());
        assert_eq!(
            paints(mounted.drawing(), &replayed),
            expected_colors(CheckboxState::Checked, true, false)
        );
        assert!(replayed.is_animating());
        sample(&mut replayed, &[draw_node], 1);
        assert!(
            replayed.value(draw_node) > drawn,
            "the check goes on drawing"
        );
        sample(&mut replayed, &[draw_node], 600);
        let settled =
            [color_node, draw_node, gravity_node].map(|node| replayed.value(node).to_bits());
        assert_eq!(settled, [1.0_f32, 1.0, 0.0].map(f32::to_bits));
        assert!(!replayed.is_animating());

        // Enabled again, the colors are the enabled ones with no motion at all.
        let before = recording.items().len();
        app.press("Enable")?;
        let items = recording.items().split_off(before);
        assert_eq!(sets(&items), Vec::new());
        assert_eq!(started_springs(&items), Vec::new());
        assert_eq!(started_timings(&items), Vec::new());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.canvas_items(&items), 1);
        replayed.apply(&items);
        assert_eq!(
            paints(mounted.drawing(), &replayed),
            expected_colors(CheckboxState::Checked, false, false)
        );
        assert!(!replayed.is_animating());
        Ok(())
    }
}
