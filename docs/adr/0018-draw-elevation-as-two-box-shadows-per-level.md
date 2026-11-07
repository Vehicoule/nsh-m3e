# 0018. Draw elevation as the two box shadows that Material Web gives each level

- Status: Accepted
- Date: 2026-11-07
- Deciders: the user

## Context

The floating action button (FAB) is the first component that must show its elevation: its container rests at level 3
and rises to level 4 under the mouse. The floating toolbar, the elevated card and button, menus and sheets will need
the same. [ADR 0011](0011-morph-shapes-with-a-drawn-container.md) left the question open ("the tokens give its levels
in points, not shadows. It is decided with that style"). No component draws a shadow today, so it has to be settled
once, before the first of them ships.

All facts below are at androidx `a8b393d51f9b` and NSH `2208136f4cdd`, unless said otherwise.

- **What the tokens give.** Levels in points, and no shadow: `ElevationTokens.Level0` to `Level5` are 0, 1, 3, 6, 8
  and 12 (`reference/androidx-material3-tokens/ElevationTokens.kt`). The FAB reads
  `FabPrimaryContainerTokens.ContainerElevation` = `Level3`, `HoveredContainerElevation` = `Level4`,
  `FocusedContainerElevation` and `PressedContainerElevation` = `Level3`. No token file holds a shadow color.
- **What androidx does with them.** `FloatingActionButton` passes the elevation to `Surface` as `shadowElevation`
  (`FloatingActionButton.kt` line 178), and `Surface` hands it to the platform as the layer's `shadowElevation`
  (`Surface.kt` lines 123, 251, 385, 521). The platform draws the shadow: Android's own, Skia's elsewhere. That code is
  not vendored, so this ADR does not describe how it draws. The FAB animates the elevation between its states
  (`FloatingActionButtonElevationAnimatable`, `FloatingActionButton.kt` line 1373) with `animateElevation`, which is
  in a file that is not vendored either (the FAB's pull request reads it and vendors it, as ADR 0012 says).
- **What NSH draws.** A view's `boxShadow` takes CSS's layers (`BoxShadow`: `offset_x`, `offset_y`, `blur`, `spread`,
  `color`, `inset`, `crates/nsh-core/src/effects.rs`), several at once, and follows the view's own corner radius.
  NSH's `docs/dev/styles.md` and `docs/dev/linux.md` say how three backends draw it: one `CALayer` per shadow with a
  `shadowPath` on iOS, `Decor` on Android, GSK nodes on Linux. The macOS, Windows and web hosts are not checked here.
  `android={{ elevation }}` is a prop that gives a view Android's material shadow, and "Android's `elevation` is not
  drawn" on Windows (`docs/dev/windows.md`). Shadows are among the properties a transition animates
  (`TransitionProps`).
- **A formula exists for the levels.** Material Web (`material-components/material-web`, `main` at
  `d872455c4be5`) draws each level as two shadows, one for the key light (opacity 0.3) and one for the ambient light
  (opacity 0.15), from the level's index 0 to 5, in `elevation/internal/_elevation.scss`. For level 3, from that
  file: `0 1px 3px 0` at 0.3 and `0 4px 8px 3px` at 0.15; for level 4: `0 2px 3px 0` and `0 6px 10px 4px`. Both
  shadows' color is `--md-sys-color-shadow`; no androidx token file has it, and `PaletteTokens.Black` is the black
  of the palette.
- **The requirements.** "The same components and behavior on desktop and mobile" and "Every motion uses M3E springs"
  ([`docs/app-requirements.md`](../app-requirements.md), Look and Motion).

## Options

1. **The two shadows of Material Web's formula, as `boxShadow` layers, on every host.**
   - `elevation.rs` turns an `ElevationTokens` level into its two `BoxShadow` layers, in `PaletteTokens.Black` at the
     formula's opacities. A component puts them on the view whose corner radius is its container's.
   - `_elevation.scss` is vendored in `reference/material-web/` at the pinned commit and checked by a
     `scripts/sync-material-web.sh` in CI, as the androidx and Material Symbols copies are. The formula is written in
     Rust with a test per level that pins its layers to the numbers worked out from the vendored file.
   - It gives one look on every host that draws a `boxShadow`, and the level is data a test can read. Its cost: the shadows are
     Material Web's, not androidx's own, and a component whose container is a canvas (the buttons that morph) casts the
     shadow of its view's radius, not of the drawn shape. The elevated button needs its own answer then.
2. **The platform's shadow where NSH offers one, the formula's elsewhere.** `android={{ elevation }}` on Android, with
   the level in points as androidx gives it, and option 1 on the other hosts.
   - It is what Android apps built with androidx show, on Android.
   - Its cost: two looks for one component, and Windows draws no Android elevation (the formula's layers there). The
     visual checks cannot read Android's shadow on NSH's CI hosts, which run no Android.
3. **No shadow.** The levels stay tokens, and the surface colors carry the hierarchy.
   - It adds nothing to maintain and nothing to check.
   - Its cost: a FAB with no depth over a list is not Material 3, and the hover feedback that raises the FAB from level 3
     to 4 disappears (the state layer remains).

## Decision

**Option 1**, chosen by the user on 2026-11-07. It is the only one where every host shows the same elevation from one
pinned source.

## Consequences

- **If option 1 is chosen:**
  - **Vendoring.** `reference/material-web/_elevation.scss`, `scripts/sync-material-web.sh` and its row in
    `reference/README.md` and `docs/references.md`; CI runs its `check` as it runs the others.
  - **The module.** `elevation.rs` has one function from a level to its layers; its tests pin levels 0 to 5, and a test
    ties each `ElevationTokens` value to its index.
  - **Components.** Each component that has an elevation reads its token, asks the module for the layers and sets them
    on the view whose radius is its container's.
  - **Motion.** The shadow follows the elevation as it changes between states. How it moves is settled when the FAB
    does it, from androidx's `animateElevation` and ADR 0004; the visual checks read its two ends.
  - **The visual checks.** A shadow tints the pixels around the container, so a check of "the surface outside the
    corner" reads beyond the shadow's reach. The ambient layer reaches furthest: at level 3, 11 points to the sides
    (spread 3, blur 8) and 15 below (plus its 4-point offset); at level 4, 14 and 20.
- **If option 2 is chosen:** the components also take the `android` prop, and the checks cover the formula's layers
  only.
- **If option 3 is chosen:** the components carry their level and no shadow; this ADR is superseded when NSH or the
  user wants one.
