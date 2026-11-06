# 0006. Pin every version and keep dependencies few

- Status: Proposed
- Date: 2026-11-06
- Deciders: the user

## Context

An agent in an ambiguous environment guesses; the first app's agents invented APIs and drifted between versions. NSH is
young (repository created on 2026-10-14) and changes daily on `main`.

## Decision (recommended)

- Rust toolchain pinned in `rust-toolchain.toml`; `Cargo.lock` committed; `--locked` everywhere in CI.
- NSH used as a git dependency pinned by `rev`, the same revision as the vendored NSH skills. Updating NSH is one pull
  request that moves both together and lists what changed.
- Vendored references (`reference/`, `.claude/skills/`) carry their source revision in a `PROVENANCE.md` and are never
  edited by hand.
- Dependencies are declared once in the workspace; `cargo deny` checks advisories, licenses (GPL-3.0 compatible) and
  sources; a new dependency needs a one-line justification. A dependency written in C, or exposed to untrusted
  content, needs the user's explicit agreement.
- GitHub Actions are pinned by commit SHA.

## Consequences

- Upgrades are deliberate and reviewable; nothing changes under an agent's feet.
- The pinned NSH revision must be bumped regularly, since NSH promises no compatibility until its first recorded
  release.
