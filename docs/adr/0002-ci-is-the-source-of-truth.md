# 0002. Treat CI as the only source of truth

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

In the first version, work was verified on the developer's machine only. Measurements were skewed by load, power
saving and debug builds, merges broke the main branch twice, and slow tests were skipped without anyone seeing them
red. The user decided that only CI decides what is accepted; local runs are for development.

## Decision

Every change reaches `main` through a pull request whose CI is green. CI runs the commands listed in `AGENTS.md`.
`main` is protected: no direct push, CI required.

## Consequences

- Any check that matters must exist in CI; a check that only runs locally does not count.
- Signing (releases, feeds) will happen in CI, with secrets limited to the signing job and actions pinned by SHA.
- Open: performance gates measured on a real device (Xiaomi 14, Mac with a 120 Hz display) cannot run on GitHub-hosted
  runners. What runs in CI instead (headless `nsh test --trace` item counts and timings, per-frame work counters) is
  to be decided in its own ADR before the first animated component.
