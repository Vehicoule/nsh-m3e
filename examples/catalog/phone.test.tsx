// The catalog on a phone (ADR 0013): `nsh test examples/catalog/phone.test.tsx --target <ios|android>`, run from the
// root of the repository, saves a screenshot of each appearance in `target/visual`. On iOS and Android the app fills
// the device's screen: `open` takes no size there, and a phone has no mouse to hover with.
import { expect, open, test } from "nsh/test";

for (const scheme of ["light", "dark"] as const) {
  test(`the catalog on a phone in ${scheme} appearance`, async () => {
    const app = await open(".", { colorScheme: scheme });
    // The first column, the filled buttons, starts within any phone's width.
    expect(app.texts()).toContain("Filled");
    await app.screenshot(`target/visual/phone-${scheme}-rest.png`);
  });
}
