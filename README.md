# Kuerdas Studio

A KPT Pro recording studio for macOS Apple Silicon. Built with Tauri 2, React, TypeScript and a Rust audio engine.

## Current build · recording preview

Start with the [recording quick start](docs/quick-start.md). See [implementation session notes](docs/session-notes.md) for delivered behavior, validation and remaining work.

- KPT Pro required for the live engine; recognizes the observed **Sinco-Audio** USB identity at **44.1 kHz**.
- Guitar recording to mono float32 WAV using a bounded background writer. Stop saves each take as a new track.
- Upload one or multiple WAV, MP3, M4A (AAC/ALAC), FLAC or Ogg tracks, including file drops; playback, loop, rewind and overdub at the playhead.
- Multi-clip Canvas timeline: drag clips in time/between tracks, rename tracks/recordings, split at the playhead, and combine selected audio into a new track while retaining the originals.
- Empty tracks and selectable recording destinations, per-track volume/pan/mute/solo, track and stereo-mix WAV export.
- Named local workspaces with independent saved tracks, managed imports and takes. Create or open a workspace before recording.
- Clear selected-track/clip highlights, transparent overlapping waveforms and overlap markers.
- Optional drive/EQ on a processed copy of recorded audio; original recordings remain available.

The KPT Pro supplies the live guitar tone; Kuerdas preserves its USB signal apart from input trim. Choose hardware patches on the pedal. There is no tone library or Development Plan in the app; the longer-term plan remains in [docs/roadmap.md](docs/roadmap.md). Recorded-audio processing provides basic drive and tone shaping, not proprietary amp/cabinet simulation.

This is an early recording workflow, **not a finished DAW**. Files are cached in memory with 16-track / 128-clip / 512 MB decoded-session limits, a one-hour timeline, and ten-minute import/take/combine-span limits. There is no latency-compensated overdubbing, crash-take recovery, portable project import/export, advanced clip editing, undo/redo, cabinet convolution or hardware preset control yet. Physical round-trip latency and sustained hardware recording still need testing. See [recording details](docs/recording.md) and the [remaining plan](docs/roadmap.md).

## Run

Prerequisites: Node.js 22.12+ (or a newer supported LTS), npm, a stable Rust toolchain and Xcode command-line tools. Install Rust from [rustup](https://rustup.rs/) if needed. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for native dependencies.

```sh
npm ci
npm run desktop
```

For the interface alone:

```sh
npm run dev
```

Open http://127.0.0.1:1420. In the desktop app, create a custom-named workspace (or open an existing one), then open **Audio settings**, refresh devices, connect KPT Pro over USB, select its input, choose output and a mono input channel, then click **Start monitoring**. Begin with headphones and low interface volume. Choose either interface direct monitoring or app monitoring to avoid doubled input. Unsupported formats/rates return an explicit error; there is no silent resampling.

The connected KPT Pro is advertised by macOS as **Sinco-Audio**, with a Jieli USB Composite Device CoreAudio identifier. Kuerdas matches that exact name plus USB driver identity, in addition to explicit KPT Pro model names. It does not accept arbitrary USB audio. Sinco-Audio reports stereo float32 at **44.1 kHz only**; the settings select a rate shared by the selected input/output. A native watchdog checks the selected KPT Pro every second and stops monitoring on disconnection, independently of React. Reconnection requires explicit monitoring restart.

The alias/driver combination was observed on this setup; full USB vendor/product enforcement and other hardware revisions remain to be validated. It is a compatibility check, not hardware authentication.

See [testing](docs/testing.md) for a quick manual walkthrough.

## Validate and package

```sh
npm run check
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri -- build --target aarch64-apple-darwin
```

The macOS build generates local app/DMG artifacts. Distribution signing and notarization are release work; an unsigned local build is not a signed release. CI checks the frontend and native workspace on Apple Silicon; no hardware audio tests are claimed by CI.

## Repository

| Path                        | Responsibility                                                             |
| --------------------------- | -------------------------------------------------------------------------- |
| `src/`                      | React UI, feature views, Zustand preferences, Tauri bridge                 |
| `src-tauri/`                | Desktop configuration, input permission, command handlers                  |
| `crates/kuerdas-audio/`     | Stream lifetime, bounded routing, atomic meters and diagnostics            |
| `crates/kuerdas-dsp/`       | Allocation-free noise gate and overdrive                                   |
| `crates/kuerdas-devices/`   | CPAL discovery and device resolution                                       |
| `crates/kuerdas-project/`   | Versioned session/track/clip model                                         |
| `assets/impulse-responses/` | Reserved for licensed cabinet IRs                                          |
| `docs/`                     | Architecture, implementation plan, audio contracts and hardware test sheet |

## Engineering contract

The real-time callback must never wait for React, locks, disk writes, database access or network requests. Stream setup and teardown use a control-thread mutex; callbacks never acquire it. Audio travels through a preallocated SPSC queue, and parameters/meters use atomics. See [architecture](docs/architecture.md) and [audio engine](docs/audio-engine.md).

## References

- [Tauri project configuration](https://v2.tauri.app/reference/config/)
- [CPAL 0.17.3](https://docs.rs/cpal/0.17.3/cpal/)
- [rtrb real-time SPSC buffer](https://docs.rs/rtrb/latest/rtrb/)
