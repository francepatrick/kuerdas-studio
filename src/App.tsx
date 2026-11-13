import { useCallback, useEffect, useRef, useState } from 'react';
import {
  AudioLines,
  Cable,
  ChevronDown,
  Guitar,
  Headphones,
  RefreshCw,
  Settings2,
  X,
} from 'lucide-react';
import {
  audio,
  channelCount,
  canMonitor,
  commonSampleRates,
  deviceLabel,
  desktop,
  idleStatus,
  statusError,
  emptySession,
  type SessionState,
  type Track,
  type AudioDevice,
  type EngineStatus,
  type WorkspaceView,
  type WorkspaceInfo,
} from './lib/audio';
import { useStudio } from './stores/studio';
import { Meter } from './components/Meter';
import { Timeline } from './features/timeline/Timeline';
import { RecordedEffects } from './features/effects/RecordedEffects';
import {
  WorkspaceSidebar,
  WorkspaceStart,
  WorkspaceNameDialog,
  type WorkspaceDialogState,
} from './features/workspaces/Workspaces';
import { Transport } from './features/transport/Transport';
export default function App() {
  const { config, setConfig, inputTrim, setInputTrim } = useStudio();
  const [library, setLibrary] = useState<WorkspaceView>({
    items: [],
    active: null,
  });
  const [workspaceDialog, setWorkspaceDialog] =
    useState<WorkspaceDialogState | null>(null);
  const [selectedTrack, setSelectedTrack] = useState('');
  useEffect(() => {
    void audio
      .workspaces()
      .then(setLibrary)
      .catch((e) => setError(String(e)));
  }, []);
  const [session, setSession] = useState<SessionState>(emptySession);
  const [recordTarget, setRecordTarget] = useState('');
  const hasAudio = session.tracks.some((t) => t.clips.length > 0);
  useEffect(() => {
    setRecordTarget((current) =>
      current && !session.tracks.some((track) => track.id === current)
        ? ''
        : current,
    );
  }, [session.tracks]);
  const pendingTracks = useRef<Record<string, Track>>({});
  const trackTimers = useRef<Record<string, ReturnType<typeof setTimeout>>>({});
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [status, setStatus] = useState<EngineStatus>(idleStatus);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [settings, setSettings] = useState(false);
  const mounted = useRef(true);
  const previousDeviceError = useRef(false);
  const receiveStatus = useCallback((next: EngineStatus) => {
    setStatus(next);
    const message = statusError(next, previousDeviceError.current);
    previousDeviceError.current = next.deviceError;
    if (message) setError(message);
  }, []);
  const refresh = useCallback(async () => {
    try {
      const found = await audio.devices();
      if (mounted.current) setDevices(found);
    } catch (e) {
      setError(String(e));
    }
  }, []);
  useEffect(() => {
    mounted.current = true;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const scan = async () => {
      await refresh();
      if (!cancelled) timer = setTimeout(scan, 2000);
    };
    void scan();
    return () => {
      cancelled = true;
      clearTimeout(timer);
      mounted.current = false;
    };
  }, [refresh]);
  useEffect(() => {
    if (!desktop) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const s = await audio.status();
        if (!cancelled) {
          receiveStatus(s);
        }
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
      if (!cancelled) timer = setTimeout(poll, 100);
    };
    void poll();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);
  useEffect(() => {
    if (!desktop) return;
    const timer = setTimeout(() => {
      void audio.setInputTrim(inputTrim).catch((e) => {
        if (mounted.current) setError(String(e));
      });
    }, 25);
    return () => clearTimeout(timer);
  }, [inputTrim]);
  useEffect(() => {
    if (!settings && !workspaceDialog) return;
    const previous = document.activeElement as HTMLElement | null;
    const dialog = document.querySelector<HTMLElement>('[role="dialog"]');
    const focusable = () =>
      Array.from(
        dialog?.querySelectorAll<HTMLElement>(
          'button:not(:disabled), select:not(:disabled), input:not(:disabled), [href]',
        ) ?? [],
      );
    focusable()[0]?.focus();
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setSettings(false);
        setWorkspaceDialog(null);
      }
      if (e.key === 'Tab') {
        const elements = focusable();
        const first = elements[0];
        const last = elements.at(-1);
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    };
    document.addEventListener('keydown', handleKey);
    return () => {
      document.removeEventListener('keydown', handleKey);
      previous?.focus();
    };
  }, [settings, workspaceDialog]);
  useEffect(() => {
    let cancelled = false;
    if (!desktop) return;
    void audio
      .session()
      .then((value) => {
        if (!cancelled) setSession(value);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [status.sessionRevision]);
  useEffect(() => {
    if (
      hasAudio &&
      session.sampleRate &&
      config.sampleRate !== session.sampleRate &&
      !status.running
    )
      setConfig({ sampleRate: session.sampleRate });
  }, [
    session.sampleRate,
    hasAudio,
    config.sampleRate,
    status.running,
    setConfig,
  ]);
  useEffect(
    () => () => {
      Object.values(trackTimers.current).forEach(clearTimeout);
    },
    [],
  );
  const run = async (operation: () => Promise<unknown>) => {
    setBusy(true);
    setError('');
    try {
      await operation();
      const next = desktop ? await audio.status() : idleStatus;
      receiveStatus(next);
      setSession(desktop ? await audio.session() : emptySession);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const updateTrack = (track: Track) => {
    setSession((s) => ({
      ...s,
      tracks: s.tracks.map((t) => (t.id === track.id ? track : t)),
    }));
    pendingTracks.current[track.id] = track;
    clearTimeout(trackTimers.current[track.id]);
    trackTimers.current[track.id] = setTimeout(() => {
      delete pendingTracks.current[track.id];
      void audio.updateTrack(track).catch((e) => {
        setError(String(e));
        void audio.session().then(setSession);
      });
    }, 60);
  };
  const edit = async (operation: () => Promise<unknown>) => {
    if (status.playing || status.recording) {
      setError('Stop playback or recording before editing.');
      return;
    }
    await run(async () => {
      if (status.running) await audio.stopTransport();
      return operation();
    });
  };
  const importDropped = useRef<(paths: string[]) => void>(() => {});
  importDropped.current = (paths) => {
    if (!busy && library.active)
      void edit(() => audio.importPaths(paths, config.sampleRate));
  };
  useEffect(() => {
    if (!desktop) return;
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void import('@tauri-apps/api/webviewWindow')
      .then(async (api) => {
        const stop = await api
          .getCurrentWebviewWindow()
          .onDragDropEvent((event) => {
            if (event.payload.type === 'drop')
              importDropped.current(event.payload.paths);
          });
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch((e) => setError(String(e)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
  const input = devices.find(
    (d) => d.direction === 'input' && d.id === config.inputId,
  );
  const output = devices.find(
    (d) => d.direction === 'output' && d.id === config.outputId,
  );
  const channels = channelCount(input, config.sampleRate);
  const sampleRates = commonSampleRates(input, output);
  useEffect(() => {
    if (status.running || busy || hasAudio) return;
    if (sampleRates.length && !sampleRates.includes(config.sampleRate))
      setConfig({ sampleRate: sampleRates[0], inputChannel: 0 });
  }, [
    sampleRates.join(','),
    hasAudio,
    config.sampleRate,
    status.running,
    busy,
    setConfig,
  ]);
  const kptConnected = devices.some(
    (d) => d.direction === 'input' && d.isKptPro,
  );
  const canStart = !!library.active && desktop && canMonitor(devices, config);
  const toggle = async () => {
    setBusy(true);
    setError('');
    try {
      if (status.monitoring) await audio.stop();
      else {
        await audio.setInputTrim(inputTrim);
        await audio.start(config);
      }
      receiveStatus(await audio.status());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  async function changeWorkspace(operation: () => Promise<WorkspaceView>) {
    if (status.playing || status.recording) return;
    await edit(async () => {
      for (const timer of Object.values(trackTimers.current))
        clearTimeout(timer);
      const pending = Object.values(pendingTracks.current);
      pendingTracks.current = {};
      for (const track of pending) await audio.updateTrack(track);
      const next = await operation();
      setLibrary(next);
      setRecordTarget('');
      setSelectedTrack('');
      setWorkspaceDialog(null);
    });
  }
  async function createWorkspace(name: string) {
    if (desktop) await changeWorkspace(() => audio.createWorkspace(name));
    else {
      const item: WorkspaceInfo = {
        id: crypto.randomUUID(),
        name,
        createdAt: Date.now() / 1000,
        legacy: false,
        requiresName: false,
      };
      setLibrary((s) => ({ items: [item, ...s.items], active: item }));
      setWorkspaceDialog(null);
    }
  }
  function openWorkspace(item: WorkspaceInfo) {
    if (item.requiresName) {
      setWorkspaceDialog({ mode: 'recover', id: item.id, name: '' });
      return;
    }
    if (item.id === library.active?.id) return;
    if (desktop) void changeWorkspace(() => audio.openWorkspace(item.id));
    else setLibrary((s) => ({ ...s, active: item }));
  }
  return (
    <div className="app-shell">
      <WorkspaceSidebar
        library={library}
        locked={busy || status.playing || status.recording}
        onHome={() =>
          void changeWorkspace(() =>
            desktop
              ? audio.closeWorkspace()
              : Promise.resolve({ ...library, active: null }),
          )
        }
        onCreate={() => setWorkspaceDialog({ mode: 'create', name: '' })}
        onOpen={openWorkspace}
        onRename={(item) =>
          setWorkspaceDialog({
            mode: item.requiresName ? 'recover' : 'rename',
            id: item.id,
            name: item.requiresName ? '' : item.name,
          })
        }
      />
      <main>
        <header className="topbar">
          <div className="breadcrumbs">
            Workspace <span>/</span>{' '}
            <strong>{library.active?.name ?? 'Your workspaces'}</strong>
            <ChevronDown size={13} />
          </div>
          <span className="local-badge">
            <span className="small-dot" /> LOCAL WORKSPACE
          </span>
          <button
            className="icon-button"
            aria-label="Open audio settings"
            onClick={() => setSettings(true)}
          >
            <Settings2 size={18} />
          </button>
        </header>
        <div className="page-content">
          {error && (
            <div className="error" role="alert">
              <span>{error}</span>
              <button aria-label="Dismiss error" onClick={() => setError('')}>
                <X size={16} />
              </button>
            </div>
          )}
          {!library.active ? (
            <WorkspaceStart
              library={library}
              busy={busy}
              onCreate={(name) => void createWorkspace(name)}
              onOpen={openWorkspace}
            />
          ) : (
            <>
              <div className="page-heading">
                <div>
                  <div className="eyebrow">RECORDING WORKSPACE</div>
                  <h1>
                    {library.active.name}
                    <span>.</span>
                  </h1>
                  <p>
                    Record with KPT Pro. Arrange your takes. Build your mix.
                  </p>
                </div>
                <span className="session-badge">
                  <Guitar size={16} /> Recording studio
                </span>
              </div>
              {!desktop && (
                <div className="notice">
                  <Headphones size={16} />
                  <span>
                    Browser preview · Open the desktop app and connect KPT Pro
                    to enable audio.
                  </span>
                </div>
              )}
              {desktop && !kptConnected && (
                <div className="notice" role="status">
                  <Cable size={17} />
                  <div>
                    <strong>Connect KPT Pro to enable audio</strong>
                    <p className="connection-copy">
                      Connect it by USB, then select its input in Audio
                      settings. Monitoring requires KPT Pro.
                    </p>
                  </div>
                  <button
                    className="subtle-button"
                    onClick={() => setSettings(true)}
                  >
                    Audio settings
                  </button>
                </div>
              )}
              <Transport
                status={status}
                busy={busy}
                canStart={canStart}
                kptConnected={kptConnected}
                sampleRate={session.sampleRate || config.sampleRate}
                hasTracks={hasAudio}
                toggle={() => void toggle()}
                onRecord={() =>
                  void run(async () => {
                    await audio.setInputTrim(inputTrim);
                    await audio.record(
                      config,
                      'KPT Pro USB input',
                      recordTarget || null,
                    );
                  })
                }
                onPlay={() => void run(() => audio.play(config))}
                onStop={() => void run(() => audio.stopTransport())}
                onRewind={() => void run(() => audio.seek(0))}
                onLoop={() => void run(() => audio.loop(!status.looping))}
              />
              <div className="device-strip">
                <span
                  className={
                    status.running ? 'status-light running' : 'status-light'
                  }
                />
                <strong>
                  {status.running
                    ? 'Audio engine running'
                    : !kptConnected
                      ? 'KPT Pro required'
                      : 'Audio engine stopped'}
                </strong>
                <span className="strip-separator" />
                <span>
                  IN <b>{input ? deviceLabel(input) : 'No input selected'}</b>
                </span>
                <span>
                  OUT{' '}
                  <b>{output ? deviceLabel(output) : 'No output selected'}</b>
                </span>
                <button onClick={() => setSettings(true)}>
                  Configure <ChevronDown size={12} />
                </button>
              </div>
              <Timeline
                key={library.active.id}
                selectedTrack={selectedTrack}
                onSelectTrack={setSelectedTrack}
                session={session}
                status={status}
                busy={busy}
                desktop={desktop}
                onImport={() =>
                  void edit(() => audio.importBacking(config.sampleRate))
                }
                onExport={(id) => void edit(() => audio.export(id ?? null))}
                onUpdate={updateTrack}
                onRemove={(id) => void edit(() => audio.removeTrack(id))}
                recordTarget={recordTarget}
                onRecordTarget={setRecordTarget}
                onCreate={() =>
                  void edit(async () => {
                    const id = await audio.createTrack(
                      `Audio track ${session.tracks.length + 1}`,
                      config.sampleRate,
                    );
                    setRecordTarget(id);
                  })
                }
                onRenameTrack={(id, name) =>
                  void edit(() => audio.renameTrack(id, name))
                }
                onRenameClip={(id, name) =>
                  void edit(() => audio.renameClip(id, name))
                }
                onMove={(id, target, frame) =>
                  void edit(() => audio.moveClip(id, target, frame))
                }
                onSplit={(id, frame) =>
                  void edit(() => audio.splitClip(id, frame))
                }
                onCombine={(ids, name) =>
                  void edit(async () => {
                    const id = await audio.combineTracks(ids, name);
                    setRecordTarget(id);
                  })
                }
                onSeek={(frame) => void run(() => audio.seek(frame))}
              />

              <div className="lower-workspace">
                <RecordedEffects
                  track={session.tracks.find((t) => t.id === selectedTrack)}
                  disabled={
                    !desktop || busy || status.playing || status.recording
                  }
                  onApply={(id, settings) =>
                    void edit(async () => {
                      const next = await audio.processTrack(id, settings);
                      setSelectedTrack(next);
                    })
                  }
                />
                <aside className="levels">
                  <div className="section-heading">
                    <div>
                      <span className="eyebrow">SIGNAL CHECK</span>
                      <h2>Levels</h2>
                    </div>
                    <AudioLines size={18} />
                  </div>
                  <label className="input-trim">
                    Input trim <strong>{Math.round(inputTrim * 100)}%</strong>
                    <input
                      aria-label="Input trim"
                      type="range"
                      min={0}
                      max={1}
                      step={0.01}
                      value={inputTrim}
                      onChange={(e) => setInputTrim(Number(e.target.value))}
                    />
                  </label>
                  <Meter label="INPUT" value={status.inputPeak} />
                  <Meter label="OUTPUT" value={status.outputPeak} />
                  <div className="meter-scale">
                    <span>−60</span>
                    <span>−24</span>
                    <span>−12</span>
                    <span>0 dB</span>
                  </div>
                  <div className="engine-detail">
                    <span>
                      Sample rate{' '}
                      <strong>{config.sampleRate.toLocaleString()} Hz</strong>
                    </span>
                    <span>
                      Requested buffer{' '}
                      <strong>{config.bufferFrames} frames</strong>
                    </span>
                    <span>
                      Underrun samples <strong>{status.underruns}</strong>
                    </span>
                    <span>
                      Overrun samples <strong>{status.overruns}</strong>
                    </span>
                  </div>
                  <p className="level-hint">
                    Use headphones. Choose either KPT Pro direct monitoring or
                    app monitoring to avoid hearing the input twice.
                  </p>
                </aside>
              </div>
              <footer>
                <span>
                  <span className="small-dot" />{' '}
                  {status.running
                    ? 'Listening for your next idea'
                    : 'Ready when you are'}
                </span>
                <span>
                  KUERDAS STUDIO <i>/</i> BUILT AROUND YOUR GUITAR
                </span>
              </footer>
            </>
          )}
        </div>
      </main>
      {settings && (
        <div className="modal-backdrop" onClick={() => setSettings(false)}>
          <section
            role="dialog"
            aria-modal="true"
            aria-labelledby="settings-title"
            className="modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-heading">
              <div>
                <span className="eyebrow">MAKE THE CONNECTION</span>
                <h2 id="settings-title">Audio settings</h2>
              </div>
              <button
                className="icon-button"
                aria-label="Close settings"
                onClick={() => setSettings(false)}
              >
                <X size={20} />
              </button>
            </div>
            <p>
              Select an interface and use headphones. Monitoring is always off
              when the app starts.
            </p>
            {!desktop && (
              <div className="notice">
                Audio devices are available in the Tauri desktop app.
              </div>
            )}
            <fieldset disabled={busy || status.running}>
              <label>
                KPT Pro input device
                <select
                  value={config.inputId}
                  onChange={(e) =>
                    setConfig({ inputId: e.target.value, inputChannel: 0 })
                  }
                >
                  <option value="">Select KPT Pro…</option>
                  {devices
                    .filter((d) => d.direction === 'input' && d.isKptPro)
                    .map((d) => (
                      <option key={d.id} value={d.id}>
                        {deviceLabel(d)}
                      </option>
                    ))}
                </select>
              </label>
              <label>
                Input channel
                <select
                  value={config.inputChannel}
                  onChange={(e) =>
                    setConfig({ inputChannel: Number(e.target.value) })
                  }
                >
                  {channels ? (
                    Array.from({ length: channels }, (_, i) => (
                      <option key={i} value={i}>
                        Channel {i + 1}
                      </option>
                    ))
                  ) : (
                    <option value={0}>No supported channel</option>
                  )}
                </select>
              </label>
              <label>
                Output device
                <select
                  value={config.outputId}
                  onChange={(e) => setConfig({ outputId: e.target.value })}
                >
                  <option value="">Select output…</option>
                  {devices
                    .filter((d) => d.direction === 'output')
                    .map((d) => (
                      <option key={d.id} value={d.id}>
                        {deviceLabel(d)}
                      </option>
                    ))}
                </select>
              </label>
              <div className="settings-grid">
                <label>
                  Sample rate
                  <select
                    value={config.sampleRate}
                    disabled={!sampleRates.length || hasAudio}
                    onChange={(e) =>
                      setConfig({ sampleRate: Number(e.target.value) })
                    }
                  >
                    {sampleRates.length ? (
                      sampleRates.map((rate) => (
                        <option key={rate} value={rate}>
                          {rate.toLocaleString()} Hz
                        </option>
                      ))
                    ) : (
                      <option value={config.sampleRate}>
                        Select compatible devices
                      </option>
                    )}
                  </select>
                </label>
                <label>
                  Requested buffer
                  <select
                    value={config.bufferFrames}
                    onChange={(e) =>
                      setConfig({ bufferFrames: Number(e.target.value) })
                    }
                  >
                    {[64, 128, 256].map((n) => (
                      <option key={n} value={n}>
                        {n} frames
                      </option>
                    ))}
                  </select>
                </label>
              </div>
            </fieldset>
            <p className="settings-note">
              Rates shown are supported by the selected devices. Sinco-Audio
              uses 44.1 kHz. Different input/output hardware clocks may drift.
              Actual round-trip latency needs a loopback measurement. KPT Pro is
              required for monitoring. The connected model appears as
              Sinco-Audio; Kuerdas also checks its native Jieli USB driver
              identifier.
            </p>
            <div className="modal-actions">
              <button
                className="subtle-button"
                disabled={!desktop || busy || status.running}
                onClick={() => void refresh()}
              >
                <RefreshCw size={14} /> Refresh devices
              </button>
              <button
                className="primary-button"
                onClick={() => setSettings(false)}
              >
                Done
              </button>
            </div>
          </section>
        </div>
      )}
      {workspaceDialog && (
        <WorkspaceNameDialog
          key={`${workspaceDialog.mode}-${workspaceDialog.id ?? ''}`}
          dialog={workspaceDialog}
          busy={busy}
          onClose={() => setWorkspaceDialog(null)}
          onSubmit={(name) => {
            if (workspaceDialog.mode === 'create') void createWorkspace(name);
            else
              void changeWorkspace(async () => {
                const renamed = await audio.renameWorkspace(
                  workspaceDialog.id!,
                  name,
                );
                return workspaceDialog.mode === 'recover'
                  ? audio.openWorkspace(workspaceDialog.id!)
                  : renamed;
              });
          }}
        />
      )}
    </div>
  );
}
