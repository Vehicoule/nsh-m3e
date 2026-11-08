// The switch's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test examples/catalog/switch.test.tsx
// --target <linux|web>`, run from the root of the repository, saves them in `target/visual`; `cargo run -p
// visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only inside its app, so this file is
// beside the catalog's `nsh.json`.
//
// The catalog shows every value, handle and availability of the switch on a grid below the icon buttons'. The switches
// at rest are in the screenshots of `button.test.tsx` (the catalog is one page); this takes one screenshot with the mouse
// over the off switch with a plain handle and one with the mouse over the on switch with an icon, in each appearance,
// for the state layer that hover lays over the handle. The ids and the file names are those `visual-checks` reads
// (`Shot::file_name`, `switch::HOVERED`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport (`catalog::grid::viewport`); `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3072 };

const HOVERED = ["switch-off-plain-enabled", "switch-on-icon-enabled"];

for (const scheme of ["light", "dark"] as const) {
  test(`the switches in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const id of HOVERED) {
      // A switch shows no text to wait for: its view is there once the catalog is.
      expect(app.count(`id:${id}`)).toBe(1);
      // The switch itself, in the view with the id: the mouse comes over the middle of its track.
      await app.hover(`id:${id} >> role:switch`);
      await app.screenshot(`target/visual/${scheme}-hover-${id}.png`);
    }
  });
}
