import type {
  AudioConfig,
  EngineStatus,
} from "@/src/services/engine/engine.interfaces";

export const defaultAudioConfig: AudioConfig = {
  bufferFrames: 128,
  inputChannel: 0,
  inputId: "",
  outputId: "",
  sampleRate: 44100,
};

export const idleStatus: EngineStatus = {
  deviceError: false,
  durationFrames: 0,
  error: null,
  inputPeak: 0,
  looping: false,
  monitoring: false,
  outputPeak: 0,
  overruns: 0,
  playing: false,
  positionFrames: 0,
  recording: false,
  running: false,
  sessionRevision: 0,
  underruns: 0,
};
