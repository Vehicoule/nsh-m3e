# What Vehicoule needs from this library

Taken from the app's design decisions (2026-11). These are requirements on the components; where a number is given,
it is a budget the components must allow the app to meet.

## Look

- Material 3 **Expressive**, not classic M3: Expressive springs, shapes, sizes and components (button groups, FAB menu,
  loading indicator, flexible app bars, floating toolbars).
- Expressive shapes (cookie, clover, ...) are for artwork, loading and empty states only.
- Always pastel: chroma capped, contrast guaranteed by the scheme's tones, never by the hue. One source of color at a
  time: the app's scheme, which follows what is playing ("content" scheme from artwork, "tonal spot" for presets and
  system color). A pure black option for dark mode.
- Contrast at least 4.5:1 for text (WCAG AA), 7:1 in high contrast.
- The same components and behavior on desktop and mobile. Hover is an extra layer on desktop, never another design.
  Layouts differ between mobile and desktop; components do not.

## Motion

- Every motion uses M3E springs from the tokens ([ADR 0004](adr/0004-spring-damping-conversion.md)), through transform
  and opacity, with no layout per frame.
- No dropped frame at the display's refresh rate (120 Hz on the reference devices); never a fixed 60 Hz.
- Nothing animates without a visible consumer; nothing is drawn when nothing changes or the window is hidden.
- "Reduce motion" (system or setting) replaces motion with short fades.
- No amateur defects: an icon that jumps at the end of a transition is a bug even if barely visible. Transitions survive
  interruption, spamming and going back.

## Accessibility and text

- Every control has a label; every gesture has a menu equivalent.
- Text at 200 % never overflows or clips; layouts use start/end, and right-to-left text is isolated correctly.
- Touch targets at least 48 dp.

## First components the app needs (music first)

Navigation bar and rail, top app bars, buttons and icon buttons (including toggle and button groups), FAB and FAB menu,
sliders (seek and volume), lists, cards, chips, menus, bottom sheets, dialogs, snackbars, search, tabs, progress and
loading indicators, switches and checkboxes. The order is decided per pull request with the user.
