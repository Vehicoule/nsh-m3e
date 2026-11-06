# 0011. Morph a component's shape on press with a drawn container whose corners follow the M3E spring

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

The M3E button changes shape when it is pressed, and so will other components (icon buttons, button groups, FABs).
The button is the first.

- **What androidx does** (Compose Material 3, `Button.kt`, at the pinned commit `a8b393d51f9b`):
  - The Expressive `Button(onClick, shapes: ButtonShapes, …)` morphs between `shapes.shape` at rest and
    `shapes.pressedShape` while pressed (`shapeByInteraction`). It animates the corner sizes with
    `MotionSchemeKeyTokens.DefaultEffects`, "intentional here to prevent any bounce in this component".
  - The shapes come from the tokens. For example, `ButtonSmallTokens.ContainerShapeRound` is `CornerFull` (a pill) and
    `PressedContainerShape` is `CornerSmall` (8 points). Medium presses to `CornerMedium` (12), large and extra large
    to `CornerLarge` (16).
  - `DefaultEffects` is k = 1600, ζ = 1, so c = 80 after the ADR 0004 conversion (`nsh_m3e::motion::SpringToken`).
- **What the motion must do** ([`docs/app-requirements.md`](../app-requirements.md), Motion): M3E springs from the
  tokens, no layout per frame, no jump at the end, continuity when interrupted. It also says "through transform and
  opacity". A corner radius is neither.

### What NSH can animate

All facts below are at the pinned revision `897fd9c2d968`.

- **Animated values cannot drive a view's corner radius.** The properties an animated value can drive are
  `AnimationProp`: opacity, background, border and text colors, the steps of a transform, a canvas argument, sticky
  offsets and a native component's slot (`crates/nsh-core/src/animation/ops.rs`).
- **Style transitions can animate it, but not with a spring.**
  - `Transition` has a `SHAPE` group ("Corner radii and border widths", `crates/nsh-core/src/effects.rs`).
  - Its timing is a duration in milliseconds and a cubic Bézier `easing`. There is no spring.
  - The desktop backends step it with their frame clock (`crates/nsh-core/src/transition.rs`), and iOS hands it to
    Core Animation. None of this goes through `nsh_core::animation::Graph`, so ADR 0010's replay cannot measure it.
- **A canvas can draw a shape whose numbers are animated values.**
  - Any number argument of a drawing call can be an animated value or a node derived from it. The platform draws the
    canvas again on each frame while the value moves, "without recording the drawing again or running any of your
    code" (`website/docs/canvas.md`, "Driving drawings with animated values").
  - `Painter::move_to`, `line_to` and `arc` take such arguments. `round_rect` takes constant radii only
    (`crates/nsh-core/src/element/host.rs`), so a rounded rectangle with a moving radius is written as four lines and
    four arcs.
  - `Painter::fill_background_from_view` fills the path with the canvas view's own background, which can be an
    `AdaptiveColor`. The `md-sys-color-*` tokens of ADR 0008 therefore still apply, with their light and dark values.

### A trial at the pinned revision

A test, not committed, drew a 120 × 40 point container in a canvas:

- The path was four lines and four arcs. Its radius `r` was an animated value starting at 20 (a pill), and the far
  edges were `r · (−1) + 120` and `r · (−1) + 40`.
- It was filled with the view's background, an `AdaptiveColor`.
- A press sprang `r` to 8 with the `DefaultEffects` spring, converted.

It ran under `nsh_testing::Headless` with a manual clock and was replayed with ADR 0010's helper.

- **Mount.** The engine sent one `Canvas` item. Its drawing read three slots: `r` and the two derived nodes.
- **Press.** The press sent one item: `Start { driver: Spring { to: 8, stiffness: 1600, damping: 80, mass: 1,
  frame_rate: None } }`.
- **While it played.** During the 1.5 s of `run_for` that followed, the engine sent no batch.
- **Curve.** Replayed at 120 Hz, `r` matched the closed-form spring within 0.000023 points over a travel of
  12 points. It ended on exactly 8.

How this was run: in the development container (`cargo test`, debug profile), once. Wall-clock time plays no part, so
every run gives the same numbers.

The trial does not show the drawing on a device: how much the platform's redraw of one path costs per frame, and how
the edges look. Those are device measurements.

## Options

1. **A drawn container.** The component's container is a canvas behind its content, filling it. The canvas draws the
   container's path, with the corner radius as an animated value that the M3E spring moves.
   - The background comes from the view (`fill_background_from_view`), and so does the outline
     (`stroke_style_from_view`).
   - It gives the M3E morph, with the token's spring, measured by ADR 0010's checks (the trial above). There is no
     layout and no engine work per frame.
   - Its costs:
     - The canvas needs the container's size: `on_layout` keeps it in a signal, and the drawing is recorded again
       when the size changes, not per frame.
     - The platform redraws one path per frame while the shape moves. This animates a drawing argument, not a
       transform or an opacity, so the Motion requirement needs this exception.
     - Each component that morphs uses the same shared drawn container, which this library maintains.
2. **A style transition of the shape.** The view's corner radius changes on press, with `transition` on the `SHAPE`
   group.
   - It is the simplest, and the container stays a plain view.
   - It needs a duration and a Bézier curve that M3E does not give for `DefaultEffects` (springs have no duration).
     These would be numbers made up, against AGENTS.md rules 3 and 6.
   - It is not a spring, and ADR 0010 cannot measure it.
3. **No morph until NSH can animate a corner radius.** The button ships with its rest shape only, like androidx's
   non-Expressive overload `Button(onClick, shape = ButtonDefaults.shape)`. Later, NSH would add a corner radius to
   `AnimationProp`, through an issue opened upstream.
   - It adds no code and no exception now.
   - The pressed shape, one of M3E's signatures, is missing until NSH has the property, with no date.
   - Opening the issue is a message outside the project, which needs the user's go.
4. **Switch the shape on press with no animation** (`active:` style). This is the "jump" defect the requirements
   forbid, listed only to reject it.

## Decision

**Option 1**, chosen by the user on 2026-11-06, with the Motion requirement amended for it: a drawn container may move
a drawing argument (here a corner radius) with an M3E spring, measured as ADR 0010 says.

The button's first pull request ships the component at rest: the sizes, the styles, the colors, the states and the
label. The morph follows in its own pull request.

## Consequences

- **What option 1 brings:**
  - **One shared drawn container** draws a rounded rectangle whose radius is an animated value, with the view's
    background and outline. Every component that morphs uses it.
  - **Tests.** Its tests are ADR 0010's: exact parameters, the curve within 0.1 % of the travel, no batch while it
    plays, `frame_rate: None`, quiet at rest, interruption from the current value and velocity.
  - **A rest shape that is a pill** (`Shape::Circle`) is half the container's height. The radius therefore depends on
    the size the canvas is given.
  - **Reduce motion.** With reduce motion on, the morph sends no spring, as ADR 0010's check 6 asks. What replaces it
    is decided with the reduce-motion fade, when it comes.
  - **Not proved by CI.** The cost of the redraw and how the edges look stay device measurements, reported as AGENTS.md
    rule 8 asks.
  - **Elevation.** The elevated button's shadow is a separate question in every option: the tokens give its levels in
    points, not shadows. It is decided with that style, and must follow the drawn shape if option 1 is chosen.
- **Requirements.** `docs/app-requirements.md` (Motion) allows this exception, in the same pull request as this
  decision.
