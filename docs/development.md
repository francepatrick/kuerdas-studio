# Developing Kuerdas

For dependencies and launch commands, start with [Run locally](run-local.md). The [main README](../README.md) describes the app's purpose and user-facing features; this guide covers the codebase and engineering workflow.

## Stack

Tauri 2 hosts a React/TypeScript interface styled with Tailwind CSS and app styles. Zustand stores UI preferences. Rust owns CPAL audio streams, DSP, WAV recording/export and workspace persistence; Canvas displays waveform summaries. See [architecture](architecture.md) for responsibilities and data flow.

## Validate changes

```sh
npm run check
npm run format:check
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

These commands check frontend types/build, unit tests, formatting and Rust warnings. See [testing](testing.md) for hardware and manual acceptance checks. Automated checks do not establish physical latency, audible quality or sustained recording reliability.

## Package a local build

```sh
npm run tauri -- build --target aarch64-apple-darwin
```

For the local debug app bundle:

```sh
npm run tauri -- build --debug --bundles app
```

The debug bundle is written to `target/debug/bundle/macos/Kuerdas Studio.app`. Open it from Finder; the built app does not need a running Vite server or installed Rust toolchain.

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

The real-time callback must never wait for React, locks, disk writes, database access or network requests. Stream setup and teardown use a control-thread mutex; callbacks never acquire it. Audio travels through a preallocated SPSC queue, and parameters/meters use atomics. See [architecture](architecture.md) and [audio engine](audio-engine.md).

## References

- [Tauri project configuration](https://v2.tauri.app/reference/config/)
- [CPAL 0.17.3](https://docs.rs/cpal/0.17.3/cpal/)
- [rtrb real-time SPSC buffer](https://docs.rs/rtrb/latest/rtrb/)
