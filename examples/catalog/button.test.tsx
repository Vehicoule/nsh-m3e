// The button's, icon button's and FAB's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/button.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every size, style, shape and availability of the button on a grid, and every size, width,
// style, shape and availability of the icon button on a second grid below it, and every size and style of the FAB in
// a row below that, to the right of the checkboxes' grid (`checkbox.test.tsx`).
// Each appearance gets one screenshot at rest, one with the mouse over the small round enabled button of each style,
// one with the mouse over the small uniform round enabled icon button of each style, and one with the mouse over the
// baseline FAB of each style. The file names are those `visual-checks` reads (`Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport (`catalog::grid::viewport`); `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 2800 };

const STYLES = ["filled", "tonal", "outlined", "text"];

const ICON_STYLES = ["filled", "tonal", "outlined", "standard"];

const FAB_STYLES = ["primary-container", "secondary-container"];

for (const scheme of ["light", "dark"] as const) {
  test(`the buttons in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const label of ["Filled", "Tonal", "Outlined", "Text"]) {
      expect(app.texts()).toContain(label);
    }
    await app.screenshot(`target/visual/${scheme}-rest.png`);
    for (const style of STYLES) {
      await app.hover(`id:${style}-small-round-enabled`);
      await app.screenshot(`target/visual/${scheme}-hover-${style}.png`);
    }
    for (const style of ICON_STYLES) {
      await app.hover(`id:icon-${style}-small-uniform-round-enabled`);
      await app.screenshot(`target/visual/${scheme}-hover-icon-${style}.png`);
    }
    for (const style of FAB_STYLES) {
      await app.hover(`id:fab-${style}-baseline`);
      await app.screenshot(`target/visual/${scheme}-hover-fab-${style}.png`);
    }
  });
}
