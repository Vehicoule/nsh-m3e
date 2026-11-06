# 0006. Pin every version and keep dependencies few

- Status: Accepted; the bump's merge amended by [0014](0014-merge-nsh-bumps-on-green.md)
- Date: 2026-11-06
- Deciders: the user

## Context

An agent in an ambiguous environment guesses; the first app's agents invented APIs and drifted between versions. NSH is
young (repository created on 2026-10-14) and changes daily on `main`.

## Decision

- Rust toolchain pinned in `rust-toolchain.toml`; `Cargo.lock` committed; `--locked` everywhere in CI.
- NSH follows its `main` branch, not its releases (the user tested NSH on `main`, and it changes almost daily). It is a
  git dependency pinned by `rev` to a commit of `main`, declared once in the root `Cargo.toml`; the vendored NSH skills
  are always those of that same commit (`scripts/sync-nsh.sh check` runs in CI).
- Every day, `.github/workflows/nsh-bump.yml` moves the pin to the newest commit of NSH's `main`, re-copies the skills
  from it and opens (or updates) one pull request with the link to NSH's changes. CI runs on it; the user merges it
  when green. A red bump shows exactly which NSH change breaks us.
- Vendored references (`reference/`, `.claude/skills/`) carry their source revision in a `PROVENANCE.md` and are never
  edited by hand.
- Dependencies are declared once in the workspace; `cargo deny` checks advisories, licenses (GPL-3.0 compatible) and
  sources; a new dependency needs a one-line justification. A dependency written in C, or exposed to untrusted
  content, needs the user's explicit agreement.
- GitHub Actions are pinned by commit SHA.

## Consequences

- Upgrades are deliberate and reviewable; nothing changes under an agent's feet, and the docs agents read always match
  the code they build against.
- We stay at most a day behind NSH's `main` as long as bump pull requests are merged. NSH promises no compatibility
  until its first recorded release, so a bump may need code changes; those go in the bump's pull request.
