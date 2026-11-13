# Implementation session notes

## 2026-11-13 — recording studio foundation

Delivered the first Tauri/React/Rust recording build for macOS Apple Silicon. The current product is a KPT Pro recording studio with named local workspaces, backing audio, multitrack recording, arrangement, mixing and WAV export.

### Decisions

- Require KPT Pro for live audio. The observed unit advertises Sinco-Audio with a Jieli USB CoreAudio identifier and supports 44.1 kHz; generic USB devices are not accepted by the alias check.
- Preserve the pedal's live tone apart from input trim. Remove the tone library, live pedalboard and Development Plan from the app. Keep longer-term plans in documentation.
- Require a custom-named workspace before starting. Isolate each workspace's session and media. Copy the legacy session once into a recovered workspace without modifying its originals.
- Capture recording samples directly from the input callback into a dedicated bounded background-writer queue. Monitoring queue faults are diagnostics, distinct from recording loss or device faults.
- Keep source audio immutable. Moves/splits edit metadata; combine and recorded drive/EQ create new WAVs and retain muted original tracks.

### Delivered workflows

Create/open/rename workspaces; monitor and record KPT Pro; upload multiple common audio formats; play/loop backing tracks; record overdubs to new or existing tracks; rename/move/split clips; combine tracks; adjust volume/pan/mute/solo; render processed copies; export tracks or stereo mixes. Selected rows/clips are highlighted and overlapping waveforms are translucent.

### Validation evidence

- 28 Rust tests and 13 Vitest tests passed; frontend build, formatting and Clippy with warnings denied passed.
- Built and opened the macOS debug app. Verified the workspace chooser, blank-name gate, workspace-only sidebar, KPT Pro selection and recording workspace UI.
- Verified recovery of the existing five tracks/five clips. All 16 files copied from the legacy session matched their original SHA-256 content; the original directory was retained.
- The user created a named workspace and accepted the current build. This does not replace sustained hardware/latency testing.

### Remaining work

Measure round-trip latency and compensate overdubs; handle independent device-clock drift; run long recording, disconnect, sleep/wake and disk-full tests; add undo/recovery and portable project packaging. Signing/notarization remains release work. See [roadmap](roadmap.md) and [hardware validation](hardware-validation.md).
