// The catalog's screenshots on NSH's desktop and web hosts (ADR 0013): `nsh test
// examples/catalog/button.test.tsx --target <linux|web>`, run from the root of the repository, saves them in
// `target/visual`; `cargo run -p visual-checks -- target/visual` checks their known pixels. `nsh test` takes a test only
// inside its app, so this file is beside the catalog's `nsh.json`.
//
// The catalog shows every size, style, shape and availability of the button on a grid, every size, width, style,
// shape and availability of the icon button on a second grid below it, the switches on a third (which
// `switch.test.tsx` hovers; they are in these screenshots at rest), every arrangement, icon position and label
// visibility of the navigation bar on a fourth, and every kind of row of tabs with a label, an icon and both on a fifth.
// Each appearance gets one screenshot at rest, one with the mouse over the small round enabled button of each style,
// one with the mouse over the small uniform round enabled icon button of each style, one with the mouse over each of
// four items of the navigation bars, and one with the mouse over each of four tabs. The file names are those
// `visual-checks` reads (`Shot::file_name`).
import { expect, open, test } from "nsh/test";

// The catalog's viewport (`catalog::grid::viewport`); `visual-checks` refuses a screenshot of another size.
const VIEWPORT = { width: 2608, height: 3072 };

const STYLES = ["filled", "tonal", "outlined", "text"];

const ICON_STYLES = ["filled", "tonal", "outlined", "standard"];

// The navigation bar items the mouse goes over, each as its bar's test id and its label (`visual-checks`'
// `hovered_items`): all three items of the first bar, and the unselected one of the bar with its icons at the start.
const NAVIGATION_ITEMS = [
  ["navigation-equal-top-shown", "Home"],
  ["navigation-equal-top-shown", "Search"],
  ["navigation-equal-top-shown", "Library"],
  ["navigation-equal-start-shown", "Search"],
] as const;

// The tabs the mouse goes over, each as its row's test id and its label (`visual-checks`' `hovered_tabs`): all three tabs
// of the first primary row with its icons alone, and the unselected one of the secondary row with its icons alone.
const TABS = [
  ["tabs-primary-icon", "Songs"],
  ["tabs-primary-icon", "Albums"],
  ["tabs-primary-icon", "Artists"],
  ["tabs-secondary-icon", "Albums"],
] as const;

for (const scheme of ["light", "dark"] as const) {
  test(`the buttons in ${scheme} appearance`, async () => {
    const app = await open(".", { ...VIEWPORT, colorScheme: scheme });
    for (const label of ["Filled", "Tonal", "Outlined", "Text", "Home", "Search", "Songs"]) {
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
    for (const [bar, item] of NAVIGATION_ITEMS) {
      await app.hover(`id:${bar} >> label:${item}`);
      await app.screenshot(
        `target/visual/${scheme}-hover-${bar}-${item.toLowerCase()}.png`,
      );
    }
    for (const [row, tab] of TABS) {
      await app.hover(`id:${row} >> label:${tab}`);
      await app.screenshot(
        `target/visual/${scheme}-hover-${row}-${tab.toLowerCase()}.png`,
      );
    }
  });
}
