import { expect, it } from "vitest";
import { migrateStudioPreferences } from "@/src/stores/studio.store";
it("preserves audio devices and input level when removing the tone-library UI", () => {
  const migrated = migrateStudioPreferences({
    config: { inputId: "kpt-input", outputId: "headphones", sampleRate: 44100 },
    tone: { output: 0.67, drive: 4 },
    preset: "My saved patch",
    savedTones: [{ name: "My tone" }],
    patchReferences: { "My tone": "Bank 2 / B" },
  });
  expect(migrated.config.inputId).toBe("kpt-input");
  expect(migrated.inputTrim).toBe(0.67);
  expect(migrated.archivedToneLibrary).toMatchObject({
    savedTones: [{ name: "My tone" }],
  });
});
it("retains an existing hidden archive and validates input trim", () => {
  expect(
    migrateStudioPreferences({
      inputTrim: 9,
      archivedToneLibrary: { preserved: true },
    }),
  ).toMatchObject({ inputTrim: 1, archivedToneLibrary: { preserved: true } });
  expect(migrateStudioPreferences(null).inputTrim).toBe(1);
});
