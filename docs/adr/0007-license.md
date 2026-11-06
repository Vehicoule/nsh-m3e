# 0007. License the library under MPL-2.0

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

The Vehicoule app is licensed under GPL-3.0. This library is a separate building block that other NSH apps may want to
use. NSH, the androidx tokens vendored here and most of the Rust ecosystem are under Apache-2.0 or MIT.

## Options

1. GPL-3.0: every app using the library must be GPL-3.0; keeps the NSH community away.
2. MIT OR Apache-2.0: the most permissive; changes to the library may stay closed.
3. MPL-2.0: file-level copyleft. Changes to the library's own files stay open; apps using it keep any license.

## Decision

MPL-2.0. It is compatible with the app's GPL-3.0 (MPL-2.0 section 3.3) and with the Apache-2.0 files it vendors, which
keep their own license and notices.

## Consequences

- `LICENSE` holds the MPL-2.0 text; `Cargo.toml` declares `license = "MPL-2.0"`.
- Vendored files (`reference/`, `.claude/skills/`) keep their own licenses, listed in their provenance files.
- Contributions are accepted under the same license.
