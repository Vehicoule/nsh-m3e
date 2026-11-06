// The catalog on a phone (ADR 0013): `nsh test examples/catalog/phone.test.tsx --target <ios|android>`, run from the
// root of the repository, saves a screenshot of each appearance in `target/visual`. On iOS and Android the app fills
// the device's screen: `open` takes no size there, and a phone has no mouse to hover with.
import { expect, open, test } from "nsh/test";

// The first app on a freshly booted iOS simulator took longer than `nsh test`'s default 30 s on GitHub's macos-15
// runner (31.8 s, where the next one took 7.1 s). This bounds the launch, not anything the catalog draws.
const TIMEOUT_MS = 120_000;

for (const scheme of ["light", "dark"] as const) {
  test(`the catalog on a phone in ${scheme} appearance`, async () => {
    const app = await open(".", { colorScheme: scheme });
    // The first column, the filled buttons, starts within any phone's width.
    expect(app.texts()).toContain("Filled");
    await app.screenshot(`target/visual/phone-${scheme}-rest.png`);
  }, TIMEOUT_MS);
}
