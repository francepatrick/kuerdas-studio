import { useState } from "react";
import { AudioLines, Sparkles } from "lucide-react";

import type {
  RecordedEffects as EffectSettings,
  Track,
} from "@/src/services/session/session.interfaces";

interface RecordedEffectsProps {
  disabled: boolean;
  onApply: (id: string, settings: EffectSettings) => void;
  track?: Track;
}

export function RecordedEffects({
  track,
  disabled,
  onApply,
}: RecordedEffectsProps) {
  const [settings, setSettings] = useState<EffectSettings>({
    drive: 3,
    bass: 0,
    treble: 0,
    level: 0.5,
  });
  return (
    <section className="recorded-effects">
      <div className="section-heading">
        <div>
          <span className="eyebrow">AFTER RECORDING</span>
          <h2>Recorded audio effects</h2>
        </div>
        <AudioLines size={18} />
      </div>
      <div className="recorded-effects-body">
        <div className="effects-track-name">
          <span className="small-dot" />
          <strong>{track?.name ?? "Select a track"}</strong>
        </div>
        <p>
          Add drive and tone shaping to a copy of the recorded audio. The
          original KPT Pro take is kept.
        </p>
        <div className="recorded-effects-controls">
          {(
            [
              { key: "drive", label: "Drive", min: 1, max: 20, step: 0.1 },
              { key: "bass", label: "Bass", min: -1, max: 1, step: 0.05 },
              { key: "treble", label: "Treble", min: -1, max: 1, step: 0.05 },
              { key: "level", label: "Output", min: 0, max: 1, step: 0.01 },
            ] as const
          ).map((control) => (
            <label key={control.key}>
              <span>
                {control.label}
                <b>
                  {control.key === "level"
                    ? `${Math.round(settings.level * 100)}%`
                    : settings[control.key].toFixed(1)}
                </b>
              </span>
              <input
                aria-label={`Recorded audio ${control.label.toLowerCase()}`}
                type="range"
                min={control.min}
                max={control.max}
                step={control.step}
                value={settings[control.key]}
                disabled={disabled || !track?.clips.length}
                onChange={(e) =>
                  setSettings((s) => ({
                    ...s,
                    [control.key]: Number(e.target.value),
                  }))
                }
              />
            </label>
          ))}
        </div>
        <div className="recorded-effects-footer">
          <span>The original is muted for comparison.</span>
          <button
            className="primary-button"
            disabled={disabled || !track?.clips.length}
            onClick={() => track && onApply(track.id, settings)}
          >
            <Sparkles size={14} /> Create processed copy
          </button>
        </div>
      </div>
    </section>
  );
}
