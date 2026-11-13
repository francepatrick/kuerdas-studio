# Testing Kuerdas

## Quick UI preview

```sh
npm ci
npm run dev
```

Open http://127.0.0.1:1420. Create a named workspace and open Audio settings. Verify the sidebar contains workspaces only and no tone library or Development Plan. Browser preview cannot access native audio and monitoring stays disabled.

## Desktop / KPT Pro test

```sh
npm run desktop
```

Requires stable Rust, Node and Xcode prerequisites from the README. Stop a separately running `npm run dev` first so port 1420 is available. Alternatively, open a locally built `target/debug/bundle/macos/Kuerdas Studio.app` from Finder; Rust is not needed just to run the built app.

1. Create or open a named workspace. With KPT Pro disconnected, expect **Connect KPT Pro to enable audio** and a disabled monitoring button.
2. Connect KPT Pro by USB. Wait up to two seconds for discovery, then open **Audio settings**.
3. Select **KPT Pro · Sinco-Audio** input and your output/headphones, mono input channel, and 128 requested frames. The connected interface requires **44,100 Hz**; Kuerdas selects a shared rate automatically. No other input model is allowed.
4. Allow microphone access when macOS requests it. Keep physical output low and choose either hardware direct monitoring or app monitoring.
5. Click **Monitor off** to enable monitoring. Play guitar; real input/output meters should move. Adjust input trim; the live sound should remain the KPT Pro signal.
6. Click **Monitor on** to disable monitoring. The app's live sound should stop.
7. Start again, disconnect KPT Pro. Expect monitoring to stop, an error and the connection-required state. Reconnect and explicitly restart; it must never resume automatically.
8. Close/reopen. Device/input-trim preferences remain, monitoring is off; choose the named workspace to reopen its tracks.

Initial compatibility: float32 at 44.1/48 kHz with requested 64/128/256-frame buffers. The observed Sinco-Audio alias is accepted only with the Jieli USB Composite Device CoreAudio UID lineage. Other generic USB names remain blocked. Model/driver matching is provisional; it is not proof of physical model authenticity.

## Recording / backing-track smoke test

1. Stop audio and use Upload audio to import one or multiple backing tracks. Verify its waveform, stereo routing and duration.
2. Play, stop, rewind and loop; transport time must follow audio. Confirm track gain, pan, mute and solo.
3. Rewind and record guitar over the backing track. Press Stop; verify a new mono take and saved waveform.
4. Solo the take: confirm Kuerdas has not mixed the backing into its recording. If backing remains, investigate the pedal's USB-loopback routing.
5. Rewind and record another take, then mix both with the backing. Check uncompensated input latency explicitly; alignment compensation is not implemented.
6. Export the take and stereo mix and inspect sample count/rate in another audio tool. Restart the app and verify tracks, controls and source notes reopen.
7. Select a recorded track and create a processed copy with drive/EQ. Confirm its placement, source mute, and unchanged original audio; compare by muting the copy and unmuting the source.

Use [recording.md](recording.md) for storage, file/memory limits, error behavior and remaining production limitations. Use the full [hardware sheet](hardware-validation.md) for sustained operation and faults.

## Automated checks

```sh
npm run check
npm run pretty:check
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

These check UI types/build, supported input-channel selection, KPT Pro eligibility, model-name matching, finite/bounded DSP, bypass gain and session serialization. They do not prove device compatibility, low latency or audible quality.

## Track-editing smoke test

1. Rename an existing track and a recording clip. Reopen and verify names persist.
2. Add an empty track, choose it under Record to (or R), record and Stop. Confirm a new clip lands there without replacing existing clips.
3. Drag a clip horizontally and into another lane; verify playback follows its new location. Test Snap, Alt-drag, arrow nudging and numeric placement.
4. Split a clip at the playhead and move one part. Confirm source offsets and sample continuity survive reopen.
5. Select tracks and Combine selected. Verify a new stereo track, preserved timing/gaps, muted originals and existing source files.
6. Upload multiple WAV/MP3/FLAC/M4A/Ogg files, including a native file drop; reject corrupt/oversized files without changing the current layout.
7. Reopen a schema-v1 session with actual takes: verify `session-v1-backup.json` and migration to schema v2 after an edit. Never remove the user's recordings as part of testing.

## Workspace and selection regression

- Empty/whitespace workspace names must not enable Create. Create two differently named workspaces; tracks added in one must not appear in the other. Rename and restart: each name/session must persist.
- Upgrade with existing recordings: a recovered item should require a name, preserve every track/clip/file, and leave the old Sessions/current untouched. A second launch must not create a duplicate recovery.
- During playback/recording workspace switches are disabled. Stop first; switching from idle monitoring stops streams safely.
- Click track controls and clips: the correct row is highlighted. Place clips on top of each other: translucent waveforms and MIX ×N strips must show overlap, including during drag previews. Touching boundaries are not overlaps.
