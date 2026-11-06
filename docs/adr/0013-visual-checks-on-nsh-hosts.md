# 0013. Take the visual checks on NSH's real hosts, with test scripts in TypeScript

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

[ADR 0005](0005-verification.md) asks for visual checks of every state and transition: screenshots, then sequences
with interruption, spamming and going back, never static goldens alone. The button at rest is the first component
(pull request #23). On 2026-11-06 the user chose to ship it with its behavior tests and to settle the visual checks
here, then add the button's.

All facts below are at NSH's pinned revision `897fd9c2d968`.

- **The Rust test runner takes no picture.** `nsh_testing::Headless` runs the real engine with no window: it reads
  what the engine mounts (styles, frames, texts), not what a platform draws.
- **NSH takes pictures on its real hosts.** `nsh test --target <macos|linux|windows|gpui|web|ios|android>` runs the
  same tests in the platform's development host, with the platform's own input, and `app.screenshot(path)` saves a
  PNG of the window (`website/docs/testing-and-trace.md`, "On a real platform"). `app.spanStyle(selector)` reads a
  text's resolved color, size and weight on every target.
- **Those tests are TypeScript.** `nsh test` runs `.test.tsx` files in the CLI's QuickJS. A Rust app is tested this
  way too: on a real host, it runs in a development host built with its crates, which runs its root
  (`website/docs/testing-and-trace.md`, "On a real platform"; `website/docs/rust.md`, "Testing").
- **NSH tests its own Rust components that way.** `examples/gallery-rs` is a Rust app with no TSX in it
  (`"rust": "native"`, `"root": "Gallery"`), whose tests are `.test.tsx` files, about one per demo.
- **What NSH's CI runs on real hosts**: GTK windows on Linux under `xvfb-run` and Win32 windows on Windows
  (`.github/workflows/linux.yml`, `windows.yml`), and the web in headless Chromium (`rust-app.yml`). It runs no test on
  the iOS simulator or the Android emulator: whether `nsh test --target ios` and `--target android` work on
  GitHub-hosted runners is not found.
- **The hosts draw differently.** On iOS and Android NSH shows native views (UIKit, Android views); on Linux GTK; on
  the web the DOM; GPUI draws its own. A picture from one host proves nothing about another
  (`website/docs/coding-guides.md`: "Record any unavailable platform verification rather than assuming one screenshot
  proves all targets").
- **Rust only** ([ADR 0001](0001-rust-only.md), AGENTS.md rule 5): no TSX, no QuickJS, no JavaScript. ADR 0001's
  reasons are the app's binary (no JavaScript engine written in C in it) and one language. A test script run by the
  `nsh` CLI is in neither the library nor the app's binary.

## Options

1. **TypeScript test scripts on NSH's hosts, the components in Rust.** As NSH does for `examples/gallery-rs`:
   - a small Rust app in the repository, `examples/catalog`, mounts each component in each of its states (its crate
     depends on `nsh-m3e`; `nsh.json` names its root);
   - `.test.tsx` files open it, set each state (appearance, size, style, disabled, hover, press), save a screenshot,
     and check what selectors can say (`app.spanStyle`);
   - a Rust `xtask` reads the screenshots and checks pixels where a value is known: the container's color at its
     center, the label's color, transparent pixels outside a rounded corner. This needs a PNG decoder, a new
     dependency (the `png` crate is the likely one);
   - the screenshots are kept as CI artifacts, for review in the pull request.
   - Cost: a second language for test scripts only, the `nsh` CLI in CI (built from NSH at the pinned revision), one
     more job per host. Gives: the real drawing on each host NSH supports, as its authors check their own components.
2. **Rust only, through GPUI's test platform.** `nsh-platform-gpui` with its feature `shot` renders a window offscreen
   in Rust tests (`.github/workflows/gpui.yml`, "Test (GPUI's test platform)"). Cost: it pulls GPUI into the
   dev-dependencies, and it is not a public API for apps (`shot` is described as development support). Gives:
   pictures with no TypeScript, but of GPUI's drawing only, which is none of the app's targets (iOS, Android).
3. **No pictures: checks of what is mounted only**, as the button's tests do now, plus `app.spanStyle`-like reads.
   Cost: ADR 0005's visual checks are dropped, a check removed, which AGENTS.md rule 2 forbids without the user.
   Gives: nothing new to set up.

## Decision

Option 1, chosen by the user on 2026-11-06. It is what NSH's authors do for their own Rust components, and the only
one that shows the native views the app ships on. The library and the app stay Rust; only test scripts run by the `nsh` CLI are
TypeScript.

Which hosts run in CI:

- on each pull request touching `crates/`: Linux (GTK under `xvfb-run`) and the web (headless Chromium), which NSH's
  own CI runs on the same runners;
- iOS (simulator, macOS runner) and Android (emulator, Linux runner with KVM): tried first in a pull request of their
  own, since NSH does not run them in CI. If they work, on each pull request touching a component; if not, the pull
  request says so and they run by hand on the reference devices, reported as AGENTS.md rule 8 asks.

## Consequences

- ADR 0001 gets an exception, and AGENTS.md rule 5 says: "Rust only, except `.test.tsx` files run by `nsh test` for
  the visual checks of ADR 0013." No TypeScript in `crates/`, and nothing the library or the app ships runs JavaScript.
- New: `examples/catalog` (Rust), `tests/visual/*.test.tsx`, an `xtask` command that checks the screenshots, a CI job
  per host, and the `nsh` CLI installed in CI at the pinned revision (kept in step by `scripts/sync-nsh.sh`).
- The `png` crate (or the decoder chosen) is a new dependency, with its line of justification and `cargo deny`.
- The button's pull request after this ADR adds its screenshots: every size, style, shape, appearance, disabled, and
  hover, on each host that runs.
- A screenshot is never a golden alone: each comes with the checks of its known pixels, and the artifacts are what a
  reviewer looks at.
- Not decided here: the sequences of transitions (interruption, spamming, going back) come with the press morph
  (ADR 0011), measured as ADR 0010 says.
