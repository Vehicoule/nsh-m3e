# What Vehicoule needs from this library

Taken from the Vehicoule rewrite's decision records of 2026-11 (`decisions/design.md`, `decisions/produit.md`,
`decisions/perf.md` in the project's documentation), which are the product requirement. These are requirements on the
components; where a number is given, it is a budget the components must allow the app to meet. Each threshold names
the standard or decision it comes from, so a test can cite it.

## Look

- Material 3 **Expressive**, not classic M3: Expressive springs, shapes, sizes and components (button groups, FAB menu,
  loading indicator, flexible app bars, floating toolbars).
- Expressive shapes (cookie, clover, ...) are for artwork, loading and empty states only.
- Always pastel: chroma capped, contrast guaranteed by the scheme's tones, never by the hue. One source of color at a
  time: the app's scheme, which follows what is playing ("content" scheme from artwork, "tonal spot" for presets and
  system color). A pure black option for dark mode.
- Contrast at least 4.5:1 for text, 7:1 in high contrast: [WCAG 2.2 SC 1.4.3 Contrast (Minimum)](https://www.w3.org/TR/WCAG22/#contrast-minimum)
  and [SC 1.4.6 Contrast (Enhanced)](https://www.w3.org/TR/WCAG22/#contrast-enhanced); required by `decisions/produit.md`
  and `decisions/design.md`.
- The same components and behavior on desktop and mobile. Hover is an extra layer on desktop, never another design.
  Layouts differ between mobile and desktop; components do not.

## Motion

- Every motion uses M3E springs from the tokens ([ADR 0004](adr/0004-spring-damping-conversion.md)), through transform
  and opacity, with no layout per frame. The one exception is a shape that changes, such as a button pressed: a drawn
  container may move a drawing argument (its corner radius) with the spring, measured as
  [ADR 0010](adr/0010-measure-animations-in-ci.md) says ([ADR 0011](adr/0011-morph-shapes-with-a-drawn-container.md)).
- No dropped frame at the display's refresh rate (120 Hz on the reference devices); never a fixed 60 Hz
  (`decisions/perf.md`).
- Nothing animates without a visible consumer; nothing is drawn when nothing changes or the window is hidden.
- "Reduce motion" (system or setting) replaces motion with short fades.
- No amateur defects: an icon that jumps at the end of a transition is a bug even if barely visible. Transitions survive
  interruption, spamming and going back.

## Accessibility and text

- Every control has a label; every gesture has a menu equivalent (`decisions/produit.md`).
- Text at 200 % never overflows or clips: [WCAG 2.2 SC 1.4.4 Resize Text](https://www.w3.org/TR/WCAG22/#resize-text),
  required by `decisions/produit.md`. Layouts use start/end, and right-to-left text is isolated correctly.
- Touch targets at least 48 × 48 dp: Material 3's accessibility guidance (https://m3.material.io/foundations/designing/structure,
  "touch targets"), stricter than [WCAG 2.2 SC 2.5.8 Target Size (Minimum)](https://www.w3.org/TR/WCAG22/#target-size-minimum)
  (24 × 24 CSS px); kept from the first app's UI spec.

## First components the app needs (music first)

Navigation bar and rail, top app bars, buttons and icon buttons (including toggle and button groups), FAB and FAB menu,
sliders (seek and volume), lists, cards, chips, menus, bottom sheets, dialogs, snackbars, search, tabs, progress and
loading indicators, switches and checkboxes. The order is decided per pull request with the user.
