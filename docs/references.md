# References

Every source an agent may rely on, with the version it is pinned to. Use the pinned copy when there is one; the live
site may describe a newer version than the one this repository builds against.

## NSH

| What | Where | Pinned |
|---|---|---|
| Source | https://github.com/longbridge/nsh, branch `main` | `897fd9c2d968b341fcd81ceb34e991b13167e558`, the `rev` in the root `Cargo.toml`, bumped daily ([ADR 0006](adr/0006-pinned-versions-and-dependencies.md)) |
| Coding and Design Guides, mode pages, recipes | [`.claude/skills/nsh/`](../.claude/skills/nsh/), [`.claude/skills/nsh-design-guides/`](../.claude/skills/nsh-design-guides/) | same revision |
| Docs site (newest release, may be ahead) | https://nsh.dev, index for agents: https://nsh.dev/llms.txt | not pinned |
| Animation and `Spring` | https://nsh.dev/docs/animation.md, `crates/nsh-core/src/element/animated.rs` | same revision |
| Base components (behavior, no look) | https://nsh.dev/modules/nsh-base.md, `crates/nsh-base/` | same revision |
| Headless engine and test driver (animation measurements, ADR 0010) | [`.claude/skills/nsh/references/rust-testing.md`](../.claude/skills/nsh/references/rust-testing.md), `crates/nsh-testing/`, `crates/nsh-engine/`, `crates/nsh-core/src/animation/` | same revision |

## Material 3 Expressive

| What | Where | Pinned |
|---|---|---|
| Official token values | [`reference/androidx-material3-tokens/`](../reference/androidx-material3-tokens/) (from `androidx/androidx`, `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/`) | `a8b393d51f9bdcfcac8f0a4a0992ecad1b27ba55`, moved by `scripts/sync-androidx.sh` when the tokens change |
| Official implementation (behavior, layout, how tokens are used) | https://github.com/androidx/androidx/tree/androidx-main/compose/material3/material3 | read at the same revision |
| Guidelines (anatomy, states, usage, motion) | https://m3.material.io, sections Foundations, Styles, Components | not pinned (web) |
| Dynamic color algorithm (HCT, schemes) | https://github.com/material-foundation/material-color-utilities | to pin when used (ADR to come) |
| Figma | The Material 3 Design Kit on Figma Community, read through the Figma MCP when available | to pin when used |

The androidx tokens win over the website when they differ: they are what the official library ships.

## Rust

| What | Where |
|---|---|
| Google's Rust style guide (Chrome and Fuchsia tagged rules do not apply here) | [`reference/rust/google-rust-style-guide.md`](../reference/rust/google-rust-style-guide.md), from https://google.github.io/styleguide/rustguide.html |
| Apollo's Rust best practices | [`.claude/skills/rust-best-practices/`](../.claude/skills/rust-best-practices/) |
| Rust coding guidelines (50 core rules) | [`.claude/skills/rust-coding-guidelines/`](../.claude/skills/rust-coding-guidelines/) |
| Rust API Guidelines | https://rust-lang.github.io/api-guidelines/ |
| Clippy lints | https://rust-lang.github.io/rust-clippy/ |

The full `actionbook/rust-skills` plugin (https://github.com/actionbook/rust-skills) can be installed as a Claude Code
plugin by whoever wants it; only its `coding-guidelines` skill is vendored here.

## Verification tools

| What | Where | Pinned |
|---|---|---|
| Property tests | https://docs.rs/proptest | `1.12.0` in the root `Cargo.toml`, exact version in `Cargo.lock` |
| Mutation testing | https://mutants.rs | `27.1.0`, installed by CI (`.github/workflows/ci.yml`) |
