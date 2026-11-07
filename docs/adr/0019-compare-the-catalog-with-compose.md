# 0019. Compare the catalog with androidx's own Compose components, on the same Android emulator

- Status: Proposed
- Date: 2026-11-07
- Deciders: the user

## Context

On 2026-11-07 the user asked whether the CI is enough to show that the components reproduce M3E. What it shows today:

- **The values.** The tokens are generated from androidx's files at a pinned commit, and CI fails when they drift
  (`scripts/sync-androidx.sh check`, `cargo xtask gen-tokens --check`).
- **The behavior, headless.** Sizes, colors and shapes per state, roles and labels for screen readers, touch targets,
  spring conversion; mutation testing makes every line of it tested (ADR 0005).
- **The pixels on Linux and the web.** `visual-checks` reads known pixels of the catalog's screenshots (container,
  corners, outline, icon; at rest and hovered) and compares them with the colors the tokens give (ADR 0013).
- **Screenshots on phones**, in the iOS simulator and the Android emulator (`visual-phone.yml`), with no pixel check.

What none of these shows is that the components look like M3E's own. Every check compares the library with the
tokens, read by the same person who wrote the component. A token read the wrong way, a padding or a layout rule that
lives in androidx's component code and not in a token (ADR 0012), or a shape drawn a little differently, passes every
check. The reference that settles it is the component Google ships: androidx's `compose.material3`.

Facts, checked on 2026-11-07:

- **The pinned commit is not a release.** `a8b393d51f9b` builds `COMPOSE_MATERIAL3 = "1.6.0-alpha01"`
  (`libraryversions.toml` at that commit), which Google's Maven repository does not have. Its latest version of
  `androidx.compose.material3:material3` is `1.5.0-rc01` (`maven-metadata.xml`, last updated 2026-11-04).
- **That release has our tokens.** The sources jar of `material3-android:1.5.0-rc01` holds the same 120 token files as
  `reference/androidx-material3-tokens/`, byte for byte. Of the vendored components, `IconButtonDefaults.kt` is
  identical. `Button.kt` and `IconButton.kt` differ only in KDoc tags (`@material3expressive`) and in how a missing
  interaction source is created (`remember { MutableInteractionSource() }`), not in a size, color or shape.
- **Our Android screenshots are taken on an emulator** in `visual-phone.yml`: API 35, `x86_64`, `google_apis`, through
  `reactivecircus/android-emulator-runner` v2.38.0. On Android, NSH shows native Android views (ADR 0013).
- **A Compose app needs the Android toolchain**: Gradle, the Android Gradle plugin, Kotlin and a JDK. None of these is
  in the repository, and AGENTS.md rule 5 allows no language but Rust, and `.test.tsx` for the visual checks.

## Options

1. **A Compose reference app on the same emulator.** A small Android app in `tests/compose-reference/` shows, with
   `androidx.compose.material3` itself, the same components in the same states on the same grid as `examples/catalog`
   (the same frames, in dp where the catalog uses points). The phone job installs it on the emulator it already boots
   and takes its screenshots with `adb`. A Rust check crops each component from both screenshots by its frame and
   compares the pixels, with a tolerance.
   - **States:** at rest, disabled, checked. Pressed and focused are forced through the component's
     `MutableInteractionSource` (a `PressInteraction.Press` or a `FocusInteraction.Focus` emitted, and the screenshot
     taken once the spring has settled).
   - **Text is masked at first:** the label's box is left out of the comparison until the typefaces are known to be
     the same on both sides. The container, its shape, the outline, the state layer and the icon are compared.
   - **Cost:**
     - a second toolchain in CI (Gradle and its plugins, a JDK) on a job that already runs; the added time is not
       measured, since nothing is built yet;
     - Kotlin in the repository, as a test fixture that nothing ships;
     - a second catalog to keep in step with the first, which the check enforces by failing on a component with no
       reference.
   - **Gives:** the same device, density and Android drawing on both sides. A difference is then the component's,
     not the host's.
2. **Paparazzi on the JVM.** The same reference app's composables are rendered by Paparazzi (Cash App), which draws
   Android views and Compose with Android Studio's layoutlib, with no emulator. They are compared with our emulator
   screenshots.
   - **Cost:** the same toolchain and Kotlin as option 1. layoutlib is not the device's drawing, so part of any
     difference is the renderer's. Which Paparazzi version supports this Compose version is not checked here.
   - **Gives:** a fast job with no emulator, and pictures that are the same on every run.
3. **No Compose reference.** Only the checks of our own pixels against the tokens, extended to phones and to the
   pressed and focused states.
   - **Cost:** the gap described above stays open.
   - **Gives:** nothing new to install.

Which version of `compose.material3` the reference uses, for options 1 and 2:

- **a.** A release, `1.5.0-rc01` now, moved by hand or by Dependabot once it is set up. A check reads the sources jar of
  that version and fails unless its token files and vendored components equal `reference/`. This is true of `1.5.0-rc01`
  today, save the differences listed above for `Button.kt` and `IconButton.kt`, which the check allows by name.
- **b.** Build `compose.material3` from androidx's sources at the pinned commit. That needs androidx's own build and
  prebuilts, which this CI does not have, and whether it runs on a GitHub runner is not checked.

## Decision

Proposed. **Recommendation: option 1 with version a.**

Comparing on one emulator leaves only the components' differences. The emulator is already there. The released
artifact carries our exact tokens today, and the sources-jar check keeps that true or says when it stops being true.

## Consequences

If accepted:

- **AGENTS.md rule 5** gets a second exception: "Kotlin in `tests/compose-reference/`, an Android app that draws M3E's
  own components for ADR 0019's comparison. Nothing the library or the app ships uses it." ADR 0001 is amended in the
  same way.
- **New:**
  - `tests/compose-reference/`: the Gradle project, its wrapper and the Kotlin app;
  - a Rust check beside `visual-checks` that compares the crops;
  - a step in `visual-phone.yml`'s Android job;
  - the sources-jar check.
  - The workflow change goes through whoever owns `visual-phone.yml` at the time.
- **Dependencies:**
  - Gradle, the Android Gradle plugin, Kotlin and `compose.material3` are pinned (ADR 0006) and declared once, in the
    project's version catalog.
  - Dependabot follows them once it is set up. A bump that changes a token fails the sources-jar check, until
    `sync-androidx.sh` moves the pin to match.
  - None of them is a Cargo dependency, and `cargo deny` does not see them.
- **Licensing:** `tests/**` is already MPL-2.0 in `REUSE.toml`. The Gradle wrapper jar needs its own entry, under
  Apache-2.0.
- **Each component pull request** adds its states to both catalogs. The comparison's tolerance is measured on the
  first run, on the emulator, and stated with its source as AGENTS.md rule 8 asks.
- **Not covered by this ADR:** motion. The comparison looks at settled states. The springs stay measured as
  ADR 0010 says.
