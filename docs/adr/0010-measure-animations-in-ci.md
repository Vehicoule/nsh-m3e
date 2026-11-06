# 0010. Measure animations in CI by what the component sends and how NSH's graph plays it

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

[ADR 0002](0002-ci-is-the-source-of-truth.md) leaves one question open: the reference devices (Xiaomi 14, a Mac with
a 120 Hz display) cannot run on GitHub-hosted runners, so what does CI measure for an animated component? It must be
settled before the first one. The button is that component: its shape morphs when it is pressed (androidx
`ButtonSmallTokens.ContainerShapeRound` = `CornerFull` at rest, `PressedContainerShape` = `CornerSmall`).

What the motion must do ([`docs/app-requirements.md`](../app-requirements.md), Motion):

- M3E springs from the tokens ([ADR 0004](0004-spring-damping-conversion.md)), through transform and opacity, with no
  layout per frame.
- No dropped frame at the display's refresh rate (120 Hz on the reference devices), never a fixed 60 Hz.
- Nothing animates without a visible consumer.
- "Reduce motion" replaces motion with short fades.
- No icon that jumps at the end. Transitions survive interruption, spamming and going back.

The first app broke these in ways no one measured. It passed the damping ratio as the coefficient, and it used the
standard springs instead of the Expressive ones (ADR 0004). Its timings were taken on the developer's machine, which
was skewed by load, power saving and debug builds (ADR 0002).

### What NSH does

All facts below are at the pinned revision `897fd9c2d968`.

- **One description, played on the UI thread.** An animation is described once and played frame by frame on the UI
  thread, "without calling back into your code" (`website/docs/animation.md`).
  - The engine sends one operation for it. For a spring this is `AnimationOp::Start` with
    `Driver::Spring { to, stiffness, damping, mass, velocity, frame_rate }` (`crates/nsh-core/src/animation/ops.rs`).
- **Every platform plays it with the same graph:** `nsh_core::animation::Graph`, a public type that has no platform
  code (`crates/nsh-core/src/animation/mod.rs`). Who drives the graph's frames:

  | Platform | Frame source | Where in NSH |
  |---|---|---|
  | iOS | a `CADisplayLink` | `crates/nsh-platform-ios/src/animation.rs` |
  | Android | the `Choreographer` | `crates/nsh-platform-android/src/animation.rs` |
  | Linux, Windows, macOS, GPUI | the backend's own frame loop | each backend |
  | Headless test runner | a 60 Hz test clock | `crates/nsh-testing/src/headless/animation.rs` |

  `Graph::apply` takes the operations, `Graph::tick(now)` advances it, and `Graph::value(node)` reads a value back.
- **Frame rate.** `frame_rate: None` asks for the screen's highest rate (`Driver::frame_rate`). On iOS the display link
  asks for the full rate. Above 60 Hz on a ProMotion iPhone, it also needs `CADisableMinimumFrameDurationOnPhone` in
  the app's `Info.plist`, which `nsh package` sets.
- **Headless runner.** `nsh_testing::Headless` runs the real engine without a window.
  - With `Clock::manual`, time moves only by `run_for`.
  - `observe` hands the test every mount batch and the step it came in.
  - `Environment` sets `reduce_motion`.
- **NSH's own testing guide** (`.claude/skills/nsh/references/rust-testing.md`, "What headless does not prove") says:
  - A headless run proves nothing visual and nothing about performance.
  - App tests should not assert on animation frames.

  The headless runner's frames are 60 Hz and follow its clock. The method below does not read those frames. It replays
  the operations in its own `Graph`, on its own clock.

### A trial at the pinned revision

A test, not committed, built a view whose translation is an animated value, and a press that starts the default
spatial spring: k = 380, c = 0.8 · 2 · √380, m = 1. It ran under `Headless` with a manual clock and recorded every
batch with `observe`. Then it replayed the animation operations in a `Graph` ticked every 1/120 s for 1.5 s.

- The press sent one batch, with one item: `Start { driver: Spring { stiffness: 380, damping: 31.189743, mass: 1,
  frame_rate: None } }`.
- During the 1.5 s of `run_for` that followed, the engine sent **no batch at all**: no layout, no text and no
  operation per frame.
- For a travel of 100 points, the replayed values matched the closed-form solution of `m·x″ + c·x′ + k·x = 0` within
  **0.00035 points** at every frame. The overshoot peaks near t = 0.3 s (101.31 at the 0.3 s frame), and the value is
  100.000 from t = 0.8 s.

How this was run: in the development container (`cargo test`, debug profile), once. Wall-clock time plays no part in
any of these numbers, so every run gives the same ones; CI would reproduce them.

### Constraints of the runners and the dependencies

- **Runners.** GitHub-hosted runners are shared virtual machines with no GPU and no 120 Hz display. A frame time
  measured there measures the runner.
- **Dependencies.** `nsh-engine` (`default-features = false`) passes `cargo deny check` with the exceptions of
  [ADR 0009](0009-depend-on-nsh-with-targeted-deny-exceptions.md). `nsh-testing` does not: it turns on `nsh-core`'s
  `shader` feature, which brings `naga` and, through it, `hexf-parse` 0.2.1. That crate is licensed `CC0-1.0`, which
  `deny.toml` does not allow. This was checked by adding each dev-dependency to a copy of the repository and running
  `cargo deny check`.

## Options

1. **Deterministic headless measurement.** For each animated component, check what it sends and how the graph plays
   it, without wall-clock time:
   - Record its batches under `Headless` with a manual clock.
   - Replay the animation operations in a `Graph` at 120 Hz.
   - Assert on the operations, the sampled curve, and the engine's work per frame.

   It gives a fast check, in seconds, on every pull request, that catches each defect of the first app. It costs two
   dev-dependencies, a license exception for `hexf-parse` (or more test code, see the decision), and a small test
   helper. It does not prove frame pacing on a device, the cost of drawing, or what the screen shows.
2. **Wall-clock timings on hosted runners.** Gate on `nsh test --trace` commit times, or on frame times under `xvfb`
   with `--target linux`. It is cheap, but the numbers come from a shared virtual machine with a 60 Hz virtual
   display. A threshold is then either flaky or too loose to catch anything.
3. **Real devices in CI.** Use a self-hosted runner with the reference devices, or a device farm. It gives the only
   true frame-rate measurement. It costs hardware or a subscription, upkeep, secrets, and the security of a
   self-hosted runner. None of these is available today.
4. **No CI measurement.** Measure on a device by hand and report it in the pull request. This contradicts ADR 0002:
   a check that only runs locally does not count.

## Decision

**Option 1** is the CI gate for every animated component, with the `nsh-testing` harness and its `CC0-1.0` exception
for `hexf-parse`. The user chose it on 2026-11-06.

- **Device measurements** stay manual, reported as AGENTS.md rule 8 asks: the gesture, the device, the build profile
  and the number of runs. They stay manual until a device runner (option 3) is decided in its own ADR.
- **Harness:** use `nsh-testing`'s `Headless`, which ADR 0005 already plans for behavior tests. This needs
  a `CC0-1.0` exception for `hexf-parse` only, used by tests only, written in `deny.toml` like the `webpki-roots` one.
  - The alternative not taken: `nsh-engine` alone, with a test sink written here, as NSH's own engine tests do
    (`crates/nsh-engine/tests/engine/rust_animation.rs`). That needs no exception, but adds more test code to
    maintain, and loses selectors, presses and hovers.

### What each animated component's tests check

These run with `cargo test` and in the `mutants` job, so a check that asserts nothing is caught (ADR 0005).

1. **Parameters.**
   - Every driver the component sends is the one its spec names, built from the M3E token through the ADR 0004
     conversion.
   - Stiffness, damping and mass equal the `f32` values the helper produces, exactly.
   - A standard spring, a raw number or a ratio passed as the coefficient fails.
2. **Curve.** The sampled values follow the closed-form spring within 0.1 % of the travel at every 1/120 s frame,
   and end exactly on the target.
   - The trial's error was 0.00035 % of the travel.
   - Over the same 1.5 s, a ratio passed as the coefficient (c = 0.8) misses the curve by up to 102 % of the travel,
     and the standard M3 spring (k = 700, ζ = 0.9) by up to 14 %. Both are computed from the closed form.
   - The last step into rest is no larger than the tolerance, which is the "jump at the end" defect.
3. **Engine work per frame.**
   - While the graph plays an animation, the engine sends no mount batch.
   - Hover and press send no `Frame` and no `Text` item, so there is no relayout.

   These are counts, exact, not timings.
4. **Frame rate.** Every driver asks for `frame_rate: None`. A fixed rate such as `Some(60.0)` fails.
5. **Nothing animates for nothing.**
   - Mounting a component starts no animation.
   - Once it settles, `Graph::is_animating()` is false.
6. **Reduce motion.** With `reduce_motion` on, the component sends no spring and no transform motion, only a short
   opacity fade with the duration its spec gives, from the tokens.
7. **Interruption and spamming.** A new target in mid-flight starts from the current value and velocity
   (`Spring::velocity: None` keeps the current one). No frame jumps by more than the spring's own largest step, and N
   presses end in the state N presses mean.

## Consequences

- **Dependencies and deny.** The first animated component's pull request adds `nsh-engine` and `nsh-testing` as
  workspace dev-dependencies, with `default-features = false` and the pinned `rev`. It also adds the
  `CC0-1.0`/`hexf-parse` exception, with this ADR as its reason.
- **One shared test helper.** It records batches, replays them in a `Graph` at 120 Hz, and gives the closed-form spring
  to compare with. Components do not each write their own.
- **What CI proves, and what it does not.**
  - Proved: the right springs, played by the same graph every platform runs, with no engine work per frame, at the
    screen's rate, quiet when idle, faded when motion is reduced, and continuous when interrupted.
  - Not proved: frame pacing and dropped frames on a device, the cost of drawing, and what the screen shows. Pull
    requests say so in their Verification section. The screens are ADR 0005's visual checks; the rest is the device
    runner's ADR.
- **NSH bumps.** A daily NSH bump that changes the graph's behavior fails these tests in its own pull request, before
  any component is affected.
- **ADR 0002.** This decision closes its open question once accepted.
