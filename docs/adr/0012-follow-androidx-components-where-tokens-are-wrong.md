# 0012. Where androidx's component and its token files disagree, follow the component

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

[ADR 0003](0003-tokens-from-androidx.md) takes every M3 value from the androidx token files, and AGENTS.md rule 3
says to "take every M3 value from the vendored tokens". Building the button shows that androidx's own component does
not always read those files. Sometimes it reads another file, and sometimes it writes a value of its own, with a
comment saying that the token is wrong.

All facts below are at the pinned androidx commit `a8b393d51f9b`. `Button.kt` is
`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt`. The token files are those
vendored in `reference/androidx-material3-tokens/`.

| What | The token file says | `Button.kt` uses |
|---|---|---|
| Text button label color | `TextButtonTokens.LabelColor` = `OnSurfaceVariant` (v0_11_0) | `ColorSchemeKeyTokens.Primary`, under "TODO replace with the token value once it's corrected" (line 1423) |
| Extra small button side padding | `ButtonXSmallTokens.LeadingSpace` / `TrailingSpace` = 16 | 12, under "TODO update with the value from ButtonXSmallTokens.kt once it's been corrected" (line 972) |
| Extra small icon to label space | `ButtonXSmallTokens.IconLabelSpace` | 4, under "TODO use the value from ButtonXSmallTokens.kt once it's been corrected" (line 1117) |
| Tonal button colors | `TonalButtonTokens` (v0_11_0): disabled container opacity 0.1, disabled label `OnSurfaceVariant` | `FilledTonalButtonTokens` (v0_103): disabled container opacity 0.12, disabled label `OnSurface` |
| Outlined button outline when disabled | `OutlinedButtonTokens.DisabledOutlineColor` = `OutlineVariant`, no opacity | `OutlineColor` at `DisabledContainerOpacity` (0.1) |

- **Tonal tokens.** In androidx's Material 3 code, only the toggle button (`ToggleButton.kt`) reads
  `TonalButtonTokens`. The button reads `FilledTonalButtonTokens`.
- **The extra small paddings.** With them, an extra small button with one line of label is 6 + 20 + 6 = 32 points
  tall, its token height.
- **The other styles agree.** The filled, elevated and outlined buttons read the v0_11_0 file of their own name.

Material's specification pages are not vendored and cannot be checked in CI, so they cannot settle this. The Figma
kit is only a visual reference.

## Options

1. **Follow the androidx component.**
   - Where `Button.kt` (or, later, another component's file) reads a token, read that token. Where it writes a value
     under a comment saying the token is wrong, use that value and cite the line.
   - The component's Kotlin files are vendored beside the tokens, at the same commit, and checked by
     `scripts/sync-androidx.sh`, so that every value still has a pinned, checkable source.
   - It gives what Android apps built with androidx show, and follows Google's own corrections.
   - Its cost: a few values are literals with a citation instead of generated tokens. Each one is listed in its
     component's module, and a test pins it to the cited value.
   - When androidx corrects a token, the daily bump moves the vendored component and the test fails, so the literal
     is replaced.
2. **Follow the token files only.**
   - It keeps rule 3 as written and adds nothing to vendor.
   - It reproduces the errors androidx flags: a text button label in `OnSurfaceVariant` instead of `Primary`, and an
     extra small button with 16-point sides.
   - The tonal button gets 0.1 and `OnSurfaceVariant` when disabled, values no androidx component uses.
3. **Decide case by case.** Each disagreement goes to the user as it comes. It is the most control, at the cost of a
   question per component.

## Decision

**Option 1**, chosen by the user on 2026-11-06. AGENTS.md rule 3 now reads: "Take every M3 value from the vendored
tokens, or from the vendored androidx component where it overrides a token, and cite it."

## Consequences

- **If option 1 is chosen:**
  - **Vendoring.** `scripts/sync-androidx.sh` vendors each component file a component of this library follows,
    starting with `Button.kt`, into `reference/androidx-material3-components/`, at the same pinned commit.
  - **Generated tokens.** `FilledTonalButtonTokens.kt` is added to the generator's `SOURCES` for the button.
    `tonal_button_tokens` stays, for the toggle button.
  - **Citing.** Every overriding value cites its file and line, and its test pins the value.
  - **Rule 3.** AGENTS.md rule 3 is amended in the same pull request as this decision.
- **If option 2 is chosen:** the button uses the token files as they are, and the disagreements above are listed in
  its module's documentation.
- **In either case,** a later component that meets a disagreement of a new kind (not a token override) comes back to
  the user.
