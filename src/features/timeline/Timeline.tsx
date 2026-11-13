import { useEffect, useRef, useState, type PointerEvent } from 'react';
import {
  Guitar,
  AudioLines,
  Plus,
  Download,
  Trash2,
  Upload,
  Merge,
  Scissors,
  Pencil,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import {
  formatTime,
  type EngineStatus,
  type SessionState,
  type Track,
  type Clip,
} from '../../lib/audio';
import {
  draggedPosition,
  droppedLane,
  LANE_HEIGHT,
  RULER_HEIGHT,
  overlapSegments,
} from './editing';
type Drag = { id: string; start: number; row: number };
type DragOrigin = {
  id: string;
  start: number;
  clientX: number;
  clientY: number;
  width: number;
  top: number;
  span: number;
  moved: boolean;
};
interface Props {
  session: SessionState;
  status: EngineStatus;
  busy: boolean;
  desktop: boolean;
  recordTarget: string;
  selectedTrack: string;
  onSelectTrack: (id: string) => void;
  onImport: () => void;
  onExport: (id?: string) => void;
  onUpdate: (track: Track) => void;
  onRemove: (id: string) => void;
  onSeek: (frame: number) => void;
  onCreate: () => void;
  onRecordTarget: (id: string) => void;
  onRenameTrack: (id: string, name: string) => void;
  onRenameClip: (id: string, name: string) => void;
  onMove: (id: string, track: string, frame: number) => void;
  onSplit: (id: string, frame: number) => void;
  onCombine: (ids: string[], name: string) => void;
}
export function Timeline({
  session,
  status,
  busy,
  desktop,
  recordTarget,
  selectedTrack,
  onSelectTrack,
  onImport,
  onExport,
  onUpdate,
  onRemove,
  onSeek,
  onCreate,
  onRecordTarget,
  onRenameTrack,
  onRenameClip,
  onMove,
  onSplit,
  onCombine,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const origin = useRef<DragOrigin | null>(null);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [selectedClip, setSelectedClip] = useState('');
  const [selectedTracks, setSelectedTracks] = useState<string[]>([]);
  const [snap, setSnap] = useState(true);
  const [zoom, setZoom] = useState(1);
  const [combineName, setCombineName] = useState('');
  const [rename, setRename] = useState<{ id: string; name: string } | null>(
    null,
  );
  const [moveSeconds, setMoveSeconds] = useState('0');
  const [moveTrack, setMoveTrack] = useState('');
  const rate = session.sampleRate || 44100;
  const editable = desktop && !busy && !status.playing && !status.recording;
  const recordStart = useRef(status.positionFrames);
  useEffect(() => {
    if (status.recording) recordStart.current = status.positionFrames;
  }, [status.recording]);
  useEffect(() => {
    setSelectedTracks((current) =>
      current.filter((id) => session.tracks.some((t) => t.id === id)),
    );
  }, [session.tracks]);
  const entries = session.tracks.flatMap((track, row) =>
    track.clips.map((clip) => ({ clip, track, row })),
  );
  const chosen = entries.find((e) => e.clip.id === selectedClip);
  useEffect(() => {
    if (chosen) {
      setMoveSeconds((chosen.clip.startFrame / rate).toFixed(3));
      setMoveTrack(chosen.track.id);
    }
  }, [chosen?.clip.id, chosen?.clip.startFrame, chosen?.track.id, rate]);
  const selectedCount = entries.filter((e) =>
    selectedTracks.includes(e.track.id),
  ).length;
  const end =
    Math.max(
      rate * 30,
      status.durationFrames + rate * 2,
      status.positionFrames + rate * 2,
    ) * zoom;
  const targetRow = recordTarget
    ? session.tracks.findIndex((t) => t.id === recordTarget)
    : session.tracks.length;
  const rows =
    session.tracks.length + (status.recording && !recordTarget ? 1 : 0);
  const height = Math.max(242, rows * LANE_HEIGHT + RULER_HEIGHT);
  useEffect(() => {
    const canvas = canvasRef.current!;
    const draw = () => {
      const r = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      canvas.width = r.width * dpr;
      canvas.height = r.height * dpr;
      const c = canvas.getContext('2d');
      if (!c) return;
      c.scale(dpr, dpr);
      c.clearRect(0, 0, r.width, r.height);
      for (let i = 0; i <= 8; i++) {
        const x = (i * r.width) / 8;
        c.strokeStyle = '#2c323c';
        c.beginPath();
        c.moveTo(x, RULER_HEIGHT);
        c.lineTo(x, r.height);
        c.stroke();
        c.fillStyle = '#78859a';
        c.font = '9px monospace';
        c.fillText(formatTime((end * i) / 8, rate).slice(0, 5), x + 6, 18);
      }
      session.tracks.forEach((track, row) => {
        c.strokeStyle = '#2b313c';
        c.beginPath();
        c.moveTo(0, RULER_HEIGHT + (row + 1) * LANE_HEIGHT);
        c.lineTo(r.width, RULER_HEIGHT + (row + 1) * LANE_HEIGHT);
        c.stroke();
        if (track.id === selectedTrack) {
          c.fillStyle = 'rgba(132,173,212,0.09)';
          c.fillRect(0, RULER_HEIGHT + row * LANE_HEIGHT, r.width, LANE_HEIGHT);
        }
        track.clips.forEach((clip, index) => {
          const preview = drag?.id === clip.id ? drag : null;
          const start = ((preview?.start ?? clip.startFrame) / end) * r.width;
          const width = Math.max(3, (clip.frames / end) * r.width);
          const y = RULER_HEIGHT + (preview?.row ?? row) * LANE_HEIGHT;
          const palette = [
            '210,181,129',
            '119,192,207',
            '180,150,218',
            '135,196,159',
          ];
          const color = palette[index % palette.length];
          c.fillStyle = `rgba(${color},${track.mute ? 0.08 : 0.18})`;
          c.fillRect(start, y + 12, width, 81);
          c.fillStyle = `rgba(${color},${track.mute ? 0.25 : 0.65})`;
          const peaks = track.waveforms[clip.id] ?? [];
          for (let i = 0; i < peaks.length; i++) {
            const amplitude = Math.min(1, peaks[i]) * 26;
            c.fillRect(
              start + (i / peaks.length) * width,
              y + 53 - amplitude,
              Math.max(1, width / peaks.length),
              Math.max(1, amplitude * 2),
            );
          }
        });
      });
      session.tracks.forEach((_, row) => {
        const clips = entries
          .filter((e) => (drag?.id === e.clip.id ? drag.row : e.row) === row)
          .map((e) => ({
            startFrame: drag?.id === e.clip.id ? drag.start : e.clip.startFrame,
            frames: e.clip.frames,
          }));
        for (const segment of overlapSegments(clips)) {
          const x = (segment.start / end) * r.width,
            w = ((segment.end - segment.start) / end) * r.width,
            y = RULER_HEIGHT + row * LANE_HEIGHT;
          c.fillStyle = 'rgba(141,197,223,0.22)';
          c.fillRect(x, y + 94, w, 10);
          if (w > 38) {
            c.font = '8px monospace';
            c.fillStyle = '#b9dfed';
            c.fillText(`MIX ×${segment.count}`, x + 3, y + 102);
          }
        }
      });
      if (status.recording) {
        const x = (recordStart.current / end) * r.width;
        const width = Math.max(
          3,
          ((status.positionFrames - recordStart.current) / end) * r.width,
        );
        const y = RULER_HEIGHT + Math.max(0, targetRow) * LANE_HEIGHT;
        c.fillStyle = '#6b303b';
        c.fillRect(x, y + 12, width, 81);
        c.fillStyle = '#f0acb6';
        c.font = '10px sans-serif';
        c.fillText('Recording…', x + 8, y + 34);
      }
    };
    const observer = new ResizeObserver(draw);
    observer.observe(canvas);
    draw();
    return () => observer.disconnect();
  }, [
    session.tracks,
    end,
    rate,
    status.recording,
    status.positionFrames,
    drag,
    targetRow,
    selectedTrack,
  ]);
  const selectTrack = (id: string, checked: boolean) =>
    setSelectedTracks((current) =>
      checked
        ? [...new Set([...current, id])]
        : current.filter((value) => value !== id),
    );
  function pointerDown(event: PointerEvent<HTMLDivElement>, clip: Clip) {
    event.stopPropagation();
    setSelectedClip(clip.id);
    const parent = entries.find((e) => e.clip.id === clip.id);
    if (parent) onSelectTrack(parent.track.id);
    if (
      !editable ||
      event.button !== 0 ||
      (event.target as HTMLElement).closest('button')
    )
      return;
    const r = gridRef.current!.getBoundingClientRect();
    event.currentTarget.setPointerCapture(event.pointerId);
    event.currentTarget.focus();
    origin.current = {
      id: clip.id,
      start: clip.startFrame,
      clientX: event.clientX,
      clientY: event.clientY,
      width: r.width,
      top: r.top,
      span: end,
      moved: false,
    };
  }
  function pointerMove(event: PointerEvent<HTMLDivElement>) {
    const initial = origin.current;
    if (!initial) return;
    if (
      Math.abs(event.clientX - initial.clientX) +
        Math.abs(event.clientY - initial.clientY) <
        4 &&
      !initial.moved
    )
      return;
    initial.moved = true;
    setDrag({
      id: initial.id,
      start: draggedPosition(
        initial.start,
        event.clientX - initial.clientX,
        initial.width,
        initial.span,
        rate,
        snap && !event.altKey,
      ),
      row: droppedLane(
        event.clientY,
        gridRef.current?.getBoundingClientRect().top ?? initial.top,
        session.tracks.length,
      ),
    });
  }
  function pointerUp(event: PointerEvent<HTMLDivElement>) {
    const initial = origin.current;
    origin.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
    if (initial?.moved) {
      const frame = draggedPosition(
        initial.start,
        event.clientX - initial.clientX,
        initial.width,
        initial.span,
        rate,
        snap && !event.altKey,
      );
      const row = droppedLane(
        event.clientY,
        initial.top,
        session.tracks.length,
      );
      onMove(initial.id, session.tracks[row].id, frame);
    }
    setDrag(null);
  }
  return (
    <section className="arrangement">
      <div className="section-heading">
        <div>
          <span className="eyebrow">RECORD • ARRANGE • COMBINE</span>
          <h2>
            Arrangement{' '}
            <span className="heading-note">
              / {session.tracks.length} tracks · {entries.length} clips
            </span>
          </h2>
        </div>
        <div className="arrangement-actions">
          <button
            className="subtle-button"
            disabled={!editable || session.tracks.length >= 16}
            onClick={onCreate}
          >
            <Plus size={14} /> New track
          </button>
          <button
            className="subtle-button"
            disabled={!editable}
            onClick={onImport}
          >
            <Upload size={14} /> Upload audio
          </button>
          <button
            className="subtle-button"
            disabled={!editable || !entries.length}
            onClick={() => onExport()}
          >
            <Download size={14} /> Export mix
          </button>
        </div>
      </div>
      <div className="edit-toolbar">
        <label>
          Record to
          <select
            aria-label="Recording destination"
            value={recordTarget}
            onChange={(e) => onRecordTarget(e.target.value)}
            disabled={busy || status.recording}
          >
            <option value="">New track per take</option>
            {session.tracks.map((t) => (
              <option value={t.id} key={t.id}>
                {t.name}
              </option>
            ))}
          </select>
        </label>
        <label className="snap-toggle">
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => setSnap(e.target.checked)}
          />{' '}
          Snap 0.1 s
        </label>
        <div className="zoom-controls">
          <button
            title="Zoom in"
            aria-label="Zoom in"
            disabled={zoom <= 0.25}
            onClick={() => setZoom((v) => v / 2)}
          >
            <ZoomIn size={14} />
          </button>
          <button
            title="Zoom out"
            aria-label="Zoom out"
            disabled={zoom >= 32}
            onClick={() => setZoom((v) => v * 2)}
          >
            <ZoomOut size={14} />
          </button>
        </div>
        <div className="combine-controls">
          <input
            aria-label="Combined track name"
            placeholder="Combined track name"
            value={combineName}
            onChange={(e) => setCombineName(e.target.value)}
            maxLength={80}
          />
          <button
            className="subtle-button"
            disabled={
              !editable || selectedCount < 2 || session.tracks.length >= 16
            }
            onClick={() =>
              onCombine(selectedTracks, combineName.trim() || 'Combined track')
            }
            title="Render selected clips using track volume/pan. Originals are kept and muted."
          >
            <Merge size={14} /> Combine selected ({selectedCount})
          </button>
        </div>
      </div>
      <div className="timeline session-timeline" style={{ height }}>
        <div className="track-list" style={{ minHeight: height }}>
          <div className="track-ruler">
            SELECT TRACKS TO COMBINE{' '}
            <span>{String(rows).padStart(2, '0')}</span>
          </div>
          {session.tracks.map((track) => (
            <div
              className={`session-track ${track.kind} ${selectedTrack === track.id ? 'selected-row' : ''} ${recordTarget === track.id ? 'record-destination' : ''}`}
              key={track.id}
              onClick={() => onSelectTrack(track.id)}
              onFocus={() => onSelectTrack(track.id)}
            >
              <div className="session-track-title">
                <input
                  className="track-select"
                  type="checkbox"
                  aria-label={`Select ${track.name} to combine`}
                  checked={selectedTracks.includes(track.id)}
                  disabled={!editable || !track.clips.length}
                  onChange={(e) => selectTrack(track.id, e.target.checked)}
                />
                {track.kind === 'take' ? (
                  <Guitar size={14} />
                ) : (
                  <AudioLines size={14} />
                )}
                <TrackName
                  track={track}
                  disabled={!editable}
                  onRename={onRenameTrack}
                />
                <button
                  aria-label={`Export ${track.name}`}
                  title="Export track audio"
                  disabled={!editable || !track.clips.length}
                  onClick={() => onExport(track.id)}
                >
                  <Download size={12} />
                </button>
                <button
                  aria-label={`Remove ${track.name}`}
                  title="Remove track; source files are retained"
                  disabled={!editable}
                  onClick={() => onRemove(track.id)}
                >
                  <Trash2 size={12} />
                </button>
              </div>
              <div className="track-mix">
                <button
                  aria-pressed={track.mute}
                  className={track.mute ? 'lit' : ''}
                  onClick={() => onUpdate({ ...track, mute: !track.mute })}
                >
                  M
                </button>
                <button
                  aria-pressed={track.solo}
                  className={track.solo ? 'lit' : ''}
                  onClick={() => onUpdate({ ...track, solo: !track.solo })}
                >
                  S
                </button>
                <label>
                  Vol
                  <input
                    aria-label={`${track.name} volume`}
                    type="range"
                    min={0}
                    max={1.5}
                    step={0.01}
                    value={track.gain}
                    onChange={(e) =>
                      onUpdate({ ...track, gain: Number(e.target.value) })
                    }
                  />
                </label>
              </div>
              <label className="track-pan">
                Pan
                <input
                  aria-label={`${track.name} pan`}
                  type="range"
                  min={-1}
                  max={1}
                  step={0.05}
                  value={track.pan}
                  onChange={(e) =>
                    onUpdate({ ...track, pan: Number(e.target.value) })
                  }
                />
                <span>{track.clips.length} clips</span>
                <button
                  className={
                    recordTarget === track.id ? 'arm-track armed' : 'arm-track'
                  }
                  aria-label={`Record into ${track.name}`}
                  aria-pressed={recordTarget === track.id}
                  disabled={busy || status.recording}
                  onClick={() =>
                    onRecordTarget(recordTarget === track.id ? '' : track.id)
                  }
                >
                  R
                </button>
              </label>
            </div>
          ))}
          {status.recording && !recordTarget && (
            <div className="session-track recording-track">
              <span className="record-dot" /> Recording guitar…
              <small>Stop to save WAV</small>
            </div>
          )}
          {!rows && (
            <div className="empty-session-tracks">
              <Guitar size={25} />
              <strong>Ready for your first take</strong>
              <span>
                Record, create a track,
                <br />
                or upload audio files.
              </span>
            </div>
          )}
        </div>
        <div
          className="timeline-grid"
          style={{ minHeight: height }}
          ref={gridRef}
          onClick={(e) => {
            if (status.playing || busy) return;
            const r = e.currentTarget.getBoundingClientRect();
            const row = droppedLane(e.clientY, r.top, session.tracks.length);
            if (session.tracks[row]) onSelectTrack(session.tracks[row].id);
            onSeek(
              Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * end,
            );
          }}
        >
          <canvas
            ref={canvasRef}
            aria-label="Audio clip waveforms and timeline"
          />
          {entries.map(({ clip, row, track }) => {
            const preview = drag?.id === clip.id ? drag : null;
            return (
              <div
                key={clip.id}
                role="group"
                tabIndex={0}
                aria-label={`Audio clip ${clip.name}`}
                title="Drag to move in time or to another track. Use arrow keys to nudge. Alt disables snap."
                className={`audio-clip ${selectedClip === clip.id ? 'selected-clip' : ''} ${preview ? 'dragging-clip' : ''}`}
                style={{
                  left: `${((preview?.start ?? clip.startFrame) / end) * 100}%`,
                  width: `${Math.max(0.35, (clip.frames / end) * 100)}%`,
                  top: RULER_HEIGHT + (preview?.row ?? row) * LANE_HEIGHT + 12,
                }}
                onClick={(e) => e.stopPropagation()}
                onPointerDown={(e) => pointerDown(e, clip)}
                onPointerMove={pointerMove}
                onPointerUp={pointerUp}
                onPointerCancel={() => {
                  origin.current = null;
                  setDrag(null);
                }}
                onFocus={() => {
                  setSelectedClip(clip.id);
                  onSelectTrack(track.id);
                }}
                onKeyDown={(e) => {
                  if (!editable) return;
                  if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
                    e.preventDefault();
                    onMove(
                      clip.id,
                      session.tracks[row].id,
                      Math.max(
                        0,
                        clip.startFrame +
                          (e.key === 'ArrowLeft' ? -1 : 1) *
                            Math.round(rate * (e.shiftKey ? 1 : 0.1)),
                      ),
                    );
                  }
                }}
              >
                <span
                  className="clip-name"
                  onDoubleClick={() =>
                    editable && setRename({ id: clip.id, name: clip.name })
                  }
                >
                  {clip.name}
                </span>
                <button
                  className="clip-rename"
                  aria-label={`Rename clip ${clip.name}`}
                  disabled={!editable}
                  onClick={() => setRename({ id: clip.id, name: clip.name })}
                >
                  <Pencil size={10} />
                </button>
              </div>
            );
          })}
          {!rows && (
            <div className="timeline-empty">
              <AudioLines size={26} />
              <strong>Drop a backing track and start playing.</strong>
              <span>
                WAV, MP3, M4A, FLAC or Ogg.
                <br />
                Drag recordings between tracks to arrange them.
              </span>
            </div>
          )}
          <div
            className="playhead"
            style={{
              left: `${Math.min(100, (status.positionFrames / end) * 100)}%`,
            }}
          />
        </div>
      </div>
      {chosen && (
        <div className="clip-inspector">
          <strong>{chosen.clip.name}</strong>
          <label>
            Track
            <select
              aria-label="Selected clip destination track"
              value={moveTrack}
              disabled={!editable}
              onChange={(e) => setMoveTrack(e.target.value)}
            >
              {session.tracks.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Start (s)
            <input
              aria-label="Selected clip start in seconds"
              type="number"
              min={0}
              max={3600}
              step={0.01}
              value={moveSeconds}
              disabled={!editable}
              onChange={(e) => setMoveSeconds(e.target.value)}
            />
          </label>
          <button
            className="subtle-button"
            disabled={
              !editable ||
              !moveTrack ||
              !Number.isFinite(Number(moveSeconds)) ||
              Number(moveSeconds) < 0
            }
            onClick={() =>
              onMove(
                chosen.clip.id,
                moveTrack,
                Math.round(Number(moveSeconds) * rate),
              )
            }
          >
            Apply position
          </button>
          <button
            className="subtle-button"
            disabled={
              !editable ||
              status.positionFrames <= chosen.clip.startFrame ||
              status.positionFrames >=
                chosen.clip.startFrame + chosen.clip.frames
            }
            onClick={() => onSplit(chosen.clip.id, status.positionFrames)}
          >
            <Scissors size={13} /> Split at playhead
          </button>
        </div>
      )}
      <div className="session-footer">
        <span>
          Drag clips · click track names to rename · uploads stay on this Mac
        </span>
        <span>{formatTime(status.durationFrames, rate)}</span>
      </div>
      {rename && (
        <div className="modal-backdrop" onClick={() => setRename(null)}>
          <form
            className="modal clip-name-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="clip-name-title"
            onClick={(e) => e.stopPropagation()}
            onSubmit={(e) => {
              e.preventDefault();
              if (rename.name.trim()) {
                onRenameClip(rename.id, rename.name);
                setRename(null);
              }
            }}
          >
            <h2 id="clip-name-title">Rename recording</h2>
            <input
              autoFocus
              aria-label="Recording clip name"
              value={rename.name}
              onChange={(e) => setRename({ ...rename, name: e.target.value })}
              maxLength={80}
              onKeyDown={(e) => {
                if (e.key === 'Escape') setRename(null);
              }}
            />
            <div className="modal-actions">
              <button
                type="button"
                className="subtle-button"
                onClick={() => setRename(null)}
              >
                Cancel
              </button>
              <button className="primary-button" disabled={!rename.name.trim()}>
                Save name
              </button>
            </div>
          </form>
        </div>
      )}
    </section>
  );
}
function TrackName({
  track,
  disabled,
  onRename,
}: {
  track: Track;
  disabled: boolean;
  onRename: (id: string, name: string) => void;
}) {
  const [value, setValue] = useState(track.name);
  const cancelled = useRef(false);
  useEffect(() => setValue(track.name), [track.name]);
  return (
    <input
      className="track-name-input"
      aria-label={`Rename track ${track.name}`}
      title="Click to rename track"
      maxLength={80}
      disabled={disabled}
      value={value}
      onChange={(e) => setValue(e.target.value)}
      onBlur={() => {
        if (cancelled.current) {
          cancelled.current = false;
          setValue(track.name);
          return;
        }
        if (value.trim() && value.trim() !== track.name)
          onRename(track.id, value.trim());
        else setValue(track.name);
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter') e.currentTarget.blur();
        if (e.key === 'Escape') {
          cancelled.current = true;
          e.currentTarget.blur();
        }
      }}
    />
  );
}
