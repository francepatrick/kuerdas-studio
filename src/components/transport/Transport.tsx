import {
  Circle,
  Play,
  Square,
  SkipBack,
  Headphones,
  Repeat,
} from "lucide-react";

import type { EngineStatus } from "@/src/services/engine/engine.interfaces";
import { formatTime } from "@/src/utils/format-time";

interface TransportProps {
  busy: boolean;
  canStart: boolean;
  hasTracks: boolean;
  kptConnected: boolean;
  onLoop: () => void;
  onPlay: () => void;
  onRecord: () => void;
  onRewind: () => void;
  onStop: () => void;
  sampleRate: number;
  status: EngineStatus;
  toggle: () => void;
}

export function Transport({
  status,
  busy,
  canStart,
  kptConnected,
  sampleRate,
  hasTracks,
  toggle,
  onRecord,
  onPlay,
  onStop,
  onRewind,
  onLoop,
}: TransportProps) {
  return (
    <div className={`transport ${status.recording ? "is-recording" : ""}`}>
      <div className="transport-controls">
        <button
          disabled={busy || status.playing}
          onClick={onRewind}
          title="Return to start"
          aria-label="Return to start"
        >
          <SkipBack size={18} />
        </button>
        <button
          disabled={busy || !status.running}
          onClick={onStop}
          title="Stop and save recording"
          aria-label="Stop"
        >
          <Square size={16} />
        </button>
        <button
          disabled={busy || !canStart || !hasTracks || status.playing}
          onClick={onPlay}
          title="Play session"
          aria-label="Play"
        >
          <Play size={19} />
        </button>
        <button
          className={status.recording ? "recording-active" : ""}
          disabled={busy || !canStart || status.recording}
          onClick={onRecord}
          title="Record a new guitar take at the playhead"
          aria-label="Record"
        >
          <Circle size={19} className="record-icon" />
        </button>
        <button
          aria-label="Loop playback"
          title="Loop playback (overdubs record continuously)"
          aria-pressed={status.looping}
          disabled={busy || status.recording}
          onClick={onLoop}
          className={status.looping ? "loop-active" : ""}
        >
          <Repeat size={16} />
        </button>
      </div>
      <div className="time-display">
        <span>
          {status.recording
            ? "RECORDING"
            : status.playing
              ? "PLAYING"
              : "POSITION"}
        </span>
        <strong>{formatTime(status.positionFrames, sampleRate)}</strong>
      </div>
      <div className="tempo-display">
        <strong>
          {sampleRate / 1000} <small>kHz</small>
        </strong>
        <span>{status.recording ? "New take" : "WAV session"}</span>
      </div>
      <div className="transport-spacer" />
      <button
        className={
          status.monitoring ? "monitor-button active" : "monitor-button"
        }
        onClick={toggle}
        disabled={busy || (!status.monitoring && !canStart) || status.recording}
      >
        <Headphones size={16} />
        {busy
          ? "Please wait…"
          : status.monitoring
            ? "Monitor on"
            : !kptConnected
              ? "Connect KPT Pro"
              : "Monitor off"}
      </button>
    </div>
  );
}
