# Architecture Decision Records

One decision per file, numbered, never renumbered. A decision is changed by a new ADR that supersedes it, never by
editing the old one beyond its status line.

Status: **Proposed** (written, waiting for the user), **Accepted** (the user decided), **Superseded by NNNN**.
Only the user moves an ADR from Proposed to Accepted.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-rust-only.md) | Components and app in Rust, no TSX | Accepted |
| [0002](0002-ci-is-the-source-of-truth.md) | CI is the only source of truth | Accepted |
| [0003](0003-tokens-from-androidx.md) | M3 values come from the androidx tokens, generated into Rust | Accepted |
| [0004](0004-spring-damping-conversion.md) | Spring damping ratio converted to NSH's coefficient | Accepted |
| [0005](0005-verification.md) | How correctness is shown: property tests, mutation testing, no Lean | Accepted |
| [0006](0006-pinned-versions-and-dependencies.md) | Pinned versions and the dependency policy | Accepted |
| [0007](0007-license.md) | The library is licensed under MPL-2.0 | Accepted |
| [0009](0009-depend-on-nsh-with-targeted-deny-exceptions.md) | Depend on NSH, with targeted `cargo deny` exceptions | Accepted |
