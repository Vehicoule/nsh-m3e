# Vendored skills

Never edited by hand. The NSH skills are copied by `scripts/sync-nsh.sh` from the NSH commit pinned in the root
`Cargo.toml`, and CI checks that they match it; the others are updated by copying a newer revision in a pull request of
their own.

| Skill | Source | Revision | License |
|---|---|---|---|
| `nsh` | https://github.com/longbridge/nsh/tree/main/skills/nsh | `fec005467bbe84a659c56534ff60f119ecd43209` | Apache-2.0, © Longbridge (`LICENSE-APACHE`) |
| `nsh-design-guides` | https://github.com/longbridge/nsh/tree/main/skills/nsh-design-guides | `fec005467bbe84a659c56534ff60f119ecd43209` | Apache-2.0, © Longbridge (`LICENSE-APACHE`) |
| `rust-best-practices` | https://github.com/stolinski/s-stack/tree/main/skills/rust-best-practices, based on Apollo GraphQL's Rust Best Practices handbook | `3cc9e7c22e7e1039297ac6aa98923dc1ac963351` | MIT (declared in the skill's front matter) |
| `rust-coding-guidelines` | https://github.com/actionbook/rust-skills/tree/main/skills/coding-guidelines (renamed from `coding-guidelines`) | `5c40d3ad785193231b7d0dbfb8e1eb447e5edd94` | MIT (declared in the project's README) |

The `nsh` skill also describes TSX and mixed modes; this repository uses Rust only (ADR 0001).
