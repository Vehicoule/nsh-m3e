// The checkbox's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test examples/catalog/checkbox.test.tsx
// --target <linux|web>`, run from the root of the repository, saves them in `target/visual`; `cargo run -p
// visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only inside its app, so this
// file is beside the catalog's `nsh.json`.
//
// The catalog shows every state of the checkbox, enabled and disabled, on a small grid below the buttons' and the
// icon buttons'. Each appearance gets one screenshot at rest and one with the mouse over each checkbox. The file
// names are those `visual-checks` reads (`checkbox::Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport with the checkboxes' grid (`catalog::checkbox::viewport`); the checkboxes' grid is below
// the one `button.test.tsx` opens, so this viewport is taller. `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3088 };

const STATES = ["unchecked", "checked", "indeterminate"];

const AVAILABILITIES = ["enabled", "disabled"];

for (const scheme of ["light", "dark"] as const) {
  test(`the checkboxes in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const state of STATES) {
      for (const availability of AVAILABILITIES) {
        expect(app.shows(`id:checkbox-${state}-${availability}`)).toBe(true);
      }
    }
    await app.screenshot(`target/visual/checkbox-${scheme}-rest.png`);
    for (const availability of AVAILABILITIES) {
      for (const state of STATES) {
        await app.hover(`id:checkbox-${state}-${availability}`);
        await app.screenshot(`target/visual/checkbox-${scheme}-hover-${state}-${availability}.png`);
      }
    }
  });
}
