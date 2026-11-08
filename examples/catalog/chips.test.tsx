// The chips' screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test examples/catalog/chips.test.tsx
// --target <linux|web>`, run from the root of the repository, saves them in `target/visual`; `cargo run -p
// visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only inside its app, so this file
// is beside the catalog's `nsh.json`.
//
// The catalog shows the chips in a block below its other grids: each kind unselected and, for the filter and input
// chips, selected, enabled and disabled, with its icons and its avatar. Each appearance gets one screenshot at rest,
// and one with the mouse over the enabled chip that shows only its label, for each kind and for each selection of the
// filter and input chips. The file names are those `visual-checks` reads (`chips::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The viewport the chips' screenshots are taken in (`catalog::chips::viewport`): the buttons' (`button.test.tsx`) with
// the chips' block and a margin below it. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3272 };

// The test id of the chip the mouse is over, and the end of the file name: the kind, then `-selected` for a selected
// chip (`catalog::chips::ChipCell::test_id`).
const HOVERED = [
  { id: "chip-assist-unselected-label-enabled", name: "assist" },
  { id: "chip-filter-unselected-label-enabled", name: "filter" },
  { id: "chip-filter-selected-label-enabled", name: "filter-selected" },
  { id: "chip-input-unselected-label-enabled", name: "input" },
  { id: "chip-input-selected-label-enabled", name: "input-selected" },
  { id: "chip-suggestion-unselected-label-enabled", name: "suggestion" },
];

for (const scheme of ["light", "dark"] as const) {
  test(`the chips in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const label of ["Assist", "Filter", "Input", "Suggestion"]) {
      expect(app.texts()).toContain(label);
    }
    await app.screenshot(`target/visual/chips-${scheme}-rest.png`);
    for (const { id, name } of HOVERED) {
      await app.hover(`id:${id}`);
      await app.screenshot(`target/visual/chips-${scheme}-hover-${name}.png`);
    }
  });
}
