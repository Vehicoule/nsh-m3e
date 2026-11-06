//! Design tokens: Material 3's values, as Rust constants.
//!
//! [`androidx`] is generated from the vendored androidx token files and never edited by hand (ADR 0003). The types
//! its constants use are defined here.

pub mod androidx;

use androidx::motion_tokens::EASING_LINEAR_CUBIC_BEZIER;
use androidx::palette_tokens::BLACK;

/// A cubic Bézier easing curve from `(0, 0)` to `(1, 1)` through the control points `(x1, y1)` and `(x2, y2)`, as in
/// CSS `cubic-bezier()` and NSH's `Easing::Bezier(x1, y1, x2, y2)`.
///
/// Only tokens build one, so every curve in this crate is Material 3's:
///
/// ```
/// use nsh_m3e::tokens::androidx::motion_tokens::EASING_EMPHASIZED_DECELERATE_CUBIC_BEZIER as CURVE;
///
/// assert_eq!((CURVE.x1, CURVE.y1, CURVE.x2, CURVE.y2), (0.05, 0.7, 0.1, 1.0));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    /// The first control point's time, from 0 to 1.
    pub x1: f32,
    /// The first control point's progress.
    pub y1: f32,
    /// The second control point's time, from 0 to 1.
    pub x2: f32,
    /// The second control point's progress.
    pub y2: f32,
}

impl CubicBezier {
    pub(crate) const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

impl Default for CubicBezier {
    /// Constant speed: androidx `MotionTokens.EasingLinearCubicBezier`.
    fn default() -> Self {
        EASING_LINEAR_CUBIC_BEZIER
    }
}

/// An opaque color in 8-bit sRGB, as the androidx palette gives it.
///
/// Only tokens build one, so every color in this crate is Material 3's. [`Srgb::to_hex`] gives the `0xRRGGBB` that
/// NSH's `color::rgb` and `AdaptiveColor::rgb` take:
///
/// ```
/// use nsh_m3e::tokens::androidx::palette_tokens::PRIMARY40;
///
/// assert_eq!((PRIMARY40.red, PRIMARY40.green, PRIMARY40.blue), (103, 80, 164));
/// assert_eq!(PRIMARY40.to_hex(), 0x6750a4);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb {
    /// The red channel, from 0 to 255.
    pub red: u8,
    /// The green channel, from 0 to 255.
    pub green: u8,
    /// The blue channel, from 0 to 255.
    pub blue: u8,
}

impl Srgb {
    pub(crate) const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// The color as `0xRRGGBB`.
    #[must_use]
    pub const fn to_hex(self) -> u32 {
        // Bit positions of the `0xRRGGBB` layout, not design values. The channels never overlap, so `+` is `|`; it
        // is written `+` because `|` and `^` would give the same result, and no test could tell them apart.
        ((self.red as u32) << 16) + ((self.green as u32) << 8) + self.blue as u32
    }
}

impl Default for Srgb {
    /// Black: androidx `PaletteTokens.Black`.
    fn default() -> Self {
        BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use androidx::motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;
    use androidx::{color_dark_tokens, color_light_tokens, palette_tokens};

    #[test]
    fn new_keeps_each_control_point_in_place() {
        // androidx: `MotionTokens.EasingEmphasizedAccelerateCubicBezier` = (0.3, 0.0, 0.8, 0.15). Every point
        // differs, so a swapped argument shows.
        let curve = EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;
        assert_eq!(
            [curve.x1, curve.y1, curve.x2, curve.y2].map(f32::to_bits),
            [0.3_f32, 0.0, 0.8, 0.15].map(f32::to_bits)
        );
    }

    #[test]
    fn the_default_is_the_linear_token_and_not_another_curve() {
        let default = CubicBezier::default();
        assert_eq!(default, EASING_LINEAR_CUBIC_BEZIER);
        assert_eq!(
            [default.x1, default.y1, default.x2, default.y2].map(f32::to_bits),
            [0.0_f32, 0.0, 1.0, 1.0].map(f32::to_bits)
        );
    }

    #[test]
    fn new_keeps_each_channel_in_place() {
        // androidx: `PaletteTokens.Primary40` = (103, 80, 164). Every channel differs, so a swapped argument shows.
        let color = palette_tokens::PRIMARY40;
        assert_eq!((color.red, color.green, color.blue), (103, 80, 164));
    }

    #[test]
    fn to_hex_puts_each_channel_in_its_own_byte() {
        assert_eq!(Srgb::new(0x12, 0x34, 0x56).to_hex(), 0x0012_3456);
        assert_eq!(Srgb::new(0xff, 0, 0).to_hex(), 0x00ff_0000);
        assert_eq!(Srgb::new(0, 0xff, 0).to_hex(), 0x0000_ff00);
        assert_eq!(Srgb::new(0, 0, 0xff).to_hex(), 0x0000_00ff);
        assert_eq!(Srgb::new(0xff, 0xff, 0xff).to_hex(), 0x00ff_ffff);
    }

    #[test]
    fn the_default_is_black_and_not_another_color() {
        assert_eq!(Srgb::default(), BLACK);
        assert_eq!(Srgb::default().to_hex(), 0);
        assert_ne!(Srgb::default(), palette_tokens::WHITE);
    }

    #[test]
    fn scheme_roles_resolve_to_their_palette_tones() {
        // androidx: `ColorLightTokens.Primary` = `PaletteTokens.Primary40`, `ColorDarkTokens.Primary` =
        // `PaletteTokens.Primary80`, `ColorLightTokens.Background` = `PaletteTokens.Neutral98`.
        assert_eq!(color_light_tokens::PRIMARY, palette_tokens::PRIMARY40);
        assert_eq!(color_dark_tokens::PRIMARY, palette_tokens::PRIMARY80);
        assert_eq!(color_light_tokens::BACKGROUND, palette_tokens::NEUTRAL98);
        assert_ne!(color_light_tokens::PRIMARY, color_dark_tokens::PRIMARY);
    }
}
