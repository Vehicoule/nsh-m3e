// The radio buttons' screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/radio.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows the radio buttons in a block below its other grids: selected and unselected, enabled and disabled.
// Each appearance gets one screenshot at rest, and one with the mouse over each of the unselected enabled, the
// selected enabled and the unselected disabled radio button, which shows no state layer. The file names are those
// `visual-checks` reads (`radio::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The viewport the radio buttons' screenshots are taken in (`catalog::radio::viewport`): the buttons' (`button.test.tsx`)
// with the radio buttons' block and a margin below it. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 2792 };

const HOVERED = ["unselected-enabled", "selected-enabled", "unselected-disabled"];

for (const scheme of ["light", "dark"] as const) {
  test(`the radio buttons in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    // A radio button shows no text: the catalog's buttons are what says it has loaded.
    expect(app.texts()).toContain("Filled");
    await app.screenshot(`target/visual/radio-${scheme}-rest.png`);
    for (const radio of HOVERED) {
      await app.hover(`id:radio-${radio}`);
      await app.screenshot(`target/visual/radio-${scheme}-hover-${radio}.png`);
    }
  });
}
