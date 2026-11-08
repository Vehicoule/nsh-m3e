// The floating action button's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/fab.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every size and style of the FAB in a row below the checkboxes' grid, the last block of the page.
// Each appearance gets one screenshot at rest and one with the mouse over the baseline FAB of each style, which raises
// it to the hovered elevation (ADR 0018). The file names are those `visual-checks` reads (`fab::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the FABs' row (`catalog::fab::viewport`); the row is below the grids the other test
// files open, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3008 };

const STYLES = ["primary-container", "secondary-container"];

const SIZES = ["small", "baseline", "medium", "large"];

for (const scheme of ["light", "dark"] as const) {
  test(`the FABs in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const style of STYLES) {
      for (const size of SIZES) {
        expect(app.shows(`id:fab-${style}-${size}`)).toBe(true);
      }
    }
    await app.screenshot(`target/visual/fab-${scheme}-rest.png`);
    for (const style of STYLES) {
      await app.hover(`id:fab-${style}-baseline`);
      await app.screenshot(`target/visual/fab-${scheme}-hover-${style}.png`);
    }
  });
}
