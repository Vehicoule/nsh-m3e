# AGENTS.md

`nsh-m3e` is a Material 3 Expressive (M3E) component library for [NSH](https://github.com/longbridge/nsh) (Native
Shell), written in Rust. It is the first building block of the Vehicoule rewrite: every screen of the app is made of
these components. NSH gives behavior with no look (`nsh-base`); this crate gives the look, M3E's, and nothing else.

This file is the rule book. Every agent, human or AI, reads it before touching the repository, and follows it over
any habit, skill or training data. When a rule here conflicts with another document, this file wins; when it
conflicts with what the user says in the task, the user wins and the rule gets an ADR.

## Read first, in this order

1. This file.
2. The decisions: [`docs/adr/`](docs/adr/). They say what is settled and why. Do not reopen a decision without a new
   ADR that the user accepts.
3. The NSH skills in [`.claude/skills/`](.claude/skills/): `nsh` (Coding Guides, Rust mode) and `nsh-design-guides`.
   Read the guide files themselves, not a summary.
4. For any M3 value (color, type, shape, elevation, motion, size, state): the vendored tokens in
   [`reference/androidx-material3-tokens/`](reference/androidx-material3-tokens/). Never from memory.
5. For Rust style: the `rust-best-practices` and `rust-coding-guidelines` skills, and
   [`reference/rust/google-rust-style-guide.md`](reference/rust/google-rust-style-guide.md).
6. What the app needs from this library: [`docs/app-requirements.md`](docs/app-requirements.md).

[`docs/references.md`](docs/references.md) lists every source with its pinned version.

## Non-negotiable rules

1. **CI is the only truth.** Nothing reaches `main` without a pull request whose CI is green. A local run is a
   convenience, never evidence. Never push to `main`.
2. **Never weaken a check to get green.** No lowered lint level, no `#[allow]` without an ADR, no skipped or ignored
   test, no raised performance ceiling, no shortened animation, no reduced scope. If a check is wrong, say so and stop:
   changing a check is the user's decision.
3. **Never invent an API or a value.** Look an NSH name up in its source at the pinned revision before using it. Take
   every M3 value from the vendored tokens, or from the vendored androidx component where it overrides a token, and
   cite it ([ADR 0012](docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md)). If something cannot be
   found, write "not found" and stop. Do not write it by analogy with Compose, Flutter, React or another framework.
4. **No hard-coded design values in components.** Colors, sizes, radii, durations, springs and type styles come from
   the token modules. A literal number in a component needs a comment saying why it is not a token.
5. **Rust only.** No TSX, no QuickJS, no JavaScript ([ADR 0001](docs/adr/0001-rust-only.md)). One exception: the
   `.test.tsx` files that `nsh test` runs for the visual checks
   ([ADR 0013](docs/adr/0013-visual-checks-on-nsh-hosts.md)). No TypeScript in `crates/`, and nothing the library or
   the app ships runs JavaScript.
6. **Springs use M3E's values, converted.** M3 gives a damping *ratio*; NSH's `Spring` takes a damping *coefficient*.
   Always convert with the tested helper ([ADR 0004](docs/adr/0004-spring-damping-conversion.md)). Passing a ratio
   as a coefficient is the bug that made the first app's animations look wrong.
7. **A test proves what must happen and what must not.** A test that only confirms the intended action proves little.
   Never change a test to make it pass: a changed or deleted assertion is listed and justified in the pull request.
8. **Every number has a source.** A token cites its token name. A measurement says what was measured (the real
   gesture or interaction), on which device or runner, in which build profile, and how many runs. If something was not
   measured, write "not measured" and the technical reason.
9. **No panics in library code.** No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` or indexing that can fail
   outside tests. No `unsafe` (`forbid`). Errors are typed (`thiserror`).
10. **Every public item is documented**, with a doc test when it has a usage. `missing_docs` is an error.
11. **Dependencies are rare.** A new dependency is declared once in the root `Cargo.toml`, passes `cargo deny`, and its
    pull request says in one line why it is needed and what was considered instead.
12. **One pull request does one thing.** No drive-by refactors, renames or formatting.
13. **Decisions belong to the user.** Anything that changes the architecture, the scope, a budget, a license, a
    dependency policy or a rule here is written as an ADR with options and a recommendation, and waits for the user.
14. **Do not guess the environment.** Use the pinned toolchain and versions. If a tool is missing or a command fails,
    report the exact output instead of working around it.

## Commands

The same commands run in CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked   # CI sets PROPTEST_CASES and PROPTEST_RNG_SEED (ci.yml): the same cases each run
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
cargo deny check
scripts/sync-androidx.sh check
cargo xtask gen-tokens --check
scripts/sync-material-symbols.sh check
cargo xtask gen-icons --check
scripts/sync-nsh.sh check
cargo mutants --workspace --cargo-arg=--locked   # cargo-mutants 27.1.0
```

The visual checks ([ADR 0013](docs/adr/0013-visual-checks-on-nsh-hosts.md)) run in
[`.github/workflows/visual.yml`](.github/workflows/visual.yml), with the `nsh` CLI at the pinned NSH revision and the
host's system libraries:

```sh
# under xvfb-run with a screen larger than 2608 x 2800 points; or --target web
nsh test examples/catalog/button.test.tsx --target linux
cargo run -p visual-checks --locked -- target/visual
```

On phones, [`.github/workflows/visual-phone.yml`](.github/workflows/visual-phone.yml) takes the catalog's screenshots
in the iOS simulator and the Android emulator, with the CLI built from an NSH checkout at the pinned revision:

```sh
nsh test examples/catalog/phone.test.tsx --target ios   # or --target android, with an emulator running
```

The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). Run the full list before opening a pull
request, and report what you ran.

## Layout

| Path | What it holds |
|---|---|
| `crates/nsh-m3e/` | The library |
| `crates/nsh-m3e/src/tokens/androidx/` | M3 tokens generated by `cargo xtask gen-tokens`; never edited by hand |
| `crates/nsh-m3e/src/material_symbols/` | Material Symbols generated by `cargo xtask gen-icons` ([ADR 0017](docs/adr/0017-draw-icons-from-path-data.md)); never edited by hand |
| `examples/catalog/` | A Rust app showing every component in each of its states, and the `.test.tsx` files that take its screenshots ([ADR 0013](docs/adr/0013-visual-checks-on-nsh-hosts.md)) |
| `tests/visual/` | `visual-checks`: checks the known pixels of the catalog's screenshots against the tokens |
| `xtask/` | Repository tooling in Rust (`cargo xtask gen-tokens`: generate the tokens from the vendored androidx files; `cargo xtask gen-icons`: generate the icons from the vendored Material Symbols) |
| `docs/adr/` | Decisions, numbered, one per file ([template](docs/adr/0000-template.md)) |
| `docs/` | Requirements and references |
| `reference/` | Vendored sources of truth, pinned, never edited by hand |
| `.claude/skills/` | Vendored skills for coding agents, pinned (the NSH ones are synced by `scripts/sync-nsh.sh`) |
| `scripts/` | Repository tooling (`sync-nsh.sh`: keep the NSH skills on the pinned NSH commit; `sync-androidx.sh`: keep `reference/androidx-material3-tokens/` and `reference/androidx-material3-components/` on the pinned androidx commit; `sync-material-symbols.sh`: keep `reference/material-symbols/` on the pinned Material Symbols commit) |

A token module that a component needs and that does not exist yet is added by listing its androidx file in
`SOURCES` (`xtask/src/main.rs`) and running `cargo xtask gen-tokens`, never by typing the values. The generator
rejects any Kotlin form it does not know; teaching it a new one is a change to `xtask/` with its tests.

## Conventions

- Everything in the repository is written in English: code, comments, docs, commit messages. Comments say why, not
  what.
- Commit subjects and pull request titles: `<area>: <What the change does>`, imperative, capitalized, no trailing
  period, about 72 characters (`tokens: Generate spring tokens from androidx`). Areas: `tokens`, `motion`, `color`,
  `shape`, `type`, a component name in snake case (`button`, `navigation_bar`), `docs`, `ci`, `deps`.
- The body explains why. A pull request that adds or changes public API lists every item under `## Public API`.
- New public structs are `#[non_exhaustive]` and implement `Default`. Public enums are `#[non_exhaustive]`.
- Component states are enums, never several booleans.
- Units are typed as NSH types them (`Points`, `Hsla`, milliseconds); no bare `f32` for a length in public API.
- Follow NSH's naming: names come from `nsh-base` / `gpui-base` and M3's own component names.
