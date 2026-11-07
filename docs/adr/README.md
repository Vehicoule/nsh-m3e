# Architecture Decision Records

One decision per file, numbered, never renumbered. A decision is changed by a new ADR that supersedes it, never by
editing the old one beyond its status line.

Status: **Proposed** (written, waiting for the user), **Accepted** (the user decided), **Superseded by NNNN**.
Only the user moves an ADR from Proposed to Accepted.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-rust-only.md) | Components and app in Rust, no TSX (test scripts excepted by 0013) | Accepted |
| [0002](0002-ci-is-the-source-of-truth.md) | CI is the only source of truth | Accepted |
| [0003](0003-tokens-from-androidx.md) | M3 values come from the androidx tokens, generated into Rust | Accepted |
| [0004](0004-spring-damping-conversion.md) | Spring damping ratio converted to NSH's coefficient | Accepted |
| [0005](0005-verification.md) | How correctness is shown: property tests, mutation testing, no Lean | Accepted |
| [0006](0006-pinned-versions-and-dependencies.md) | Pinned versions and the dependency policy (bump merge amended by 0014) | Accepted |
| [0007](0007-license.md) | The library is licensed under MPL-2.0 | Accepted |
| [0008](0008-color-scheme-in-nsh-tokens.md) | The M3 color scheme lives in NSH's tokens as `md-sys-color-*` | Accepted |
| [0009](0009-depend-on-nsh-with-targeted-deny-exceptions.md) | Depend on NSH, with targeted `cargo deny` exceptions | Accepted |
| [0010](0010-measure-animations-in-ci.md) | Animations measured in CI from their operations, replayed in NSH's graph at 120 Hz | Accepted |
| [0011](0011-morph-shapes-with-a-drawn-container.md) | Shapes morph on press in a drawn container whose corners follow the M3E spring | Accepted |
| [0012](0012-follow-androidx-components-where-tokens-are-wrong.md) | Where androidx's component and its token files disagree, follow the component | Accepted |
| [0013](0013-visual-checks-on-nsh-hosts.md) | Visual checks on NSH's real hosts, with test scripts in TypeScript | Accepted |
| [0014](0014-merge-nsh-bumps-on-green.md) | The NSH bump pull request merges itself when every check is green | Accepted |
| [0015](0015-declare-every-file-license-with-reuse.md) | Every file's license declared with REUSE, checked in CI | Accepted |
| [0016](0016-adopt-the-contributor-covenant.md) | The Contributor Covenant 3.0 is the code of conduct, under its own CC BY-SA 4.0 license | Accepted |
| [0017](0017-draw-icons-from-path-data.md) | Icons are SVG path data drawn on a canvas in a token color; the library ships every Material Symbol in its three styles | Accepted |
| [0018](0018-draw-elevation-as-two-box-shadows-per-level.md) | Elevation is drawn as the two box shadows Material Web gives each level | Accepted |
