# 0004. Convert M3's spring damping ratio to NSH's damping coefficient

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

M3 defines springs by stiffness and damping **ratio** (ζ): `ExpressiveMotionTokens.SpringDefaultSpatialStiffness =
380`, `SpringDefaultSpatialDamping = 0.8`. NSH's `Spring` (`crates/nsh-core/src/element/animated.rs`) takes stiffness
`k`, damping **coefficient** `c` and mass `m`, with defaults 170 / 26 / 1. Passing 0.8 as `c` gives an almost
undamped spring that oscillates for seconds. The first app also used the standard M3 springs (700 / 0.9) instead of
the Expressive ones; that was the main cause of "not the real animations".

## Decision

One function converts: `c = ζ · 2 · √(k · m)`, with `m = 1`. Components never build an NSH `Spring` from raw numbers;
they take a motion token (default, fast or slow; spatial or effects) and the conversion builds it.

| Expressive token | k | ζ | c |
|---|---|---|---|
| Default spatial | 380 | 0.8 | 31.19 |
| Fast spatial | 800 | 0.6 | 33.94 |
| Slow spatial | 200 | 0.8 | 22.63 |
| Default effects | 1600 | 1.0 | 80.00 |
| Fast effects | 3800 | 1.0 | 123.29 |
| Slow effects | 800 | 1.0 | 56.57 |

## Consequences

- A property test checks the round trip (`c / (2·√(k·m)) == ζ` within tolerance) for every token, and that effects
  springs (ζ = 1) never overshoot.
- The Expressive scheme is the default; the standard scheme exists only if the user asks for it.
