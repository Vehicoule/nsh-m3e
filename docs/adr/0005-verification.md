# 0005. Show correctness with types, property tests and mutation testing, not formal proofs

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

The first app had green static goldens that hid a hard-coded shadow, and tests that only confirmed the intended action.
Formal tools exist for Rust: Kani (bounded model checking), Verus, Aeneas (Rust to Lean 4). They cost a lot of effort
per property and suit small, critical algorithms more than UI components.

## Decision

For this library:

1. **Types first**: states as enums, typed units, `#[non_exhaustive]`; invalid states unrepresentable.
2. **Property tests** (`proptest`) on token logic: every foreground/background role pair meets WCAG AA contrast (and
   7:1 in high contrast), every converted spring keeps its ratio, generated tokens match the vendored source.
3. **Behavior tests** with NSH's `Headless` runner: what must happen *and* what must not (no relayout on hover, no
   extra mount items, no animation when "reduce motion" is on).
4. **Visual checks** of every state and transition (screenshots, then sequences with interruption, spamming and going
   back), never static goldens alone.
5. **Mutation testing** (`cargo-mutants`) in CI, so a test that asserts nothing is caught.
6. **API stability** with `cargo-semver-checks` once the first version is tagged.

Kani is kept for the app's core (queue and player state machine, sync merge rules), in the Vehicoule repository. Lean
and Aeneas are not used.

## Consequences

- `proptest` and `cargo-mutants` are added with the first code that needs them, each with its justification.
- Mutation testing is slow; it runs on pull requests touching `crates/` and its survivors fail the check.
