import { describe, expect, it } from 'vitest';
import {
  channelCount,
  canMonitor,
  commonSampleRates,
  type AudioDevice,
} from './audio';
describe('48 kHz input eligibility', () => {
  it('excludes incompatible formats and rates from channel selection', () => {
    const device: AudioDevice = {
      id: 'test',
      name: 'Interface',
      direction: 'input',
      isKptPro: false,
      configs: [
        {
          channels: 8,
          minSampleRate: 44100,
          maxSampleRate: 44100,
          sampleFormat: 'f32',
        },
        {
          channels: 4,
          minSampleRate: 48000,
          maxSampleRate: 96000,
          sampleFormat: 'i16',
        },
        {
          channels: 2,
          minSampleRate: 44100,
          maxSampleRate: 96000,
          sampleFormat: 'f32',
        },
      ],
    };
    expect(channelCount(device)).toBe(2);
  });
  it('does not expose phantom channels when a device is absent', () => {
    expect(channelCount()).toBe(0);
  });
});

describe('KPT Pro monitoring requirement', () => {
  const config = {
    inputId: 'input',
    outputId: 'output',
    inputChannel: 0,
    sampleRate: 48000,
    bufferFrames: 128,
  };
  const input: AudioDevice = {
    id: 'input',
    name: 'KPT Pro',
    direction: 'input',
    isKptPro: true,
    configs: [
      {
        channels: 2,
        minSampleRate: 48000,
        maxSampleRate: 48000,
        sampleFormat: 'f32',
      },
    ],
  };
  const output: AudioDevice = {
    ...input,
    id: 'output',
    name: 'Headphones',
    direction: 'output',
    isKptPro: false,
  };
  it('requires the selected input itself to be KPT Pro', () => {
    expect(canMonitor([input, output], config)).toBe(true);
    expect(canMonitor([{ ...input, isKptPro: false }, output], config)).toBe(
      false,
    );
    expect(
      canMonitor(
        [
          { ...input, id: 'another-kpt' },
          { ...input, name: 'Built-in microphone', isKptPro: false },
          output,
        ],
        config,
      ),
    ).toBe(false);
  });
  it('disables monitoring when KPT Pro disconnects or a channel is invalid', () => {
    expect(canMonitor([output], config)).toBe(false);
    expect(canMonitor([input, output], { ...config, inputChannel: 2 })).toBe(
      false,
    );
    expect(canMonitor([input, output], { ...config, inputChannel: -1 })).toBe(
      false,
    );
  });
});

describe('Sinco-Audio 44.1 kHz compatibility', () => {
  const input: AudioDevice = {
    id: 'sinco-in',
    name: 'Sinco-Audio',
    direction: 'input',
    isKptPro: true,
    configs: [
      {
        channels: 2,
        minSampleRate: 44100,
        maxSampleRate: 44100,
        sampleFormat: 'f32',
      },
    ],
  };
  const output: AudioDevice = {
    ...input,
    id: 'out',
    direction: 'output',
    configs: [
      {
        channels: 2,
        minSampleRate: 44100,
        maxSampleRate: 96000,
        sampleFormat: 'f32',
      },
    ],
  };
  it('negotiates 44.1 kHz with a wider-rate output', () => {
    expect(commonSampleRates(input, output)).toEqual([44100]);
    expect(channelCount(input, 44100)).toBe(2);
    expect(
      canMonitor([input, output], {
        inputId: input.id,
        outputId: output.id,
        inputChannel: 1,
        sampleRate: 44100,
        bufferFrames: 128,
      }),
    ).toBe(true);
  });
  it('rejects mismatched selected rates and incompatible outputs', () => {
    expect(
      canMonitor([input, output], {
        inputId: input.id,
        outputId: output.id,
        inputChannel: 0,
        sampleRate: 48000,
        bufferFrames: 128,
      }),
    ).toBe(false);
    expect(
      commonSampleRates(input, {
        ...output,
        configs: [
          {
            channels: 2,
            minSampleRate: 48000,
            maxSampleRate: 48000,
            sampleFormat: 'f32',
          },
        ],
      }),
    ).toEqual([]);
  });
});

import { statusError, idleStatus } from './audio';
describe('audio fault messages', () => {
  it('preserves the specific recording error over a generic device message', () => {
    expect(
      statusError(
        { ...idleStatus, deviceError: true, error: 'Disk full; take retained' },
        false,
      ),
    ).toBe('Disk full; take retained');
  });
  it('does not reassert a dismissed device error on every poll', () => {
    expect(statusError({ ...idleStatus, deviceError: true }, true)).toBeNull();
    expect(statusError({ ...idleStatus, deviceError: true }, false)).toContain(
      'stopped responding',
    );
    expect(statusError({ ...idleStatus, overruns: 1024 }, false)).toBeNull();
  });
});
