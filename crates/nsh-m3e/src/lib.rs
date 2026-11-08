//! Material 3 Expressive components for [NSH](https://github.com/longbridge/nsh).
//!
//! NSH's `nsh-base` gives each control its behavior, state, focus and accessibility, and no look. This crate gives
//! them the look of Material 3 Expressive: its tokens (color, type, shape, elevation, motion, state) and the styled
//! components built on them.
//!
//! Every design value comes from the Material 3 tokens vendored in `reference/androidx-material3-tokens/`, never from
//! memory. The rules for working on this crate are in `AGENTS.md` at the root of the repository.

#[cfg(test)]
mod animation_probe;
pub mod button;
pub mod checkbox;
pub mod color;
pub mod elevation;
pub mod icon;
pub mod icon_button;
pub mod icon_toggle_button;
pub mod material_symbols;
mod morph;
#[cfg(test)]
mod morph_probe;
pub mod motion;
pub mod snackbar;
pub mod switch;
mod tint;
pub mod toggle_button;
pub mod tokens;
