import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AudioConfig } from '../lib/audio';
export const defaultAudioConfig: AudioConfig = {
  inputId: '',
  outputId: '',
  inputChannel: 0,
  sampleRate: 44100,
  bufferFrames: 128,
};
export function migrateStudioPreferences(value: unknown) {
  const old = (value && typeof value === 'object' ? value : {}) as Record<
    string,
    unknown
  >;
  const tone = old.tone as { output?: number } | undefined;
  const storedTrim =
    typeof old.inputTrim === 'number' ? old.inputTrim : tone?.output;
  const inputTrim =
    typeof storedTrim === 'number' && Number.isFinite(storedTrim)
      ? Math.max(0, Math.min(1, storedTrim))
      : 1;
  return {
    config: {
      ...defaultAudioConfig,
      ...((old.config as Partial<AudioConfig>) ?? {}),
    },
    inputTrim,
    archivedToneLibrary:
      old.archivedToneLibrary ??
      (old.tone || old.savedTones
        ? {
            tone: old.tone,
            preset: old.preset,
            savedTones: old.savedTones,
            patchReferences: old.patchReferences,
          }
        : null),
  };
}
interface Studio {
  config: AudioConfig;
  inputTrim: number;
  archivedToneLibrary: unknown;
  setConfig: (patch: Partial<AudioConfig>) => void;
  setInputTrim: (value: number) => void;
}
export const useStudio = create<Studio>()(
  persist(
    (set) => ({
      config: defaultAudioConfig,
      inputTrim: 1,
      archivedToneLibrary: null,
      setConfig: (patch) => set((s) => ({ config: { ...s.config, ...patch } })),
      setInputTrim: (inputTrim) =>
        set({ inputTrim: Math.max(0, Math.min(1, inputTrim)) }),
    }),
    {
      name: 'kuerdas-studio-v1',
      version: 3,
      migrate: migrateStudioPreferences,
    },
  ),
);
