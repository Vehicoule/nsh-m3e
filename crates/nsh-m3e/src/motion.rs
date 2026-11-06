//! Motion: Material 3 Expressive's springs, ready for NSH.
//!
//! M3 gives a spring as a stiffness and a damping *ratio* (ζ). NSH's `Spring` takes a stiffness, a damping
//! *coefficient* (c) and a mass. Passing the ratio as the coefficient makes a spring that bounces for seconds: that
//! was the first app's bug. So components never take spring numbers; they take a [`SpringToken`], and
//! [`SpringSpec::damping_coefficient`] does the conversion (ADR 0004).

use crate::tokens::androidx::expressive_motion_tokens as expressive;

/// The mass of every M3 spring: the springs are defined at unit mass, and ADR 0004 converts with `m = 1`. It is also
/// the default mass of NSH's `Spring`.
pub const SPRING_MASS: f32 = 1.0;

/// One of the six springs of the Expressive motion scheme (androidx `MotionSchemeKeyTokens`).
///
/// *Spatial* springs move things (position, size, rotation, shape) and may overshoot; *effects* springs change color
/// and opacity and never overshoot. *Fast* suits small components, *slow* full-screen changes.
///
/// ```
/// use nsh_m3e::motion::SpringToken;
///
/// let spring = SpringToken::DefaultSpatial.spec();
/// assert!((spring.stiffness - 380.0).abs() < 1e-3);
/// assert!((spring.damping_ratio - 0.8).abs() < 1e-6);
/// // What NSH's `Spring::damping` takes (ADR 0004): 0.8 · 2 · √380.
/// assert!((spring.damping_coefficient() - 31.19).abs() < 0.01);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpringToken {
    /// androidx: `MotionSchemeKeyTokens.DefaultSpatial`.
    #[default]
    DefaultSpatial,
    /// androidx: `MotionSchemeKeyTokens.FastSpatial`.
    FastSpatial,
    /// androidx: `MotionSchemeKeyTokens.SlowSpatial`.
    SlowSpatial,
    /// androidx: `MotionSchemeKeyTokens.DefaultEffects`.
    DefaultEffects,
    /// androidx: `MotionSchemeKeyTokens.FastEffects`.
    FastEffects,
    /// androidx: `MotionSchemeKeyTokens.SlowEffects`.
    SlowEffects,
}

impl SpringToken {
    /// The spring's stiffness and damping ratio in the Expressive scheme (androidx `ExpressiveMotionTokens`).
    #[must_use]
    pub const fn spec(self) -> SpringSpec {
        let (stiffness, damping_ratio) = match self {
            SpringToken::DefaultSpatial => (
                expressive::SPRING_DEFAULT_SPATIAL_STIFFNESS,
                expressive::SPRING_DEFAULT_SPATIAL_DAMPING,
            ),
            SpringToken::FastSpatial => (
                expressive::SPRING_FAST_SPATIAL_STIFFNESS,
                expressive::SPRING_FAST_SPATIAL_DAMPING,
            ),
            SpringToken::SlowSpatial => (
                expressive::SPRING_SLOW_SPATIAL_STIFFNESS,
                expressive::SPRING_SLOW_SPATIAL_DAMPING,
            ),
            SpringToken::DefaultEffects => (
                expressive::SPRING_DEFAULT_EFFECTS_STIFFNESS,
                expressive::SPRING_DEFAULT_EFFECTS_DAMPING,
            ),
            SpringToken::FastEffects => (
                expressive::SPRING_FAST_EFFECTS_STIFFNESS,
                expressive::SPRING_FAST_EFFECTS_DAMPING,
            ),
            SpringToken::SlowEffects => (
                expressive::SPRING_SLOW_EFFECTS_STIFFNESS,
                expressive::SPRING_SLOW_EFFECTS_DAMPING,
            ),
        };
        SpringSpec {
            stiffness,
            damping_ratio,
        }
    }
}

/// A spring as M3 defines it: a stiffness and a damping ratio, at [`SPRING_MASS`].
///
/// Only a [`SpringToken`] makes one, so every spring in this crate is M3's.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSpec {
    /// Stiffness `k`, what NSH's `Spring::stiffness` takes.
    pub stiffness: f32,
    /// Damping ratio `ζ`: below 1 the spring overshoots, at 1 it settles as fast as it can without overshooting.
    /// This is **not** what NSH's `Spring::damping` takes; that is [`SpringSpec::damping_coefficient`].
    pub damping_ratio: f32,
}

impl Default for SpringSpec {
    /// The default spatial spring, [`SpringToken::DefaultSpatial`].
    fn default() -> Self {
        SpringToken::DefaultSpatial.spec()
    }
}

impl SpringSpec {
    /// The damping coefficient `c = ζ · 2 · √(k · m)` with `m` = [`SPRING_MASS`]: what NSH's `Spring::damping` takes.
    #[must_use]
    pub fn damping_coefficient(self) -> f32 {
        damping_coefficient(self.stiffness, self.damping_ratio, SPRING_MASS)
    }
}

/// `c = ζ · 2 · √(k · m)`: critical damping is `2 · √(k · m)`, and `ζ` is the fraction of it. The `2` is part of the
/// formula, not a design value.
fn damping_coefficient(stiffness: f32, damping_ratio: f32, mass: f32) -> f32 {
    damping_ratio * 2.0 * (stiffness * mass).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const ALL: [SpringToken; 6] = [
        SpringToken::DefaultSpatial,
        SpringToken::FastSpatial,
        SpringToken::SlowSpatial,
        SpringToken::DefaultEffects,
        SpringToken::FastEffects,
        SpringToken::SlowEffects,
    ];

    /// ADR 0004's table: token, k, ζ, c rounded to two decimals.
    const ADR_0004: [(SpringToken, f32, f32, f32); 6] = [
        (SpringToken::DefaultSpatial, 380.0, 0.8, 31.19),
        (SpringToken::FastSpatial, 800.0, 0.6, 33.94),
        (SpringToken::SlowSpatial, 200.0, 0.8, 22.63),
        (SpringToken::DefaultEffects, 1600.0, 1.0, 80.00),
        (SpringToken::FastEffects, 3800.0, 1.0, 123.29),
        (SpringToken::SlowEffects, 800.0, 1.0, 56.57),
    ];

    fn is_spatial(token: SpringToken) -> bool {
        matches!(
            token,
            SpringToken::DefaultSpatial | SpringToken::FastSpatial | SpringToken::SlowSpatial
        )
    }

    /// The highest point of a unit step: a mass `m` at rest at 0 pulled to 1 by
    /// `m·x'' = -k·(x - 1) - c·x'`, integrated with RK4 for `seconds`. It solves the motion itself instead of
    /// trusting the formula under test.
    fn peak(stiffness: f32, damping: f32, mass: f32, seconds: f64) -> f64 {
        let (stiffness, damping, mass) =
            (f64::from(stiffness), f64::from(damping), f64::from(mass));
        let acceleration = |position: f64, velocity: f64| {
            (-stiffness * (position - 1.0) - damping * velocity) / mass
        };
        let dt = 1e-4;
        let (mut position, mut velocity, mut highest) = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut elapsed = 0.0;
        while elapsed < seconds {
            let (p1, v1) = (velocity, acceleration(position, velocity));
            let (p2, v2) = (
                velocity + 0.5 * dt * v1,
                acceleration(position + 0.5 * dt * p1, velocity + 0.5 * dt * v1),
            );
            let (p3, v3) = (
                velocity + 0.5 * dt * v2,
                acceleration(position + 0.5 * dt * p2, velocity + 0.5 * dt * v2),
            );
            let (p4, v4) = (
                velocity + dt * v3,
                acceleration(position + dt * p3, velocity + dt * v3),
            );
            position += dt / 6.0 * (p1 + 2.0 * p2 + 2.0 * p3 + p4);
            velocity += dt / 6.0 * (v1 + 2.0 * v2 + 2.0 * v3 + v4);
            highest = highest.max(position);
            elapsed += dt;
        }
        highest
    }

    #[test]
    fn every_token_has_the_expressive_values_of_adr_0004() {
        for (token, stiffness, ratio, coefficient) in ADR_0004 {
            let spec = token.spec();
            assert!((spec.stiffness - stiffness).abs() < 1e-3, "{token:?} k");
            assert!((spec.damping_ratio - ratio).abs() < 1e-6, "{token:?} ζ");
            assert!(
                (spec.damping_coefficient() - coefficient).abs() < 0.005,
                "{token:?} c = {}",
                spec.damping_coefficient()
            );
        }
    }

    #[test]
    fn no_token_uses_the_standard_scheme() {
        // androidx `StandardMotionTokens`: spatial springs 700 / 0.9, 1400 / 0.9, 300 / 0.9. The first app used them
        // instead of the Expressive ones.
        for token in ALL {
            let spec = token.spec();
            for (stiffness, ratio) in [(700.0_f32, 0.9_f32), (1400.0, 0.9), (300.0, 0.9)] {
                let same = (spec.stiffness - stiffness).abs() < 1e-3
                    && (spec.damping_ratio - ratio).abs() < 1e-6;
                assert!(!same, "{token:?} is a standard spring");
            }
        }
    }

    #[test]
    fn the_coefficient_is_never_the_ratio() {
        for token in ALL {
            let spec = token.spec();
            assert!(
                spec.damping_coefficient() > 10.0 * spec.damping_ratio,
                "{token:?}"
            );
        }
    }

    #[test]
    fn spatial_springs_overshoot_and_effects_springs_do_not() {
        for token in ALL {
            let spec = token.spec();
            let highest = peak(spec.stiffness, spec.damping_coefficient(), SPRING_MASS, 3.0);
            if is_spatial(token) {
                assert!(highest > 1.001, "{token:?} peaks at {highest}");
            } else {
                assert!(highest <= 1.0 + 1e-6, "{token:?} overshoots to {highest}");
            }
        }
    }

    #[test]
    fn passing_the_ratio_as_the_coefficient_would_bounce() {
        // The first app's bug, shown with the same simulation: the critically damped default effects spring would
        // overshoot by more than 90 %.
        let spec = SpringToken::DefaultEffects.spec();
        let highest = peak(spec.stiffness, spec.damping_ratio, SPRING_MASS, 3.0);
        assert!(highest > 1.9, "peaks at {highest}");
    }

    #[test]
    fn the_defaults_are_the_default_spatial_spring() {
        assert_eq!(SpringToken::default(), SpringToken::DefaultSpatial);
        assert_eq!(SpringSpec::default(), SpringToken::DefaultSpatial.spec());
        assert_ne!(SpringSpec::default(), SpringToken::FastSpatial.spec());
    }

    #[test]
    fn the_mass_is_nsh_s_default_unit_mass() {
        assert!((SPRING_MASS - 1.0).abs() < f32::EPSILON);
    }

    proptest! {
        #[test]
        fn the_ratio_comes_back_from_the_coefficient(
            stiffness in 1.0_f32..10_000.0,
            ratio in 0.0_f32..3.0,
            mass in 0.1_f32..10.0,
        ) {
            let coefficient = damping_coefficient(stiffness, ratio, mass);
            let back = coefficient / (2.0 * (stiffness * mass).sqrt());
            prop_assert!((back - ratio).abs() <= 1e-5 * ratio.max(1.0), "{back} != {ratio}");
        }

        #[test]
        fn an_underdamped_spring_overshoots(stiffness in 50.0_f32..5_000.0, ratio in 0.05_f32..0.9) {
            let highest = peak(stiffness, damping_coefficient(stiffness, ratio, SPRING_MASS), SPRING_MASS, 3.0);
            prop_assert!(highest > 1.0 + 1e-4, "peaks at {highest}");
        }

        #[test]
        fn a_spring_damped_at_least_critically_never_overshoots(
            stiffness in 50.0_f32..5_000.0,
            ratio in 1.0_f32..3.0,
        ) {
            let highest = peak(stiffness, damping_coefficient(stiffness, ratio, SPRING_MASS), SPRING_MASS, 3.0);
            prop_assert!(highest <= 1.0 + 1e-6, "overshoots to {highest}");
        }
    }
}
