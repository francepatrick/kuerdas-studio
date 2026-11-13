interface MeterProps {
  label: string;
  /** Linear peak, 0–1. */
  value: number;
}

const FLOOR_DB = -60;

export function Meter({ value, label }: MeterProps) {
  const db = value > 0 ? Math.max(FLOOR_DB, 20 * Math.log10(value)) : FLOOR_DB;
  return (
    <div className="meter">
      <div className="meter-label">
        <span>{label}</span>
        <span>{value > 0 ? `${db.toFixed(1)} dB` : "−∞ dB"}</span>
      </div>
      <div
        className="meter-track"
        role="meter"
        aria-label={label}
        aria-valuemin={FLOOR_DB}
        aria-valuemax={0}
        aria-valuenow={db}
      >
        <div style={{ width: `${((db - FLOOR_DB) / -FLOOR_DB) * 100}%` }} />
      </div>
    </div>
  );
}
