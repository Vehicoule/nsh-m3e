# 0022. Cast the shadow of an elevated container whose corners move

- Status: Proposed
- Date: 2026-11-08
- Deciders: the user

## Context

The FAB menu's button, androidx's `ToggleFloatingActionButton` (`FloatingActionButtonMenu.kt` lines 457 to 588), is the
first component of this library whose container both **casts a shadow** and **changes its corners while it moves**.
[ADR 0011](0011-morph-shapes-with-a-drawn-container.md) drew the morphing containers on a canvas, and
[ADR 0018](0018-draw-elevation-as-two-box-shadows-per-level.md) put a level's two shadows on "the view whose corner
radius is its container's". It said what happens when the two meet: a component whose container is a canvas "casts the
shadow of its view's radius, not of the drawn shape. The elevated button needs its own answer then." This is that
answer, for the toggle FAB, and the elevated button, the elevated card and the FAB menu's items can follow it.

- **What androidx does.** The container's size, corner radius and color are functions of a checked progress, an
  `animateFloatAsState` with `MotionSchemeKeyTokens.FastSpatial` (lines 469 to 474): `lerp(16, 28, progress)` for the
  radius of the default size (20 to 28 for the medium one, 28 to 28 for the large one), `lerp(80, 56, progress)` for the
  medium size (`lerp(96, 56, progress)` for the large one, 56 to 56 for the default one), and `lerp(primaryContainer,
  primary, progress)` for the color (lines 593 to 613, 793 to 814). The spring overshoots, so the radius and the size go
  past their end for a moment. The shadow is one of level 3 (`FabShadowElevation`, line 815, the container's
  `ContainerElevation`), drawn by the platform for the shape the radius draws: `graphicsLayer { shadowElevation = …;
  shape = shape; clip = true }` with a `GenericShape` that reads the radius at each frame (lines 540 to 553). The
  elevation does not change with the interaction.
- **What NSH can animate** (at the pinned revision `c85e3f7dc878`; ADR 0011 read an earlier one and its facts hold):
  - A canvas draws a path whose numbers are animated values, which a spring moves on the platform with no batch from
    the engine. This is what [`DrawnShape`](../../crates/nsh-m3e/src/morph.rs) does for the buttons' corners, and what
    ADR 0010's replay measures.
  - A view's style `transition` can animate its `border-radius` and its `box-shadow` natively (`TransitionProps`,
    `crates/nsh-core/src/transition.rs`, `lerp_points` of `border_radius`), on the desktop backends' shared stepper and,
    on iOS, with Core Animation. Its timing is a duration and a cubic Bézier, never a spring, and ADR 0010's replay
    cannot measure it.
  - A canvas has `shadowColor`, `shadowBlur` and `shadowOffset`, like an HTML canvas (`CanvasOp::ShadowColor`,
    `ShadowBlur`, `ShadowOffset`, `Painter::shadow`), and no spread. Level 3's ambient layer is `0 4px 8px 3px` (ADR 0018),
    so a canvas would need the path grown by 3 points, drawn off the canvas and its shadow offset back, to give that
    layer. No host has been asked to draw such a shadow here.
  - A `boxShadow` is drawn outside the view's border box only, so a view with no background that carries the layers
    casts exactly the shadow of its radius and shows nothing else.
- **The requirement.** "Every motion uses M3E springs" ([`docs/app-requirements.md`](../app-requirements.md), Motion).

## Options

1. **The canvas draws the container and its shadow.**
   - The container's path is drawn twice more, as the shadow's layers, with the canvas's shadow operations and the path
     grown by the ambient layer's spread, so the shadow follows the spring of the corners exactly.
   - It keeps every motion a spring, and nothing outside the canvas moves.
   - Its cost: ADR 0018's formula is written for `boxShadow`, so the canvas version is a second implementation of it that
     has to agree on every host. Nothing here shows that the four hosts (iOS, Android, Linux, the web) draw a canvas
     shadow alike, or draw it at all for a path that lies off the canvas: it would be known from CI's screenshots, after
     the component is written. If a host drew it wrong, the component would be reworked, with another decision.
2. **A plain container whose corners change at once.**
   - The container is a view with a corner radius and ADR 0018's `boxShadow`; the radius takes the checked state's value
     when the state changes, and the size, the colors and the icon move with their springs.
   - It is the simplest, and every host draws it as it draws the FAB's.
   - Its cost: the corners, which androidx moves with the spring, jump. For the default size, 16 to 28 points on a 56-point
     square. The requirement that every motion is a spring is not met for this one.
3. **The canvas draws the container; a view behind it casts the shadow and moves its radius with a style transition.**
   - The drawn container is ADR 0011's, with the spring, exact. Behind it, a view of the same frame with no background
     carries the level's `boxShadow` and a `transition` on its radius, whose duration is the time the spring takes to
     settle and whose easing is a cubic Bézier that overshoots as the spring does.
   - The container, which is what is seen, moves with the spring; the shadow is soft (opacity 0.3 and 0.15, blurred 3 to 8
     points) and follows the same radius at the same pace, only not by the same curve.
   - Its cost: the shadow's radius is not a spring and cannot be measured by ADR 0010's replay, and a shadow that follows a
     Bézier is not androidx's, which follows the spring. Whether each host animates `border-radius` as asked is checked
     from the screenshots, not assumed. The transition's end state is checked by tests, and its curve is not.

## Decision

**Proposed: option 3**, because it is the only one that moves the visible container with a spring today and uses ADR
0018's `boxShadow` as it is, so nothing is built on a canvas feature that no host has been asked for. Option 1 is where
the library should arrive if the hosts draw canvas shadows alike: the component's API would not change, only how the
shadow is drawn, and a later ADR would supersede this one. Option 2 is the choice if the user prefers that this
component takes no step outside the springs and the formula, and accepts that its corners jump.

## Consequences

- **If option 3 is chosen:**
  - The toggle FAB (and, once they need it, the elevated button and card) has a drawn container and a shadow view behind it
    with a `transition` on its radius. A shape change sends one style change to that view and one animated value to the
    canvas, and the transition's duration is `SpringToken::FastSpatial.spec().layout_transition().duration_ms`, the time
    the spring takes to settle.
  - The visual checks read the shadow's pixels at the end of the motion at both states, at rest and checked; no check reads
    a pixel during the motion.
  - The module documentation of each component that does this names the one thing that is not a spring.
- **If option 1 is chosen:** the shadow's formula is written a second time for canvases and its numbers are tested against
  `elevation.rs`; the visual checks are the first measure of whether the hosts agree, and the component waits for it.
- **If option 2 is chosen:** the toggle FAB is a plain container; its documentation says its corners do not move.
