import { invoke, isTauri } from '@tauri-apps/api/core';
export const desktop = isTauri();
export interface AudioDevice {
  id: string;
  name: string;
  direction: 'input' | 'output';
  isKptPro: boolean;
  configs: {
    channels: number;
    minSampleRate: number;
    maxSampleRate: number;
    sampleFormat: string;
  }[];
}
export interface EngineStatus {
  running: boolean;
  inputPeak: number;
  outputPeak: number;
  underruns: number;
  overruns: number;
  deviceError: boolean;
  playing: boolean;
  recording: boolean;
  monitoring: boolean;
  looping: boolean;
  positionFrames: number;
  durationFrames: number;
  sessionRevision: number;
  error: string | null;
}
export interface AudioConfig {
  inputId: string;
  outputId: string;
  inputChannel: number;
  sampleRate: number;
  bufferFrames: number;
}
export const idleStatus: EngineStatus = {
  running: false,
  inputPeak: 0,
  outputPeak: 0,
  underruns: 0,
  overruns: 0,
  deviceError: false,
  playing: false,
  recording: false,
  monitoring: false,
  looping: false,
  positionFrames: 0,
  durationFrames: 0,
  sessionRevision: 0,
  error: null,
};
export const audio = {
  session: () =>
    desktop
      ? invoke<SessionState>('session_state')
      : Promise.resolve(emptySession),
  record: (
    config: AudioConfig,
    tone: string,
    targetTrack: string | null = null,
  ) => invoke<void>('start_recording', { config, tone, targetTrack }),
  play: (config: AudioConfig) => invoke<void>('play_transport', { config }),
  stopTransport: () => invoke<void>('stop_transport'),
  seek: (frame: number) =>
    invoke<void>('seek_transport', { frame: Math.round(frame) }),
  loop: (enabled: boolean) => invoke<void>('set_loop', { enabled }),
  importPaths: (paths: string[], sampleRate: number) =>
    invoke<void>('import_audio_paths', { paths, sampleRate }),
  createTrack: (name: string, sampleRate: number) =>
    invoke<string>('create_track', { name, sampleRate }),
  renameTrack: (id: string, name: string) =>
    invoke<void>('rename_track', { id, name }),
  renameClip: (id: string, name: string) =>
    invoke<void>('rename_clip', { id, name }),
  moveClip: (id: string, targetTrack: string, frame: number) =>
    invoke<void>('move_clip', { id, targetTrack, frame: Math.round(frame) }),
  splitClip: (id: string, frame: number) =>
    invoke<void>('split_clip', { id, frame: Math.round(frame) }),
  combineTracks: (ids: string[], name: string) =>
    invoke<string>('combine_tracks', { ids, name }),
  importBacking: (sampleRate: number) =>
    invoke<boolean>('import_backing', { sampleRate }),
  updateTrack: (track: Track) =>
    invoke<void>('update_track', {
      id: track.id,
      gain: track.gain,
      pan: track.pan,
      mute: track.mute,
      solo: track.solo,
    }),
  removeTrack: (id: string) => invoke<void>('remove_track', { id }),
  export: (id: string | null = null) => invoke<boolean>('export_audio', { id }),
  devices: () =>
    desktop ? invoke<AudioDevice[]>('list_devices') : Promise.resolve([]),
  start: (config: AudioConfig) => invoke<void>('start_monitoring', { config }),
  stop: () => invoke<void>('stop_monitoring'),
  status: () => invoke<EngineStatus>('engine_status'),
  setInputTrim: (inputTrim: number) =>
    desktop ? invoke<void>('set_parameters', { inputTrim }) : Promise.resolve(),
  workspaces: () =>
    desktop
      ? invoke<WorkspaceView>('workspace_list')
      : Promise.resolve({ items: [], active: null } as WorkspaceView),
  createWorkspace: (name: string) =>
    invoke<WorkspaceView>('workspace_create', { name }),
  openWorkspace: (id: string) =>
    invoke<WorkspaceView>('workspace_open', { id }),
  renameWorkspace: (id: string, name: string) =>
    invoke<WorkspaceView>('workspace_rename', { id, name }),
  closeWorkspace: () => invoke<WorkspaceView>('workspace_close'),
  processTrack: (id: string, settings: RecordedEffects) =>
    invoke<string>('process_recorded_track', { id, settings }),
};
export function channelCount(device?: AudioDevice, sampleRate = 48000) {
  return Math.max(
    0,
    ...(device?.configs
      .filter(
        (c) =>
          c.sampleFormat.toLowerCase() === 'f32' &&
          c.minSampleRate <= sampleRate &&
          c.maxSampleRate >= sampleRate,
      )
      .map((c) => c.channels) ?? []),
  );
}

export function canMonitor(devices: AudioDevice[], config: AudioConfig) {
  const input = devices.find(
    (d) => d.direction === 'input' && d.id === config.inputId,
  );
  const output = devices.find(
    (d) => d.direction === 'output' && d.id === config.outputId,
  );
  return (
    !!input?.isKptPro &&
    config.inputChannel >= 0 &&
    Number.isInteger(config.inputChannel) &&
    channelCount(input, config.sampleRate) > config.inputChannel &&
    channelCount(output, config.sampleRate) > 0
  );
}

export function commonSampleRates(input?: AudioDevice, output?: AudioDevice) {
  if (!input) return [];
  return [48000, 44100].filter(
    (rate) =>
      channelCount(input, rate) > 0 &&
      (!output || channelCount(output, rate) > 0),
  );
}
export function deviceLabel(device: AudioDevice) {
  return device.isKptPro && device.name.toLowerCase() !== 'kpt pro'
    ? `KPT Pro · ${device.name}`
    : device.name;
}

export interface Clip {
  id: string;
  name: string;
  file: string;
  startFrame: number;
  frames: number;
  sourceOffset: number;
  channels: number;
  tone: string;
}
export interface Track {
  id: string;
  name: string;
  kind: 'take' | 'backing' | 'audio' | 'combined' | 'processed';
  file: string;
  startFrame: number;
  frames: number;
  channels: number;
  gain: number;
  pan: number;
  mute: boolean;
  solo: boolean;
  tone: string;
  waveform: number[];
  clips: Clip[];
  waveforms: Record<string, number[]>;
}
export interface SessionState {
  sampleRate: number;
  tracks: Track[];
  storagePath: string;
}
export const emptySession: SessionState = {
  sampleRate: 0,
  tracks: [],
  storagePath: '',
};
export function formatTime(frames: number, rate: number) {
  const seconds = frames / Math.max(1, rate);
  return `${Math.floor(seconds / 60)
    .toString()
    .padStart(2, '0')}:${Math.floor(seconds % 60)
    .toString()
    .padStart(2, '0')}.${Math.floor((seconds % 1) * 1000)
    .toString()
    .padStart(3, '0')}`;
}

export function statusError(
  status: EngineStatus,
  previousDeviceError: boolean,
): string | null {
  if (status.error) return status.error;
  return status.deviceError && !previousDeviceError
    ? 'The audio device stopped responding. Check the selected input/output devices, then restart audio.'
    : null;
}

export interface WorkspaceInfo {
  id: string;
  name: string;
  createdAt: number;
  legacy: boolean;
  requiresName: boolean;
}
export interface WorkspaceView {
  items: WorkspaceInfo[];
  active: WorkspaceInfo | null;
}
export interface RecordedEffects {
  drive: number;
  bass: number;
  treble: number;
  level: number;
}
