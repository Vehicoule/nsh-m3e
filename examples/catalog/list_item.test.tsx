// The list item's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/list_item.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every kind and number of lines of the list item in a grid below the FABs' row, the last block of
// the page. Each appearance gets one screenshot at rest and one with the mouse over the two-line clickable list item
// and the two-line selected one. The file names are those `visual-checks` reads (`list_item::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the list items' grid (`catalog::list_item::viewport`); the grid is below the rows the
// other test files open, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3320 };

const KINDS = ["clickable", "selected"];

const ALL_KINDS = ["plain", "clickable", "selected", "disabled"];

const LINES = ["one", "two", "three"];

for (const scheme of ["light", "dark"] as const) {
  test(`the list items in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const kind of ALL_KINDS) {
      for (const lines of LINES) {
        expect(app.shows(`id:list-${lines}-${kind}`)).toBe(true);
      }
    }
    await app.screenshot(`target/visual/list-${scheme}-rest.png`);
    for (const kind of KINDS) {
      await app.hover(`id:list-two-${kind}`);
      await app.screenshot(`target/visual/list-${scheme}-hover-${kind}.png`);
    }
  });
}
