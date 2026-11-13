import {
  ENGINE_SAMPLE_FORMAT,
  SUPPORTED_SAMPLE_RATES,
} from "@/src/constants/Audio";
import type {
  AudioConfig,
  AudioDevice,
} from "@/src/services/engine/engine.interfaces";

/** Largest channel count a device offers in the engine format at `sampleRate`. */
export function channelCount(device?: AudioDevice, sampleRate = 48000): number {
  return Math.max(
    0,
    ...(device?.configs
      .filter(
        (config) =>
          config.sampleFormat.toLowerCase() === ENGINE_SAMPLE_FORMAT &&
          config.minSampleRate <= sampleRate &&
          config.maxSampleRate >= sampleRate,
      )
      .map((config) => config.channels) ?? []),
  );
}

/** Monitoring requires a connected KPT Pro input with a valid channel and a usable output. */
export function canMonitor(
  devices: readonly AudioDevice[],
  config: AudioConfig,
): boolean {
  const input = devices.find(
    (device) => device.direction === "input" && device.id === config.inputId,
  );
  const output = devices.find(
    (device) => device.direction === "output" && device.id === config.outputId,
  );

  return (
    !!input?.isKptPro &&
    config.inputChannel >= 0 &&
    Number.isInteger(config.inputChannel) &&
    channelCount(input, config.sampleRate) > config.inputChannel &&
    channelCount(output, config.sampleRate) > 0
  );
}

export function commonSampleRates(
  input?: AudioDevice,
  output?: AudioDevice,
): number[] {
  if (!input) return [];

  return SUPPORTED_SAMPLE_RATES.filter(
    (rate) =>
      channelCount(input, rate) > 0 &&
      (!output || channelCount(output, rate) > 0),
  );
}

export function deviceLabel(device: AudioDevice): string {
  return device.isKptPro && device.name.toLowerCase() !== "kpt pro"
    ? `KPT Pro · ${device.name}`
    : device.name;
}
