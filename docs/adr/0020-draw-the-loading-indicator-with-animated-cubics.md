# 0020. Draw the loading indicator's morphing shapes as one canvas of cubics whose points are animated values

- Status: Proposed
- Date: 2026-11-07
- Deciders: the user

## Context

The loading indicator is one of the components the app needs ([`docs/app-requirements.md`](../app-requirements.md):
"progress and loading indicators", and "Expressive shapes (cookie, clover, ...) are for artwork, loading and empty
states only"). It is the one M3E component whose motion is a shape morph: a filled shape turns into the next shape of a
sequence while it rotates. How the morph is drawn decides the component's code, its tests, the sources it vendors and
their licenses, and NSH's canvas draws a Bézier curve from constant points only (`Painter::bezier_curve_to`). It is
settled once, before the component.

### What androidx does

At the pinned commit `a8b393d51f9b`. `LoadingIndicator.kt` is in
`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/`; line numbers are its own unless said
otherwise.

- **Entry points.** `LoadingIndicator` and `ContainedLoadingIndicator`, each with an indeterminate overload and a
  determinate one that takes a `progress` (l.99, 137, 185, 228). Size, shape and colors come from the tokens
  (`LoadingIndicatorTokens.kt` l.23-44, v0_7_0): a 48 × 48 container of shape `CornerFull`, an active size of 38 dp,
  `Primary` for the shape, and for the contained one `PrimaryContainer` behind `OnPrimaryContainer`. That file is
  vendored in `reference/androidx-material3-tokens/`; the `SOURCES` of `xtask` do not list it yet, so no module is
  generated from it.
- **Indeterminate** (l.372-504).
  - Seven shapes, `SoftBurst`, `Cookie9Sided`, `Pentagon`, `Pill`, `Sunny`, `Cookie4Sided` and `Oval` (l.548-557), in a
    circular sequence: seven morphs, the last one back to the first (l.382-388, 613-627).
  - A morph's progress runs from 0 to 1 with
    `spring(dampingRatio = 0.6f, stiffness = 200f, visibilityThreshold = 0.1f)` (l.407-408), and a new morph starts
    every 650 ms (`MorphIntervalMillis`, l.426, 707). The spring and the period are literals of the component, not
    tokens.
  - The shape turns by `progress * 90 + morphRotationTargetAngle + globalRotation` degrees (l.480). The target angle
    starts at 90 and grows by 90 at each morph, modulo 360 (l.399, 422-423, 711). The global rotation is linear, 0 to
    360 in 4,666 ms, repeated (l.434-440, 706).
  - **It draws the progress as the spring gives it, overshoot included.** The comment above the call says it uses "the
    coerced progress value" (l.486-488), but the code passes `progress`, which is `morphProgress.value` (l.478, 489).
    `Morph.asCubics` interpolates the control points without clamping (graphics-shapes `Morph.kt` l.119-150), so the
    shape goes past its end shape. Nothing scales the drawing "to simulate some bounciness" (l.480-500).
  - **It cuts the spring.** `SpringSpec.vectorize` builds a `VectorizedSpringSpec` (androidx `AnimationSpec.kt`
    l.191-194, `VectorizedAnimationSpec.kt` l.946) whose duration is `FloatSpringSpec.getDurationNanos`: the time at
    which the spring's envelope falls to the visibility threshold (`FloatAnimationSpec.kt` l.189-212,
    `SpringEstimation.kt` l.172-185). `TargetBasedAnimation.getValueFromNanos` returns the target exactly once that
    time has passed (`Animation.kt` l.265-285). The progress therefore goes from about 1.09 to 1 in one frame. Then
    `snapTo(0)` shows the next morph at its start, which is the same outline at the same angle (l.419-424), and only
    the global rotation moves until the next morph begins at 650 ms. These files are at the same commit, in
    `compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/`, and are not vendored.
  - **It has no reduce-motion handling:** the file has no fade and no mention of motion settings (searched for
    `reduce`, `fade`, `alpha`, `accessib` and `motion`: not found).
  - Each frame the path is scaled by `size × calculateScaleFactor × ActiveIndicatorScale` (l.396, 579-580, 641-657,
    696) and **centered on its exact bounds**: `path.translate(size.center - path.getBounds().center)` (l.702), where
    `getBounds` is `computeBounds(exact = true)` on Android (`ui-graphics` `AndroidPath.android.kt` l.214-219).
- **Determinate** (l.261-357). Two shapes, a circle of ten vertices turned by 18° and `SoftBurst` (l.567-573), so one
  morph, whose progress is the app's `progress` coerced to 0..1 (l.272, 317), and a turn of `-progress * 180` degrees
  (l.334). It has no spring and no animation of its own.
- **Where the shapes come from.** `MaterialShapes.kt` (`Oval` l.391, `Pill` l.396, `Pentagon` l.434, `Sunny` l.459,
  `Cookie4Sided` l.477, `Cookie9Sided` l.508, `SoftBurst` l.570, helpers l.721-786): each is one to three vertices
  with a corner rounding, a repeat count and sometimes a rotation, built with graphics-shapes
  (`androidx.graphics.shapes`, another androidx module, in
  `graphics/graphics-shapes/src/commonMain/kotlin/androidx/graphics/shapes/`). `RoundedPolygon` rounds the corners
  into cubics. `Morph` measures two polygons and maps their features onto each other, which gives pairs of cubics, one
  of the start shape and one of the end shape; `asCubics(progress)` interpolates the eight points of each pair
  linearly. The Kotlin the morph needs is 3,263 lines in eleven files (`Cubic`, `RoundedPolygon`, `Features`,
  `FeatureMapping`, `FloatMapping`, `PolygonMeasure`, `Morph`, `Shapes`, `Utils`, `Point`, `CornerRounding`), and
  `internal/ShapeUtil.kt` (178 lines) makes a path from it.

### What NSH can draw and animate

At the pinned revision `2208136f4cdd`.

- **`Painter`'s curve calls take constants.** `bezier_curve_to(c1, c2, to)` and `quadratic_curve_to(control, to)` take
  `Point<Points>` (`crates/nsh-core/src/element/host.rs` l.1917-1936), while `move_to` and `line_to` take numbers that
  can be animated values (l.1905-1914). This is the constraint the morph meets.
- **The operation behind them does not.**
  - `PathOp::CubicTo([Num; 6])` holds six `Num`s, each a `Const(f32)` or a `Slot(u16)`, "the current value of an
    animated value" (`crates/nsh-core/src/canvas.rs` l.31-36, 76).
  - A drawing can build any operation by hand: `Painter::num(value)` gives the `Num` of an animated value and
    `Painter::op(op)` records an operation, "for an operation built by hand" (`host.rs` l.1778, 1810).
  - NSH's own plots do it for cubics: `crates/nsh-base/src/plot/radial_line.rs` l.373-395 moves a path from one set of
    numbers to another with `CubicTo`.
  - The core reads the dependencies of all six numbers (`canvas/damage.rs` l.718), and every backend draws `CubicTo`
    from its numbers: `nsh-platform-linux` `canvas.rs` l.974, `nsh-platform-macos` l.882, `nsh-platform-ios` l.471,
    `nsh-platform-windows` l.732, `gpui/nsh-platform-gpui` l.1343, `nsh-platform-tui` `canvas/path.rs` l.240,
    `nsh-platform-android` `encode.rs` l.1734 and `nsh-platform-web` `canvas.rs` l.75.
  - So animated cubics need no change in NSH.
- **The graph computes the numbers natively** ([ADR 0010](0010-measure-animations-in-ci.md)).
  - An animated value can be a looped linear timing (`Animation::timing(..).looped(None)`; `Timing::new` asks for
    `frame_rate: None`, `crates/nsh-core/src/element/animated.rs` l.653-662, 1200), and nodes derive from it:
    `interpolate`, `interpolate_with`, `modulo`, `add` and `mul` (l.348, 408, 480).
  - An interpolation clamps by default; `Extrapolate::Extend` continues the end segment, which the spring's overshoot
    needs.
  - Where two breakpoints share an input, the value at exactly that input is the left segment's
    (`crates/nsh-core/src/animation/graph.rs` l.1180, `partition_point(|&v| v < x)`), so a table can hold a step.
- **NSH's spring rests at absolute thresholds**, 0.001 in value and in velocity
  (`crates/nsh-core/src/animation/driver.rs` l.6-8). It cannot be cut at a visibility threshold as androidx's is: this
  spring over a travel of 1 stops animating at 833 ms (trial below).
- **NSH has no recorded release** (NSH's `AGENTS.md`, "Backward compatible"): nothing is frozen, `CanvasOp` included.

### A trial at the pinned revision

Throwaway tests, never committed and since deleted, ran the options below under `nsh_testing::Headless` with a manual
clock, recorded the mount items, and replayed the animation operations in NSH's own `Graph` at 120 Hz, as ADR 0010 does.

- **The port.** The morph's Kotlin (graphics-shapes' eleven files, the parts the morph reaches), the seven shapes and
  the circle of ten vertices, ported to Rust in `f32`: 880 lines, 812 of them code, written under `#![allow(...)]` and
  therefore not what the library would ship. It was checked against itself, not against androidx's output (there is no
  Kotlin runtime here):
  - A circle of ten vertices measures 6.2717 against 2π, an error of 0.18 %. androidx's own test accepts 1.5 %
    (`PolygonMeasureTest.kt` l.54-64).
  - The matched cubics follow the polygons: at progress 0 and at 1, every sampled point of the cubics is within 4.6e-5
    of a unit square of the polygon's outline (0.0015 pt at the path's scale), and both chains are contiguous (gaps
    under 1e-3).
  - Cubics per polygon, in the sequence's order: 41, 19, 11, 13, 33, 17 and 9. Pairs per morph: 58, 28, 22, 44, 48, 24
    and 48 (272 in all, at most 58); the determinate morph has 50.
  - `calculateScaleFactor` gives 0.86558414 for the indeterminate sequence (× 38/48 = 0.6852541: a path of 32.89 pt in
    the 48 pt container) and 1 for the determinate one (38 pt).
  - Building the seven polygons and seven morphs, outside NSH, takes a median 0.18 ms in release and 0.75 ms in debug
    (21 runs each; release 0.16 to 0.24 ms, debug 0.72 to 0.85 ms).
- **One morph, the 58-pair one,** in a canvas whose 348 cubic numbers are interpolation nodes of a progress `p`, which
  springs from 0 to 1 with k = 200 and ζ = 0.6 (c = 16.970564 by ADR 0004's conversion).
  - Mount: one `Canvas` item (63 operations, 58 cubics, 348 slots) and 348 interpolation nodes. The press sent one item,
    `Start { driver: Spring { to: 1, stiffness: 200, damping: 16.970564, mass: 1, velocity: None, frame_rate: None } }`.
  - During `clock.runFor(1500)` the engine sent no item.
  - Replayed at 120 Hz, `p` matched the closed form within 0.000851 of the travel (ADR 0010's tolerance is 0.001), and
    the 348 numbers matched a direct interpolation of the matched cubics within 0.000008 pt. The first run missed by
    0.402 pt: the interpolation clamped and cut the overshoot (peak 1.0947 at the 120 Hz frames). With
    `Extrapolate::Extend` it is exact.
  - NSH's spring stopped animating at the frame of 833 ms. Over a travel of 100 (the same spring, with the thresholds
    absolute) it stops at 1,383 ms and the gap is 0.00076 % of the travel.
- **The whole loop in one drawing, with no spring, no timer and no recording again.**
  - Design: a canvas whose path has 58 cubics (each morph padded to 58 pairs by splitting both cubics of a pair at 0.5,
    which is exact because a Bézier's points are linear in its control points). Every number is an interpolation node
    over a morph coordinate `s`. `s` is a table of the spring (48 samples over its moving part, then a step at
    androidx's cut) over `c`, a looped linear clock of 28 × 650 ms (28 = 7 morphs × 4 target angles). The turn is a
    table over `c` plus `g`, a second looped linear clock (4,666 ms, 0 to 360). The centering is two tables over `s`,
    the exact bounds' center at 24 samples of `p` per morph.
  - Mount: a canvas of 66 operations with 351 slots (the 348 numbers and three transforms), 352 interpolation nodes, 3
    math nodes, 4 other animation operations and 7,324 breakpoints in all (5,220 for the coordinates, 351 for `s`, 1,401
    for the turn, 352 for the centering).
  - During `clock.runFor(6000)` the engine sent no item. Replayed at 120 Hz for 20 s (2,400 frames) against a reference
    built from the closed-form spring and the ported morph, the worst gap was 0.0055 pt in the shape, 0.0030 pt in the
    centering and 0.0665° in the turn. The graph was still animating at 20 s, as it loops.
  - One frame, ticking the graph and reading the 351 slots, took a median 50.8 µs (min 40.9, max 52.5; 7 runs of 2,400
    frames, debug profile, this machine: not a device).
  - The table must be fine. Over its moving part, 8 segments leave a gap of 2.45 % of the travel, 16 of 0.73 %, 32 of
    0.20 %, 48 of 0.091 % and 64 of 0.052 %. Only 48 and 64 meet ADR 0010's 0.1 %.
  - The padded morphs follow the original matched cubics within 0.00114 pt, which is the resolution of the check; the
    padding itself is exact. Centering tables of 24 samples per morph follow the exact bounds' center within 0.0071 pt.
  - Why center at all: the exact bounds' center is off the middle of the unit square by up to 0.406 pt (SoftBurst to
    Cookie9Sided, at `p` = 0.905), and by 0.300, 0.179, 0.112, 0.102, 0.005 and 0.000 pt at the worst for the other six.
- **One drawing per morph, recorded again at each cycle** (a signal moved by a timer, the spring NSH's). A record
  sends one canvas item, with 348, 165, 132, 264, 288, 144 and 288 slots for the seven morphs, and creates one
  interpolation node per slot. The one exception: the first morph's 348 nodes were not created again (they stay from
  the mount; why was not looked into). That is 1,281 nodes created over the loop of seven records, 183 per cycle on
  average. Over nine windows of 650 ms the engine sent 1,162 to 1,828 mount items per window, but the timer fired four
  times per window under the manual clock (the cause was not found), so only the size of a record is a result, not the
  rate. Positive control: those items are in the step `clock.runFor(650)`, and the step `run` held none (see
  Consequences).
- **Flattening** each cubic into K line segments with every point within 0.1 pt of the true curve, for `p` from 0 to
  1.1 (0.1 pt is the trial's tolerance, not a source's): K = 4, 4, 6, 3, 4, 8 and 4 for the seven morphs, so 464, 224,
  264, 264, 384, 384 and 384 numbers, against 348, 165, 132, 264, 288, 144 and 288 slots for the cubics. The
  determinate morph needs K = 4, 400 numbers, against 300.
- **androidx's cut,** from the closed form. Compose's estimate of the spring's end is 297.7 ms (it falls between the
  120 Hz frames of 291.7 and 300.0 ms), where `p` is 1.0914.
  - The step to 1 is 0.0914 of the travel, 1.56 times the largest step the spring itself makes between two 120 Hz
    frames (0.0587). It turns the shape back by 8.23° (the spring's own largest frame step of turning: 5.28°) and moves
    the outline by 0.33 to 0.68 pt at most over the seven morphs (the largest frame step: 0.21 to 0.44 pt; the largest
    travel of a coordinate over a morph: 3.59 to 7.48 pt).
  - If the spring runs to the cycle's end instead, `p` is 0.99542 at 650 ms: a step of 0.458 % of the travel to the
    next morph's start.

How this was run: in the development container, debug profile, with `cargo test`, once, except where a number of runs is
given; the build cost was measured with `rustc` outside the repository. Wall-clock time plays no part in the counts and
the curves, which are the same on every run. The timings are this machine's, not a device's.

The trial does not show:

- what the platform's redraw of one 58-cubic path costs per frame, or how its edges look: both are device
  measurements. The trial ran headless; no backend drew these cubics;
- that the port gives androidx's cubics: the pair counts, the scale factor and the centering are the port's own;
- the determinate drawing, which was counted, not mounted;
- whether NSH stops the two looped clocks when the component is removed or hidden, or pauses them while it is off
  screen.

## Options

### A. How the morph is drawn

1. **One canvas, drawn once, whose cubic points are animated values, and the whole loop on two native clocks** (the
   trial's whole loop).
   - The cubics are built by hand (`PathOp::CubicTo` with `Painter::num` and `Painter::op`), as NSH's `radial_line`
     does. After the mount the engine does nothing: no batch, no timer, no recording again. The platform redraws one
     path of 58 cubics per frame.
   - Its costs:
     - It leans on `Painter::op`, documented "for an operation built by hand". NSH has no recorded release, so nothing
       promises that `CanvasOp` stays as it is. One function builds the path, and an NSH bump that breaks it fails the
       tests first ([ADR 0014](0014-merge-nsh-bumps-on-green.md)).
     - The spring is a table, not an NSH spring: ADR 0010's checks 1 and 2 test the table (Consequences).
     - 7,324 breakpoints are sent once, at the mount.
     - The 22-pair morph is drawn with 58 cubics too, because the loop is one drawing.
2. **One canvas per morph, recorded again at each cycle,** a timer moving a signal, the spring NSH's.
   - ADR 0010's checks 1 and 4 apply unchanged, since the spring is a real driver. Check 2 holds for the curve and
     fails at its end (below).
   - Its costs:
     - Engine work every 650 ms, for ever: one canvas item and up to 288 new nodes per record (trial).
     - A timer whose behavior under the manual clock the trial did not explain.
     - NSH's spring cannot be cut. It rests at 833 ms, so the next morph starts at `p` = 0.99542, and androidx's own end
       is not reproduced.
     - Check 3 (no batch while the animation plays) is false by design.
   - A variant that records once and restarts a real spring from a timer was not tried.
3. **Flatten each cubic into line segments whose numbers are animated values** (`line_to`, with no `Painter::op`).
   - Its costs: 1 to 2.7 times option 1's numbers per morph (trial), and an outline within 0.1 pt of the curve instead
     of the curve. It only avoids `Painter::op`, which is public API. The loop still needs the padding.
4. **Crossfade seven static outlines.** Each polygon is constant `bezier_curve_to` calls (143 cubics in all), shown and
   hidden with opacity.
   - There is no morph: it is not M3E's loading indicator. It needs no matched cubics and no tables, but still the
     rounded corners of graphics-shapes' `RoundedPolygon`, ported or generated.
   - It is [ADR 0011](0011-morph-shapes-with-a-drawn-container.md)'s option 3 again: a different component, shipped
     until a morph is possible. A morph is possible now (option 1).
5. **Ask NSH for animated points in `bezier_curve_to`, and wait.** One call per cubic instead of a hand-built
   operation. Nothing in option 1 needs it, and it is a message outside the project, which needs the user's go.
   Option 1's builder is one function that such a change would replace.

### B. Where the matched cubics come from (with any of A)

1. **A port of graphics-shapes in the library, run at first use,** the seven shapes typed in Rust as literals, each
   cited, with a test that compares their numbers with the vendored `MaterialShapes.kt`.
   - Its costs:
     - The port is 880 lines in the trial and must be rewritten for the library's lints (no indexing that can fail, no
       `unwrap`, `clippy::pedantic` denied, `Cargo.toml`), then tested until `cargo mutants` has no survivor.
     - It runs at first use, 0.18 ms in release and 0.75 ms in debug.
   - It gives: an app could later supply its own polygons, as androidx's `polygons` parameter allows. That is a public
     API with its own ADR.
2. **The port in `xtask`, the matched cubics generated and checked in.** `cargo xtask` reads the seven shapes' numbers
   (typed in `xtask` under the same comparing test, or parsed from the vendored Kotlin), runs the port and writes
   constants: the matched pairs of both sequences (272 + 50 pairs of 16 floats, about 5,200 numbers), the scale factors
   and the centering samples. `--check` fails when they drift, as for the tokens and the icons
   ([ADR 0003](0003-tokens-from-androidx.md), [ADR 0017](0017-draw-icons-from-path-data.md)).
   - Its costs:
     - A numerics module in `xtask`, under the same lints and the same mutation testing (`xtask` is a workspace member).
     - A generated file of about 5,200 numbers (about 21 KB of `f32` data; its size as source was not measured).
     - No custom polygons without a new ADR.
   - It gives: the library ships data and a drawing, no geometry port, and no first-use cost.

Both need the port tested with the cases of androidx's own tests (`CubicTest`, `RoundedPolygonTest`,
`FeatureMappingTest`, `MorphTest`, `PolygonMeasureTest`), which the trial did not do.

## Decision

Proposed, not chosen: **A1 with B2**. The user decides; this ADR changes to Accepted with their choice.

- **A1** is the only option that morphs exactly while the engine stays idle. It keeps ADR 0010's check 3 and "nothing
  animates for nothing" true, and the animation runs at the screen's rate with no engine work per frame. Option 2 sends
  items for ever, option 3 is option 1 with approximations, option 4 drops the morph, and option 5 waits for a change
  the trial showed is not needed.
- **B2** follows the repository's habit for values that come from vendored sources (tokens and icons are generated, and
  CI checks them) and keeps a geometry port out of the shipped library. What it gives up, custom polygons, is not in
  `docs/app-requirements.md`, which names the M3E loading indicator and not custom shapes.

Inside it, the sources leave these choices to the user, each with a recommendation. Whichever is chosen sits behind one
function or constant of the component, so changing it later is local.

1. **How a morph's spring ends.**
   - (a) As androidx: cut at 297.7 ms, a step of 0.0914 of the travel, then still until 650 ms. It reproduces androidx's
     frames, but the step is what ADR 0010's check 2 ("the last step into rest is no larger than the tolerance") and
     `docs/app-requirements.md` ("an icon that jumps at the end of a transition is a bug even if barely visible") call a
     defect, so it needs the user to accept an exception.
   - (b) The spring runs to the cycle's end: a step of 0.458 % of the travel at the next morph, above ADR 0010's 0.1 %
     and well below what shows (0.00458 of the largest coordinate travel, 7.48 pt, is 0.034 pt). It differs from
     androidx's pixels by at most 0.68 pt and 8.23° between 297.7 and 650 ms: the size of androidx's own step, which
     (b) does not make. **Recommended: (b),** and the user accepts a tolerance of 0.5 % for this one step.
2. **The overshoot.** (a) Draw it as androidx's code does, and follow the code over its comment, as
   [ADR 0012](0012-follow-androidx-components-where-tokens-are-wrong.md) follows a component over its tokens.
   (b) Clamp the progress, as the comment says: no bounce. **Recommended: (a).**
3. **The centering.** (a) Per frame, on the exact bounds, as androidx does: two more tables, 352 breakpoints.
   (b) Fixed at the middle of the unit square: an offset of up to 0.406 pt. **Recommended: (a).**
4. **The sequences offered.** The two default sequences only (the indeterminate seven shapes, the determinate two), no
   `polygons` parameter, which B2 implies. **Recommended.**

## Consequences

- **This ADR changes no code.** The component and its generator come in their own pull requests, after the user's
  choice.
- **Reduce motion.** `docs/app-requirements.md` (Motion) replaces motion with short fades, and androidx's indicator has
  none (not found, above). A fade does not fit a loop that never ends, so the proposal, in force until the user decides
  otherwise, is a still pose: androidx's first frame, taken from the animation's own initial values, the first morph at
  progress 0, target angle 90 and global rotation 0 (`LoadingIndicator.kt` l.398-401), which is `SoftBurst` turned by
  90°. It is drawn by the same builder with constants, so the mount starts no animation and `Graph::is_animating()` is
  false. The busy semantics stay (`progressSemantics()`, l.465). One function builds the pose, so replacing it is
  local. The visual checks run with `reducedMotion: true` ([ADR 0013](0013-visual-checks-on-nsh-hosts.md)) and so get
  this still picture; they read only the token colors of the shape and the container.
- **ADR 0010's checks, adapted for option 1** (the user accepts them with this ADR):
  1. *Parameters.* There is no spring driver. The literals ζ = 0.6 and k = 200 (l.407-408) go through ADR 0004's
     conversion, and the table's samples equal the closed form of that spring exactly.
  2. *Curve.* The replayed values follow the closed form within 0.1 % of the travel at every 1/120 s frame. 48 segments
     give 0.091 %, with no margin; **64 give 0.052 %** and are the recommendation. The same applies to the turn and the
     centering, in points.
  3. *Engine work.* While the loop plays, the engine sends no item (trial: 0 over 6 s), checked in the step
     `clock.runFor(<ms>)`.
  4. *Frame rate.* Both looped clocks are `Timing::new`, which asks for `frame_rate: None`.
  5. *Nothing animates for nothing.* An indeterminate indicator never settles, so the check becomes: with reduce
     motion on nothing animates, and removing the indicator stops both clocks (the second part was not shown by the
     trial).
  6. *Reduce motion.* As above: no animation and no fade. ADR 0010 asks for a fade "with the duration its spec gives";
     the indicator's sources give none.
  7. *Interruption.* Nothing interrupts the loop. The determinate indicator follows its `progress` one to one, as
     androidx's does, with no spring to interrupt.
- **A defect found in the existing checks, not fixed here.** `Headless::run_for` renames the step to
  `clock.runFor(<ms>)` (`nsh-testing` `headless/mod.rs` l.407-412), so the assertions
  `recording.in_step("run").is_empty()` in `crates/nsh-m3e/src/animation_probe.rs` l.401,
  `crates/nsh-m3e/src/button.rs` l.1099 and `crates/nsh-m3e/src/icon_button.rs` l.1183 look at a step that stays empty
  whatever the engine sends: they prove nothing. The trial's positive control shows it (`run` empty, `clock.runFor(650)`
  holding 1,426 items). The property itself held where it was measured under the right step name (trial: 0 items).
  Fixing them is a change to existing assertions, which AGENTS.md rule 7 asks to be listed and justified: a pull request
  of its own.
- **Vendored sources and licenses.** Option B2 and the tests need, at the same androidx commit and under the same
  Apache-2.0, each listed in `reference/README.md` and `docs/references.md` and declared in `REUSE.toml` (ADR 0015):
  - `LoadingIndicator.kt`, `MaterialShapes.kt` and `internal/ShapeUtil.kt` in
    `reference/androidx-material3-components/`, copied by `scripts/sync-androidx.sh`;
  - graphics-shapes' eleven files and the five test files named above in a new `reference/androidx-graphics-shapes/`,
    copied by the same script;
  - `LoadingIndicatorTokens.kt`, already vendored, added to `xtask`'s `SOURCES`, so that `cargo xtask gen-tokens`
    generates its module.
- **Dependencies.** None added; the trial used the existing ones.
- **The determinate indicator** uses the same builder: one morph (50 pairs, 300 numbers) whose nodes take the app's
  progress, with two breakpoints each, no clock and no table. It was not mounted in the trial.
- **The contained variant's container** is `CornerFull` at rest: ADR 0011's drawn container with its rest shape, in the
  tokens' `PrimaryContainer` and `OnPrimaryContainer`.
- **What CI will prove, and what it will not.** Proved: the table follows the closed form, the matched cubics follow the
  polygons, no engine work while the loop plays, the still pose under reduce motion, and the generated data matches the
  generator. Not proved: androidx's own cubics (the port's tests are androidx's cases, not androidx's output for these
  shapes), the redraw cost of one 58-cubic path per frame at 120 Hz on a device, and how the edges look. The pull
  request that builds the component reports those as AGENTS.md rule 8 asks.
