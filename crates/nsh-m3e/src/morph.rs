//! The drawn container of [ADR 0011]: a component's container drawn by a canvas, whose corner radius is an animated
//! value that the M3E spring moves, so that the container can change shape when it is pressed.
//!
//! NSH animates no view's corner radius with a spring, but any number a canvas draws with can be an animated value,
//! which the platform moves frame by frame without the engine (ADR 0011, "What NSH can animate"). A [`DrawnShape`]
//! keeps that value and the container's size, and two canvases draw the container's path with it: the container
//! itself ([`DrawnShape::container`]), filled with the canvas view's background and stroked with its outline, and the
//! state layer over it ([`DrawnShape::state_layer`]), so both keep the same corners while they move.
//!
//! A shape that can morph has its two start corners (top and bottom) alike and its two end corners alike: a
//! [`SideCorners`]. Most have their four corners alike, which [`Corners::of`] tells; every container shape of the
//! button and icon button tokens is one. A connected button group's items round their start and end corners apart
//! (androidx `ButtonGroupDefaults.connectedLeadingButtonShape` and the others in `ButtonGroup.kt`): their drawn
//! container moves a second radius for its end corners, and a right-to-left layout draws the start corners on the
//! right.
//!
//! [ADR 0011]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0011-morph-shapes-with-a-drawn-container.md

use std::f32::consts::{FRAC_PI_2, PI};

use nsh_core::SvgPaint;
use nsh_core::element::layout_rtl;
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

/// The corners of a shape that can morph, side by side: its top and bottom corners at the start alike, and its top and
/// bottom corners at the end alike.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SideCorners {
    /// The corners at the start: on the left, or on the right in a right-to-left layout.
    pub(crate) start: Corners,
    /// The corners at the end.
    pub(crate) end: Corners,
}

impl SideCorners {
    /// Whether its start and end corners are alike.
    fn even(self) -> bool {
        self.start == self.end
    }
}

impl From<Corners> for SideCorners {
    fn from(corners: Corners) -> Self {
        Self {
            start: corners,
            end: corners,
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

/// A container drawn with animated corner radii, and the state that moves them: its shape at rest, while pressed and
/// while checked, whether it is pressed and checked, its size, and the spring that moves it. A `Copy` handle, like the
/// signals it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DrawnShape {
    /// The radius of the start corners, in points; of all four while it is not [`DrawnShape::sided`].
    start: AnimatedValue,
    /// The radius of the end corners, in points, which only a sided container moves and draws.
    end: AnimatedValue,
    /// Whether a shape it was given has its start and end corners apart. Until then only the start radius moves, so
    /// that a container whose corners are all alike starts one spring, not two; once it is, it stays so.
    sided: Signal<bool>,
    /// The container's size, from its layout.
    size: Signal<Size<Points>>,
    /// The corners at rest.
    rest: Signal<SideCorners>,
    /// The corners while pressed.
    pressed_corners: Signal<SideCorners>,
    /// Whether a press is down.
    pressed: Signal<bool>,
    /// The corners while checked and not pressed: a toggle button's; a button, never checked, keeps its rest corners.
    checked_corners: Signal<SideCorners>,
    /// Whether it is checked.
    checked: Signal<bool>,
    /// The spring that moves the radii.
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
    pub(crate) fn toggle<C: Into<SideCorners>>(
        shapes: [C; 3],
        checked: bool,
        size: Size<Points>,
        spring: SpringToken,
    ) -> Self {
        let [rest, pressed, checked_corners] = shapes.map(Into::into);
        let first = if checked { checked_corners } else { rest };
        Self {
            start: animated_value(first.start.radius(size)),
            end: animated_value(first.end.radius(size)),
            sided: signal(!(rest.even() && pressed.even() && checked_corners.even())),
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
    fn target(self) -> SideCorners {
        if self.pressed.get() {
            self.pressed_corners.get()
        } else {
            self.unpressed()
        }
    }

    /// The corners it has when no press is down: checked or at rest.
    fn unpressed(self) -> SideCorners {
        if self.checked.get() {
            self.checked_corners.get()
        } else {
            self.rest.get()
        }
    }

    /// The corners change (another size or shape of the component): the radii go to the new target at once,
    /// stopping any spring still moving them, as androidx starts its shape animation again when the shapes change
    /// (`key(shapes)` in `Button.kt`'s `shapeByInteraction`). This holds even when the radii they go to are the ones
    /// they were going to, as when a press is down and both shapes press into the same corners. Nothing is sent when
    /// the corners stay the same.
    pub(crate) fn set_corners(self, rest: impl Into<SideCorners>, pressed: impl Into<SideCorners>) {
        let (rest, pressed) = (rest.into(), pressed.into());
        untracked(|| {
            if self.rest.get() == rest && self.pressed_corners.get() == pressed {
                return;
            }
            self.rest.set(rest);
            self.pressed_corners.set(pressed);
            self.take_sides(&[rest, pressed]);
            self.jump_to(self.target());
        });
    }

    /// The checked corners change, as [`DrawnShape::set_corners`] does for the others.
    pub(crate) fn set_checked_corners(self, checked_corners: impl Into<SideCorners>) {
        let checked_corners = checked_corners.into();
        untracked(|| {
            if self.checked_corners.get() == checked_corners {
                return;
            }
            self.checked_corners.set(checked_corners);
            self.take_sides(&[checked_corners]);
            self.jump_to(self.target());
        });
    }

    /// It was checked (`true`) or unchecked: the radii spring to the corners it now has, from where they are and as
    /// fast as they move. With reduce motion on, they go at once to the corners it has when no press is down, which a
    /// press does not change then: being checked is a state the shape shows, where a press is feedback that can be
    /// left out.
    pub(crate) fn set_checked(self, checked: bool) {
        untracked(|| {
            if self.checked.get() == checked {
                return;
            }
            self.checked.set(checked);
            if reduce_motion() {
                self.jump_to(self.unpressed());
            } else if !self.pressed.get() {
                // A press down keeps the pressed corners whatever it is: its end springs to the new ones.
                self.spring_to(self.target());
            }
        });
    }

    /// The container was laid out at `size`. A full radius follows its shorter side at once; a radius in points does
    /// not change.
    pub(crate) fn resize(self, size: Size<Points>) {
        untracked(|| {
            let target = self.target();
            let before = self.size.get();
            self.size.set(size);
            let follow = |radius: AnimatedValue, corners: Corners| {
                let after = corners.radius(size);
                if after.to_bits() != corners.radius(before).to_bits() {
                    radius.set(after);
                }
            };
            follow(self.start, target.start);
            if self.sided.get() {
                follow(self.end, target.end);
            }
        });
    }

    /// A press went down (`true`) or ended (`false`): the radii spring to the pressed or the resting corners, from
    /// where they are and as fast as they move. With reduce motion on, nothing moves and the shape stays as it is
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
            self.spring_to(self.target());
        });
    }

    /// Becomes sided if one of `shapes` has its start and end corners apart. The end radius, which did not move
    /// until then, is set by the [`DrawnShape::jump_to`] that follows.
    fn take_sides(self, shapes: &[SideCorners]) {
        if !self.sided.get() && shapes.iter().any(|shape| !shape.even()) {
            self.sided.set(true);
        }
    }

    /// The radii go to `corners` at once.
    fn jump_to(self, corners: SideCorners) {
        let size = self.size.get();
        self.start.set(corners.start.radius(size));
        if self.sided.get() {
            self.end.set(corners.end.radius(size));
        }
    }

    /// The radii spring to `corners`, from where they are and as fast as they move.
    fn spring_to(self, corners: SideCorners) {
        let size = self.size.get();
        let spring = morph_spring(self.spring);
        let _ = self.start.spring(corners.start.radius(size), spring);
        if self.sided.get() {
            let _ = self.end.spring(corners.end.radius(size), spring);
        }
    }

    /// The container: a canvas filling the component (place it first among its children), which draws the
    /// container's path with the current radii, filled with the canvas view's background and, when `outline` gives
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
                Some(width) => self
                    .trace(filled.begin_path(), size, width.0 / 2.0)
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
        self.trace(painter.begin_path(), size, inset)
            .fill_background_from_view(area)
    }

    /// Adds to `painter` the container's path `inset` points inside its edge: [`trace`] with the start radius on the
    /// left and the end one on the right, or the other way round in a right-to-left layout. A container that is not
    /// sided draws the start radius at all four corners. Reading the layout's direction subscribes, as `nsh-base`'s
    /// slider mirrors itself by it.
    fn trace(self, painter: Painter, size: Size<Points>, inset: f32) -> Painter {
        let end = if self.sided.get() {
            &self.end
        } else {
            &self.start
        };
        let (left, right) = if layout_rtl() {
            (end, &self.start)
        } else {
            (&self.start, end)
        };
        trace(painter, [left, right], size, inset)
    }
}

/// `canvas` filling the component: `absolute`, inset by 0.
fn layer(canvas: CanvasView) -> CanvasView {
    canvas.absolute().inset(px(0.0))
}

/// Adds to `painter` the outline of a rounded rectangle `inset` points inside a container of `size`, its left corners
/// rounded by the first of `radii` less `inset` and its right corners by the second: four lines and four arcs,
/// clockwise from the end of the top-left corner (ADR 0011). Each radius is clamped to what the inset rectangle can
/// take, as a view's corners are; two clamped radii fit side by side, since each is at most half the shorter side. A
/// container too small to hold the inset rectangle adds nothing.
fn trace(painter: Painter, radii: [&AnimatedValue; 2], size: Size<Points>, inset: f32) -> Painter {
    let half = half_shorter_side(size);
    if half <= inset {
        return painter;
    }
    // A radius from `inset` to `half` draws as 0 to `half - inset`: the stroke's middle stays parallel to the
    // container's edge, and the clamp keeps the corners within the rectangle.
    let [l, r]: [AnimatedNode; 2] =
        radii.map(|radius| radius.interpolate(&[inset, half], &[0.0, half - inset]));
    let (left, top) = (inset, inset);
    let (right, bottom) = (size.width.0 - inset, size.height.0 - inset);
    let left_x = l.add(left);
    let left_top_y = l.add(top);
    let left_bottom_y = l.mul(-1.0).add(bottom);
    let right_x = r.mul(-1.0).add(right);
    let right_top_y = r.add(top);
    let right_bottom_y = r.mul(-1.0).add(bottom);
    painter
        .move_to(&left_x, top)
        .line_to(&right_x, top)
        .arc(&right_x, &right_top_y, &r, -FRAC_PI_2, 0.0)
        .line_to(right, &right_bottom_y)
        .arc(&right_x, &right_bottom_y, &r, 0.0, FRAC_PI_2)
        .line_to(&left_x, bottom)
        .arc(&left_x, &left_bottom_y, &l, FRAC_PI_2, PI)
        .line_to(left, &left_top_y)
        .arc(&left_x, &left_top_y, &l, PI, PI + FRAC_PI_2)
        .close_path()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::platform::{Environment, Os};
    use nsh_core::{MountItem, NodeId};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::{Recording, Replay, StartedSpring, layout_items, started_springs};
    use crate::morph_probe::{Step, assert_traces, paths, replay, rounded_sides, set_values};

    /// The probe's container: as wide as a connected group's leading button might be, 40 points high
    /// (`ConnectedButtonGroupSmallTokens.ContainerHeight`), so that a full radius is 20.
    const SIZE: Size<Points> = Size {
        width: Points(100.0),
        height: Points(40.0),
    };

    /// The width the probe narrows its container to: narrower than high, so that a full radius is 15.
    const NARROW: f32 = 30.0;

    /// The outline width the probe's state layer is drawn inside.
    const OUTLINE: f32 = 1.0;

    /// A connected group's leading button (androidx `ButtonGroupDefaults` in `ButtonGroup.kt`): fully rounded at its
    /// start and rounded by `InnerCornerCornerSize` (8) at its end; pressed, by `PressedInnerCornerCornerSize` (4);
    /// checked, fully rounded. The values are written here, not taken from tokens, because only the drawing of two
    /// sides is under test; the group takes them from `ConnectedButtonGroupSmallTokens`.
    const LEADING: [SideCorners; 3] = [
        SideCorners {
            start: Corners::Full,
            end: Corners::Radius(Points(8.0)),
        },
        SideCorners {
            start: Corners::Full,
            end: Corners::Radius(Points(4.0)),
        },
        SideCorners {
            start: Corners::Full,
            end: Corners::Full,
        },
    ];

    /// A shape whose four corners are alike at rest, pressed and checked.
    const EVEN: [SideCorners; 3] = [
        SideCorners {
            start: Corners::Radius(Points(12.0)),
            end: Corners::Radius(Points(12.0)),
        },
        SideCorners {
            start: Corners::Radius(Points(8.0)),
            end: Corners::Radius(Points(8.0)),
        },
        SideCorners {
            start: Corners::Full,
            end: Corners::Full,
        },
    ];

    /// Shapes whose start and end corners are apart only at rest, only pressed or only checked.
    const APART_AT_REST: [SideCorners; 3] = [LEADING[0], EVEN[1], EVEN[2]];
    const APART_PRESSED: [SideCorners; 3] = [EVEN[0], LEADING[1], EVEN[2]];
    const APART_CHECKED: [SideCorners; 3] = [
        EVEN[0],
        EVEN[1],
        SideCorners {
            start: Corners::Full,
            end: Corners::Radius(Points(16.0)),
        },
    ];

    /// The shapes the probe can start with.
    const SHAPES: [[SideCorners; 3]; 5] =
        [EVEN, LEADING, APART_AT_REST, APART_PRESSED, APART_CHECKED];

    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        /// Which of [`SHAPES`] it starts with.
        shapes: Prop<u32>,
    }

    /// A drawn container and its state layer, and lines that press it, end the press, check it, give it new corners
    /// and narrow it.
    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let shapes = usize::try_from(props.shapes.get_untracked())
            .ok()
            .and_then(|at| SHAPES.get(at))
            .copied()
            .unwrap_or(EVEN);
        let drawn = untracked(|| DrawnShape::toggle(shapes, false, SIZE, SpringToken::FastSpatial));
        let width = signal(SIZE.width.0);
        view()
            .child(
                view()
                    .w(move || px(width.get()))
                    .h(px(SIZE.height.0))
                    .child(drawn.container(|| None))
                    .child(drawn.state_layer(|| Some(Points(OUTLINE)))),
            )
            .child(text("Down").on_press(move |(): &()| drawn.press(true)))
            .child(text("Up").on_press(move |(): &()| drawn.press(false)))
            .child(text("Check").on_press(move |(): &()| drawn.set_checked(true)))
            .child(text("Even").on_press(move |(): &()| drawn.set_corners(EVEN[0], EVEN[1])))
            .child(
                text("Leading").on_press(move |(): &()| drawn.set_corners(LEADING[0], LEADING[1])),
            )
            .child(text("Square").on_press(move |(): &()| {
                drawn.set_checked_corners(Corners::Radius(Points(16.0)));
            }))
            .child(text("Narrow").on_press(move |(): &()| width.set(NARROW)))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        shapes: [SideCorners; 3],
        rtl: bool,
        reduce_motion: bool,
    }

    impl Config {
        fn of(shapes: [SideCorners; 3]) -> Self {
            Self {
                shapes,
                rtl: false,
                reduce_motion: false,
            }
        }
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            shapes: SHAPES
                .iter()
                .position(|shapes| *shapes == config.shapes)
                .and_then(|at| u32::try_from(at).ok())
                .map(Prop::new)
                .expect("one of the probe's shapes"),
        });
        let environment = Environment {
            os: Os::Web,
            rtl: config.rtl,
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
        app.wait_for("text:Narrow").unwrap();
        (app, recording)
    }

    /// The paths the container and the state layer last traced in `items`, the container first, with the animated
    /// values as `replay` has them.
    fn traced(items: &[MountItem], replay: &Replay) -> [Vec<Step>; 2] {
        let mut canvases: Vec<NodeId> = Vec::new();
        let mut drawings = HashMap::new();
        for item in items {
            if let MountItem::Canvas { id, drawing } = item {
                if !canvases.contains(id) {
                    canvases.push(*id);
                }
                drawings.insert(*id, Arc::clone(drawing));
            }
        }
        let [container, layer] = canvases[..] else {
            panic!("two canvases: {canvases:?}");
        };
        let path = |id: NodeId| {
            let mut traced = paths(&drawings[&id], replay);
            assert_eq!(traced.len(), 1, "one path: {traced:?}");
            traced.remove(0)
        };
        [path(container), path(layer)]
    }

    /// The container traces `[left, right]` radii on a container of `size`, and the state layer the same inside the
    /// outline, with the animated values as `replay` has them.
    fn assert_sides(
        recording: &Recording,
        replay: &Replay,
        size: Size<Points>,
        [l, r]: [f32; 2],
        what: &str,
    ) {
        let [container, layer] = traced(&recording.items(), replay);
        assert_traces(&container, &rounded_sides(size, 0.0, [l, r]), what);
        assert_traces(
            &layer,
            &rounded_sides(size, OUTLINE, [l - OUTLINE, r - OUTLINE]),
            what,
        );
    }

    /// `items` start one spring for each of `to`, in order, all with the probe's spring (`FastSpatial`, converted as
    /// ADR 0004 says), each on its own value, and set no value at once.
    fn springs_to(items: &[MountItem], to: &[f32], what: &str) -> Vec<StartedSpring> {
        let springs = started_springs(items);
        let targets: Vec<f32> = springs.iter().map(|spring| spring.to).collect();
        assert_eq!(targets, to, "{what}");
        let spec = SpringToken::FastSpatial.spec();
        for spring in &springs {
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
        }
        if let [first, second] = springs[..] {
            assert_ne!(first.node, second.node, "{what}");
        }
        assert_eq!(set_values(items), 0, "{what}: {items:?}");
        springs
    }

    /// Presses `selector` and gives what the engine sent meanwhile.
    fn press(app: &mut Headless, recording: &Recording, selector: &str) -> Vec<MountItem> {
        let (pressed, sent) = recording.during(|| app.press(selector));
        pressed.unwrap();
        sent
    }

    /// The recorded animations, played until `springs` settle.
    fn settled(recording: &Recording, springs: &[StartedSpring]) -> Replay {
        let mut replay = replay(recording);
        for spring in springs {
            let _ = replay.run(spring.node, 600);
        }
        assert!(!replay.is_animating());
        replay
    }

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

    #[test]
    fn a_shape_with_its_sides_apart_draws_its_start_on_the_left_or_on_the_right_to_left() {
        let (_app, recording) = start(Config::of(LEADING));
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [20.0, 8.0],
            "left to right",
        );
        let (_app, recording) = start(Config {
            rtl: true,
            ..Config::of(LEADING)
        });
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [8.0, 20.0],
            "right to left",
        );
    }

    #[test]
    fn a_press_springs_each_side_to_its_own_corners_and_lays_nothing_out() {
        for rtl in [false, true] {
            let what = format!("right to left: {rtl}");
            let (mut app, recording) = start(Config {
                rtl,
                ..Config::of(LEADING)
            });
            let mirror = |[start, end]: [f32; 2]| if rtl { [end, start] } else { [start, end] };
            let sent = press(&mut app, &recording, "Down");
            let down = springs_to(&sent, &[20.0, 4.0], &what);
            // The platform moves the corners: nothing is drawn again or laid out.
            assert!(
                !sent
                    .iter()
                    .any(|item| matches!(item, MountItem::Canvas { .. })),
                "{what}: {sent:?}"
            );
            assert_eq!(layout_items(&sent), 0, "{what}: {sent:?}");
            assert_sides(
                &recording,
                &settled(&recording, &down),
                SIZE,
                mirror([20.0, 4.0]),
                &what,
            );
            // Checked during the press: the corners wait for its end, then spring to the checked ones.
            let sent = press(&mut app, &recording, "Check");
            assert_eq!(started_springs(&sent), [], "{what}");
            assert_eq!(set_values(&sent), 0, "{what}");
            let sent = press(&mut app, &recording, "Up");
            let up = springs_to(&sent, &[20.0, 20.0], &what);
            assert_eq!(
                up.iter().map(|spring| spring.node).collect::<Vec<_>>(),
                down.iter().map(|spring| spring.node).collect::<Vec<_>>(),
                "{what}"
            );
            assert_sides(
                &recording,
                &settled(&recording, &up),
                SIZE,
                [20.0, 20.0],
                &what,
            );
        }
    }

    #[test]
    fn a_shape_with_four_alike_corners_moves_one_radius() {
        let (mut app, recording) = start(Config::of(EVEN));
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [12.0, 12.0],
            "at rest",
        );
        let sent = press(&mut app, &recording, "Down");
        let down = springs_to(&sent, &[8.0], "pressed");
        assert_sides(
            &recording,
            &settled(&recording, &down),
            SIZE,
            [8.0, 8.0],
            "pressed",
        );
        let sent = press(&mut app, &recording, "Up");
        springs_to(&sent, &[12.0], "released");
        let sent = press(&mut app, &recording, "Check");
        let checked = springs_to(&sent, &[20.0], "checked");
        assert_sides(
            &recording,
            &settled(&recording, &checked),
            SIZE,
            [20.0, 20.0],
            "checked",
        );
        // New checked corners and a new width each set the one radius at once.
        let sent = press(&mut app, &recording, "Square");
        assert_eq!(started_springs(&sent), [], "square");
        assert_eq!(set_values(&sent), 1, "square: {sent:?}");
        let sent = press(&mut app, &recording, "Narrow");
        assert_eq!(set_values(&sent), 0, "a radius in points stays: {sent:?}");
    }

    #[test]
    fn new_corners_go_at_once_and_the_first_with_its_sides_apart_moves_the_end_too() {
        let (mut app, recording) = start(Config::of(EVEN));
        let sent = press(&mut app, &recording, "Even");
        assert!(sent.is_empty(), "the same corners send nothing: {sent:?}");
        let sent = press(&mut app, &recording, "Leading");
        assert_eq!(started_springs(&sent), [], "{sent:?}");
        assert_eq!(set_values(&sent), 2, "{sent:?}");
        assert_eq!(layout_items(&sent), 0, "{sent:?}");
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [20.0, 8.0],
            "leading",
        );
        let sent = press(&mut app, &recording, "Down");
        let down = springs_to(&sent, &[20.0, 4.0], "pressed");
        assert_sides(
            &recording,
            &settled(&recording, &down),
            SIZE,
            [20.0, 4.0],
            "pressed",
        );
        // Back to corners all alike: it stays sided, and both sides go at once to the pressed ones.
        let sent = press(&mut app, &recording, "Even");
        assert_eq!(set_values(&sent), 2, "{sent:?}");
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [8.0, 8.0],
            "even again",
        );
    }

    #[test]
    fn new_checked_corners_go_at_once_on_both_sides() {
        let (mut app, recording) = start(Config::of(LEADING));
        let sent = press(&mut app, &recording, "Square");
        assert_eq!(started_springs(&sent), [], "{sent:?}");
        assert_eq!(set_values(&sent), 2, "{sent:?}");
        let sent = press(&mut app, &recording, "Check");
        let checked = springs_to(&sent, &[16.0, 16.0], "checked");
        assert_sides(
            &recording,
            &settled(&recording, &checked),
            SIZE,
            [16.0, 16.0],
            "checked",
        );
    }

    #[test]
    fn with_reduce_motion_checked_goes_at_once_on_both_sides_and_a_press_moves_nothing() {
        let (mut app, recording) = start(Config {
            reduce_motion: true,
            ..Config::of(LEADING)
        });
        let sent = press(&mut app, &recording, "Down");
        assert_eq!(started_springs(&sent), [], "{sent:?}");
        assert_eq!(set_values(&sent), 0, "{sent:?}");
        let sent = press(&mut app, &recording, "Check");
        assert_eq!(started_springs(&sent), [], "{sent:?}");
        assert_eq!(set_values(&sent), 2, "{sent:?}");
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [20.0, 20.0],
            "checked",
        );
    }

    #[test]
    fn a_full_side_follows_the_shorter_side_at_once_and_a_side_in_points_does_not() {
        let (mut app, recording) = start(Config::of(LEADING));
        let sent = press(&mut app, &recording, "Narrow");
        assert_eq!(started_springs(&sent), [], "{sent:?}");
        assert_eq!(set_values(&sent), 1, "{sent:?}");
        let narrow = Size::new(Points(NARROW), SIZE.height);
        assert_sides(
            &recording,
            &replay(&recording),
            narrow,
            [15.0, 8.0],
            "narrow",
        );
        // Checked, both sides are full: both follow.
        let (mut app, recording) = start(Config::of(LEADING));
        let sent = press(&mut app, &recording, "Check");
        let checked = springs_to(&sent, &[20.0, 20.0], "checked");
        let _ = settled(&recording, &checked);
        let sent = press(&mut app, &recording, "Narrow");
        assert_eq!(set_values(&sent), 2, "{sent:?}");
        assert_sides(
            &recording,
            &replay(&recording),
            narrow,
            [15.0, 15.0],
            "narrow and checked",
        );
    }

    #[test]
    fn corners_alike_make_alike_sides() {
        let full = SideCorners::from(Corners::Full);
        assert_eq!(full.start, Corners::Full);
        assert_eq!(full.end, Corners::Full);
        assert!(full.even());
        assert!(!LEADING[0].even());
        assert!(EVEN.iter().all(|shape| shape.even()));
    }

    #[test]
    fn one_shape_with_its_sides_apart_is_enough_to_move_both() {
        // At rest only: a press springs both sides to the pressed corners, alike.
        let (mut app, recording) = start(Config::of(APART_AT_REST));
        assert_sides(
            &recording,
            &replay(&recording),
            SIZE,
            [20.0, 8.0],
            "at rest",
        );
        let sent = press(&mut app, &recording, "Down");
        springs_to(&sent, &[8.0, 8.0], "apart at rest, pressed");
        // Pressed only.
        let (mut app, recording) = start(Config::of(APART_PRESSED));
        let sent = press(&mut app, &recording, "Down");
        let down = springs_to(&sent, &[20.0, 4.0], "apart pressed");
        assert_sides(
            &recording,
            &settled(&recording, &down),
            SIZE,
            [20.0, 4.0],
            "pressed",
        );
        // Checked only.
        let (mut app, recording) = start(Config::of(APART_CHECKED));
        let sent = press(&mut app, &recording, "Check");
        let checked = springs_to(&sent, &[20.0, 16.0], "apart checked");
        assert_sides(
            &recording,
            &settled(&recording, &checked),
            SIZE,
            [20.0, 16.0],
            "checked",
        );
    }
}
