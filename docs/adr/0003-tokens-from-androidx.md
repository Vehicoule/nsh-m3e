# 0003. Take M3 values from the androidx tokens and generate the Rust tokens from them

- Status: Proposed
- Date: 2026-11-06
- Deciders: the user

## Context

Material 3's values (colors, type scale, shapes, elevations, motion springs, component sizes) are published as design
tokens. Jetpack Compose Material 3 ships them as generated Kotlin files (`// GENERATED CODE - DO NOT MODIFY BY HAND`)
in `androidx/compose/material3/tokens/`, under Apache-2.0. They are the official library's own values, including the
Expressive ones (`ExpressiveMotionTokens`, `ButtonXLargeTokens`, `LoadingIndicatorTokens`, ...). The first app copied
432,000 lines of a third-party M3 library and still used the wrong springs.

## Options

1. A hand-written `tokens.json`, then Rust generated from it: one more copy of values that already exist, typed by hand.
2. Generate the Rust token modules from the vendored androidx Kotlin files with a small tool in this repository, and
   check in CI that the generated code matches.
3. Write the Rust constants by hand: fastest to start, and exactly how wrong values creep in.

## Decision (recommended)

Option 2. The vendored files in `reference/androidx-material3-tokens/` are the single source; a generator turns them
into Rust; CI regenerates and fails on any difference. Each Rust constant keeps the androidx token name in its doc
comment.

## Consequences

- No design value is ever typed by hand; updating M3 is updating the pinned androidx revision and regenerating.
- Dynamic color (schemes from a seed color, the "content" and "tonal spot" schemes the app uses) is an algorithm, not
  a token: it needs Google's material-color-utilities or a faithful Rust port, decided in its own ADR.
- Vehicoule-specific values (the pastel chroma cap, Mochi's peach) live in a separate, clearly named module, never mixed
  with M3's.
