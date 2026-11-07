//! What the navigation bar's items share with the other navigation components' (androidx `NavigationItem.kt`): where
//! an item's icon sits, the opacity of a disabled item, and the indicator that grows behind a selected item's icon.
//!
//! The indicator is a pill drawn by a canvas, not a view whose width is laid out: androidx measures it again on every
//! frame of its animation (`NavigationItem.kt`, `animatedIndicatorWidth`), which the Motion requirement of
//! `docs/app-requirements.md` forbids, so its width is an animated value that the M3E spring moves and the canvas draws
//! with, as the button's corner radius is ([ADR 0011]).
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md

use std::f32::consts::{FRAC_PI_2, PI};

use nsh_core::animation::{Easing, Extrapolate};
use nsh_core::element::{AnimatedNode, AnimationOutput, Interpolation};
use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::prelude::*;

use crate::motion::{SPRING_MASS, SpringToken};

/// Where the icon of a navigation item sits (androidx `NavigationItemIconPosition`).
///
/// Material recommends the icon on top for a bar in a small window (a phone held upright) and at the start for a bar
/// in a medium one (a phone held sideways).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum NavigationItemIconPosition {
    /// The icon above the label.
    #[default]
    Top,
    /// The icon before the label.
    Start,
}

/// Whether a navigation item shows its label. Its label is its name for screen readers either way.
///
/// androidx takes a label that may be missing (`label: (@Composable () -> Unit)?`) and recommends "for navigation
/// items to always have a text label" (`ShortNavigationBar.kt`, `ShortNavigationBarItem`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum NavigationItemLabel {
    /// The label is shown: Material's recommendation.
    #[default]
    Shown,
    /// Only the icon is shown.
    Hidden,
}

/// The opacity of a disabled item's icon and label: androidx `DisabledAlpha`, "a low level of alpha used to represent
/// disabled components" (`ColorScheme.kt`, line 1544). No token holds it for the navigation components.
pub const DISABLED_OPACITY: f32 = 0.38;

/// The spring of the indicator's growth and fade: androidx `NavigationItem.kt`, `animateIndicatorProgressAsState`
/// animates the indicator's progress with `MotionSchemeKeyTokens.DefaultSpatial`, converted as ADR 0004 says. Every
/// field is written out, none taken from NSH's defaults: it starts from the speed the width already has, at the
/// screen's own rate (ADR 0010).
fn indicator_spring() -> Spring {
    let spec = SpringToken::DefaultSpatial.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// The indicator of a navigation item: a pill that grows from nothing to the item's indicator area, and fades in, as
/// the item is selected, and the other way as it is not. A `Copy` handle, like the signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Indicator {
    /// How far the indicator has grown: 0 for none, 1 for the whole area. The spring overshoots 1 when it is going
    /// there (the pill is briefly wider than its area, as in androidx) and undershoots 0 when it is going back,
    /// where the drawing stops at 0.
    progress: AnimatedValue,
    /// The area the indicator fills, from its layout.
    size: Signal<Size<Points>>,
    /// Whether the item is selected: where the progress is going.
    selected: Signal<bool>,
}

impl Indicator {
    /// An indicator `selected` or not, already grown or not, expected to fill `size` until its layout says otherwise.
    pub(crate) fn new(selected: bool, size: Size<Points>) -> Self {
        Self {
            progress: animated_value(if selected { 1.0 } else { 0.0 }),
            size: signal(size),
            selected: signal(selected),
        }
    }

    /// The item is selected (`true`) or not: the indicator springs to its full width, or to none, from where it is
    /// and as fast as it moves, as androidx's `animateFloatAsState` does. With reduce motion on it goes there at once,
    /// as ADR 0011 says for the button's corners. Nothing is sent when the item stays as it is.
    pub(crate) fn select(self, selected: bool) {
        untracked(|| {
            if self.selected.get() == selected {
                return;
            }
            self.selected.set(selected);
            let to = if selected { 1.0 } else { 0.0 };
            if reduce_motion() {
                self.progress.set(to);
            } else {
                let _ = self.progress.spring(to, indicator_spring());
            }
        });
    }

    /// The area was laid out at `size`.
    fn resize(self, size: Size<Points>) {
        untracked(|| {
            if self.size.get() != size {
                self.size.set(size);
            }
        });
    }

    /// The indicator: a canvas filling its area (place it first among the area's children, under the icon), whose
    /// layout gives the area's size. It fills the pill with the canvas view's background and fades with the progress.
    /// It takes no presses: the item does.
    pub(crate) fn canvas(self) -> CanvasView {
        // The fade is the progress clamped to 0 and 1: a view's opacity is no more than 1.
        let fade: AnimatedNode = self.progress.interpolate(&[0.0, 1.0], &[0.0, 1.0]);
        canvas(move || self.drawing())
            .absolute()
            .inset(px(0.0))
            .pointer_events(PointerEvents::None)
            .bind_opacity(&fade)
            .on_layout(move |bounds: &Bounds<Points>| self.resize(bounds.size))
    }

    /// The pill, as wide as the progress says and as tall as the area, centered in it, filled with the canvas view's
    /// background: a line and a half circle for each end, clockwise from the end of the top left corner, as ADR 0011
    /// traces a rounded rectangle.
    fn drawing(self) -> Painter {
        let size = self.size.get();
        let (width, height) = (size.width.0, size.height.0);
        if !(width > 0.0 && height > 0.0) {
            return Painter::new();
        }
        // `Painter` draws nothing below zero width: the spring undershoots 0 when the pill goes away, and a width
        // below 0 would draw it mirrored. Above the area it goes on, as the spring overshoots.
        let wide: AnimatedNode = self.progress.interpolate_with(
            &[0.0, 1.0],
            AnimationOutput::Numbers(vec![0.0, width]),
            Interpolation {
                extrapolate_left: Extrapolate::Clamp,
                extrapolate_right: Extrapolate::Extend,
                easing: Easing::Linear,
            },
        );
        // androidx's `CornerFull` is half the shorter side: the pill narrower than it is tall is a capsule on its end.
        let radius: AnimatedNode = wide.interpolate(&[0.0, height], &[0.0, height / 2.0]);
        let left = wide.mul(-0.5).add(width / 2.0);
        let right = wide.mul(0.5).add(width / 2.0);
        let near_x = left.add(&radius);
        let far_x = right.sub(&radius);
        let near_y = radius.add(0.0);
        let far_y = radius.mul(-1.0).add(height);
        let area = Bounds::new(point(px(0.0), px(0.0)), size);
        Painter::new()
            .begin_path()
            .move_to(&near_x, 0.0)
            .line_to(&far_x, 0.0)
            .arc(&far_x, &near_y, &radius, -FRAC_PI_2, 0.0)
            .line_to(&right, &far_y)
            .arc(&far_x, &far_y, &radius, 0.0, FRAC_PI_2)
            .line_to(&near_x, height)
            .arc(&near_x, &far_y, &radius, FRAC_PI_2, PI)
            .line_to(&left, &near_y)
            .arc(&near_x, &near_y, &radius, PI, PI + FRAC_PI_2)
            .close_path()
            .fill_background_from_view(area)
    }
}
