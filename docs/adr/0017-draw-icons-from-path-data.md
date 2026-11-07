# 0017. Draw icons from SVG path data on a canvas, filled with a token color, and ship every Material Symbol in its three styles

- Status: Accepted
- Date: 2026-11-07
- Deciders: the user

## Context

The icon button is the next component ([`docs/app-requirements.md`](../app-requirements.md), "First components the app
needs"). After it come the navigation bar and rail, the top app bars, the FAB, chips and lists, which all show icons.
How an icon is given and drawn is public API that every one of them shares. It has to be settled once, before the
first of them ships.

- **What M3 asks of an icon.** An icon is one shape in one color: the component's content color in each state, from
  the tokens. For example, `SmallIconButtonTokens.IconSize` is the size, and `FilledIconButtonTokens.Color` /
  `DisabledColor` / `DisabledOpacity` are the colors (androidx `a8b393d51f9b`). The icons M3 itself uses are
  Material Symbols. Each one is a single filled path, published as SVG files by Google under Apache-2.0
  (https://github.com/google/material-design-icons, commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`). Of the 24,942
  files the library ships (4,157 icons, 3 styles, filled and not, at 24 px), 24,348 draw in a 960 × 960 view box
  (`viewBox="0 -960 960 960"`), 192 in `"0 96 960 960"`, and 402 have no view box and draw in their 24-point square.
  (Corrected when the icons were vendored: the first version said every file had the first view box.)
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

And for where the icons come from. Material Symbols, at the pinned commit, has 4,157 icons, each in three styles
(outlined, rounded, sharp) and 168 variants per style (optical size, weight, grade, fill). A 24-point outlined SVG file
is 484 bytes on average (40 icons drawn at random). Measured in this repository's development container (4 cores, not a
CI runner), 3 builds from scratch each: a crate of 4,000 path constants of 700 bytes (3 MB of source, more than the real
icons) builds in 0.3 s, debug or release, and documents in 1 s. An app that depends on it and shows one of those paths
grows by 936 bytes in release (445,688 bytes against 444,752 with none), and its binary holds the path it uses and none
of the others, debug or release: a Rust constant is copied where it is used and nowhere else.

- **a. The library ships no icon set.** The app copies each icon's path data itself. The catalog and the tests copy a
  few Material Symbols with their license.
- **b. The library ships every Material Symbol in the outlined style, filled and not.** The SVG files are copied from
  the pinned commit into `reference/material-symbols/` (about 4 MB), as the androidx files are, and `cargo xtask`
  generates one constant per icon and fill from them, as it generates the tokens.
  - The app has every icon of M3's default style without a pull request each time it needs one.
  - The library's API and documentation get 8,314 more names. An app pays only for the icons it shows.
  - The generated files carry Material Symbols' license, Apache-2.0, beside MPL-2.0, as the generated tokens carry
    androidx's ([ADR 0007](0007-license.md), `REUSE.toml`).
- **c. As b, in the three styles**: about 12 MB copied, 24,942 names (4,157 icons × 2 fills × 3 styles), and the app
  picks its style.
- **d. As b, for a chosen list of icons only**, which grows with the app: the smallest copy, and a pull request for each
  new batch of icons.

## Decision

**1 and c**, chosen by the user. Icons are path data drawn on a canvas and filled with a token color, and the library
ships every Material Symbol in the three styles (outlined, rounded, sharp), filled and not, generated from the files
copied at the pinned commit. Option b was recommended; the user chose c. The app can pick its style, and change it,
without a pull request to the library.

## Consequences

- `nsh_m3e::icon::Icon` holds path data and a view box. Every component that shows an icon takes an `Icon`. The icon
  button is the first.
- A test checks the icon's drawing: its path data, its scale to the token size and its fill from the token color, in
  every state.
- The visual checks read the icon's pixels against its token color on every host, the web included.
- Each style is its own module of generated constants, so the app names the style it uses. Outlined is M3's default
  style; the library sets no default and the app chooses.
- A script keeps `reference/material-symbols/` (the three styles, about 12 MB) on the pinned commit and CI checks it,
  as `scripts/sync-androidx.sh` does; `cargo xtask` generates the icons from it and `--check` fails when they drift.
  The generated icons come in a pull request of their own, after the icon button.
- Multicolor images stay with NSH's `image` and `svg`. They are not icons in this library.
