# 0021. Draw containers with four corner radii of their own

- Status: Proposed
- Date: 2026-11-07
- Deciders: the user

## Context

The drawn container of [ADR 0011](0011-morph-shapes-with-a-drawn-container.md) has one corner radius for a side:
its two start corners are the same, and so are its two end corners, and an animated value moves each side's. Most
shapes are the same at all four corners, so one value moves them all; the connected button group's items, whose start
and end corners differ, are the only shapes that use both. The button, the icon button, the toggle buttons and the list
item use it.

The M3E list also has items whose four corners differ, the segmented ones. This ADR decides whether the drawn container
learns to draw them. It decides nothing else.

- **What androidx does** (Compose Material 3, at the pinned commit `a8b393d51f9b`):
  - `ListItemDefaults.segmentedShapes(index, count)` (`ListItemDefaults.kt`, line 449) gives the first item of a list
    the list container's shape on its top corners and its own shape on its bottom corners, the last item the reverse,
    and a single item the container's shape on all four. The middle items keep `shapes().shape`.
  - The list container's shape is `ListTokens.ContainerShape`, `CornerLarge` (16 points); the item's shape at rest is
    `ListTokens.ItemContainerExpressiveShape`, `CornerExtraSmall` (4 points). The first item of a segmented list is
    therefore 16, 16, 4, 4 at rest. Between the items there is `ListTokens.SegmentedGap`, 2 points.
  - `shapeForInteraction` (`ListItemDefaults.kt`, line 1136) morphs those shapes by state. For corner-based shapes it
    calls `rememberAnimatedShape` (`AnimatedShape.kt`, internal to androidx, not vendored here), which keeps one
    progress value and interpolates each corner linearly from the shape it started from to the shape it goes to
    (`Interpolatable.lerp`). Each corner thus has a radius of its own along the way.
  - A pressed or focused first item goes from 16, 16, 4, 4 to 16, 16, 16, 16 (`ItemPressedContainerExpressiveShape`,
    `ItemFocusedContainerExpressiveShape`: `CornerLarge`): two corners move and two do not.
- **What this library does:**
  - `Corners::of` (`crates/nsh-m3e/src/morph.rs`) answers `None` for a shape whose corners differ. `corners` then falls
    back to fully rounded corners, which is a pill: a token with uneven corners would give a wrong container, not an
    error. Every container token of the button, the icon button and the non-segmented list item has equal corners, and
    the components' `every_*_shape_can_morph` tests check it.
  - `DrawnShape` holds an `AnimatedValue` for the start corners' radius and one for the end corners', which only a
    shape with its sides apart moves (`SideCorners`): a corner at the top is never different from the one below it. A
    press or a state change starts one spring on each radius that moves, from the value and the speed it has. The
    tests of ADR 0010 (`spring_in`, `spring_with` in `morph_probe.rs`) read exactly one started spring per change of a
    shape whose corners are alike, and replay its curve against the closed form.
  - A corner radius larger than half the shorter side is drawn as that half. For four radii that rule is not enough:
    two radii on one edge can add up to more than the edge. How androidx's `CornerBasedShape` reduces them is in its
    foundation and UI libraries, which are not vendored here: it would have to be read at the pinned commit and written
    down before anything is drawn.
- **Not ported in the list item's pull request:** `SegmentedListItem` and `segmentedShapes`. The pull request adds a
  list item whose shapes have equal corners, and states that.

## Options

1. **Four animated radii.** `Corners` holds a radius per corner, and the drawn container's path takes one radius per
   arc, each an animated value or a node derived from one.
   - Either each corner is moved by its own spring from its own value and speed, or one progress value is moved and each
     corner is interpolated from where it started to where it goes, as androidx does. The pull request that builds it
     picks one and measures it with ADR 0010's checks.
   - It gives the segmented list the look and the motion androidx has, for its first, last and single items too.
   - Its costs:
     - `morph.rs` changes, and the button and icon button with it. With four springs, a press that moves four equal
       corners starts four springs where their tests read one (`spring_in`); either `DrawnShape` keeps one value while
       the four corners are equal, or those tests change, and each changed assertion is listed in the pull request.
     - The path needs the rule for radii that do not fit an edge, taken from androidx's `CornerBasedShape` at the
       pinned commit, written as a pure function and tested, with a property test on its result.
     - The platform draws four arcs with moving radii instead of one value's four; the cost per frame is not measured
       and stays a device measurement, as ADR 0011 says for one.
2. **Morph only equal-corner shapes, and draw the others as they are.** `Corners` gains a variant for four fixed radii:
   a shape whose corners differ is drawn with them and does not move; when the component changes state, the container
   goes to the new shape at once.
   - It is a small change: one variant and its path, no new animated value, and the button and icon button are not
     touched.
   - Its costs:
     - The first, last and single items of a segmented list jump when they are hovered, focused, pressed or selected,
       and the middle ones spring. `docs/app-requirements.md` (Motion) calls a jump a defect; this would be a listed
       exception to it, in the pull request that adds it.
     - The jump cannot be measured by ADR 0010, which reads springs.
3. **Nothing for segmented items.** `DrawnShape` stays as it is. The library has no `SegmentedListItem`: an app that
   wants the gap between items builds its list of list items, with `SEGMENTED_GAP` between them, and every item has
   equal corners in every state.
   - It adds no code and no exception.
   - The segmented list's rounded outer corners are missing, which is its signature, until one of the other options is
     chosen. The pinned `SegmentedGap` token stays unused.

## Decision

Proposed: **option 1**, in a pull request of its own that comes before `SegmentedListItem`, and **option 3** until then,
which is what the list item's pull request does. Option 2 gives the look and breaks the motion rule for the items that
are most seen; it is listed to be rejected, unless the user wants the look sooner than the morph.

The user chooses. Until the user does, no code in this repository draws uneven corners.

## Consequences

- **If option 1 is chosen:**
  - `Corners` and `DrawnShape` are generalized in one pull request that does only that, with ADR 0010's checks for the
    new shapes: the exact parameters of each spring, the curve against the closed form within 0.1 % of its travel, no
    batch while it plays, quiet at rest, and interruption from the current value and speed.
  - The rule for radii that do not fit an edge has its own tests, and a property test: the radii it returns fit every
    edge, and radii that already fit are returned unchanged.
  - The button's and icon button's tests that read one spring are kept unchanged if `DrawnShape` keeps one value while
    the corners are equal; if not, the pull request lists each changed assertion and why.
  - `SegmentedListItem` follows in its own pull request, with `segmentedShapes` ported from `ListItemDefaults.kt`.
- **If option 2 is chosen:** `docs/app-requirements.md` (Motion) gets the exception in the same pull request, and
  `SegmentedListItem` ships with items that jump at their ends.
- **If option 3 is chosen:** nothing changes in code. `docs/app-requirements.md` is not touched, and the decision is
  taken again when the app needs a segmented list.
