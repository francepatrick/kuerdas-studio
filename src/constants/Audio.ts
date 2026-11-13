/** Sample rates the engine negotiates, in preference order. */
export const SUPPORTED_SAMPLE_RATES = [48000, 44100] as const;

export const DEFAULT_SAMPLE_RATE = 48000;

/** CPAL sample format the engine streams in. */
export const ENGINE_SAMPLE_FORMAT = "f32";
