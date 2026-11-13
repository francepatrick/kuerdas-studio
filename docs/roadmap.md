# Kuerdas Studio · v1.0 implementation plan

Owner: solo developer · Target: macOS Apple Silicon · Estimate: 14 focused development weeks.

This is an editable delivery plan, not a promise of completed functionality. Hardware behavior, DSP quality and available weekly hours may extend the schedule. Check a task only after its acceptance criteria and evidence are recorded. Relative weeks begin at project kickoff, not a fixed release date.

## Current product scope — recording studio

The user narrowed the current app to KPT Pro recording and mixing. The live tone library, software-recreation pedalboard and in-app Development Plan have been removed. Live input preserves the pedal sound apart from input trim; optional drive/EQ works on a copy of recorded audio. Required custom-named workspaces replace the single untitled session. Sidebar navigation contains workspaces only. Track/clip selection and translucent overlap visualization support arranging a mix.

The original long-term phases below remain documentation for future consideration; they do not imply a live-effects or tone-library feature in the current app. New work should prioritize reliable recording, workspace persistence, backing audio, arrangement and mixing.

## Product goals

Kuerdas is a GarageBand-inspired desktop DAW, initially focused on electric guitar recording and software effects, with KPT Pro required as the live-monitoring input (user requirement, 2026-11-13). Long-term scope includes multitrack production, plugins and MIDI instruments.

v1.0 succeeds when it can:

- Discover audio interfaces and require KPT Pro as the selected monitoring input; allow explicit output routing. Broader input-interface support is deferred.
- Monitor live guitar through software effects with measured, acceptable latency.
- Build and reuse amp/cabinet/pedalboard tones.
- Record and play multiple guitar and backing tracks with aligned overdubs.
- Edit, mix, save, reopen and export projects without losing audio.
- Run reliably on Apple Silicon through sustained recording, disconnects and sleep/wake.

## Stack and architectural constraints

| Layer       | Technology                | Responsibility                        |
| ----------- | ------------------------- | ------------------------------------- |
| Desktop     | Tauri 2                   | Native shell and commands             |
| Frontend    | React + TypeScript        | DAW UI and interactions               |
| Styling     | Tailwind CSS + app styles | Interface design                      |
| State       | Zustand                   | UI/project interaction state          |
| Audio I/O   | Rust + CPAL               | Native streams and routing            |
| DSP         | Rust                      | Guitar effects and mixing             |
| Audio files | WAV + hound (planned M5)  | Recording and export                  |
| Database    | SQLite (planned M4)       | Presets, metadata, preferences        |
| Waveforms   | Canvas                    | Timeline visualization                |
| Tests       | Vitest + Rust tests       | State, audio logic, integration       |
| CI/CD       | GitHub Actions            | Automated checks and build validation |

**Hard rule:** the real-time audio thread never waits for React, disk, database, network or blocking locks. Allocate buffers off-thread; use bounded real-time-safe communication and deferred graph reclamation.

## Delivery gates

| Release   | Scope                                                     | Required evidence                                                                   |
| --------- | --------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| v0.1      | Live monitoring + basic gate/drive                        | Measured latency, correct channel routing, 30-minute stable hardware test           |
| v0.5      | Full pedalboard, amp/cab IRs, reusable presets            | DSP benchmarks, graph-change stability, preset/IR round trips                       |
| v1.0 beta | Multitrack recording/editing/mixing, projects, WAV export | Recovery tests, export reference comparison, signed/notarized Apple Silicon package |

The current repo now also implements basic WAV recording, backing import/playback, linear overdubbing, waveform summaries, per-track mixer controls, current-session autosave/reopen and track/mix WAV export, plus KPT Pro reference/software tone modes. These are early implementations; latency compensation, long-session/fault acceptance and advanced editing/undo remain open. Hardware acceptance remains open. Presets currently use local UI storage; this does not complete M4's portable/native persistence work.

## M1 · Foundation & device discovery · Week 1

Dependencies: none. Outcome: the desktop app detects audio hardware.

- [x] **F01** Initialize Tauri 2, React/TS, Tailwind and Zustand with separate audio/DSP/device/project crates.
  - Acceptance: frontend compiles, native workspace compiles, app launches on Apple Silicon; required checks run in CI.
- [x] **F02** Build the DAW shell, empty arrangement, tone controls, audio settings and signal meters.
  - Acceptance: settings/presets/controls are operable; unavailable recording functions are labeled; browser preview never fabricates a connected interface.
- [x] **F03** Implement device enumeration and explicit selection using device identifiers.
  - Acceptance: input/output devices expose name, channels and supported sample-rate/format ranges; a missing selection fails rather than silently choosing another device.
- [x] **F03a** Require KPT Pro for live monitoring in Rust and UI; stop when it disconnects.
  - Acceptance: other selected inputs cannot start streams; disconnected state disables monitoring; native watchdog works independently of React. Observed Sinco-Audio + Jieli USB UID matching added after hardware discovery; full acceptance remains under F04.
- [ ] **F04** Verify KPT Pro over USB and complete the hardware test sheet.
  - Acceptance: record device identity, input/output channels, formats, rates and requested-vs-observed buffer behavior; test dry/bypass routing; record whether USB audio is processed only. Do not infer compatibility from the product name.

**Gate:** native discovery smoke test passes with the actual KPT Pro. Other inputs do not satisfy the connection requirement. Record evidence in hardware-validation.md. F01–F03 are implemented; hardware checks remain necessary before closing M1.

## M2 · Real-time engine · Weeks 2–3

Depends on M1. Outcome: hear live guitar through Kuerdas.

- [x] **A01** Create explicit input/output streams with bounded SPSC routing and mono input selection.
  - Acceptance: supported float32/44.1 or 48 kHz configurations start; 64/128/256-frame requests validate; failed start leaves streams stopped; no callback locks/allocations/I/O.
- [x] **A02** Add opt-in monitoring, peak meters, under/overrun sample counters and stream failure flags.
  - Acceptance: UI polling never blocks callbacks; stopped engine outputs no live signal; stream errors silence output and require explicit restart.
- [ ] **A03** Add further device sample formats/rates and a documented configuration negotiation policy.
  - Acceptance: conversions tested at numeric extremes; mismatches rejected clearly or handled by explicit conversion; selected channel remains valid.
- [ ] **A04** Resolve independent-device clock drift with synchronized duplex I/O or bounded adaptive resampling.
  - Acceptance: 30-minute separate-device test shows no cumulative queue growth, no periodic drift glitches and no unbounded latency.
- [ ] **A05** Measure physical round-trip latency and callback CPU deadlines.
  - Acceptance: default gate/drive rig targets ≤12 ms measured round-trip on reference hardware, with median/p95 and buffer details recorded; 30-minute test has no steady-state sample losses; document hardware limits.

**Gate v0.1:** hear real guitar with software distortion reliably; do not proceed to broad DAW feature work before this is demonstrated.

## M3 · Guitar DSP & pedalboard · Weeks 4–6

Depends on M2. Outcome: playable software guitar rig.

- [x] **D01** Starter envelope gate, smoothed drive/output and effects bypass.
  - Acceptance: finite bounded samples for pathological inputs; bypass preserves polarity and output gain; no per-sample allocations.
- [ ] **D02** Implement musical noise-gate attack/release/hysteresis, EQ and compressor.
  - Acceptance: sample-rate-aware coefficients, response fixtures, no NaN/instability at parameter limits; listening checks at low/high guitar levels.
- [ ] **D03** Improve overdrive with oversampling/anti-alias filtering and add amp gain stages/tone stack.
  - Acceptance: amplitude/frequency/aliasing measurements plus listening comparisons; gain staging avoids unintended output jumps.
- [ ] **D04** Add cabinet IR loader and efficient partitioned convolution.
  - Acceptance: off-thread decode/resample/preparation; impulse response matches a direct-convolution reference; long IR CPU/memory bounded; no callback allocations.
- [ ] **D05** Implement delay, chorus and reverb.
  - Acceptance: sample-rate-aware timing; stability at extremes; tails handled on bypass, transport changes and project switches.
- [ ] **D06** Enable per-effect bypass and reordering through prepared DSP graphs.
  - Acceptance: swaps occur at block boundaries, smoothing/crossfades prevent clicks; old graph freed off-thread; stress rapid reorder while playing.

**Gate:** representative full rig stays within 50% of the callback deadline at the agreed reference buffer on Apple Silicon, with documented tone and DSP quality checks.

## M4 · Presets & device settings · Week 7

Depends on M3; schema planning may overlap. Outcome: reusable guitar tone library.

- [ ] **P01** Define versioned preset JSON with strict finite/range/schema validation and migration policy.
  - Acceptance: export/import preserves graph order, controls and references; malformed/future versions fail clearly without changing the active tone.
- [ ] **P02** Add native SQLite preset/metadata/preferences repository and migrations.
  - Acceptance: transaction rollback and migration fixtures pass; DB never touched from callbacks; device choices persist but monitoring always starts off.
- [ ] **P03** Build amp/pedal editors, preset CRUD/duplicate/search and dirty-state feedback.
  - Acceptance: edited tone can be saved under a new name, reopened and compared; factory tones cannot be accidentally overwritten.
- [ ] **P04** Import/manage IR assets with licensing/source metadata and missing-file handling.
  - Acceptance: invalid/channel/length cases rejected; relative managed references survive project move; missing asset prompts repair.

**Gate v0.5:** full pedalboard + amp/cab tone saves, reloads and sounds equivalent after restart.

## M5 · Multitrack recording · Weeks 8–10

Depends on reliable M2 engine; use M3/M4 for tone rendering. Outcome: rhythm/lead over a backing track.

- [ ] **R01** Implement bounded block pool and background `hound` WAV writer.
  - Acceptance: no disk I/O/allocation in callback; disk-full/slow disk surfaces take failure; finalized headers are valid; no silent sample drops.
- [ ] **R02** Add sample-clock transport, record arm, stop and seek, plus track playback.
  - Acceptance: start/stop/seek sample positions deterministic; React timers never drive audio position; tail/end-of-file cases safe.
- [ ] **R03** Import backing WAV tracks with metadata validation and worker decoding/resampling.
  - Acceptance: imports preserve source media; unsupported files report an error; long files do not freeze UI or exhaust memory.
- [ ] **R04** Generate cached multiresolution waveform peaks off-thread; draw viewport with Canvas.
  - Acceptance: waveform aligns to recorded samples, zoom/scroll remains responsive for long sessions, no original audio decoding on UI frame loop.
- [ ] **R05** Implement aligned overdubbing and monitoring routing policy.
  - Acceptance: loopback alignment within one frame after compensation, repeated takes stay aligned, no doubled software/direct monitor path.
- [ ] **R06** Add non-destructive move/trim/split/delete and snap, with immutable source offsets.
  - Acceptance: edits preserve audio; sample-accurate boundaries and overlapping clips have specified behavior; undo/redo compatible.

**Gate:** record separate rhythm and lead takes over an imported backing track, replay without loss or alignment drift, complete a 60-minute recording test.

## M6 · Mixing, projects & export · Weeks 11–12

Depends on M5; model already scaffolded. Outcome: usable guitar-focused DAW.

- [ ] **S01** Add per-track gain/pan/mute/solo and master bus headroom/metering.
  - Acceptance: pan law specified; solo/mute precedence tested; automation-free controls smoothed; clipping visible.
- [ ] **S02** Save/reopen portable project directories with relative audio/IR references.
  - Acceptance: full session round trip, moved project reopens, missing media repair, version validation and atomic metadata writes pass.
- [ ] **S03** Add command-based undo/redo for edits, mixer and tone/project changes.
  - Acceptance: branching after undo clears redo; media referenced by history retained; bounded history memory.
- [ ] **S04** Add autosave, interrupted-take recovery and crash-safe checkpoints.
  - Acceptance: force termination during capture/save and recover valid audio/metadata; original project remains intact; explicit recovery UI.
- [ ] **S05** Offline stereo WAV export using `hound` and the same processing graph.
  - Acceptance: correct sample count/rate/channel format, deterministic reference mix, tail length/pan/mute/solo honored, cancel and disk-full paths safe.

**Gate:** create a session, record/import/edit/mix, save/close/reopen, export and compare rendered audio to expected output.

## M7 · Stability & release · Weeks 13–14

Depends on all preceding gates. Outcome: Kuerdas v1.0 beta.

- [ ] **Q01** Profile latency, callback deadline, memory and long-session CPU with full rigs/multitrack projects.
  - Acceptance: reference-machine performance budget documented; one-hour stress session without unreported loss; repeatable benchmarks checked in.
- [ ] **Q02** Handle device disconnect, sample-rate change, sleep/wake and reconnect.
  - Acceptance: no crash, no full-volume burst, takes recover/finalize safely, explicit new device choice/restart when needed.
- [ ] **Q03** Complete interaction/accessibility/error-state checks.
  - Acceptance: keyboard workflows, focus management, readable meters/errors and no misleading controls; configuration denial paths work.
- [ ] **Q04** Build, sign and notarize macOS Apple Silicon app/DMG with versioned artifacts.
  - Acceptance: Gatekeeper verification on a clean Mac, microphone permission works, release reproducible from lockfiles, credentials only in CI secrets.
- [ ] **Q05** Publish beta notes, known limitations, recovery/help and test matrix.
  - Acceptance: supported interfaces/OS versions grounded in actual tests; release checklist reviewed and remaining blockers recorded.

## Risk register

| Risk                                      | Mitigation / proof                                                                               |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------ |
| KPT Pro exposes only processed USB signal | Test dry/bypass options and document signal; broader input support needs explicit scope approval |
| Monitoring latency                        | Start 48 kHz, test 64–256 frames, measure physical round trip                                    |
| Separate hardware clocks drift            | Prefer one duplex interface initially; add adaptive conversion/synchronized backend              |
| Audio callback glitches                   | No allocation/locks/I/O; audit and profile callback work                                         |
| Long cabinet IR CPU load                  | Partitioned convolution, benchmark full rig at target buffer                                     |
| Corrupt/incomplete recordings             | Worker writer, checkpoints, journal, forced-crash recovery tests                                 |
| Device disconnect/sleep                   | Failure flag + silence, controlled teardown, recover takes, explicit restart                     |
| Feedback/doubled monitoring               | Explicit devices, opt-in monitoring, headphone/direct-monitor guidance                           |
| Scope exceeds solo schedule               | Gate v0.1 first; defer MIDI/plugin hosting and advanced automation                               |

## Deferred after v1.0

MIDI instruments, VST3/AU hosting, advanced automation, direct KPT Pro preset control, other desktop OS release support and non-WAV formats. Keep these outside beta acceptance unless the scope is explicitly revised.

## Evidence and handoff

For each milestone, append date, commit/build, reference hardware, checks run, observed results and unresolved defects to the test sheet or a linked issue. CI proves build/unit behavior only. Never mark hardware or listening acceptance complete from mock data. First next task: connect KPT Pro and complete F04/A05 before expanding the DAW.

## Recording preview implementation update

Implemented initial portions of R01–R05 and S01/S02/S05: bounded WAV writer, sample-clock transport, WAV import/basic rate conversion, cached peak waveforms, linear overdubs, gain/pan/mute/solo, atomic current-session metadata with restart restoration, track and stereo WAV export. Unit/integration fixtures pass; the milestone checkboxes remain open where broader acceptance (stress, alignment, disk faults, portability, editing, recovery) is not met. See recording.md for exact supported behavior. Tone library now supports physical KPT Pro preset references and editable software approximations; no factory list or remote patch control is claimed.

## Track-editing implementation update

Implemented schema-v2 multi-clip tracks, track/clip renaming, sample-position drag between lanes, numeric placement, snapping/nudging, source-offset splitting, empty tracks and recording destinations, combine-selected render with retained/muted originals, and multi-file upload/drop for WAV/MP3/M4A/FLAC/Ogg. Existing v1 sessions get a metadata backup before migration is saved. Automated fixtures cover edit persistence, offsets, original-file preservation, combine mix/positions, migration and compressed-format decoding. Native interaction and long-session acceptance remain distinct from unit checks.
