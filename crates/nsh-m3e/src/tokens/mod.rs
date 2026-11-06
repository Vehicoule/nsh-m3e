//! Design tokens: Material 3's values, as Rust constants.
//!
//! [`androidx`] is generated from the vendored androidx token files and never edited by hand (ADR 0003). The types
//! its constants use are defined here.

pub mod androidx;

use androidx::motion_tokens::EASING_LINEAR_CUBIC_BEZIER;

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

#[cfg(test)]
mod tests {
    use super::*;
    use androidx::motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;

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
}
