import { create } from "zustand";
import { persist } from "zustand/middleware";

import { defaultAudioConfig } from "@/src/services/engine/engine.fallback";
import type { AudioConfig } from "@/src/services/engine/engine.interfaces";
import { clamp } from "@/src/utils/clamp";
import { asRecord, readFiniteNumber } from "@/src/utils/records";

const STORAGE_KEY = "kuerdas-studio-v1";
const STORAGE_VERSION = 3;

export interface StudioPreferences {
  /** Tone-library state kept from older versions so nothing a user saved is lost. */
  archivedToneLibrary: unknown;
  config: AudioConfig;
  /** Live input level, 0–1. */
  inputTrim: number;
}

interface StudioStore extends StudioPreferences {
  setConfig: (patch: Partial<AudioConfig>) => void;
  setInputTrim: (value: number) => void;
}

/** Upgrades persisted preferences from any earlier storage version. */
export function migrateStudioPreferences(value: unknown): StudioPreferences {
  const old = asRecord(value);
  const tone = asRecord(old.tone);
  const storedTrim =
    readFiniteNumber(old, "inputTrim") ?? readFiniteNumber(tone, "output");

  return {
    archivedToneLibrary:
      old.archivedToneLibrary ??
      (old.tone || old.savedTones
        ? {
            patchReferences: old.patchReferences,
            preset: old.preset,
            savedTones: old.savedTones,
            tone: old.tone,
          }
        : null),
    config: {
      ...defaultAudioConfig,
      ...(asRecord(old.config) as Partial<AudioConfig>),
    },
    inputTrim: storedTrim === undefined ? 1 : clamp(storedTrim, 0, 1),
  };
}

export const useStudio = create<StudioStore>()(
  persist(
    (set) => ({
      archivedToneLibrary: null,
      config: defaultAudioConfig,
      inputTrim: 1,
      setConfig: (patch) =>
        set((state) => ({ config: { ...state.config, ...patch } })),
      setInputTrim: (inputTrim) => set({ inputTrim: clamp(inputTrim, 0, 1) }),
    }),
    {
      migrate: migrateStudioPreferences,
      name: STORAGE_KEY,
      version: STORAGE_VERSION,
    },
  ),
);
