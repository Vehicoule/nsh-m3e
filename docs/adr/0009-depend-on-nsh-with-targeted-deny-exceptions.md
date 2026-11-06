# 0009. Depend on NSH, with targeted `cargo deny` exceptions for NSH's own dependencies

- Status: Proposed
- Date: 2026-11-06
- Deciders: the user

## Context

A component of this library is an `nsh-base` control styled with NSH's style API, so the library must depend on
`nsh-core` (and later `nsh-base`). With `nsh-core` at the pinned revision (`897fd9c2d968`), `cargo deny check` fails on
crates that NSH brings, not on this library's code:

| Check | Crate | Why it fails | Where it comes from |
|---|---|---|---|
| advisories | `zip` 8.6.0 | RUSTSEC-2026-0367, -0368, -0369, -0429 (extraction of hostile archives); fixed in zip 9 | `nsh-updater`, which extracts downloaded update archives (`crates/nsh-updater/src/install.rs`); NSH's `main` still requires `zip = "8"` |
| advisories | `ttf-parser` 0.25 | RUSTSEC-2026-0192: unmaintained, no known vulnerability | `nsh-macros`, at build time |
| licenses | `webpki-roots` 0.26 and 1.0 | `CDLA-Permissive-2.0` is not in the allowed list | NSH's TLS (`rustls`, `tokio-tungstenite`): Mozilla's root certificates, as data |
| bans | `nsh-core` | a git dependency without a `version` counts as a wildcard | the root `Cargo.toml` |

ADR 0006 also says that a dependency written in C, or exposed to untrusted content, needs the user's explicit
agreement. `nsh-core` brings `ring` (C and assembly, built with `cc`) through `rustls`, and `rustls` and
`tokio-tungstenite` handle network input. Every NSH app has them; this library calls none of them.

The Vehicoule app, in its own repository, will meet the same failures as soon as it depends on NSH.

## Options

1. **Targeted exceptions**: ignore each advisory by its id with its reason, allow `CDLA-Permissive-2.0` for
   `webpki-roots` only, declare NSH's version beside its `rev`. Every other advisory, license and wildcard still fails.
   `cargo deny` reports an ignore that no longer matches, so each exception is removed once NSH drops the crate.
2. **Wait for NSH**: keep the library free of NSH; tokens and data can go on, components cannot.
3. **Allow the license only**: the license and the version, no ignored advisory; `deny` stays red until NSH moves to
   zip 9.

## Decision

Option 1, chosen by the user on 2026-11-06. Waiting for the user's explicit agreement on `ring` and the network
crates (ADR 0006) before this ADR is accepted.

## Consequences

- `deny.toml` lists five ignored advisories and one license exception, each with its reason and this ADR.
- The root `Cargo.toml` declares `version = "<NSH's version>"` beside the `rev`; `scripts/sync-nsh.sh update` writes it
  from NSH's workspace version, and `scripts/sync-nsh.sh check` fails when they differ.
- When a daily NSH bump makes an exception unused, its pull request removes it.
- Reporting the `zip` advisories to NSH is the user's to do; this repository cannot open issues there.
