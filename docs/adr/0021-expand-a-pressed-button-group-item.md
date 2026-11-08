# 0021. Choose how a button group widens the item a finger presses

- Status: Accepted
- Date: 2026-11-08
- Deciders: the user

## Context

Button groups are one of the components the app needs ([`docs/app-requirements.md`](../app-requirements.md): "buttons
and icon buttons (including toggle and button groups)"). Their motion is unusual: a press widens the item under the
finger and narrows its neighbors, which no other component of this library does. How that is drawn decides the
component's code and tests, and whether it keeps M3E's springs exactly. It is settled once, before the component.

### What androidx does

At the pinned commit `a8b393d51f9b`, `ButtonGroup.kt` (vendored in `reference/androidx-material3-components/` with
this proposal):

- **The standard group** (`ButtonGroup`, l.131-178) lays its items in a row, `ButtonGroupSmallTokens.BetweenSpace`
  apart (l.191-192). It measures them itself (`ButtonGroupMeasurePolicy`, l.556), with an overflow menu for the items
  that do not fit (l.159-171).
- **A press widens the item.** Each item holds an animated value from 0 to 1 (`EnlargeOnPressNode`, l.1005-1063): it
  goes to 1 when a press goes down and back to 0 when the press ends, with `MotionSchemeKeyTokens.FastSpatial` (l.140),
  and the press's end waits until the value has passed 0.75 (l.1046-1047). Every frame, the measure policy widens the
  item by that value times `ExpandedRatio`, 15 % of its width (l.188), and narrows its neighbors by the same width,
  each by no more than its own end padding (l.738-767). The row's width does not change; the items' frames do, every
  frame of the spring.
- **The connected group** is the same row, `ConnectedButtonGroupSmallTokens.BetweenSpace` apart (l.195), whose toggle
  buttons take connected shapes: full outer corners, inner corners of `InnerCornerCornerSize` (small, 8 dp) that a press
  sharpens to `PressedInnerCornerCornerSize` (extra small, 4 dp), and fully rounded when checked (l.198-279). It widens
  pressed items the same way.

### What NSH gives

At the pinned NSH revision `c85e3f7dc878`:

- A platform-run animation drives only these view properties (`nsh-core/src/animation/ops.rs`, `AnimationProp`):
  opacity, the background, border and text colors, the entries of `transform`, a canvas's animated slots, sticky
  offsets and a native component's slots. None is a width or a frame: a frame that moves every frame of a spring is a
  layout every frame.
- A layout change can be animated by the platform (`ElementRef::animate_layout`, `nsh-core/src/element/element_ref.rs`
  l.160-175, and `nsh_core::animate_layout`), with a `LayoutTransition` of a `LayoutCurve` and a duration in
  milliseconds (`nsh-core/src/props.rs` l.492-534). Its spring is `LayoutCurve::Spring { damping }`: a damping ratio
  and a duration, not a stiffness.
- [ADR 0010](0010-measure-animations-in-ci.md) measures every animation from the operations the engine sends when it
  starts, with nothing sent while it plays (the components' tests check it), and
  [ADR 0004](0004-spring-damping-conversion.md) gives every spring M3E's stiffness and its damping converted from M3E's
  ratio.

So androidx's own way, a layout per frame, is not open to this library, and NSH's layout animation cannot take
`FastSpatial` as M3E defines it (stiffness 800, damping ratio 0.6: `ExpressiveMotionTokens`, through `crate::motion`).

## Options

1. **Leave the widening out for now.** The button group is the row with its spaces, and the connected group the row
   with its shapes and their morph (which DrawnShape draws with the platform's springs, as the toggle button's); a press
   morphs the pressed item's corners but widens nothing. NSH is asked for a layout spring that takes a stiffness and a
   damping coefficient, and the widening comes in its own pull request when NSH has it. Cost: the groups miss part of
   M3E's motion until then; no new code path. Gives: the groups now, with every value exact.
2. **Animate the frames with NSH's layout animation.** A press changes the items' widths and asks
   `ElementRef::animate_layout` for `LayoutCurve::Spring { damping: 0.6 }` over a duration computed from
   `FastSpatial`'s stiffness and ratio (the time the spring takes to settle within a stated tolerance), by a tested
   helper beside ADR 0004's. Cost: a spring that is not M3E's (the platform's spring of that ratio and duration need not
   follow the same curve), a new rule to write next to ADR 0004, and the release rule (wait for 0.75) done by a timer.
   Gives: the widening now, with real frames (touch targets follow).
3. **Draw the widening.** Layout stays still: the pressed item's container canvas is drawn wider by an animated slot,
   its neighbors' containers narrower, and each item's content moves with a `transform` translation, all driven by
   platform springs with `FastSpatial`'s exact values. Cost: each item draws its container wider than its frame (the
   group leaves room for it), more code in the group and the drawn shape, touch targets that stay where the layout put
   them, and the release rule (wait for 0.75) done by a timer. Gives: M3E's exact motion, with nothing laid out while it
   plays.

## Decision

Accepted by the user on 2026-11-08: option 1. The button groups leave the widening out for now, and NSH is asked, in a
pull request to NSH, for a layout spring that takes a stiffness and a damping coefficient. It ships both groups now with
every value from androidx and no new rule. The widening comes in its own pull request once NSH has that spring. Option 3
is the fallback if NSH declines, since it keeps M3E's springs exact.

## Consequences

- With option 1, the button group's module docs list the widening as left out, citing this ADR, and its tests check
  that a press lays nothing out. The connected group needs `DrawnShape` to animate the start and end corners apart (its
  shapes round the two sides differently), which is its own change with its own tests.
- With option 2, the duration helper and its tolerance need their own tests and an addition to ADR 0004.
- With option 3, the drawn shape gains a drawn width, and the group tests check every item's drawn width and offset
  over the spring as the toggle button's tests check its corners.
- Whatever the option, the overflow menu waits for the menu component.
