//! The colors and the touch target that the components share: a color role at an opacity, resolved against the app's
//! NSH design tokens (ADR 0008), and the smallest area a press counts in.

use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::color::{ColorRole, ColorScheme};

/// A color role, made more transparent by `opacity` (1 for the role as it is).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Tint {
    pub(crate) role: ColorRole,
    pub(crate) opacity: f32,
}

impl Tint {
    pub(crate) const fn solid(role: ColorRole) -> Self {
        Self { role, opacity: 1.0 }
    }

    pub(crate) const fn faded(role: ColorRole, opacity: f32) -> Self {
        Self { role, opacity }
    }
}

/// What a color style gives a component, enabled and disabled: its container, its content (a label, an icon) and its
/// outline. No container is a transparent one; no outline is none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StyleSpec {
    pub(crate) container: Option<Tint>,
    pub(crate) content: Tint,
    pub(crate) disabled_container: Option<Tint>,
    pub(crate) disabled_content: Tint,
    pub(crate) outline: Option<Tint>,
    pub(crate) disabled_outline: Option<Tint>,
}

/// The smallest touch target, 48 points on each side: `docs/app-requirements.md` (Accessibility), from Material 3's
/// touch target guidance. No androidx token holds it.
pub(crate) const MIN_TOUCH_TARGET: Points = Points(48.0);

/// The color of `role` in the app's NSH design tokens (ADR 0008), or else in the baseline schemes, so that a component
/// shows Material's colors before the app sets its own.
pub(crate) fn role_color(role: ColorRole) -> AdaptiveColor {
    tokens().color(role.token_name()).unwrap_or_else(|| {
        AdaptiveColor::rgb(
            ColorScheme::BASELINE_LIGHT.get(role).to_hex(),
            ColorScheme::BASELINE_DARK.get(role).to_hex(),
        )
    })
}

/// The color of a tint, or transparent for none.
pub(crate) fn tint_color(tint: Option<Tint>) -> AdaptiveColor {
    tint.map_or(AdaptiveColor::TRANSPARENT, |tint| {
        role_color(tint.role).opacity(tint.opacity)
    })
}
