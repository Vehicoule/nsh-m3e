//! The drawn container of [ADR 0011]: a component's container drawn by a canvas, whose corner radius is an animated
//! value that the M3E spring moves, so that the container can change shape when it is pressed.
//!
//! NSH animates no view's corner radius with a spring, but any number a canvas draws with can be an animated value,
//! which the platform moves frame by frame without the engine (ADR 0011, "What NSH can animate"). A [`DrawnShape`]
//! keeps that value and the container's size, and two canvases draw the container's path with it: the container
//! itself ([`DrawnShape::container`]), filled with the canvas view's background and stroked with its outline, and the
//! state layer over it ([`DrawnShape::state_layer`]), so both keep the same corners while they move.
//!
//! Only shapes whose four corners are the same can morph: [`Corners::of`] tells. Every container shape of the button
//! and icon button tokens is one.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md

use std::f32::consts::{FRAC_PI_2, PI};

use nsh_core::SvgPaint;
use nsh_core::geometry::{Bounds, Points, Size};
use nsh_core::prelude::*;

use crate::motion::{SPRING_MASS, SpringToken};
use crate::tokens::Shape;

/// The corners of a shape that can morph: the same at each corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Corners {
    /// Each corner rounded by half the container's shorter side: [`Shape::Circle`].
    Full,
    /// Each corner rounded by this radius (0 for [`Shape::Rectangle`]).
    Radius(Points),
}

impl Corners {
    /// The corners of `shape`, or `None` when its corners differ, which no drawn container draws.
    pub(crate) fn of(shape: Shape) -> Option<Self> {
        match shape {
            Shape::Rectangle => Some(Self::Radius(Points(0.0))),
            Shape::Circle => Some(Self::Full),
            Shape::Rounded(c) => (c.top_start == c.top_end
                && c.top_end == c.bottom_end
                && c.bottom_end == c.bottom_start)
                .then_some(Self::Radius(c.top_start)),
        }
    }

    /// The radius of these corners on a container of `size`. A radius larger than half the shorter side is drawn as
    /// that half, as a view's rounded corners are; this gives the radius itself, so that a spring to it ends where it
    /// should.
    fn radius(self, size: Size<Points>) -> f32 {
        match self {
            Self::Full => half_shorter_side(size),
            Self::Radius(radius) => radius.0,
        }
    }
}

/// The corners the drawn container gives `shape`. Every container shape of the component tokens has the same four
/// corners (the components' `every_*_shape_can_morph` tests check theirs), so the fully rounded fallback is never
/// taken; it is there so that a token with uneven corners, which no drawn container can draw, still gives a
/// component, not a panic.
pub(crate) fn corners(shape: Shape) -> Corners {
    Corners::of(shape).unwrap_or(Corners::Full)
}

/// Half of `size`'s shorter side: the largest radius its corners can take.
fn half_shorter_side(size: Size<Points>) -> f32 {
    size.width.0.min(size.height.0) / 2.0
}

/// The spring of the morph, from `token`, converted as ADR 0004 says: androidx `Button.kt` animates the button's shape
/// with `MotionSchemeKeyTokens.DefaultEffects`, "intentional here to prevent any bounce in this component" (ADR 0011);
/// `ToggleButton.kt` animates the toggle button's with `FastSpatial`. Every field is written out, none taken from NSH's
/// defaults: it starts from the speed the corners already have, at the screen's own rate (ADR 0010).
fn morph_spring(token: SpringToken) -> Spring {
    let spec = token.spec();
    Spring {
        stiffness: spec.stiffness,
        damping: spec.damping_coefficient(),
        mass: SPRING_MASS,
        velocity: None,
        frame_rate: None,
    }
}

/// A container drawn with an animated corner radius, and the state that moves it: its shape at rest, while pressed and
/// while checked, whether it is pressed and checked, its size, and the spring that moves it. A `Copy` handle, like the
/// signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DrawnShape {
    /// The corner radius, in points.
    radius: AnimatedValue,
    /// The container's size, from its layout.
    size: Signal<Size<Points>>,
    /// The corners at rest.
    rest: Signal<Corners>,
    /// The corners while pressed.
    pressed_corners: Signal<Corners>,
    /// Whether a press is down.
    pressed: Signal<bool>,
    /// The corners while checked and not pressed: a toggle button's; a button, never checked, keeps its rest corners.
    checked_corners: Signal<Corners>,
    /// Whether it is checked.
    checked: Signal<bool>,
    /// The spring that moves the radius.
    spring: SpringToken,
}

impl DrawnShape {
    /// A container at rest with `rest`'s corners, pressed into `pressed`'s, expected to be `size` until its layout
    /// says otherwise.
    /// It is never checked, and its spring is the button's, `DefaultEffects`.
    pub(crate) fn new(rest: Corners, pressed: Corners, size: Size<Points>) -> Self {
        Self::toggle(
            [rest, pressed, rest],
            false,
            size,
            SpringToken::DefaultEffects,
        )
    }

    /// A container with the `[rest, pressed, checked]` corners, `checked` or not, expected to be `size` until its
    /// layout says otherwise, moved by `spring`.
    pub(crate) fn toggle(
        [rest, pressed, checked_corners]: [Corners; 3],
        checked: bool,
        size: Size<Points>,
        spring: SpringToken,
    ) -> Self {
        let start = if checked { checked_corners } else { rest };
        Self {
            radius: animated_value(start.radius(size)),
            size: signal(size),
            rest: signal(rest),
            pressed_corners: signal(pressed),
            pressed: signal(false),
            checked_corners: signal(checked_corners),
            checked: signal(checked),
            spring,
        }
    }

    /// The corners it is at, or going to: pressed first, as androidx's `shapeByInteraction` in `ToggleButton.kt`
    /// chooses, then checked, then at rest.
    fn target(self) -> Corners {
        if self.pressed.get() {
            self.pressed_corners.get()
        } else {
            self.unpressed()
        }
    }

    /// The corners it has when no press is down: checked or at rest.
    fn unpressed(self) -> Corners {
        if self.checked.get() {
            self.checked_corners.get()
        } else {
            self.rest.get()
        }
    }

    /// The corners change (another size or shape of the component): the radius goes to the new target at once,
    /// stopping any spring still moving it, as androidx starts its shape animation again when the shapes change
    /// (`key(shapes)` in `Button.kt`'s `shapeByInteraction`). This holds even when the radius it goes to is the one
    /// it was going to, as when a press is down and both shapes press into the same corners. Nothing is sent when
    /// the corners stay the same.
    pub(crate) fn set_corners(self, rest: Corners, pressed: Corners) {
        untracked(|| {
            if self.rest.get() == rest && self.pressed_corners.get() == pressed {
                return;
            }
            self.rest.set(rest);
            self.pressed_corners.set(pressed);
            self.radius.set(self.target().radius(self.size.get()));
        });
    }

    /// The checked corners change, as [`DrawnShape::set_corners`] does for the others.
    pub(crate) fn set_checked_corners(self, checked_corners: Corners) {
        untracked(|| {
            if self.checked_corners.get() == checked_corners {
                return;
            }
            self.checked_corners.set(checked_corners);
            self.radius.set(self.target().radius(self.size.get()));
        });
    }

    /// It was checked (`true`) or unchecked: the radius springs to the corners it now has, from where it is and as fast
    /// as it moves. With reduce motion on, it goes at once to the corners it has when no press is down, which a press
    /// does not change then: being checked is a state the shape shows, where a press is feedback that can be left out.
    pub(crate) fn set_checked(self, checked: bool) {
        untracked(|| {
            if self.checked.get() == checked {
                return;
            }
            self.checked.set(checked);
            if reduce_motion() {
                self.radius.set(self.unpressed().radius(self.size.get()));
            } else if !self.pressed.get() {
                // A press down keeps the pressed corners whatever it is: its end springs to the new ones.
                let _ = self.radius.spring(
                    self.target().radius(self.size.get()),
                    morph_spring(self.spring),
                );
            }
        });
    }

    /// The container was laid out at `size`. A full radius follows its shorter side at once; a radius in points does
    /// not change.
    pub(crate) fn resize(self, size: Size<Points>) {
        untracked(|| {
            let target = self.target();
            let before = target.radius(self.size.get());
            self.size.set(size);
            let after = target.radius(size);
            if after.to_bits() != before.to_bits() {
                self.radius.set(after);
            }
        });
    }

    /// A press went down (`true`) or ended (`false`): the radius springs to the pressed or the resting corners, from
    /// where it is and as fast as it moves. With reduce motion on, nothing moves and the shape stays as it is
    /// (ADR 0011, "Reduce motion").
    pub(crate) fn press(self, down: bool) {
        untracked(|| {
            if self.pressed.get() == down {
                return;
            }
            self.pressed.set(down);
            if reduce_motion() {
                return;
            }
            let _ = self.radius.spring(
                self.target().radius(self.size.get()),
                morph_spring(self.spring),
            );
        });
    }

    /// The container: a canvas filling the component (place it first among its children), which draws the
    /// container's path with the current radius, filled with the canvas view's background and, when `outline` gives
    /// a width, stroked inside the path with the canvas view's `stroke` color at that width. Its layout gives the
    /// container's size.
    pub(crate) fn container(
        self,
        outline: impl Fn() -> Option<Points> + Copy + 'static,
    ) -> CanvasView {
        layer(canvas(move || {
            let size = self.size.get();
            let filled = self.fill(Painter::new(), size, 0.0);
            match outline() {
                Some(width) => trace(filled.begin_path(), &self.radius, size, width.0 / 2.0)
                    .stroke_style_from_view(SvgPaint::None)
                    .line_width_from_view()
                    .stroke(),
                None => filled,
            }
        }))
        .stroke_width(move || outline().unwrap_or(Points(0.0)))
        .on_layout(move |bounds: &Bounds<Points>| self.resize(bounds.size))
    }

    /// The state layer: a canvas over the container, which fills the inside of the container's outline (all of the
    /// container when `outline` gives none) with the canvas view's background, its corners concentric with the
    /// container's. The outline stays as it is under hover, focus and press, as androidx draws a button's border
    /// over its ripple (`Modifier.border` draws after its content). It does not give the container's size: a hidden
    /// state layer is laid out at no size at all.
    pub(crate) fn state_layer(
        self,
        outline: impl Fn() -> Option<Points> + Copy + 'static,
    ) -> CanvasView {
        layer(canvas(move || {
            self.fill(
                Painter::new(),
                self.size.get(),
                outline().map_or(0.0, |width| width.0),
            )
        }))
    }

    /// Adds to `painter` the container's path `inset` points inside its edge, filled with the canvas view's
    /// background.
    fn fill(self, painter: Painter, size: Size<Points>, inset: f32) -> Painter {
        let area = Bounds::new(point(px(0.0), px(0.0)), size);
        trace(painter.begin_path(), &self.radius, size, inset).fill_background_from_view(area)
    }
}

/// `canvas` filling the component: `absolute`, inset by 0.
fn layer(canvas: CanvasView) -> CanvasView {
    canvas.absolute().inset(px(0.0))
}

/// Adds to `painter` the outline of a rounded rectangle `inset` points inside a container of `size`, its corners
/// rounded by `radius` less `inset`: four lines and four arcs, clockwise from the end of the top-left corner (ADR
/// 0011). The radius is clamped to what the inset rectangle can take, as a view's corners are. A container too small
/// to hold the inset rectangle adds nothing.
fn trace(painter: Painter, radius: &AnimatedValue, size: Size<Points>, inset: f32) -> Painter {
    let half = half_shorter_side(size);
    if half <= inset {
        return painter;
    }
    // `radius` from `inset` to `half` draws as 0 to `half - inset`: the stroke's middle stays parallel to the
    // container's edge, and the clamp keeps the corners within the rectangle.
    let r: AnimatedNode = radius.interpolate(&[inset, half], &[0.0, half - inset]);
    let (left, top) = (inset, inset);
    let (right, bottom) = (size.width.0 - inset, size.height.0 - inset);
    let near_x = r.add(left);
    let near_y = r.add(top);
    let far_x = r.mul(-1.0).add(right);
    let far_y = r.mul(-1.0).add(bottom);
    painter
        .move_to(&near_x, top)
        .line_to(&far_x, top)
        .arc(&far_x, &near_y, &r, -FRAC_PI_2, 0.0)
        .line_to(right, &far_y)
        .arc(&far_x, &far_y, &r, 0.0, FRAC_PI_2)
        .line_to(&near_x, bottom)
        .arc(&near_x, &far_y, &r, FRAC_PI_2, PI)
        .line_to(left, &near_y)
        .arc(&near_x, &near_y, &r, PI, PI + FRAC_PI_2)
        .close_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_shapes_with_four_equal_corners_can_morph() {
        assert_eq!(
            Corners::of(Shape::Rectangle),
            Some(Corners::Radius(Points(0.0)))
        );
        assert_eq!(Corners::of(Shape::Circle), Some(Corners::Full));
        let even = Shape::rounded(Points(12.0), Points(12.0), Points(12.0), Points(12.0));
        assert_eq!(Corners::of(even), Some(Corners::Radius(Points(12.0))));
        // Each corner that differs from the others makes a shape no drawn container draws.
        for uneven in [
            Shape::rounded(Points(4.0), Points(12.0), Points(12.0), Points(12.0)),
            Shape::rounded(Points(12.0), Points(4.0), Points(12.0), Points(12.0)),
            Shape::rounded(Points(12.0), Points(12.0), Points(4.0), Points(12.0)),
            Shape::rounded(Points(12.0), Points(12.0), Points(12.0), Points(4.0)),
        ] {
            assert_eq!(Corners::of(uneven), None, "{uneven:?}");
        }
    }

    #[test]
    fn a_full_radius_follows_the_shorter_side_and_a_radius_in_points_does_not() {
        let wide = Size::new(Points(100.0), Points(40.0));
        let tall = Size::new(Points(30.0), Points(40.0));
        assert_eq!(Corners::Full.radius(wide), 20.0);
        assert_eq!(Corners::Full.radius(tall), 15.0);
        assert_eq!(Corners::Radius(Points(12.0)).radius(wide), 12.0);
        assert_eq!(Corners::Radius(Points(12.0)).radius(tall), 12.0);
    }
}
