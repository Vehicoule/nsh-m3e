# 0001. Write the components and the app in Rust, with no TSX

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

NSH has three modes: Rust, Rust with TSX, and TSX. TSX runs on QuickJS, a JavaScript engine written in C. NSH's own
skill recommends Rust for an app shipped in its binary, for medium and large apps. Vehicoule has strict performance
budgets (no dropped frame at the display's refresh rate) and earlier specs excluded JavaScript engines written in C.

## Options

1. Rust only: one language, no JavaScript engine, typed styles, `cargo` everywhere.
2. Rust components with TSX screens: hot reload and screens deliverable at run time, at the cost of QuickJS and two
   languages.

## Decision

Rust only, for this library and for the app.

## Consequences

- No QuickJS in the app binary; no `nsh.json` TSX entry; `cargo test` with NSH's `Headless` for behavior.
- Edits rebuild and restart instead of hot reloading.
- Moving a screen to TSX later stays possible, since a Rust app already has a `#[native_module]`, but needs a new ADR.
