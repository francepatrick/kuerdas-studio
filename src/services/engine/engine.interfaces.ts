export type DeviceDirection = "input" | "output";

export interface AudioDeviceConfig {
  channels: number;
  maxSampleRate: number;
  minSampleRate: number;
  sampleFormat: string;
}

export interface AudioDevice {
  configs: AudioDeviceConfig[];
  direction: DeviceDirection;
  id: string;
  isKptPro: boolean;
  name: string;
}

export interface AudioConfig {
  bufferFrames: number;
  inputChannel: number;
  inputId: string;
  outputId: string;
  sampleRate: number;
}

export interface EngineStatus {
  deviceError: boolean;
  durationFrames: number;
  error: string | null;
  inputPeak: number;
  looping: boolean;
  monitoring: boolean;
  outputPeak: number;
  overruns: number;
  playing: boolean;
  positionFrames: number;
  recording: boolean;
  running: boolean;
  sessionRevision: number;
  underruns: number;
}
