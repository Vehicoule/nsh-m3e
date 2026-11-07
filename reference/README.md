# Vendored references

Sources of truth copied into the repository so every agent reads the same pinned version. Never edit them by hand:
update by copying a newer revision in a pull request of its own.

The androidx tokens, and the androidx components this library follows where they override a token (ADR 0012), are
copied by `scripts/sync-androidx.sh`, which reads the pinned commit from the table below. CI runs
`scripts/sync-androidx.sh check` on every pull request, and `.github/workflows/androidx-bump.yml` opens a pull request
each week when those files on `androidx-main` have changed.

The Material Symbols the library ships (ADR 0017) are copied by `scripts/sync-material-symbols.sh`, which reads the
pinned commit from the table below; CI runs `scripts/sync-material-symbols.sh check` on every pull request. Each style
folder holds every icon at 24 px, in the default grade and weight: `<name>_24px.svg` and, filled,
`<name>_fill1_24px.svg`.

| Folder | Source | Revision | License |
|---|---|---|---|
| `androidx-material3-tokens/` | https://github.com/androidx/androidx, `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/` | `a8b393d51f9bdcfcac8f0a4a0992ecad1b27ba55` | Apache-2.0 (`LICENSE-APACHE` is androidx's `LICENSE.txt`; see also each file's header) |
| `androidx-material3-components/` | https://github.com/androidx/androidx, `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/` (`Button.kt`, `IconButton.kt`, `IconButtonDefaults.kt`) | the same commit as `androidx-material3-tokens/` | Apache-2.0 (`LICENSE-APACHE` is androidx's `LICENSE.txt`; see also each file's header) |
| `material-symbols/` | https://github.com/google/material-design-icons, `symbols/web/<name>/materialsymbols{outlined,rounded,sharp}/` | `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` | Apache-2.0 (`LICENSE-APACHE` is the repository's `LICENSE`) |
| `rust/google-rust-style-guide.md` | https://google.github.io/styleguide/rustguide.html, converted to Markdown with pandoc on 2026-11-06 | page as of 2026-11-06 | CC BY 3.0, © Google |
