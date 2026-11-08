// The text field's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/text_field.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every style, content and status of the text field in two rows below the FABs' row. Each
// appearance gets one screenshot at rest, then one with the focus in the plain enabled and in the rich error text
// field of each style, the focus moving from one field to the next. The file names are those `visual-checks` reads
// (`text_field::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the text fields' rows (`catalog::text_field::viewport`); the rows are below the grids
// the other test files open, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3256 };

const STYLES = ["filled", "outlined"];

const CONTENTS = ["plain", "rich"];

const STATUSES = ["enabled", "error", "disabled"];

// The text fields that take the focus, by what `catalog::text_field::Cell::name` says after the style.
const FOCUSED_FIELDS = ["plain-enabled", "rich-error"];

// How long the springs of the label and of the indicator or outline get to settle after the focus moves: they settle in
// a few tenths of a second (`SpringToken::FastSpatial`, `FastEffects`, `SlowEffects`). Not a design value: how long
// the script waits before the screenshot.
const SETTLE_MS = 1000;

for (const scheme of ["light", "dark"] as const) {
  test(`the text fields in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const style of STYLES) {
      for (const content of CONTENTS) {
        for (const status of STATUSES) {
          expect(app.shows(`id:text-field-${style}-${content}-${status}`)).toBe(true);
        }
      }
    }
    await app.screenshot(`target/visual/text-field-${scheme}-rest.png`);
    for (const style of STYLES) {
      for (const field of FOCUSED_FIELDS) {
        // Typing nothing takes the focus.
        await app.type(`id:text-field-${style}-${field}`, "");
        await app.wait(SETTLE_MS);
        await app.screenshot(`target/visual/text-field-${scheme}-focus-${style}-${field}.png`);
      }
    }
  });
}
