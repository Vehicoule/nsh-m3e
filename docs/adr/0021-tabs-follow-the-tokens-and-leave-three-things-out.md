# 0021. Tabs: the tokens for unselected tabs, and three things left out of the first pull request

- Status: Proposed
- Date: 2026-11-07
- Deciders: the user

## Context

The tabs follow androidx's `Tab`, `LeadingIconTab`, `PrimaryTabRow` and `SecondaryTabRow` ([ADR 0012](0012-follow-androidx-components-where-tokens-are-wrong.md)),
vendored as `Tab.kt`, `TabRow.kt` and `Divider.kt` beside their tokens (`PrimaryNavigationTabTokens`,
`SecondaryNavigationTabTokens`, `DividerTokens`). Reading them shows five places where the component does not do what
the tokens or Material's specification say, and three things this library cannot do yet. All facts are at the pinned
androidx commit `a8b393d51f9b`; `Tab.kt` and `TabRow.kt` are in
`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/`.

| What | The token files say | The component does |
|---|---|---|
| Color of an unselected tab's label and icon | `InactiveLabelTextColor`, `InactiveIconColor` = `OnSurfaceVariant`, with `OnSurface` for hover, focus and press | `unselectedContentColor: Color = selectedContentColor` (`Tab.kt`, lines 99, 160 and 232). No `Inactive*` token is read in either file. Every tab is in the row's content color: `Primary` for the primary row, `OnSurface` for the secondary one (`TabRow.kt`, lines 1016-1021) |
| Color of the divider of the secondary row | `SecondaryNavigationTabTokens.DividerColor` = `SurfaceVariant` | `HorizontalDivider()` (`TabRow.kt`, lines 163 and 214), which is `DividerTokens.Color` = `OutlineVariant` (`Divider.kt`, lines 111-119) |
| A disabled tab | No color for it in any token file | Nothing: `enabled` only stops presses, though the documentation says the tab "will appear visually disabled" (`Tab.kt`, lines 76-78) |
| Where the primary indicator sits under its tab | Not a token | In the fixed row at the tab's left edge: the offset is the tab's `left`, and the indicator is only as wide as the content (`TabRow.kt`, lines 721 and 751). In the scrollable row, centered (lines 637-639), as Material's specification draws it |
| The fade of a tab's content color | Not a token | The color animates with `DefaultEffects` when the tab is selected and `FastEffects` when it is not (`Tab.kt`, lines 267-290) |

The three things this library cannot do yet:

- **The tab with its icon above its label.** androidx places it by the text's baselines: the label sits 14 points above
  the bottom (6 with two lines), the icon 20 sp above the first baseline, in a tab 72 points tall (`Tab.kt`, lines
  298-442). NSH's layout does not expose a text's baseline for placing another view.
- **The scrollable rows.** They need a scrolling row that brings the selected tab into view with the spatial spring,
  an edge padding of 52, a minimum tab width of 90 and a different indicator layout (`TabRow.kt`, lines 255-345 and
  512-650). It is a pull request of its own.
- **Fading the content color.** The label's color can be animated, but the icon is a canvas filled with its view's `fill`
  style, and the pinned NSH has no verified way to animate that fill: only a text's color and a view's background are
  animatable props. A label that fades next to an icon that does not is worse than neither fading.

## Options

For the unselected color, which is the one that changes how every row looks:

1. **The tokens: selected in the active color, unselected in `OnSurfaceVariant`** (recommended).
   - A primary row shows its selected tab in `Primary` and the others in the secondary gray, as Material's specification
     and its Figma kit show.
   - The state layer stays in the row's content color, as androidx's ripple is (`ripple(color = selectedContentColor)`).
   - Cost: one place where this library differs from what an androidx app shows today. androidx's own comment on the
     default (`unselectedContentColor: Color = selectedContentColor`) is that it is not opinionated, not that the
     tokens are wrong, which is why ADR 0012 does not decide it.
2. **As androidx.** Every tab in the row's content color. A primary row has every label in `Primary`, and only the
   indicator says which tab is selected. It is what an androidx app that passes no colors shows.
3. **The tokens in every state.** Option 1, and the hover, focus and press label and icon colors of the tokens
   (`OnSurface` for an unselected tab) as well. It is the most Material, and adds a color change under the pointer,
   which no component of this library has yet (the state layers only change an opacity), and so needs pixel checks of a
   second kind.

## Decision

Pending the user. The pull request that adds the tabs is written for option 1; the other two change `Kind::icon`,
`Kind::label` and their tests only.

The other four rows of the table are taken in the pull request, and the user can reverse each in review:

- **The divider is `DividerTokens.Color`, for both rows**, as ADR 0012's first option says (the component reads another
  token file).
- **A disabled tab dims at `DisabledAlpha`** (0.38, as every navigation item of this library does), because the
  documentation says it should, and a disabled tab that looks like the others fails screen-reader users who do not use
  one.
- **The primary indicator is centered under the content**, as the scrollable row of androidx does and Material draws it.
  The fixed row's left-aligned indicator is, at this commit, either a bug or a layout this commit's source does not show;
  it is a deviation from the vendored code, pinned by a test that reads the lines it deviates from.
- **The three things above are left out**, each named in the module's documentation.

## Consequences

- **Option 1 or 3:** `PrimaryNavigationTabTokens.Inactive*` and `SecondaryNavigationTabTokens.Inactive*` are read, and the
  test that pins the claim "androidx never reads them" is the record of why this is a decision and not a citation.
- **Option 2:** the tokens' inactive colors are not read, and the tests expect the row's content color for every tab.
- **The stacked tab**, **the scrollable rows** and **the fade** each come back as their own pull request, when NSH can
  place a view by a text's baseline, when the navigation family is done, and when a canvas's fill can be animated.
- **The catalog and the visual checks** show both rows with a label, an icon and both, each with a selected, an
  unselected and a disabled tab, so that every color above is read from pixels on the hosts.
