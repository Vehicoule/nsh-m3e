# 0008. Carry the M3 color scheme in NSH's design tokens, under M3's `md-sys-color-*` names

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

NSH draws with design tokens: colors with a light and a dark value, by name (`nsh_core::element::Tokens`). Tailwind
classes name them (`bg-primary`), Rust reads them with `tokens()`, and the platform switches appearance with nothing
running. `nsh-base` fills the tokens with shadcn/ui's theme (`crates/nsh-base/src/theme.rs` at the pinned NSH
revision): `background`, `primary`, `secondary`, `muted`, `accent`, ... The platform's own controls read some of those
names: Linux's menus `popover`, `accent` and `border` (`crates/nsh-platform-linux/src/menu_style.rs`), iOS's system
controls `primary` as their tint (`crates/nsh-platform-ios/src/system.rs`).

Material 3 names its color roles `primary`, `on-primary`, `secondary`, `surface-container`, ... Three of them are
also shadcn names: `background`, `primary`, `secondary`. They do not all mean the same: shadcn's `secondary` is a
muted surface for secondary buttons, M3's is an accent color. M3 itself publishes its roles as *system* tokens named
`md.sys.color.<role>`, between the reference tokens (`md.ref.palette.*`, the palette tones) and the component tokens
(`md.comp.*`); Material Web exposes them as CSS custom properties `--md-sys-color-<role>`. An NSH token name is a CSS
custom property's name without its `--`.

## Options

1. **M3's system token names, `md-sys-color-<role>`, in NSH's tokens.** No name is shared with shadcn's, so `nsh-base`
   and the platform's controls keep their tokens; the names are M3's own. Tailwind classes are longer
   (`bg-md-sys-color-primary`); Rust components read them through a typed role and do not spell them.
2. **Short M3 names (`primary`, `on-primary`, ...) in NSH's tokens.** Shorter classes, but `background`, `primary` and
   `secondary` replace shadcn's: iOS's controls take M3's `primary` as their tint without anyone choosing it, and
   `bg-secondary` silently changes meaning for code written against `nsh-base`'s theme.
3. **A context of this library's own, outside NSH's tokens.** No clash, but Tailwind classes and the platform's controls
   do not follow the scheme, and appearance changes go through app code.

## Decision

Option 1, chosen by the user on 2026-11-06. Every role of androidx `ColorSchemeKeyTokens` is an NSH color token
named `md-sys-color-` and the role in kebab case, with the light scheme's color in light appearance and the dark
scheme's in dark.

## Consequences

- `nsh_m3e::color::set_color_tokens` writes the 48 roles into a `Tokens` value; the app passes it to NSH's
  `set_tokens`. It changes no other token.
- It needs `nsh-core`, which `nsh-m3e` depends on since ADR 0009.
- Mapping M3 roles onto shadcn's names, so that `nsh-base`'s defaults and the platform's controls look M3 too, stays
  possible later; it would be a choice of its own, made with the user.
- The same rule names other system tokens when they become NSH tokens (`md-sys-shape-*`, `md-sys-typescale-*`), each
  confirmed when it comes.
