import { invoke } from "@tauri-apps/api/core";

import { isDesktop } from "@/src/config/platform";
import { emptySession } from "@/src/services/session/session.fallback";
import type {
  RecordedEffects,
  SessionState,
  Track,
} from "@/src/services/session/session.interfaces";

export function getSession(): Promise<SessionState> {
  return isDesktop
    ? invoke<SessionState>("session_state")
    : Promise.resolve(emptySession);
}

export function importAudioPaths(
  paths: string[],
  sampleRate: number,
): Promise<void> {
  return invoke<void>("import_audio_paths", { paths, sampleRate });
}

/** Opens the native file picker. Resolves `false` when the user cancels. */
export function importBacking(sampleRate: number): Promise<boolean> {
  return invoke<boolean>("import_backing", { sampleRate });
}

export function createTrack(name: string, sampleRate: number): Promise<string> {
  return invoke<string>("create_track", { name, sampleRate });
}

export function renameTrack(id: string, name: string): Promise<void> {
  return invoke<void>("rename_track", { id, name });
}

export function updateTrack(track: Track): Promise<void> {
  return invoke<void>("update_track", {
    gain: track.gain,
    id: track.id,
    mute: track.mute,
    pan: track.pan,
    solo: track.solo,
  });
}

export function removeTrack(id: string): Promise<void> {
  return invoke<void>("remove_track", { id });
}

export function combineTracks(ids: string[], name: string): Promise<string> {
  return invoke<string>("combine_tracks", { ids, name });
}

export function processTrack(
  id: string,
  settings: RecordedEffects,
): Promise<string> {
  return invoke<string>("process_recorded_track", { id, settings });
}

export function renameClip(id: string, name: string): Promise<void> {
  return invoke<void>("rename_clip", { id, name });
}

export function moveClip(
  id: string,
  targetTrack: string,
  frame: number,
): Promise<void> {
  return invoke<void>("move_clip", {
    frame: Math.round(frame),
    id,
    targetTrack,
  });
}

export function splitClip(id: string, frame: number): Promise<void> {
  return invoke<void>("split_clip", { frame: Math.round(frame), id });
}

/** Exports one track, or the full mix when `id` is null. Resolves `false` when cancelled. */
export function exportAudio(id: string | null = null): Promise<boolean> {
  return invoke<boolean>("export_audio", { id });
}
