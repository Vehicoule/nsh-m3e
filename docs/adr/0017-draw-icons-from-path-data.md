# 0017. Draw icons from SVG path data on a canvas, filled with a token color; the library ships no icon set

- Status: Proposed
- Date: 2026-11-07
- Deciders: the user

## Context

The icon button is the next component ([`docs/app-requirements.md`](../app-requirements.md), "First components the app
needs"). After it come the navigation bar and rail, the top app bars, the FAB, chips and lists, which all show icons.
How an icon is given and drawn is public API that every one of them shares. It has to be settled once, before the
first of them ships.

- **What M3 asks of an icon.** An icon is one shape in one color: the component's content color in each state, from
  the tokens. For example, `SmallIconButtonTokens.IconSize` is the size, and `FilledIconButtonTokens.IconColor` /
  `DisabledIconColor` / `DisabledIconOpacity` are the colors (androidx `a8b393d51f9b`). The icons M3 itself uses are
  Material Symbols. Each one is a single filled path in a 960 × 960 view box (`viewBox="0 -960 960 960"`), published as
  SVG files by Google under Apache-2.0 (https://github.com/google/material-design-icons, commit
  `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`).
- **What NSH offers** (pinned revision `a78c6190fcf8`):
  - **`svg(src)`** (`crates/nsh-core/src/element/host.rs`) draws an SVG file "as authored, in its own colors".
    - Its `currentColor` follows the view's text color on the native backends, where it is a canvas.
    - Its doc says: "The web runtime shows it as an image the browser draws, where `currentColor` is black."
    - A source other than inline markup or a drawing in memory shows only once it has loaded.
  - **`canvas(painter)`** draws what a `Painter` records, on every backend.
    - `Painter::fill_svg(d)` fills SVG path data, parsed by `nsh_core::canvas::parse_svg_path`.
    - `Painter::scale` maps the view box onto the icon's size.
    - `Painter::fill_style_from_view` fills with the view's `fill`, which can be an `AdaptiveColor`. The
      `md-sys-color-*` tokens of [ADR 0008](0008-color-scheme-in-nsh-tokens.md) therefore apply, light and dark.
    - The button's container is already drawn this way ([ADR 0011](0011-morph-shapes-with-a-drawn-container.md)).
- **The web is a host the visual checks run on** ([ADR 0013](0013-visual-checks-on-nsh-hosts.md)). There, an icon
  drawn by `svg()` with `currentColor` would be black whatever the token says.

## Options

1. **An `Icon` made of SVG path data and its view box, drawn on a canvas and filled with the component's token
   color.**
   - It looks the same on every host, the web included, and shows in the first frame: the path is in the binary.
   - It needs no new dependency.
   - Its numbers can later be animated values, as the container's are, for the icon motion M3E asks for.
   - Cost: an icon is one path in one color. Multicolor artwork (a logo, an album cover) is an image, not an icon, and
     goes through NSH's `image` or `svg`.
2. **An `Icon` made of an NSH `ImageSrc`, drawn by `svg()`, colored through `currentColor`.**
   - Any SVG file works, multicolor ones too.
   - Cost: on the web, the icon is black in every state. The visual checks on the web would fail on every icon, or
     would have to leave icons out, which rule 2 forbids. Icons loaded from a file or a URL also pop in after the
     first frame.
3. **Option 2, after fixing `currentColor` on NSH's web runtime upstream first.**
   - Cost: the icon button waits on an upstream pull request whose design is NSH's to choose. Option 2's pop-in after
     the first frame stays.

And for where the icons come from:

- **a. The library ships no icon set.** The app gives each icon's path data, from Material Symbols or anywhere else.
  The catalog and the tests use a few Material Symbols, copied from the pinned commit into
  `reference/material-symbols/` with their license, in the same way as the androidx files.
- **b. The library ships Material Symbols**, all of them or a chosen subset.
  - It makes a few thousand icons part of the library's API and size.
  - It adds a second license (Apache-2.0) to what ships under MPL-2.0 ([ADR 0007](0007-license.md)).

## Decision

Recommended: **1 and a**. Icons are path data drawn on a canvas and filled with a token color, and the library ships no
icon set. Only the catalog and the tests copy the few Material Symbols they show.

## Consequences

- `nsh_m3e::Icon` holds path data and a view box. Every component that shows an icon takes an `Icon`. The icon button
  is the first.
- A test checks the icon's drawing: its path data, its scale to the token size and its fill from the token color, in
  every state.
- The visual checks read the icon's pixels against its token color on every host, the web included.
- The app picks its icons and ships them itself. The Material Symbols the catalog uses are vendored at a pinned commit
  and declared in `REUSE.toml`.
- Multicolor images stay with NSH's `image` and `svg`. They are not icons in this library.
