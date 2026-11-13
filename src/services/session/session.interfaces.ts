export type TrackKind = "take" | "backing" | "audio" | "combined" | "processed";

export interface Clip {
  channels: number;
  file: string;
  frames: number;
  id: string;
  name: string;
  sourceOffset: number;
  startFrame: number;
  tone: string;
}

export interface Track {
  channels: number;
  clips: Clip[];
  file: string;
  frames: number;
  gain: number;
  id: string;
  kind: TrackKind;
  mute: boolean;
  name: string;
  pan: number;
  solo: boolean;
  startFrame: number;
  tone: string;
  waveform: number[];
  waveforms: Record<string, number[]>;
}

export interface SessionState {
  sampleRate: number;
  storagePath: string;
  tracks: Track[];
}

/** Offline drive/EQ applied to a copy of a recorded track. */
export interface RecordedEffects {
  bass: number;
  drive: number;
  level: number;
  treble: number;
}
