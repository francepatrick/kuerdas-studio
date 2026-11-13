# Run Kuerdas locally

This guide is for launching the app from source. To use an already-built desktop app, follow the [recording quick start](quick-start.md). For code changes, checks and packaging, see [development](development.md).

## Prerequisites

- macOS Apple Silicon, the current native target.
- Node.js 22.12+ or a newer supported LTS, with npm. CI uses Node.js 24.
- A stable Rust toolchain with Cargo. Install from [rustup](https://rustup.rs/) if needed.
- Xcode command-line tools and the native dependencies in [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
- KPT Pro connected by USB for live audio, plus a selected output device. The browser preview does not need the pedal.

Run the following commands from the repository root.

## Desktop app

```sh
npm ci
npm run desktop
```

The desktop command starts the Vite frontend and Tauri app together. Create a named workspace, open **Audio settings**, choose the KPT Pro input and your output, then follow the [recording guide](quick-start.md). Allow microphone access if macOS requests it. Monitoring starts only when explicitly enabled.

The observed KPT Pro advertises **Sinco-Audio** with a Jieli USB Composite Device CoreAudio identifier. It supports stereo float32 at **44.1 kHz only**; choose the mono input channel carrying your guitar and a rate supported by both input and output. The engine also supports 48 kHz for compatible devices, but generic non-KPT inputs are blocked. Other hardware revisions still need validation.

## Browser interface preview

```sh
npm ci
npm run dev
```

Open http://127.0.0.1:1420. The browser lets you preview the interface and workspace naming flow. It cannot access native audio, import/record tracks or persist native workspace files; use the desktop app for actual sessions.

## Common setup issues

- **Port 1420 is already in use:** stop a separately running `npm run dev` before starting `npm run desktop`. Tauri starts its own frontend server.
- **Rust/Cargo is missing:** install the stable toolchain and open a new terminal so Cargo is on your PATH.
- **KPT Pro is missing:** reconnect USB, refresh Audio settings and select the KPT input. Arbitrary USB audio devices do not satisfy the KPT requirement.
- **No input signal:** verify the selected mono channel, macOS microphone permission and pedal routing. Choose either hardware direct monitoring or app monitoring to avoid doubled audio.
- **Audio stops after disconnect:** reconnect, reselect devices if needed and restart explicitly. Monitoring never resumes automatically.

For a local macOS app bundle without a development server, see [packaging](development.md#package-a-local-build). Existing workspaces live in app data, outside the repository; see [backups](quick-start.md#save-and-back-up) before manual maintenance.
