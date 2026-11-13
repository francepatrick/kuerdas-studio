import { invoke } from "@tauri-apps/api/core";

import { isDesktop } from "@/src/config/platform";
import type {
  AudioConfig,
  AudioDevice,
  EngineStatus,
} from "@/src/services/engine/engine.interfaces";

export function listDevices(): Promise<AudioDevice[]> {
  return isDesktop
    ? invoke<AudioDevice[]>("list_devices")
    : Promise.resolve([]);
}

export function getEngineStatus(): Promise<EngineStatus> {
  return invoke<EngineStatus>("engine_status");
}

export function startMonitoring(config: AudioConfig): Promise<void> {
  return invoke<void>("start_monitoring", { config });
}

export function stopMonitoring(): Promise<void> {
  return invoke<void>("stop_monitoring");
}

export function setInputTrim(inputTrim: number): Promise<void> {
  return isDesktop
    ? invoke<void>("set_parameters", { inputTrim })
    : Promise.resolve();
}

export function startRecording(
  config: AudioConfig,
  tone: string,
  targetTrack: string | null = null,
): Promise<void> {
  return invoke<void>("start_recording", { config, targetTrack, tone });
}

export function playTransport(config: AudioConfig): Promise<void> {
  return invoke<void>("play_transport", { config });
}

export function stopTransport(): Promise<void> {
  return invoke<void>("stop_transport");
}

export function seekTransport(frame: number): Promise<void> {
  return invoke<void>("seek_transport", { frame: Math.round(frame) });
}

export function setLoop(enabled: boolean): Promise<void> {
  return invoke<void>("set_loop", { enabled });
}
