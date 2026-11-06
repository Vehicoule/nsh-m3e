// The button's screenshots on NSH's hosts (ADR 0013): `nsh test examples/catalog --target <host>`, run from the
// root of the repository, saves them in `target/visual`; `cargo run -p visual-checks -- target/visual` checks their
// known pixels. `nsh test` takes a test only inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every size, style, shape and availability of the button on a grid.
// Each appearance gets one screenshot at rest and one with the mouse over the small round enabled button of each
// style. The file names are those `visual-checks` reads (`Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport (`catalog::grid::viewport`); `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 928 };

const STYLES = ["filled", "tonal", "outlined", "text"];

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
  });
}
