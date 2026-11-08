// The card's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test examples/catalog/card.test.tsx --target
// <linux|web>`, run from the root of the repository, saves them in `target/visual`; `cargo run -p visual-checks --
// target/visual` checks their known pixels. `nsh test` takes a test only inside its app, so this file is beside the
// catalog's `nsh.json`.
//
// The catalog shows the three styles of the card in a grid below the FABs' row, the last block of the page: in a row a
// card that is only a container, one that takes presses and one that is disabled. Each appearance gets one screenshot
// at rest and one with the mouse over the card that takes presses of each style, which lays the state layer over it and
// raises it (the outlined card does not rise, as androidx has it). The file names are those `visual-checks` reads
// (`card::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the cards' grid (`catalog::card::viewport`); the grid is below the rows the other test
// files open, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3488 };

const STYLES = ["filled", "elevated", "outlined"];

const KINDS = ["static", "clickable", "disabled"];

for (const scheme of ["light", "dark"] as const) {
  test(`the cards in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const style of STYLES) {
      for (const kind of KINDS) {
        expect(app.shows(`id:card-${style}-${kind}`)).toBe(true);
      }
    }
    await app.screenshot(`target/visual/card-${scheme}-rest.png`);
    for (const style of STYLES) {
      await app.hover(`id:card-${style}-clickable`);
      await app.screenshot(`target/visual/card-${scheme}-hover-${style}.png`);
    }
  });
}
