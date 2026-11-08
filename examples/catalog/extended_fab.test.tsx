// The extended floating action button's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/extended_fab.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows the extended FABs in a grid below the FABs' row, the last block of the page: a row per size, and in it
// each style's FAB expanded, collapsed to its icon and with no icon. Each appearance gets one screenshot at rest and one
// with the mouse over the medium expanded FAB of each style, which raises it to the hovered elevation (ADR 0018). The
// file names are those `visual-checks` reads (`extended_fab::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the extended FABs' grid (`catalog::extended_fab::viewport`); the grid is below the blocks the
// other test files open, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3384 };

const STYLES = ["primary-container", "secondary-container"];

const SIZES = ["small", "medium", "large"];

const KINDS = ["expanded", "collapsed", "no-icon"];

for (const scheme of ["light", "dark"] as const) {
  test(`the extended FABs in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const style of STYLES) {
      for (const size of SIZES) {
        for (const kind of KINDS) {
          expect(app.shows(`id:extended-fab-${style}-${size}-${kind}`)).toBe(true);
        }
      }
    }
    await app.screenshot(`target/visual/extended-fab-${scheme}-rest.png`);
    for (const style of STYLES) {
      await app.hover(`id:extended-fab-${style}-medium-expanded`);
      await app.screenshot(`target/visual/extended-fab-${scheme}-hover-${style}.png`);
    }
  });
}
