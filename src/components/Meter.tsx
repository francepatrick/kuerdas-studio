export function Meter({ value, label }: { value: number; label: string }) {
  const db = value > 0 ? Math.max(-60, 20 * Math.log10(value)) : -60;
  return (
    <div className="meter">
      <div className="meter-label">
        <span>{label}</span>
        <span>{value > 0 ? `${db.toFixed(1)} dB` : '−∞ dB'}</span>
      </div>
      <div
        className="meter-track"
        role="meter"
        aria-label={label}
        aria-valuemin={-60}
        aria-valuemax={0}
        aria-valuenow={db}
      >
        <div style={{ width: `${((db + 60) / 60) * 100}%` }} />
      </div>
    </div>
  );
}
