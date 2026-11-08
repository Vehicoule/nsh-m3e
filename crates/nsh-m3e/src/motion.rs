//! Motion: Material 3 Expressive's springs, ready for NSH.
//!
//! M3 gives a spring as a stiffness and a damping *ratio* (ζ). NSH's `Spring` takes a stiffness, a damping
//! *coefficient* (c) and a mass. Passing the ratio as the coefficient makes a spring that bounces for seconds: that
//! was the first app's bug. So components never take spring numbers; they take a [`SpringToken`], and
//! [`SpringSpec::damping_coefficient`] does the conversion (ADR 0004).
//!
//! A *layout* animation (`ElementRef::animate_layout`) takes neither: its curve is a damping ratio and a duration, not a
//! stiffness. [`SpringSpec::layout_transition`] gives the pair that makes NSH's curve the M3 spring.

use nsh_core::element::{LayoutCurve, LayoutTransition};

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

    /// The layout transition that plays this spring: what `ElementRef::animate_layout` takes to move frames as the
    /// spring would.
    ///
    /// NSH's `LayoutCurve::Spring { damping }` is a damped spring that is fitted into the duration, as UIKit's
    /// `usingSpringWithDamping` does: its envelope has fallen to 1/1000 by the end (`nsh_core::transition::spring`,
    /// `SETTLE`, and the critically damped `w = 9.233`, at the pinned NSH revision). Its damping ratio is the spring's
    /// own, and the duration that makes it the same spring is the time the spring takes to fall to 1/1000:
    /// `ln(1000) / (ζ · ω₀)` with `ω₀ = √(k / m)`, or `9.233 / ω₀` when it is critically damped (ratio 1, an effects
    /// spring). The curve then reaches 1 at the end of the duration where the spring has 1/1000 of its distance left,
    /// the only difference.
    ///
    /// NSH takes a damping ratio from 0.05 to 1: a spring outside that range gets the nearest one. No spring of the
    /// motion scheme is.
    ///
    /// ```
    /// use nsh_m3e::motion::SpringToken;
    ///
    /// let transition = SpringToken::FastSpatial.spec().layout_transition();
    /// // ln(1000) / (0.6 · √800) seconds.
    /// assert!((transition.duration_ms - 407.04).abs() < 0.01);
    /// ```
    #[must_use]
    pub fn layout_transition(self) -> LayoutTransition {
        let natural = (self.stiffness / SPRING_MASS).sqrt();
        let (damping, rate) = if self.damping_ratio > NSH_CRITICAL_ABOVE {
            (1.0, NSH_CRITICAL_RATE)
        } else {
            let ratio = self.damping_ratio.max(NSH_MIN_DAMPING);
            (ratio, NSH_SETTLE / ratio)
        };
        // NSH's curve has `rate` radians per duration where the spring has `natural` per second.
        LayoutTransition::new(LayoutCurve::Spring { damping }, rate / natural * 1000.0)
    }
}

/// ln(1000): the rate at which NSH's underdamped layout curve settles, `w = SETTLE / ζ`, its envelope at 1/1000 by the
/// end (`nsh_core::transition::spring`, constant `SETTLE`, `crates/nsh-core/src/transition.rs` line 658 at the pinned
/// revision). Not a design value: where NSH ends its curve.
const NSH_SETTLE: f32 = 6.907_755;

/// The rate of NSH's critically damped layout curve, `(1 + w x) e^(-w x)`, which reaches 1/1000 at `w = 9.233`
/// (`nsh_core::transition::spring`, line 662 at the pinned revision). Not a design value: where NSH ends its curve.
const NSH_CRITICAL_RATE: f32 = 9.233;

/// Above this damping ratio NSH's curve is critically damped (`zeta > 0.999`, line 660 at the pinned revision).
const NSH_CRITICAL_ABOVE: f32 = 0.999;

/// The lowest damping ratio NSH's curve takes (`damping.clamp(0.05, 1.0)`, line 659 at the pinned revision).
const NSH_MIN_DAMPING: f32 = 0.05;

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

    /// Where the same mass is, at each of `seconds` (ascending), after being let go at rest at 0 and pulled to 1: the
    /// motion of [`peak`], integrated with RK4, sampled.
    fn positions(stiffness: f32, damping: f32, mass: f32, seconds: &[f64]) -> Vec<f64> {
        let (stiffness, damping, mass) =
            (f64::from(stiffness), f64::from(damping), f64::from(mass));
        let acceleration = |position: f64, velocity: f64| {
            (-stiffness * (position - 1.0) - damping * velocity) / mass
        };
        let dt: f64 = 1e-4;
        let (mut position, mut velocity) = (0.0_f64, 0.0_f64);
        let mut elapsed = 0.0;
        let mut out = Vec::with_capacity(seconds.len());
        for &until in seconds {
            // The last step of each stretch is the short one that ends exactly at the moment asked for.
            while elapsed < until {
                let h = dt.min(until - elapsed);
                let (p1, v1) = (velocity, acceleration(position, velocity));
                let (p2, v2) = (
                    velocity + 0.5 * h * v1,
                    acceleration(position + 0.5 * h * p1, velocity + 0.5 * h * v1),
                );
                let (p3, v3) = (
                    velocity + 0.5 * h * v2,
                    acceleration(position + 0.5 * h * p2, velocity + 0.5 * h * v2),
                );
                let (p4, v4) = (
                    velocity + h * v3,
                    acceleration(position + h * p3, velocity + h * v3),
                );
                position += h / 6.0 * (p1 + 2.0 * p2 + 2.0 * p3 + p4);
                velocity += h / 6.0 * (v1 + 2.0 * v2 + 2.0 * v3 + v4);
                elapsed += h;
            }
            out.push(position);
        }
        out
    }

    /// NSH's layout curve at `x`, the fraction of its duration: `nsh_core::transition::spring` as it is at the pinned
    /// revision (`crates/nsh-core/src/transition.rs`, lines 650 to 669), transcribed because NSH does not export it.
    /// The tests below compare it with a spring that is integrated, so that nothing here is taken on trust from this
    /// transcription or from the formula under test.
    fn nsh_curve(damping: f32, x: f32) -> f64 {
        const SETTLE: f64 = 6.907_755;
        let x = f64::from(x);
        let zeta = f64::from(damping).clamp(0.05, 1.0);
        if zeta > 0.999 {
            let w = 9.233;
            return 1.0 - (1.0 + w * x) * (-w * x).exp();
        }
        let w = SETTLE / zeta;
        let wd = w * (1.0 - zeta * zeta).sqrt();
        let envelope = (-zeta * w * x).exp();
        1.0 - envelope * ((wd * x).cos() + zeta * w / wd * (wd * x).sin())
    }

    /// The most that NSH's curve, over the transition's duration, is ever away from `spec` played as the spring it
    /// is, at a hundred moments of the motion short of its end (the end is where the spring has 1/1000 of its
    /// distance left, and the curve has none).
    fn worst_distance(spec: SpringSpec, transition: &LayoutTransition) -> f64 {
        let LayoutCurve::Spring { damping } = transition.curve else {
            panic!("not a spring: {transition:?}");
        };
        let duration = f64::from(transition.duration_ms) / 1000.0;
        let fractions: Vec<f64> = (1..=100).map(|i| f64::from(i) / 100.0 * 0.99).collect();
        let seconds: Vec<f64> = fractions.iter().map(|x| x * duration).collect();
        let played = positions(
            spec.stiffness,
            spec.damping_coefficient(),
            SPRING_MASS,
            &seconds,
        );
        fractions
            .iter()
            .zip(played)
            .map(|(x, played)| (nsh_curve(damping, *x as f32) - played).abs())
            .fold(0.0, f64::max)
    }

    /// Each token's layout transition: the spring curve, its damping ratio (an effects spring's is 1, NSH's critical
    /// curve) and the time it takes to fall to 1/1000, in milliseconds: `ln(1000) / (ζ·√k)`, or `9.233 / √k`.
    const LAYOUT_TRANSITIONS: [(SpringToken, f32, f32); 6] = [
        (SpringToken::DefaultSpatial, 0.8, 442.95),
        (SpringToken::FastSpatial, 0.6, 407.04),
        (SpringToken::SlowSpatial, 0.8, 610.57),
        (SpringToken::DefaultEffects, 1.0, 230.83),
        (SpringToken::FastEffects, 1.0, 149.78),
        (SpringToken::SlowEffects, 1.0, 326.44),
    ];

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
    fn every_token_has_the_layout_transition_worked_out() {
        for (token, damping, milliseconds) in LAYOUT_TRANSITIONS {
            let transition = token.spec().layout_transition();
            assert_eq!(
                transition.curve,
                LayoutCurve::Spring { damping },
                "{token:?}"
            );
            assert!(
                (transition.duration_ms - milliseconds).abs() < 0.01,
                "{token:?} takes {} ms",
                transition.duration_ms
            );
            // Nothing else is asked of NSH: no delay, and appearing views fade as NSH's default has them do.
            assert!(
                (transition.delay_ms - 0.0).abs() < f32::EPSILON,
                "{token:?}"
            );
            assert_eq!(
                transition,
                LayoutTransition::new(transition.curve, transition.duration_ms),
                "{token:?}"
            );
        }
    }

    #[test]
    fn every_layout_transition_plays_its_spring() {
        for token in ALL {
            let spec = token.spec();
            let worst = worst_distance(spec, &spec.layout_transition());
            assert!(worst < 1e-3, "{token:?} is {worst} away from its spring");
        }
    }

    #[test]
    fn a_layout_transition_of_the_wrong_length_is_not_the_spring() {
        // The comparison can tell: the same curves over a duration 10 % too long or too short are visibly another
        // motion, and so is the ratio taken as the duration's scale (the first app's mistake, once more).
        for token in ALL {
            let spec = token.spec();
            let right = spec.layout_transition();
            for factor in [0.9_f32, 1.1] {
                let wrong = LayoutTransition {
                    duration_ms: right.duration_ms * factor,
                    ..right
                };
                let worst = worst_distance(spec, &wrong);
                assert!(worst > 0.01, "{token:?} x{factor}: only {worst} away");
            }
        }
    }

    #[test]
    fn a_spatial_spring_overshoots_as_a_layout_curve_and_an_effects_spring_does_not() {
        for token in ALL {
            let transition = token.spec().layout_transition();
            let LayoutCurve::Spring { damping } = transition.curve else {
                panic!("not a spring: {token:?}");
            };
            let highest = (1..1000)
                .map(|i| nsh_curve(damping, i as f32 / 1000.0))
                .fold(0.0, f64::max);
            if is_spatial(token) {
                assert!(highest > 1.001, "{token:?} peaks at {highest}");
            } else {
                assert!(highest <= 1.0, "{token:?} overshoots to {highest}");
            }
        }
    }

    #[test]
    fn a_ratio_outside_what_nsh_takes_gets_the_nearest_one() {
        let spec = |damping_ratio| SpringSpec {
            damping_ratio,
            ..SpringToken::FastSpatial.spec()
        };
        assert_eq!(
            spec(0.0).layout_transition().curve,
            LayoutCurve::Spring { damping: 0.05 }
        );
        assert_eq!(
            spec(2.0).layout_transition().curve,
            LayoutCurve::Spring { damping: 1.0 }
        );
        // An overdamped spring is NSH's critical one: the same duration as one at ratio 1.
        assert!(
            (spec(2.0).layout_transition().duration_ms - spec(1.0).layout_transition().duration_ms)
                .abs()
                < f32::EPSILON
        );
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

        #[test]
        fn an_underdamped_layout_transition_plays_any_spring(
            stiffness in 50.0_f32..5_000.0,
            ratio in 0.1_f32..0.998,
        ) {
            let spec = SpringSpec { stiffness, damping_ratio: ratio };
            let worst = worst_distance(spec, &spec.layout_transition());
            prop_assert!(worst < 1e-3, "{worst}");
        }

        #[test]
        fn a_critically_damped_layout_transition_plays_any_spring(stiffness in 50.0_f32..5_000.0) {
            let spec = SpringSpec { stiffness, damping_ratio: 1.0 };
            let worst = worst_distance(spec, &spec.layout_transition());
            prop_assert!(worst < 1e-3, "{worst}");
        }
    }
}
