//! The Material 3 Expressive floating action button (FAB): an [`Icon`] in a container of four sizes and two color
//! styles, and the state layer of hover, focus and press.
//!
//! The FAB follows androidx's `FloatingActionButton`, `SmallFloatingActionButton`, `MediumFloatingActionButton` and
//! `LargeFloatingActionButton` in `FloatingActionButton.kt`, at the pinned commit, as [ADR 0012] says. The extended
//! FAB and the FAB menu are other components.
//!
//! # Where this FAB follows `FloatingActionButton.kt` and not the token files
//!
//! Each of these is a value of `FloatingActionButtonDefaults` that is not the token of its size ([ADR 0012]). The
//! module's tests pin each one to the line it cites.
//!
//! - **The large icon is 36 points**, not the 32 of `FabLargeTokens.IconSize`: `LargeIconSize` is written
//!   `36.dp // TODO: FabLargeTokens.IconSize is incorrect` (line 1010). `LARGE_ICON_SIZE` holds it.
//! - **The medium shape is `CornerLargeIncreased`** (20 points): `FabMediumTokens` has no container shape (its
//!   `ContainerShape` is commented out), and `mediumShape` is `ShapeDefaults.LargeIncreased` under "TODO: update to
//!   use token" (line 1022). `ShapeDefaults.LargeIncreased` is `ShapeTokens.CornerLargeIncreased` in androidx's
//!   `Shapes.kt` (line 292 at the pinned commit). That file is not vendored: it was read once, there, and the
//!   generated token holds the value. The commented-out line of `FabMediumTokens.kt` names the same key.
//!
//! The medium icon is `FabMediumTokens.IconSize` (`MediumIconSize`, line 1007), and the small and the default FAB's
//! icons are their tokens' 24 points, which is also the size of the `Icon` androidx's samples put in them.
//!
//! # Color styles
//!
//! androidx's FAB takes the container color of `FabPrimaryContainerTokens` by default
//! (`FloatingActionButtonDefaults.containerColor`) and the content color that goes with it, the icon color of the
//! same file. [`FabStyle::SecondaryContainer`] is the same with `FabSecondaryContainerTokens`, which the vendored
//! token files have and `FloatingActionButton.kt` does not read. A tertiary or a surface FAB token file: not found
//! at the pinned commit, so there is no such style. Neither style has a disabled state: androidx's FAB has no
//! `enabled`.
//!
//! # Shape
//!
//! The FAB's shape does not change when it is pressed: androidx's FAB has no pressed shape (the button's
//! `shapes.pressedShape` has no counterpart in `FloatingActionButton.kt`). So its container is a plain view with a
//! corner radius, and not the drawn container of [ADR 0011], which exists to move a radius with a spring. The shadow
//! of the elevation follows that radius with no extra work.
//!
//! # Touch target
//!
//! The 40-point small FAB takes presses 4 points around it, so that its target is 48 points, as androidx's
//! `minimumInteractiveComponentSize` does for the components that are smaller; its layout and look do not change
//! (NSH's accessibility guide: extend the touch area with the hit slop, not the size).
//!
//! # Showing and hiding
//!
//! [`FabVisibility`] shows or hides the FAB with an animation, as androidx's `Modifier.animateFloatingActionButton`
//! does (`FabVisibleNode`, lines 1185 to 1274): the scale springs with the fast spatial spring of the motion scheme
//! and the opacity with the fast effects spring, both converted as [ADR 0004] says. The scale is
//! `lerp(ShowHideTargetScale, 1, progress)` with no clamp, so the spatial spring's overshoot makes the FAB larger than
//! its size for a moment when it appears. `ShowHideTargetScale` is 0.2 (`SHOW_HIDE_TARGET_SCALE`, line 1004: the one
//! literal besides the large icon's that comes from the component). Once it has faded out, a hidden FAB is not laid out,
//! as androidx lays it out at 0 by 0 when its alpha is 0. With reduce motion on, it appears and disappears at once.
//! Not ported: `alignment`, the corner the scale goes toward; the FAB scales around its center.
//!
//! # Elevation
//!
//! The container casts the shadow of its elevation ([ADR 0018]): the tokens give its length in each state
//! ([`FabStyle::elevation`]), [`FabStyle::level`] finds the level of the elevation scale that length is, and the
//! container carries that level's two shadow layers ([`Elevation::shadows`]). No shadow value is written here. The
//! levels are level 3 at rest, focused and pressed, and level 4 under the pointer, for both styles.
//!
//! - **A token that is not a level cannot be built.** [`FabStyle::level`] reads constants that are worked out when the
//!   crate is compiled: an elevation token that is none of the six levels' lengths fails the build, so there is no
//!   FAB with a made-up level and no arm of the code that guesses one. (`Elevation::from_points` is what a test ties
//!   them to; it cannot run in a constant.)
//! - **A change of level snaps.** The container's shadow changes at once when the state does. androidx animates the
//!   elevation between its states (`FloatingActionButtonElevationAnimatable`, line 1373, with `animateElevation`);
//!   whether this library follows that animation or the rule that every motion is a spring of the motion scheme is
//!   not decided, so none is invented.
//! - **The states are NSH's variants of the container**, `hover`, `focus_within` and `active`, resolved by the platform
//!   with nothing running. When several apply, NSH's order of the variants decides: the pressed level over the hovered
//!   one over the focused one. androidx takes the interaction that came last (`interactions.lastOrNull()`, line
//!   1346), which the variants do not record.
//!
//! [ADR 0004]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0004-spring-damping-conversion.md
//! [ADR 0018]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0018-draw-elevation-as-two-box-shadows-per-level.md
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::geometry::{Points, Size};
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::elevation::Elevation;
use crate::icon::Icon;
use crate::morph::{Corners, corners};
use crate::motion::{SPRING_MASS, SpringToken};
use crate::tint::{MIN_TOUCH_TARGET, role_color};
use crate::tokens::Shape;
use crate::tokens::androidx::{
    fab_baseline_tokens, fab_large_tokens, fab_medium_tokens, fab_primary_container_tokens,
    fab_secondary_container_tokens, fab_small_tokens, shape_tokens, state_tokens,
};

/// The size of a [`Fab`]: the four of androidx's `FloatingActionButton.kt`, from 40 to 96 points.
///
/// Each size takes its container, icon size and shape from its token file (`FabSmallTokens`, `FabBaselineTokens`,
/// `FabMediumTokens` and `FabLargeTokens`), except where `FloatingActionButton.kt` writes its own (this module's
/// documentation lists them).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum FabSize {
    /// 40 points square, a 24-point icon: androidx `SmallFloatingActionButton`.
    Small,
    /// 56 points square, a 24-point icon: androidx `FloatingActionButton`, its default.
    #[default]
    Baseline,
    /// 80 points square, a 28-point icon: androidx `MediumFloatingActionButton`.
    Medium,
    /// 96 points square, a 36-point icon: androidx `LargeFloatingActionButton`.
    Large,
}

/// The color style of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum FabStyle {
    /// A container in the primary container color, its icon in the color on it: androidx's default
    /// (`FabPrimaryContainerTokens`).
    #[default]
    PrimaryContainer,
    /// A container in the secondary container color, its icon in the color on it
    /// (`FabSecondaryContainerTokens`).
    SecondaryContainer,
}

/// Whether a [`Fab`] is shown: androidx's `visible` of `Modifier.animateFloatingActionButton`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum FabVisibility {
    /// At its size, in its place.
    #[default]
    Shown,
    /// Scaled down and faded out; once it has faded out it takes no place in the layout.
    Hidden,
}

impl FabVisibility {
    /// How far in the FAB is when it settles: 1 shown, 0 hidden (androidx `FabVisibleNode`'s `if (visible) 1f else
    /// 0f`, lines 1193, 1246 and 1256).
    const fn progress(self) -> f32 {
        match self {
            FabVisibility::Shown => 1.0,
            FabVisibility::Hidden => 0.0,
        }
    }

    /// Whether the FAB takes its place once it has settled.
    const fn presence(self) -> Presence {
        match self {
            FabVisibility::Shown => Presence::InLayout,
            FabVisibility::Hidden => Presence::OutOfLayout,
        }
    }
}

/// What a [`Fab`] is going through: the interactions that change its elevation (androidx's
/// `FloatingActionButtonElevation`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FabState {
    /// Nothing: no pointer over it, no focus, no press.
    #[default]
    Rest,
    /// A pointer is over it (androidx `HoverInteraction.Enter`).
    Hovered,
    /// It has the focus (androidx `FocusInteraction.Focus`).
    Focused,
    /// A press is down on it (androidx `PressInteraction.Press`).
    Pressed,
}

impl FabStyle {
    /// How far above the surface the container sits in `state`, from the style's token file
    /// (`ContainerElevation`, `HoveredContainerElevation`, `FocusedContainerElevation` and
    /// `PressedContainerElevation`): the level 3 of the elevation tokens at rest, focused and pressed, and level 4
    /// hovered, for both styles.
    ///
    /// This is the token's length; [`FabStyle::level`] is the level of the elevation scale it is, which the
    /// container casts the shadow of.
    ///
    /// ```
    /// use nsh_m3e::fab::{FabState, FabStyle};
    ///
    /// let style = FabStyle::PrimaryContainer;
    /// assert_eq!(style.elevation(FabState::Rest).0, 6.0);
    /// assert!(style.elevation(FabState::Hovered) > style.elevation(FabState::Rest));
    /// ```
    #[must_use]
    pub const fn elevation(self, state: FabState) -> Points {
        match (self, state) {
            (FabStyle::PrimaryContainer, FabState::Rest) => {
                fab_primary_container_tokens::CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Hovered) => {
                fab_primary_container_tokens::HOVERED_CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Focused) => {
                fab_primary_container_tokens::FOCUSED_CONTAINER_ELEVATION
            }
            (FabStyle::PrimaryContainer, FabState::Pressed) => {
                fab_primary_container_tokens::PRESSED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Rest) => {
                fab_secondary_container_tokens::CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Hovered) => {
                fab_secondary_container_tokens::HOVERED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Focused) => {
                fab_secondary_container_tokens::FOCUSED_CONTAINER_ELEVATION
            }
            (FabStyle::SecondaryContainer, FabState::Pressed) => {
                fab_secondary_container_tokens::PRESSED_CONTAINER_ELEVATION
            }
        }
    }

    /// The level of the elevation scale the container sits at in `state`: [`FabStyle::elevation`]'s length as a
    /// level, whose [two shadow layers](Elevation::shadows) the container casts.
    ///
    /// The levels are worked out when the crate is compiled, from the same tokens: a token that is none of the six
    /// levels' lengths fails the build, so a FAB never has a level that is not its token's.
    ///
    /// ```
    /// use nsh_m3e::elevation::Elevation;
    /// use nsh_m3e::fab::{FabState, FabStyle};
    ///
    /// let style = FabStyle::SecondaryContainer;
    /// assert_eq!(style.level(FabState::Rest), Elevation::Level3);
    /// assert_eq!(style.level(FabState::Hovered), Elevation::Level4);
    /// ```
    #[must_use]
    pub const fn level(self, state: FabState) -> Elevation {
        let levels = match self {
            FabStyle::PrimaryContainer => PRIMARY_LEVELS,
            FabStyle::SecondaryContainer => SECONDARY_LEVELS,
        };
        match state {
            FabState::Rest => levels.rest,
            FabState::Hovered => levels.hovered,
            FabState::Focused => levels.focused,
            FabState::Pressed => levels.pressed,
        }
    }
}

/// The level of each state of one style.
#[derive(Debug, Clone, Copy)]
struct Levels {
    rest: Elevation,
    hovered: Elevation,
    focused: Elevation,
    pressed: Elevation,
}

/// The levels of `style`, worked out from its tokens.
const fn levels(style: FabStyle) -> Levels {
    Levels {
        rest: level_of(style.elevation(FabState::Rest)),
        hovered: level_of(style.elevation(FabState::Hovered)),
        focused: level_of(style.elevation(FabState::Focused)),
        pressed: level_of(style.elevation(FabState::Pressed)),
    }
}

/// The levels of the primary container style, worked out when the crate is compiled.
const PRIMARY_LEVELS: Levels = levels(FabStyle::PrimaryContainer);

/// The levels of the secondary container style, worked out when the crate is compiled.
const SECONDARY_LEVELS: Levels = levels(FabStyle::SecondaryContainer);

/// The level whose length is `points`, exactly: the levels' own lengths are compared bit for bit, as
/// `Elevation::from_points` compares them, so no length is taken for its neighbor.
///
/// Only the constants above call it, so that it runs while the crate is compiled and a length that is none of the
/// six levels' fails the build (the assertion). Called with such a length at run time it would panic: it is private
/// for that reason.
const fn level_of(points: Points) -> Elevation {
    let bits = points.0.to_bits();
    if bits == Elevation::Level0.points().0.to_bits() {
        Elevation::Level0
    } else if bits == Elevation::Level1.points().0.to_bits() {
        Elevation::Level1
    } else if bits == Elevation::Level2.points().0.to_bits() {
        Elevation::Level2
    } else if bits == Elevation::Level3.points().0.to_bits() {
        Elevation::Level3
    } else if bits == Elevation::Level4.points().0.to_bits() {
        Elevation::Level4
    } else {
        assert!(
            bits == Elevation::Level5.points().0.to_bits(),
            "an elevation token of a FAB is not a level of the elevation scale"
        );
        Elevation::Level5
    }
}

/// What a size gives the FAB.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SizeSpec {
    /// The container's width: androidx's `sizeIn(minWidth = Fab*Tokens.ContainerWidth)`.
    width: Points,
    /// The container's height.
    height: Points,
    /// The icon's side.
    icon: Points,
    /// The container's shape, which no state changes.
    shape: Shape,
}

/// The large FAB's icon, 36 points: androidx `FloatingActionButtonDefaults.LargeIconSize`, which
/// `FloatingActionButton.kt` writes as `36.dp // TODO: FabLargeTokens.IconSize is incorrect` (line 1010), where the
/// token says 32 (ADR 0012).
const LARGE_ICON_SIZE: Points = Points(36.0);

/// The medium FAB's shape: androidx `FloatingActionButtonDefaults.mediumShape`, which is
/// `ShapeDefaults.LargeIncreased` under "TODO: update to use token" (`FloatingActionButton.kt` line 1022). That is
/// `ShapeTokens.CornerLargeIncreased` (`Shapes.kt` line 292, not vendored), and the key that the commented-out
/// `ContainerShape` of `FabMediumTokens.kt` names (ADR 0012).
const MEDIUM_SHAPE: Shape = shape_tokens::CORNER_LARGE_INCREASED;

/// The values of `size`, from its token file, or from the component where it overrides the token.
fn size_spec(size: FabSize) -> SizeSpec {
    match size {
        FabSize::Small => SizeSpec {
            width: fab_small_tokens::CONTAINER_WIDTH,
            height: fab_small_tokens::CONTAINER_HEIGHT,
            icon: fab_small_tokens::ICON_SIZE,
            shape: fab_small_tokens::CONTAINER_SHAPE,
        },
        FabSize::Baseline => SizeSpec {
            width: fab_baseline_tokens::CONTAINER_WIDTH,
            height: fab_baseline_tokens::CONTAINER_HEIGHT,
            icon: fab_baseline_tokens::ICON_SIZE,
            shape: fab_baseline_tokens::CONTAINER_SHAPE,
        },
        FabSize::Medium => SizeSpec {
            width: fab_medium_tokens::CONTAINER_WIDTH,
            height: fab_medium_tokens::CONTAINER_HEIGHT,
            icon: fab_medium_tokens::ICON_SIZE,
            shape: MEDIUM_SHAPE,
        },
        FabSize::Large => SizeSpec {
            width: fab_large_tokens::CONTAINER_WIDTH,
            height: fab_large_tokens::CONTAINER_HEIGHT,
            icon: LARGE_ICON_SIZE,
            shape: fab_large_tokens::CONTAINER_SHAPE,
        },
    }
}

/// What a color style gives the FAB: the role of its container and of its icon.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StyleSpec {
    container: ColorRole,
    /// The icon, and the state layer over the container, which is the icon's color as androidx's ripple is the
    /// content color.
    icon: ColorRole,
}

/// The colors of `style`. A state does not change them: each style's token file gives the same icon color at rest,
/// hovered, focused and pressed, and `every_state_has_the_same_icon_color` would say if that changed.
fn style_spec(style: FabStyle) -> StyleSpec {
    match style {
        FabStyle::PrimaryContainer => StyleSpec {
            container: fab_primary_container_tokens::CONTAINER_COLOR,
            icon: fab_primary_container_tokens::ICON_COLOR,
        },
        FabStyle::SecondaryContainer => StyleSpec {
            container: fab_secondary_container_tokens::CONTAINER_COLOR,
            icon: fab_secondary_container_tokens::ICON_COLOR,
        },
    }
}

/// How far presses also count around a container of `size`, so that a FAB smaller than [`MIN_TOUCH_TARGET`] on a side
/// still has a 48-point target there (androidx `minimumInteractiveComponentSize`), without changing its M3 size or the
/// layout (NSH's accessibility guide: extend the touch area with the hit slop, not the size). Like the icon button's
/// and the button's.
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

/// `style` with the corners that `shape` gives a container: rounded by the shape's radius, or by half the shorter
/// side.
fn rounded<S: Styled>(style: S, shape: Shape) -> S {
    match corners(shape) {
        Corners::Full => style.rounded_full(),
        Corners::Radius(radius) => style.rounded(radius),
    }
}

/// The scale a hidden FAB has: androidx `FloatingActionButtonDefaults.ShowHideTargetScale`
/// (`FloatingActionButton.kt` line 1004), "the initial scale value when showing the FAB and the final scale value when
/// hiding the FAB" (line 1127).
const SHOW_HIDE_TARGET_SCALE: f32 = 0.2;

/// The spring that shows and hides the FAB, of `token`: androidx `FabVisibleNode` plays the scale with the motion
/// scheme's `fastSpatialSpec` and the alpha with its `fastEffectsSpec` (lines 1245 to 1260), converted as ADR 0004
/// says. Every field is written out, none taken from NSH's defaults: it keeps the speed the value already has (a show
/// that interrupts a hide goes on from there) and asks for the screen's own rate (ADR 0010).
fn reveal_spring(token: SpringToken) -> Spring {
    let spec = token.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// Whether a FAB takes its place in the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presence {
    /// Laid out at its size.
    InLayout,
    /// Not laid out: androidx lays a FAB out at 0 by 0 once its alpha is 0 (`FabVisibleNode.measure`, line 1268).
    OutOfLayout,
}

/// What shows and hides a FAB: the two animated values androidx's `FabVisibleNode` has, where the FAB is going and
/// whether it is laid out. A `Copy` handle, like the signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Reveal {
    /// From 0 (hidden) to 1 (shown), on the fast spatial spring: what the scale is made from
    /// (`scaleAnimatable`).
    progress: AnimatedValue,
    /// From 0 to 1, on the fast effects spring: the opacity (`alphaAnimatable`).
    alpha: AnimatedValue,
    /// What the FAB was last asked to be.
    target: Signal<FabVisibility>,
    presence: Signal<Presence>,
}

impl Reveal {
    /// A FAB that is `visibility` already: nothing animates at first (`Animatable(if (visible) 1f else 0f)`).
    fn new(visibility: FabVisibility) -> Self {
        Self {
            progress: animated_value(visibility.progress()),
            alpha: animated_value(visibility.progress()),
            target: signal(visibility),
            presence: signal(visibility.presence()),
        }
    }

    /// The scale, `lerp(targetScale, 1f, progress)` (line 1218). Nothing clamps it, as in androidx: the spatial
    /// spring's overshoot makes it larger than 1.
    fn scale(self) -> AnimatedNode {
        self.progress
            .mul(1.0 - SHOW_HIDE_TARGET_SCALE)
            .add(SHOW_HIDE_TARGET_SCALE)
    }

    /// The FAB is asked to be `to`. Nothing happens when it already is, or is going to be. Otherwise both values
    /// spring there from where they are; a hidden FAB leaves the layout when its opacity has reached 0, and a FAB
    /// that is shown again before that stays where it is. With reduce motion on, both values and the layout change
    /// at once and nothing springs.
    ///
    /// The task that waits for the end of a hide ends with the effect that calls this, which runs again only when the
    /// FAB is asked for the other visibility: that ask supersedes the hide.
    fn go(self, to: FabVisibility) {
        untracked(|| {
            if self.target.get() == to {
                return;
            }
            self.target.set(to);
            if reduce_motion() {
                self.progress.set(to.progress());
                self.alpha.set(to.progress());
                self.presence.set(to.presence());
                return;
            }
            if to == FabVisibility::Shown {
                self.presence.set(Presence::InLayout);
            }
            let _ = self
                .progress
                .spring(to.progress(), reveal_spring(SpringToken::FastSpatial));
            let faded = self
                .alpha
                .spring(to.progress(), reveal_spring(SpringToken::FastEffects));
            if to == FabVisibility::Hidden {
                let presence = self.presence;
                // `false` is an opacity that was stopped or replaced before it got to 0.
                drop(spawn(async move {
                    if faded.await {
                        presence.set(Presence::OutOfLayout);
                    }
                }));
            }
        });
    }
}

/// The props of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct FabProps {
    /// The icon it shows.
    pub icon: Prop<Icon>,
    /// What screen readers say: a FAB shows no text, so this is its only name.
    pub label: Prop<SharedString>,
    /// Its size.
    #[prop(default)]
    pub size: Prop<FabSize>,
    /// Its color style.
    #[prop(default)]
    pub style: Prop<FabStyle>,
    /// Whether it is shown. Asking for the other visibility scales and fades it out or in with M3E's springs; a FAB
    /// that has faded out takes no place in the layout.
    #[prop(default)]
    pub visibility: Prop<FabVisibility>,
}

/// The events of a [`Fab`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum FabEvent {
    /// The FAB was pressed.
    Press,
}

/// A Material 3 floating action button: an icon in a container of the size and color style given.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the button role, named by its label for screen
/// readers) and the look of androidx's `FloatingActionButton`. Hover, focus and press lay the icon's color over it at
/// the state layer opacities of `StateTokens`, as androidx's ripple does, and change nothing else, so nothing is laid
/// out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::fab::{FabSize, FabVisibility, fab};
/// use nsh_m3e::icon::Icon;
///
/// fn add(scrolled_down: ReadSignal<bool>) -> impl IntoElement {
///     fab()
///         // Material Symbols' `add`.
///         .icon(Icon::material_symbol(
///             "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z",
///         ))
///         .label("Add")
///         .size(FabSize::Large)
///         // Hidden with M3E's springs while the content scrolls down, shown again when it scrolls up.
///         .visibility(move || {
///             if scrolled_down.get() {
///                 FabVisibility::Hidden
///             } else {
///                 FabVisibility::Shown
///             }
///         })
///         .on_press(|| println!("added"))
/// }
/// ```
#[component]
pub fn Fab(props: FabProps, cx: Scope<FabEvent>) -> impl IntoElement {
    let size = move || size_spec(props.size.get());
    let colors = move || style_spec(props.style.get());
    let frame = move || {
        let size = size();
        Size::new(size.width, size.height)
    };
    let icon_color = move || role_color(colors().icon);
    // A memo, so that the effect below runs when the visibility changes and not when it is only set again.
    let visibility = computed(move || props.visibility.get());
    let reveal = Reveal::new(untracked(|| visibility.get()));
    effect(move || reveal.go(visibility.get()));
    // The two shadow layers of the container's level in `state` (ADR 0018), read from the style as it is now.
    let shadows = move |state| props.style.get().level(state).shadows();
    nsh_base::button()
        .accessibility_label(move || Some(props.label.get()))
        .on_press(move || cx.emit(FabEvent::Press))
        .hit_slop(move || touch_slop(frame()))
        .group()
        .items_center()
        .justify_center()
        .w(move || frame().width)
        .h(move || frame().height)
        .bg(move || role_color(colors().container))
        .style_with(move |s| {
            let s = rounded(s, size().shape).shadow(shadows(FabState::Rest));
            match reveal.presence.get() {
                Presence::InLayout => s,
                Presence::OutOfLayout => s.hidden(),
            }
        })
        // The shadow of each state the container is in: NSH resolves them natively, with nothing running, and a
        // change of level snaps (this module's documentation says why). Each is bound to the style prop, which
        // `hover` and the others would read once if they took the shadows themselves.
        .hover(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Hovered))))
        .focus_within(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Focused))))
        .active(move |s| s.style_with(move |s| s.shadow(shadows(FabState::Pressed))))
        .bind_opacity(&reveal.alpha)
        .bind_scale(&reveal.scale())
        .child(
            // The state layer: the icon's color over the container, in its shape, shown by the button's own states
            // with nothing running (androidx's ripple, `RippleDefaults.RippleAlpha`, in the content color).
            view()
                .absolute()
                .inset(px(0.0))
                .bg(icon_color)
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| rounded(s, size().shape)),
        )
        .child(
            // The icon, centered at its size, in the content color.
            canvas(move || props.icon.get().drawing(size().icon))
                .w(move || size().icon)
                .h(move || size().icon)
                .fill_color(icon_color),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use nsh_core::animation::{AnimationOp, AnimationProp, NodeId as AnimationNode};
    use nsh_core::canvas::{self, CanvasOp, Drawing, Num};
    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, BoxShadow, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;
    use proptest::prelude::*;

    use super::*;
    use crate::animation_probe::{
        REPLAY_HZ, Recording, Replay, StartedSpring, layout_items, spring_position,
        started_springs, worst_gap,
    };
    use crate::color::ColorScheme;
    use crate::morph_probe::{parameters, replay, set_values, tolerance};

    /// Material Symbols' `add` (google/material-design-icons at `10ecc147906`, `symbols/web/add/
    /// materialsymbolsoutlined/add_24px.svg`): test data, the `d` of its one path.
    const ADD: &str = "M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z";

    /// What the probe places: one FAB with these props, a line counting its presses, and two texts that change its
    /// visibility.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        size: Prop<FabSize>,
        #[prop(default)]
        style: Prop<FabStyle>,
        #[prop(default)]
        visibility: Prop<FabVisibility>,
    }

    /// The visibility other than `visibility`.
    fn other(visibility: FabVisibility) -> FabVisibility {
        match visibility {
            FabVisibility::Shown => FabVisibility::Hidden,
            _ => FabVisibility::Shown,
        }
    }

    /// How long `Blink` waits before it undoes what it did: 6 display frames, so that a spring is still moving.
    const BLINK: Duration = Duration::from_millis(50);

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        let visibility = signal(untracked(|| props.visibility.get()));
        // A row, so that the FAB keeps its own size, not stretched across a column.
        view()
            .child(
                view().flex_row().child(
                    fab()
                        .icon(Icon::material_symbol(ADD))
                        .label("Add")
                        .size(props.size)
                        .style(props.style)
                        .visibility(move || visibility.get())
                        .on_press(move || presses.update(|n| *n += 1)),
                ),
            )
            .child(text(move || format!("presses: {}", presses.get())))
            // Asks for the other visibility.
            .child(text("Flip").on_press(move |(): &()| {
                visibility.update(|visibility| *visibility = other(*visibility));
            }))
            // Asks for the other visibility, and for the first again `BLINK` later.
            .child(text("Blink").on_press(move |(): &()| {
                let before = visibility.get_untracked();
                visibility.set(other(before));
                drop(set_timeout(BLINK, move || visibility.set(before)));
            }))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        size: FabSize,
        style: FabStyle,
        visibility: FabVisibility,
        dark: bool,
        reduce_motion: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            size: Prop::new(config.size),
            style: Prop::new(config.style),
            visibility: Prop::new(config.visibility),
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

    /// The FAB's views, and the last style, frame and drawing each was sent.
    struct Mounted {
        /// The press target, with the button role: the whole FAB, which is its container.
        button: NodeId,
        /// The view of the state layer.
        layer: NodeId,
        /// The canvas that draws the icon.
        icon: NodeId,
        styles: HashMap<NodeId, ViewStyle>,
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
            let inside: Vec<NodeId> = children.get(&button).cloned().unwrap_or_default();
            // The state layer, under the icon; and nothing else in the button.
            let [layer, icon] = inside[..] else {
                panic!("two views in the FAB: {items:?}");
            };
            assert_eq!(kinds.get(&layer), Some(&ViewKind::View), "{items:?}");
            assert_eq!(kinds.get(&icon), Some(&ViewKind::Canvas), "{items:?}");
            Self {
                button,
                layer,
                icon,
                styles,
                frames,
                drawings,
                label,
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

        /// The FAB's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.button].size
        }

        /// How many of `items` lay the FAB out again: one of its views' frames.
        fn layout_items(&self, items: &[MountItem]) -> usize {
            let ours = [self.button, self.layer, self.icon];
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

    /// The opaque color of `role` in the baseline scheme.
    fn solid(role: ColorRole, dark: bool) -> Color {
        color(scheme_color(role, dark), 255)
    }

    /// The color `ops` fill with, if any.
    fn fill(ops: &[CanvasOp]) -> Option<Color> {
        ops.iter().find_map(|op| match op {
            CanvasOp::FillStyle(canvas::Paint::Color(color)) => Some(*color),
            _ => None,
        })
    }

    const STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

    const SIZES: [FabSize; 4] = [
        FabSize::Small,
        FabSize::Baseline,
        FabSize::Medium,
        FabSize::Large,
    ];

    /// Every size: its side, its icon's side and its corners' radius. androidx `FabSmallTokens`, `FabBaselineTokens`,
    /// `FabMediumTokens` and `FabLargeTokens`, and `FloatingActionButtonDefaults` where it overrides them: the large
    /// icon is 36 and the medium shape is `CornerLargeIncreased`.
    const EXPECTED: [(FabSize, f32, f32, f32); 4] = [
        (FabSize::Small, 40.0, 24.0, 12.0),
        (FabSize::Baseline, 56.0, 24.0, 16.0),
        (FabSize::Medium, 80.0, 28.0, 20.0),
        (FabSize::Large, 96.0, 36.0, 28.0),
    ];

    #[test]
    fn each_style_has_its_colors_light_and_dark() {
        for style in STYLES {
            for dark in [false, true] {
                let what = format!("{style:?} dark: {dark}");
                let mounted = mount(Config {
                    style,
                    dark,
                    ..Config::default()
                });
                let (container, icon) = match style {
                    // androidx `FabPrimaryContainerTokens`: primary container, on primary container.
                    FabStyle::PrimaryContainer => {
                        (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer)
                    }
                    // androidx `FabSecondaryContainerTokens`.
                    _ => (
                        ColorRole::SecondaryContainer,
                        ColorRole::OnSecondaryContainer,
                    ),
                };
                assert_eq!(
                    mounted.style(mounted.button).background_color,
                    solid(container, dark),
                    "{what}"
                );
                assert_eq!(
                    fill(&mounted.drawing(mounted.icon).ops),
                    Some(solid(icon, dark)),
                    "{what}"
                );
                // The state layer is the icon's color, and not the container's.
                assert_eq!(
                    mounted.style(mounted.layer).background_color,
                    solid(icon, dark),
                    "{what}"
                );
                assert_ne!(solid(container, dark), solid(icon, dark), "{what}");
            }
        }
    }

    #[test]
    fn each_size_has_its_frame_and_centers_its_icon() {
        for (size, side, icon, _) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let frame = mounted.size();
            assert_eq!((frame.width.0, frame.height.0), (side, side), "{what}");
            // The state layer covers the FAB; the icon is centered at its side.
            let layer = mounted.frames[&mounted.layer];
            assert_eq!((layer.origin.x.0, layer.origin.y.0), (0.0, 0.0), "{what}");
            assert_eq!(layer.size, frame, "{what}");
            let bounds = mounted.frames[&mounted.icon];
            assert_eq!(
                (bounds.size.width.0, bounds.size.height.0),
                (icon, icon),
                "{what}"
            );
            assert_eq!(bounds.origin.x.0, (side - icon) / 2.0, "{what}");
            assert_eq!(bounds.origin.y.0, (side - icon) / 2.0, "{what}");
        }
    }

    #[test]
    fn each_size_has_its_corners_on_the_container_and_the_state_layer() {
        for (size, .., radius) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            for id in [mounted.button, mounted.layer] {
                let style = mounted.style(id);
                assert_eq!(style.border_radius.0, radius, "{what}");
                // No corner of its own: the four are the same.
                assert_eq!(style.corner_radius, nsh_core::Corners::default(), "{what}");
            }
            // The icon's canvas has none.
            assert_eq!(mounted.style(mounted.icon).border_radius.0, 0.0, "{what}");
        }
    }

    #[test]
    fn only_the_container_casts_a_shadow_and_no_view_has_a_border_or_another_effect() {
        for (size, ..) in EXPECTED {
            for style in STYLES {
                let mounted = mount(Config {
                    size,
                    style,
                    ..Config::default()
                });
                for id in [mounted.button, mounted.layer, mounted.icon] {
                    let view = mounted.style(id);
                    let what = format!("{size:?} {style:?}");
                    // Android's own material shadow is not used: the shadow is ADR 0018's layers.
                    assert_eq!(view.elevation, Points(0.0), "{what}");
                    assert_eq!(view.borders.widths.top, px(0.0), "{what}");
                    assert_eq!(view.outline, nsh_core::Outline::default(), "{what}");
                    if id == mounted.button {
                        // The shadow is the only effect: no gradient, filter or mask.
                        let effects = view.effects.as_deref().expect("the container's shadow");
                        assert!(effects.background_images.is_empty(), "{what}");
                        assert!(effects.filters.is_empty(), "{what}");
                        assert!(effects.backdrop_filters.is_empty(), "{what}");
                        assert!(effects.mask_images.is_empty(), "{what}");
                    } else {
                        // Neither the state layer nor the icon casts one: the container's is the FAB's only.
                        assert!(view.effects.is_none(), "{what}: {:?}", view.effects);
                    }
                }
            }
        }
    }

    #[test]
    fn every_size_has_a_48_point_touch_target_without_growing() {
        for (size, side, ..) in EXPECTED {
            let what = format!("{size:?}");
            let mounted = mount(Config {
                size,
                ..Config::default()
            });
            let slop = mounted.hit_slop.expect("the button's props");
            // Each side reaches 48 points with the slop, and a side already 48 or more gets none.
            let reach = (48.0 - side).max(0.0) / 2.0;
            for around in [slop.left, slop.right, slop.top, slop.bottom] {
                assert_eq!(around.0, reach, "{what}");
            }
            assert!(side + slop.left.0 + slop.right.0 >= 48.0, "{what}");
            assert!(side + slop.top.0 + slop.bottom.0 >= 48.0, "{what}");
            assert_eq!(mounted.size().height.0, side, "{what}");
        }
        // The small FAB is the only one that needs the slop: 4 points on each side.
        let small = mount(Config {
            size: FabSize::Small,
            ..Config::default()
        });
        assert_eq!(small.hit_slop.expect("props").left.0, 4.0);
    }

    proptest! {
        #[test]
        fn the_slop_always_reaches_the_target_and_never_grows_a_big_container(
            width in 0.0_f32..400.0,
            height in 0.0_f32..400.0,
        ) {
            let slop = touch_slop(Size::new(Points(width), Points(height)));
            for around in [slop.left, slop.right, slop.top, slop.bottom] {
                prop_assert!(around.0 >= 0.0);
            }
            prop_assert!(width + slop.left.0 + slop.right.0 >= MIN_TOUCH_TARGET.0 - 1e-3);
            prop_assert!(height + slop.top.0 + slop.bottom.0 >= MIN_TOUCH_TARGET.0 - 1e-3);
            // The slop is symmetric, and only as large as it must be.
            prop_assert_eq!(slop.left.0.to_bits(), slop.right.0.to_bits());
            prop_assert_eq!(slop.top.0.to_bits(), slop.bottom.0.to_bits());
            if width >= MIN_TOUCH_TARGET.0 {
                prop_assert_eq!(slop.left.0.to_bits(), 0.0_f32.to_bits());
            }
            if height >= MIN_TOUCH_TARGET.0 {
                prop_assert_eq!(slop.top.0.to_bits(), 0.0_f32.to_bits());
            }
        }
    }

    #[test]
    fn the_icon_is_its_path_at_the_size_in_the_content_color() {
        for (size, _, icon, _) in EXPECTED {
            let mounted = mount(Config {
                size,
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
        }
    }

    #[test]
    fn screen_readers_name_it_by_its_label_as_a_button() {
        let mounted = mount(Config::default());
        assert_eq!(mounted.label.as_deref(), Some("Add"));
    }

    #[test]
    fn a_press_is_reported_once() -> Result<(), String> {
        let (mut app, _recording) = start(Config::default());
        app.press("label:Add")?;
        assert!(app.shows("presses: 1"));
        assert!(!app.shows("presses: 2"));
        Ok(())
    }

    /// The opacities the state layer was sent in `items`, in order.
    fn layer_opacities(mounted: &Mounted, items: &[MountItem]) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    /// Nothing of the FAB but the styles of its container (its shadow) and of its state layer changed in `items`:
    /// none of its frames, no drawing, and its icon not styled again. What else is on screen (the probe's counter)
    /// may change.
    fn assert_only_the_styles_change_in_the_fab(
        mounted: &Mounted,
        items: &[MountItem],
        what: &str,
    ) {
        assert_eq!(mounted.layout_items(items), 0, "{what}");
        assert!(
            items.iter().all(|item| match item {
                MountItem::ViewStyle { id, .. } => *id == mounted.layer || *id == mounted.button,
                MountItem::Canvas { id, .. } => *id == mounted.layer,
                _ => true,
            }),
            "{what}: {items:?}"
        );
    }

    /// Nothing but the styles of the FAB's container and state layer changed in `items`, anywhere on screen: no
    /// frame, no text, no drawing, and no other view styled again.
    fn assert_only_the_styles_change(mounted: &Mounted, items: &[MountItem], what: &str) {
        assert_eq!(layout_items(items), 0, "{what}");
        assert_only_the_styles_change_in_the_fab(mounted, items, what);
    }

    #[test]
    fn hover_shows_the_state_layer_in_the_icon_color_and_lays_out_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.style(mounted.layer).opacity, 0.0);
        app.hover("label:Add")?;
        let hover = recording.in_step("hover(label:Add)");
        // The state layer at `StateTokens.HoverStateLayerOpacity`, once.
        assert_eq!(layer_opacities(&mounted, &hover), [0.08]);
        assert_only_the_styles_change(&mounted, &hover, "hover");
        // It is the icon's color, which the layer already had at rest.
        assert_eq!(
            mounted.style(mounted.layer).background_color,
            solid(ColorRole::OnPrimaryContainer, false)
        );
        // A pointer that leaves takes it away.
        app.step("leave");
        app.hover("text:presses: 0")?;
        assert_eq!(
            layer_opacities(&mounted, &recording.in_step("hover(text:presses: 0)")),
            [0.0]
        );
        Ok(())
    }

    /// The step of a press that goes down on the FAB and stays down: a finger that lands and does not move.
    fn press_down(app: &mut Headless) -> String {
        let still = point(px(0.0), px(0.0));
        app.drag_and_hold("label:Add", still).unwrap();
        format!("dragAndHold(label:Add, {}, {})", still.x, still.y)
    }

    #[test]
    fn a_press_changes_nothing_that_is_laid_out_or_drawn_and_starts_no_animation()
    -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let step = press_down(&mut app);
        let down = recording.in_step(&step);
        // The FAB has no pressed shape (androidx's has none): nothing moves, so no spring starts and nothing is laid
        // out or drawn again. The state layer's pressed opacity is the platform's to apply: the headless runner
        // sends the press to the FAB's handlers, not to its styles.
        assert_eq!(started_springs(&down), Vec::new());
        assert_only_the_styles_change(&mounted, &down, "down");
        assert!(app.shows("presses: 0"));
        app.drop_held()?;
        let up = recording.in_step("drop()");
        // The release reports the press, so the probe's counter text is laid out again: only the FAB itself is held
        // to nothing.
        assert_only_the_styles_change_in_the_fab(&mounted, &up, "up");
        assert_eq!(started_springs(&up), Vec::new());
        assert!(app.shows("presses: 1"));
        assert_eq!(Mounted::of(&recording.items()).size(), mounted.size());
        Ok(())
    }

    #[test]
    fn the_button_asks_the_platform_for_its_hover_and_press_so_that_its_state_layer_shows_them() {
        // The state layer's `group_hover` and `group_active` are resolved by the platform, which the button's group
        // tells to report both.
        let (_app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        let tracked = recording.items().iter().find_map(|item| match item {
            MountItem::TrackStates { id, states } if *id == mounted.button => Some(*states),
            _ => None,
        });
        let tracked = tracked.expect("the FAB's group tracks states");
        assert_ne!(tracked & nsh_core::tailwind::state::HOVERED, 0);
        assert_ne!(tracked & nsh_core::tailwind::state::PRESSED, 0);
    }

    #[test]
    fn the_state_layer_opacities_are_the_state_tokens() {
        // androidx `StateTokens`: hover 0.08, focus 0.1, pressed 0.1.
        let bits = |opacity: f32| opacity.to_bits();
        assert_eq!(bits(state_tokens::HOVER_STATE_LAYER_OPACITY), bits(0.08));
        assert_eq!(bits(state_tokens::FOCUS_STATE_LAYER_OPACITY), bits(0.1));
        assert_eq!(bits(state_tokens::PRESSED_STATE_LAYER_OPACITY), bits(0.1));
    }

    #[test]
    fn focus_shows_the_state_layer_at_the_focus_opacity() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("Tab")?;
        let focus = recording.in_step("keyboard.press(Tab)");
        // `StateTokens.FocusStateLayerOpacity`.
        assert_eq!(layer_opacities(&mounted, &focus), [0.1]);
        assert_only_the_styles_change(&mounted, &focus, "focus");
        Ok(())
    }

    #[test]
    fn mounting_starts_nothing() {
        for (size, ..) in EXPECTED {
            let (_app, recording) = start(Config {
                size,
                ..Config::default()
            });
            assert_eq!(started_springs(&recording.items()), Vec::new(), "{size:?}");
        }
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        // androidx `FloatingActionButton` is the 56-point one, with `FloatingActionButtonDefaults.containerColor`.
        assert_eq!(FabSize::default(), FabSize::Baseline);
        assert_eq!(FabStyle::default(), FabStyle::PrimaryContainer);
        assert_eq!(FabState::default(), FabState::Rest);
        assert_eq!(size_spec(FabSize::default()).width, Points(56.0));
        assert_eq!(
            style_spec(FabStyle::default()).container,
            ColorRole::PrimaryContainer
        );
    }

    #[test]
    fn the_container_is_as_wide_as_it_is_tall_at_every_size() {
        // androidx gives the width and the height of each size their own token: they are the same today.
        for size in SIZES {
            let spec = size_spec(size);
            assert_eq!(spec.width, spec.height, "{size:?}");
        }
    }

    /// `FabStyle::elevation` of each state: level 3 at rest, focused and pressed, level 4 hovered. androidx
    /// `ElevationTokens`: `Level3` is 6 and `Level4` is 8.
    const ELEVATIONS: [(FabState, f32); 4] = [
        (FabState::Rest, 6.0),
        (FabState::Hovered, 8.0),
        (FabState::Focused, 6.0),
        (FabState::Pressed, 6.0),
    ];

    #[test]
    fn each_style_resolves_the_elevation_of_each_state() {
        for style in STYLES {
            for (state, points) in ELEVATIONS {
                assert_eq!(
                    style.elevation(state),
                    Points(points),
                    "{style:?} {state:?}"
                );
            }
        }
        // Only hovering lifts it.
        for style in STYLES {
            for state in [FabState::Rest, FabState::Focused, FabState::Pressed] {
                assert!(style.elevation(FabState::Hovered) > style.elevation(state));
            }
        }
    }

    #[test]
    fn each_style_has_the_level_of_each_state_which_is_its_token() {
        for style in STYLES {
            for (state, points) in ELEVATIONS {
                let what = format!("{style:?} {state:?}");
                // The elevation module's own lookup of a length, which a constant cannot call, finds the same level.
                assert_eq!(
                    Elevation::from_points(style.elevation(state)),
                    Some(style.level(state)),
                    "{what}"
                );
                assert_eq!(
                    Elevation::from_points(Points(points)),
                    Some(style.level(state)),
                    "{what}"
                );
            }
            // Level 3 at rest, focused and pressed; level 4 under the pointer.
            assert_eq!(style.level(FabState::Rest), Elevation::Level3);
            assert_eq!(style.level(FabState::Focused), Elevation::Level3);
            assert_eq!(style.level(FabState::Pressed), Elevation::Level3);
            assert_eq!(style.level(FabState::Hovered), Elevation::Level4);
        }
    }

    #[test]
    fn a_length_that_is_no_level_is_refused_and_not_taken_for_a_neighbor() {
        // Each level's own length gives the level.
        for level in Elevation::ALL {
            assert_eq!(level_of(level.points()), level);
        }
        // Anything else would fail the build when a constant got it: called at run time it panics. Between two
        // levels, a hair off one, negative, and not a number: none is rounded to a level.
        for points in [7.0, 6.001, 5.999, 13.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(|| level_of(Points(points))).is_err(),
                "{points}"
            );
        }
    }

    /// The shadow layers of every `ViewStyle` that was sent for the container in `items`, in order.
    fn container_shadows(mounted: &Mounted, items: &[MountItem]) -> Vec<Vec<BoxShadow>> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.button => {
                    Some(shadows_of(style))
                }
                _ => None,
            })
            .collect()
    }

    /// The box shadows `style` has, top layer first.
    fn shadows_of(style: &ViewStyle) -> Vec<BoxShadow> {
        style
            .effects
            .as_deref()
            .map(|effects| effects.box_shadows.clone())
            .unwrap_or_default()
    }

    /// The two layers of `level`, as a list.
    fn layers(level: Elevation) -> Vec<BoxShadow> {
        level.shadows().to_vec()
    }

    #[test]
    fn the_container_rests_at_level_3_with_its_two_shadow_layers() {
        for (size, ..) in EXPECTED {
            for style in STYLES {
                for dark in [false, true] {
                    let mounted = mount(Config {
                        size,
                        style,
                        dark,
                        ..Config::default()
                    });
                    let found = shadows_of(&mounted.style(mounted.button));
                    let what = format!("{size:?} {style:?} dark: {dark}");
                    // Exactly the two layers of level 3, key layer first: ADR 0018's, nothing else.
                    assert_eq!(found.len(), 2, "{what}");
                    assert_eq!(found, layers(Elevation::Level3), "{what}");
                    assert!(found.iter().all(|layer| !layer.inset), "{what}");
                }
            }
        }
        // The levels differ: a test of level 3 is not a test of level 4.
        assert_ne!(layers(Elevation::Level3), layers(Elevation::Level4));
    }

    #[test]
    fn hovering_raises_the_shadow_to_level_4_at_once_and_leaving_lowers_it() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.hover("label:Add")?;
        let hover = recording.in_step("hover(label:Add)");
        // One style for the container, with level 4's two layers; the state layer's opacity is what else changed.
        assert_eq!(
            container_shadows(&mounted, &hover),
            [layers(Elevation::Level4)]
        );
        assert_eq!(
            shadows_of(&Mounted::of(&recording.items()).style(mounted.button)),
            layers(Elevation::Level4)
        );
        // It snaps: nothing animates the shadow, no spring starts and no value is set, the container has no
        // transition, and nothing is laid out again.
        assert_eq!(started_springs(&hover), Vec::new());
        assert_eq!(set_values(&hover), 0);
        assert!(
            hover.iter().all(|item| match item {
                MountItem::ViewStyle { style, .. } => style.transition.is_none(),
                _ => true,
            }),
            "{hover:?}"
        );
        assert_only_the_styles_change(&mounted, &hover, "hover");
        // A pointer that leaves puts it back.
        app.step("leave");
        app.hover("text:presses: 0")?;
        let leave = recording.in_step("hover(text:presses: 0)");
        assert_eq!(
            container_shadows(&mounted, &leave),
            [layers(Elevation::Level3)]
        );
        assert_eq!(started_springs(&leave), Vec::new());
        Ok(())
    }

    #[test]
    fn focus_keeps_level_3() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press_key("Tab")?;
        let focus = recording.in_step("keyboard.press(Tab)");
        // `FocusedContainerElevation` is level 3, the same as at rest: whatever is sent for the container is level 3.
        for found in container_shadows(&mounted, &focus) {
            assert_eq!(found, layers(Elevation::Level3));
        }
        assert_eq!(started_springs(&focus), Vec::new());
        assert_eq!(
            shadows_of(&Mounted::of(&recording.items()).style(mounted.button)),
            layers(Elevation::Level3)
        );
        Ok(())
    }

    /// The shadow the container has once the platform reports `states` for it (bits of
    /// `nsh_core::tailwind::state`), as it does a hover, a focus or a press.
    fn shadows_in(app: &mut Headless, recording: &Recording, states: u16) -> Vec<BoxShadow> {
        let button = Mounted::of(&recording.items()).button;
        app.engine().control().set_states(button, states);
        assert!(app.settle());
        shadows_of(&Mounted::of(&recording.items()).style(button))
    }

    #[test]
    fn the_state_the_platform_reports_decides_the_level() {
        use nsh_core::tailwind::state::{FOCUSED, HOVERED, PRESSED};
        let (mut app, recording) = start(Config::default());
        let level_3 = layers(Elevation::Level3);
        let level_4 = layers(Elevation::Level4);
        // The headless runner does not apply a press to the styles, so the platform's report is sent to the engine
        // as a platform does it.
        assert_eq!(shadows_in(&mut app, &recording, HOVERED), level_4);
        // A press that goes down on a hovered FAB lowers it to the pressed level, which is level 3; the platform
        // keeps reporting the hover while the pointer is down.
        assert_eq!(shadows_in(&mut app, &recording, HOVERED | PRESSED), level_3);
        assert_eq!(shadows_in(&mut app, &recording, PRESSED), level_3);
        assert_eq!(shadows_in(&mut app, &recording, HOVERED), level_4);
        assert_eq!(shadows_in(&mut app, &recording, 0), level_3);
        // Focused alone is level 3. Focused under the pointer: NSH orders the hover's variant after the focus's, so
        // the hovered level wins, where androidx takes whichever interaction came last (`interactions.lastOrNull()`,
        // line 1346): a FAB focused first and then entered by the pointer is level 4 there too, one the pointer was
        // already over when it took the focus is level 3. NSH's variants record no history.
        assert_eq!(shadows_in(&mut app, &recording, FOCUSED), level_3);
        assert_eq!(shadows_in(&mut app, &recording, HOVERED | FOCUSED), level_4);
        assert_eq!(shadows_in(&mut app, &recording, FOCUSED | PRESSED), level_3);
        assert_eq!(
            shadows_in(&mut app, &recording, HOVERED | FOCUSED | PRESSED),
            level_3
        );
    }

    #[test]
    fn a_hidden_fab_keeps_its_two_layers_and_hiding_animates_nothing_else() {
        let hidden = mount(Config {
            visibility: FabVisibility::Hidden,
            ..Config::default()
        });
        // It is not laid out, and its container still has exactly the two layers of level 3.
        assert_eq!(hidden.size(), Size::new(px(0.0), px(0.0)));
        assert_eq!(
            shadows_of(&hidden.style(hidden.button)),
            layers(Elevation::Level3)
        );
        // Showing it is two springs, the scale's and the opacity's, as before: the shadow adds none.
        let (mut app, recording) = start(Config {
            visibility: FabVisibility::Hidden,
            ..Config::default()
        });
        let mounted = Mounted::of(&recording.items());
        app.press("text:Flip").unwrap();
        let shown = recording.in_step("press(text:Flip)");
        assert_eq!(started_springs(&shown).len(), 2);
        assert_eq!(
            shadows_of(&Mounted::of(&recording.items()).style(mounted.button)),
            layers(Elevation::Level3)
        );
    }

    #[test]
    fn every_state_has_the_same_icon_color() {
        // The component draws one icon color per style, and the state layer in it. If a state ever has another one
        // in the tokens, the component has to show it.
        let primary = [
            fab_primary_container_tokens::ICON_COLOR,
            fab_primary_container_tokens::FOCUSED_ICON_COLOR,
            fab_primary_container_tokens::HOVERED_ICON_COLOR,
            fab_primary_container_tokens::PRESSED_ICON_COLOR,
        ];
        let secondary = [
            fab_secondary_container_tokens::ICON_COLOR,
            fab_secondary_container_tokens::FOCUSED_ICON_COLOR,
            fab_secondary_container_tokens::HOVERED_ICON_COLOR,
            fab_secondary_container_tokens::PRESSED_ICON_COLOR,
        ];
        assert!(
            primary
                .iter()
                .all(|role| *role == ColorRole::OnPrimaryContainer)
        );
        assert!(
            secondary
                .iter()
                .all(|role| *role == ColorRole::OnSecondaryContainer)
        );
    }

    /// Views of one shape each, to read the corners `rounded` gives.
    #[component]
    fn Medium() -> impl IntoElement {
        rounded(view().size(px(40.0)), shape_tokens::CORNER_MEDIUM)
    }

    #[component]
    fn ExtraLarge() -> impl IntoElement {
        rounded(view().size(px(96.0)), shape_tokens::CORNER_EXTRA_LARGE)
    }

    #[component]
    fn Full() -> impl IntoElement {
        rounded(view().size(px(40.0)), shape_tokens::CORNER_FULL)
    }

    /// The largest corner radius of the views `root` mounts.
    fn radius_of(root: Root) -> f32 {
        let mut app = Headless::start(EngineConfig::new(root)).unwrap();
        let recording = Recording::attach(&mut app);
        app.first_render().unwrap();
        recording
            .items()
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { style, .. } => Some(style.border_radius.0),
                _ => None,
            })
            .fold(0.0, f32::max)
    }

    #[test]
    fn a_shape_gives_its_corners_to_a_view() {
        assert_eq!(radius_of(Root::of::<Medium>()), 12.0);
        assert_eq!(radius_of(Root::of::<ExtraLarge>()), 28.0);
        // A pill or a circle takes as much as any container can round.
        assert!(radius_of(Root::of::<Full>()) >= 1000.0);
    }

    /// The vendored `FloatingActionButton.kt`, at the pinned commit (ADR 0012).
    const FLOATING_ACTION_BUTTON_KT: &str =
        include_str!("../../../reference/androidx-material3-components/FloatingActionButton.kt");
    /// The vendored `FabMediumTokens.kt`, whose `ContainerShape` androidx comments out.
    const FAB_MEDIUM_TOKENS_KT: &str =
        include_str!("../../../reference/androidx-material3-tokens/FabMediumTokens.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_icon_sizes_are_androidxs_and_the_large_one_is_its_override() {
        // The large icon: the literal, and the token it disagrees with.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val LargeIconSize: Dp = 36.dp // TODO: FabLargeTokens.IconSize is incorrect"]
        ));
        assert_eq!(LARGE_ICON_SIZE, Points(36.0));
        assert_ne!(LARGE_ICON_SIZE, fab_large_tokens::ICON_SIZE);
        assert_eq!(fab_large_tokens::ICON_SIZE, Points(32.0));
        // The medium icon is its token.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val MediumIconSize: Dp = FabMediumTokens.IconSize"]
        ));
        assert_eq!(
            size_spec(FabSize::Medium).icon,
            fab_medium_tokens::ICON_SIZE
        );
        assert_eq!(size_spec(FabSize::Large).icon, LARGE_ICON_SIZE);
        // A line that is not there is not found.
        assert!(!cites(
            FLOATING_ACTION_BUTTON_KT,
            &["public val LargeIconSize: Dp = FabLargeTokens.IconSize"]
        ));
    }

    #[test]
    fn the_shapes_are_androidxs_and_the_medium_one_is_its_override() {
        // The medium shape: `ShapeDefaults.LargeIncreased`, and the key the commented-out token names.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "public val mediumShape: Shape",
                "@Composable get() = ShapeDefaults.LargeIncreased // TODO: update to use token",
            ]
        ));
        assert!(cites(
            FAB_MEDIUM_TOKENS_KT,
            &["// val ContainerShape = ShapeKeyTokens.CornerLargeIncreased"]
        ));
        assert_eq!(MEDIUM_SHAPE, shape_tokens::CORNER_LARGE_INCREASED);
        assert_eq!(
            Corners::of(MEDIUM_SHAPE),
            Some(Corners::Radius(Points(20.0)))
        );
        // The other sizes' shapes are their tokens.
        for (name, size, token, shape) in [
            (
                "smallShape",
                FabSize::Small,
                "FabSmallTokens.ContainerShape.value",
                fab_small_tokens::CONTAINER_SHAPE,
            ),
            (
                "shape",
                FabSize::Baseline,
                "FabBaselineTokens.ContainerShape.value",
                fab_baseline_tokens::CONTAINER_SHAPE,
            ),
            (
                "largeShape",
                FabSize::Large,
                "FabLargeTokens.ContainerShape.value",
                fab_large_tokens::CONTAINER_SHAPE,
            ),
        ] {
            let property = format!("public val {name}: Shape");
            let getter = format!("@Composable get() = {token}");
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&property, &getter]),
                "{name}"
            );
            assert_eq!(size_spec(size).shape, shape, "{name}");
        }
        assert_eq!(size_spec(FabSize::Medium).shape, MEDIUM_SHAPE);
    }

    #[test]
    fn the_default_style_is_androidxs_with_its_elevations() {
        // The default container color is the primary container's.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "public val containerColor: Color",
                "@Composable get() = FabPrimaryContainerTokens.ContainerColor.value",
            ]
        ));
        // The elevation of every state is read from the primary container's tokens by default.
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "defaultElevation: Dp = FabPrimaryContainerTokens.ContainerElevation,",
                "pressedElevation: Dp = FabPrimaryContainerTokens.PressedContainerElevation,",
                "focusedElevation: Dp = FabPrimaryContainerTokens.FocusedContainerElevation,",
                "hoveredElevation: Dp = FabPrimaryContainerTokens.HoveredContainerElevation,",
            ]
        ));
        assert_eq!(
            style_spec(FabStyle::default()).container,
            fab_primary_container_tokens::CONTAINER_COLOR
        );
    }

    #[test]
    fn each_size_is_its_tokens_width_and_height() {
        // Each size is a minimum of its tokens' width and height.
        for (token, size, width, height) in [
            (
                "FabSmallTokens",
                FabSize::Small,
                fab_small_tokens::CONTAINER_WIDTH,
                fab_small_tokens::CONTAINER_HEIGHT,
            ),
            (
                "FabMediumTokens",
                FabSize::Medium,
                fab_medium_tokens::CONTAINER_WIDTH,
                fab_medium_tokens::CONTAINER_HEIGHT,
            ),
            (
                "FabLargeTokens",
                FabSize::Large,
                fab_large_tokens::CONTAINER_WIDTH,
                fab_large_tokens::CONTAINER_HEIGHT,
            ),
        ] {
            let min_width = format!("minWidth = {token}.ContainerWidth,");
            let min_height = format!("minHeight = {token}.ContainerHeight,");
            assert!(
                cites(FLOATING_ACTION_BUTTON_KT, &[&min_width, &min_height]),
                "{token}"
            );
            let spec = size_spec(size);
            assert_eq!((spec.width, spec.height), (width, height), "{token}");
        }
        assert!(cites(
            FLOATING_ACTION_BUTTON_KT,
            &[
                "FabBaselineTokens.ContainerWidth,",
                "FabBaselineTokens.ContainerHeight,",
            ]
        ));
        let baseline = size_spec(FabSize::Baseline);
        assert_eq!(
            (baseline.width, baseline.height),
            (
                fab_baseline_tokens::CONTAINER_WIDTH,
                fab_baseline_tokens::CONTAINER_HEIGHT
            )
        );
    }

    /// The animated values the FAB's views are bound to in `items`: the opacity's, and the scale's.
    fn bound(items: &[MountItem], button: NodeId) -> (AnimationNode, AnimationNode) {
        let bindings = items
            .iter()
            .find_map(|item| match item {
                MountItem::AnimationBindings { id, bindings } if *id == button => {
                    Some(bindings.clone())
                }
                _ => None,
            })
            .expect("the FAB's animated bindings");
        let nodes = |wanted: fn(&AnimationProp) -> bool| -> Vec<AnimationNode> {
            bindings
                .iter()
                .filter(|(prop, _)| wanted(prop))
                .map(|(_, node)| *node)
                .collect()
        };
        let opacity = nodes(|prop| matches!(prop, AnimationProp::Opacity));
        let scale = nodes(|prop| matches!(prop, AnimationProp::Transform(_)));
        let ([opacity], [scale]) = (&opacity[..], &scale[..]) else {
            panic!("one opacity and one scale: {bindings:?}");
        };
        (*opacity, *scale)
    }

    /// The two springs of one show or hide, the scale's progress and then the opacity.
    fn pair(springs: [StartedSpring; 2], opacity: AnimationNode) -> (StartedSpring, StartedSpring) {
        let [first, second] = springs;
        if first.node == opacity {
            (second, first)
        } else {
            (first, second)
        }
    }

    /// The values of the reveal after each tick of the replay.
    #[derive(Debug, Default)]
    struct Curves {
        /// The progress the scale is made from.
        progress: Vec<f32>,
        opacity: Vec<f32>,
        /// What the scale transform is bound to, an interpolation of the progress.
        scale: Vec<f32>,
    }

    /// Ticks `replay` `frames` times and reads the three values after each.
    fn play(
        replay: &mut Replay,
        (progress, opacity, scale): (AnimationNode, AnimationNode, AnimationNode),
        frames: usize,
    ) -> Curves {
        let mut curves = Curves::default();
        for _ in 0..frames {
            curves.progress.extend(replay.run(progress, 1));
            curves.opacity.push(replay.value(opacity));
            curves.scale.push(replay.value(scale));
        }
        curves
    }

    /// The scale `lerp(0.2f, 1f, progress)` of androidx's `FabVisibleNode`, as `FloatingActionButton.kt` writes it.
    fn lerp_scale(progress: f32) -> f32 {
        0.2 + (1.0 - 0.2) * progress
    }

    /// Where the springs of a show or a hide go, and from where: `(visibility at rest, where it goes)`.
    const REVEALS: [(FabVisibility, f32); 2] =
        [(FabVisibility::Shown, 0.0), (FabVisibility::Hidden, 1.0)];

    #[test]
    fn showing_and_hiding_spring_the_scale_and_the_opacity_with_the_fast_springs() {
        for (visibility, to) in REVEALS {
            let what = format!("{visibility:?}");
            let (mut app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            let button = Mounted::of(&recording.items()).button;
            let (opacity, _) = bound(&recording.items(), button);
            app.press("Flip").unwrap();
            let items = recording.in_step("press(Flip)");
            let springs = started_springs(&items);
            let [progress, faded] = springs[..] else {
                panic!("{what}: two springs, not {springs:?}");
            };
            let (progress, faded) = pair([progress, faded], opacity);
            // androidx: the scale on the fast spatial spring and the alpha on the fast effects one (ADR 0004).
            for (spring, token) in [
                (progress, SpringToken::FastSpatial),
                (faded, SpringToken::FastEffects),
            ] {
                let spec = token.spec();
                assert_eq!(spring.to.to_bits(), to.to_bits(), "{what} {token:?}");
                assert_eq!(
                    spring.stiffness.to_bits(),
                    spec.stiffness.to_bits(),
                    "{what} {token:?}"
                );
                assert_eq!(
                    spring.damping.to_bits(),
                    spec.damping_coefficient().to_bits(),
                    "{what} {token:?}"
                );
                assert_eq!(
                    spring.mass.to_bits(),
                    SPRING_MASS.to_bits(),
                    "{what} {token:?}"
                );
                assert_eq!(spring.velocity, None, "{what} {token:?}");
                assert_eq!(spring.frame_rate, None, "{what} {token:?}");
            }
            assert_ne!(progress.node, faded.node, "{what}");
            assert_eq!(set_values(&items), 0, "{what}: {items:?}");
        }
    }

    #[test]
    fn the_scale_and_the_opacity_follow_the_closed_form_spring_and_cost_the_engine_nothing() {
        for (visibility, to) in REVEALS {
            let what = format!("{visibility:?}");
            let from = visibility.progress();
            let (mut app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            let button = Mounted::of(&recording.items()).button;
            let (opacity, scale) = bound(&recording.items(), button);
            app.press("Flip").unwrap();
            let springs = started_springs(&recording.in_step("press(Flip)"));
            let [first, second] = springs[..] else {
                panic!("{what}: {springs:?}");
            };
            let (progress, faded) = pair([first, second], opacity);
            // While they play, the engine sends nothing (ADR 0010).
            app.step("run");
            app.run_for(Duration::from_secs(1)).unwrap();
            assert!(recording.in_step("run").is_empty(), "{what}");

            let mut replay = replay(&recording);
            let nodes = (progress.node, opacity, scale);
            let curves = play(&mut replay, nodes, 180);
            // Both curves are the closed-form springs, within 0.1 % of their travel.
            let travel = tolerance(f64::from(from), f64::from(to));
            let gap = worst_gap(&curves.progress, f64::from(from), &progress);
            assert!(gap < travel, "{what} progress: {gap}");
            let gap = worst_gap(&curves.opacity, f64::from(from), &faded);
            assert!(gap < travel, "{what} opacity: {gap}");
            // The scale is `lerp(0.2, 1, progress)` at every frame, and moves.
            for (i, (scale, progress)) in curves.scale.iter().zip(&curves.progress).enumerate() {
                assert!(
                    (scale - lerp_scale(*progress)).abs() < 1e-5,
                    "{what} frame {i}: {scale} for {progress}"
                );
            }
            let (lowest, highest) = curves
                .scale
                .iter()
                .fold((f32::MAX, f32::MIN), |(low, high), v| {
                    (low.min(*v), high.max(*v))
                });
            assert!(highest - lowest > 0.5, "{what}: {lowest} to {highest}");
            // They settle on their targets exactly, and the graph stops.
            let rest = play(&mut replay, nodes, 600);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                rest.progress.last().map(|v| v.to_bits()),
                Some(to.to_bits()),
                "{what}"
            );
            assert_eq!(
                rest.opacity.last().map(|v| v.to_bits()),
                Some(to.to_bits()),
                "{what}"
            );
            let scale_at_rest = rest.scale.last().copied().unwrap_or(f32::NAN);
            assert!(
                (scale_at_rest - lerp_scale(to)).abs() < 1e-6,
                "{what}: {scale_at_rest}"
            );
        }
    }

    #[test]
    fn the_spatial_spring_overshoots_the_scale_and_the_effects_spring_never_the_opacity() {
        for (visibility, to) in REVEALS {
            let what = format!("{visibility:?}");
            let (mut app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            let button = Mounted::of(&recording.items()).button;
            let (opacity, scale) = bound(&recording.items(), button);
            app.press("Flip").unwrap();
            let springs = started_springs(&recording.in_step("press(Flip)"));
            let [first, second] = springs[..] else {
                panic!("{what}: {springs:?}");
            };
            let (progress, _) = pair([first, second], opacity);
            let mut replay = replay(&recording);
            let curves = play(&mut replay, (progress.node, opacity, scale), 240);
            // The fast spatial spring has a damping ratio of 0.6: it passes its target by about a tenth of its
            // travel, and nothing clamps the scale (androidx `lerp`, line 1218).
            let past = |curve: &[f32]| {
                curve
                    .iter()
                    .map(|v| if to > 0.5 { v - 1.0 } else { -v })
                    .fold(f32::MIN, f32::max)
            };
            assert!(past(&curves.progress) > 0.05, "{what}");
            let scale_past = curves
                .scale
                .iter()
                .map(|v| if to > 0.5 { v - 1.0 } else { 0.2 - v })
                .fold(f32::MIN, f32::max);
            assert!(scale_past > 0.04, "{what}: {scale_past}");
            // The opacity is critically damped: it does not pass 0 or 1 (an `f32`'s rounding apart).
            assert!(past(&curves.opacity) < 1e-5, "{what}");
        }
    }

    /// The size of the FAB in the last frame it was sent.
    fn laid_out_size(recording: &Recording) -> Option<Size<Points>> {
        let mounted = Mounted::of(&recording.items());
        mounted.frames.get(&mounted.button).map(|frame| frame.size)
    }

    #[test]
    fn a_hidden_fab_takes_no_place_and_a_shown_one_does() {
        let nothing = Size::new(px(0.0), px(0.0));
        let baseline = Size::new(px(56.0), px(56.0));
        for (visibility, before, after) in [
            (FabVisibility::Shown, baseline, nothing),
            (FabVisibility::Hidden, nothing, baseline),
        ] {
            let what = format!("{visibility:?}");
            let (mut app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            assert_eq!(laid_out_size(&recording), Some(before), "{what}");
            // The hide is over within the step, and takes the place with it.
            app.press("Flip").unwrap();
            assert_eq!(laid_out_size(&recording), Some(after), "{what}");
            // And back.
            app.press("Flip").unwrap();
            assert_eq!(laid_out_size(&recording), Some(before), "{what}");
        }
    }

    #[test]
    fn a_fab_is_made_where_it_is_and_starts_nothing() {
        for (visibility, scale) in [(FabVisibility::Shown, 1.0), (FabVisibility::Hidden, 0.2)] {
            let what = format!("{visibility:?}");
            let (_app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            let items = recording.items();
            assert_eq!(started_springs(&items), Vec::new(), "{what}");
            assert_eq!(set_values(&items), 0, "{what}");
            let button = Mounted::of(&items).button;
            let (opacity, bound_scale) = bound(&items, button);
            let replay = replay(&recording);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                replay.value(opacity).to_bits(),
                visibility.progress().to_bits(),
                "{what}"
            );
            assert!(
                (replay.value(bound_scale) - scale).abs() < 1e-6,
                "{what}: {}",
                replay.value(bound_scale)
            );
        }
    }

    #[test]
    fn a_hide_that_is_undone_in_flight_leaves_the_fab_in_its_place() {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        app.press("Blink").unwrap();
        let items = recording.in_step("press(Blink)");
        assert_eq!(started_springs(&items).len(), 4, "{items:?}");
        // The opacity never reached 0, so the FAB never left the layout.
        assert_eq!(mounted.layout_items(&items), 0, "{items:?}");
        assert_eq!(laid_out_size(&recording), Some(mounted.size()));
    }

    #[test]
    fn a_show_that_is_undone_in_flight_ends_hidden_and_out_of_the_layout() {
        let (mut app, recording) = start(Config {
            visibility: FabVisibility::Hidden,
            ..Config::default()
        });
        app.press("Blink").unwrap();
        assert_eq!(started_springs(&recording.in_step("press(Blink)")).len(), 4);
        assert_eq!(laid_out_size(&recording), Some(Size::new(px(0.0), px(0.0))));
    }

    #[test]
    fn a_change_in_flight_goes_on_from_where_and_as_fast_as_the_values_move() {
        for visibility in [FabVisibility::Shown, FabVisibility::Hidden] {
            let what = format!("{visibility:?}");
            let rest = visibility.progress();
            let (mut app, recording) = start(Config {
                visibility,
                ..Config::default()
            });
            let button = Mounted::of(&recording.items()).button;
            let (opacity, scale) = bound(&recording.items(), button);
            app.press("Blink").unwrap();
            let springs = started_springs(&recording.in_step("press(Blink)"));
            let [away_first, away_second, back_first, back_second] = springs[..] else {
                panic!("{what}: {springs:?}");
            };
            let (away_progress, away_opacity) = pair([away_first, away_second], opacity);
            let (back_progress, back_opacity) = pair([back_first, back_second], opacity);
            assert_eq!(back_progress.to.to_bits(), rest.to_bits(), "{what}");
            assert_eq!(back_opacity.to.to_bits(), rest.to_bits(), "{what}");

            // The graph plays the first change for 6 frames, and the second comes in.
            let all = recording.items();
            let cut = all
                .iter()
                .enumerate()
                .filter(|(_, item)| matches!(item, MountItem::Animation(AnimationOp::Start { .. })))
                .map(|(i, _)| i)
                .nth(2)
                .expect("the second change's springs");
            let nodes = (away_progress.node, opacity, scale);
            let mut replay = Replay::default();
            replay.apply(&all[..cut]);
            let _ = play(&mut replay, nodes, 6);
            replay.apply(&all[cut..]);
            let curves = play(&mut replay, nodes, 180);

            // The graph last ticked 5 frames into the first change: the second starts there, at that value and
            // that speed.
            let at = 5.0 / REPLAY_HZ;
            for (name, away, back, curve) in [
                ("progress", away_progress, back_progress, &curves.progress),
                ("opacity", away_opacity, back_opacity, &curves.opacity),
            ] {
                let position = |seconds: f64| {
                    spring_position(
                        f64::from(rest),
                        f64::from(away.to),
                        0.0,
                        parameters(&away),
                        seconds,
                    )
                };
                let from = position(at);
                // A central difference over 2 microseconds.
                let step = 1e-6;
                let speed = (position(at + step) - position(at - step)) / (2.0 * step);
                let gap = |velocity: f64| {
                    curve
                        .iter()
                        .enumerate()
                        .map(|(i, &value)| {
                            let seconds =
                                f64::from(u32::try_from(i).expect("few frames")) / REPLAY_HZ;
                            let closed = spring_position(
                                from,
                                f64::from(back.to),
                                velocity,
                                parameters(&back),
                                seconds,
                            );
                            (f64::from(value) - closed).abs()
                        })
                        .fold(0.0, f64::max)
                };
                // 0.1 % of the travel of the change that was interrupted, not of the shorter travel left: the second
                // spring starts from the position and the speed the graph integrated, so it inherits the error of
                // the first curve, which is held to 0.1 % of its own travel.
                let tolerance = tolerance(f64::from(rest), f64::from(away.to));
                assert!(
                    gap(speed) < tolerance,
                    "{what} {name}: {} from {from} at {speed}",
                    gap(speed)
                );
                // Starting again from rest, as a new animation would, is another curve.
                assert!(gap(0.0) > 10.0 * tolerance, "{what} {name}: {}", gap(0.0));
            }
            let settled = play(&mut replay, nodes, 600);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(
                settled.progress.last().map(|v| v.to_bits()),
                Some(rest.to_bits()),
                "{what}"
            );
        }
    }

    #[test]
    fn with_reduce_motion_the_fab_shows_and_hides_at_once() {
        let nothing = Size::new(px(0.0), px(0.0));
        let baseline = Size::new(px(56.0), px(56.0));
        for (visibility, to, size) in [
            (FabVisibility::Shown, 0.0_f32, nothing),
            (FabVisibility::Hidden, 1.0, baseline),
        ] {
            let what = format!("{visibility:?}");
            let (mut app, recording) = start(Config {
                visibility,
                reduce_motion: true,
                ..Config::default()
            });
            let button = Mounted::of(&recording.items()).button;
            let (opacity, scale) = bound(&recording.items(), button);
            app.press("Flip").unwrap();
            let items = recording.in_step("press(Flip)");
            assert_eq!(started_springs(&items), Vec::new(), "{what}");
            // The scale's progress and the opacity are set where they go, and the FAB takes or leaves its place.
            assert_eq!(set_values(&items), 2, "{what}: {items:?}");
            let replay = replay(&recording);
            assert!(!replay.is_animating(), "{what}");
            assert_eq!(replay.value(opacity).to_bits(), to.to_bits(), "{what}");
            assert!(
                (replay.value(scale) - lerp_scale(to)).abs() < 1e-6,
                "{what}: {}",
                replay.value(scale)
            );
            assert_eq!(laid_out_size(&recording), Some(size), "{what}");
        }
    }

    #[test]
    fn the_choices_of_the_show_and_hide_are_androidxs() {
        for lines in [
            // The target scale, and how the scale is made from the progress.
            &["internal val ShowHideTargetScale = 0.2f"][..],
            &["scaleX = lerp(targetScale, 1f, scaleAnimatable.value)"],
            &["scaleY = lerp(targetScale, 1f, scaleAnimatable.value)"],
            // The scale on the fast spatial spring, the alpha on the fast effects one.
            &[
                "scaleAnimatable.animateTo(",
                "targetValue = if (visible) 1f else 0f,",
                "animationSpec =",
                "scaleAnimationSpec",
                "?: currentValueOf(LocalMaterialTheme).motionScheme.fastSpatialSpec<Float>(),",
            ],
            &[
                "alphaAnimatable.animateTo(",
                "targetValue = if (visible) 1f else 0f,",
                "animationSpec =",
                "alphaAnimationSpec",
                "?: currentValueOf(LocalMaterialTheme).motionScheme.fastEffectsSpec<Float>(),",
            ],
            // Both start where the FAB is, and a FAB with no alpha is laid out at 0 by 0.
            &["private val scaleAnimatable = Animatable(if (visible) 1f else 0f)"],
            &["private val alphaAnimatable = Animatable(if (visible) 1f else 0f)"],
            &[
                "if (alphaAnimatable.value == 0f) {",
                "return layout(0, 0) {}",
            ],
        ] {
            assert!(cites(FLOATING_ACTION_BUTTON_KT, lines), "{lines:?}");
        }
        assert_eq!(SHOW_HIDE_TARGET_SCALE.to_bits(), 0.2_f32.to_bits());
        assert_eq!(FabVisibility::default(), FabVisibility::Shown);
        assert_eq!(FabVisibility::Shown.progress().to_bits(), 1.0_f32.to_bits());
        assert_eq!(
            FabVisibility::Hidden.progress().to_bits(),
            0.0_f32.to_bits()
        );
    }
}
