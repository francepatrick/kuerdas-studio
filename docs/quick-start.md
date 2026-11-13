# Recording quick start

## Create your workspace

Open the desktop app, enter your own workspace name and choose **Create workspace**. The sidebar lists your saved workspaces; use the pencil to rename one. Each workspace keeps its own tracks, clips and audio files.

If upgrading from the earlier build, select **Recovered recordings** and give it a name. Kuerdas copies the earlier session into a workspace and leaves the original files intact.

## Connect KPT Pro

Connect the pedal by USB. In **Audio settings** (top-right sliders), choose **KPT Pro · Sinco-Audio**, the input channel carrying your guitar and your output device. The observed pedal supports **44,100 Hz**; start with **128 frames**. Allow microphone access if macOS asks.

Choose your guitar sound on the physical pedal. Kuerdas records that USB signal with input trim; it does not change hardware patches. Use headphones and low physical volume. Choose either hardware direct monitoring or the app's **Monitor off/on** button to avoid hearing the input twice.

## Record with a backing track

1. Choose **Upload audio** or drop audio files into the desktop app. WAV, MP3, M4A, FLAC and Ogg are supported. Each file becomes a separate track.
2. Press **Play** to check the backing. Stop, then use **Return to start** or click the timeline to choose where to record.
3. Under **Record to**, choose **New track per take** or an existing track. A track's **R** button also selects the recording destination.
4. Press the red circle **Record** and play. Existing tracks play alongside your guitar.
5. Press the square **Stop** to finalize the WAV and display the take. Return to the start and press **Play** to listen.

**Loop** repeats playback for practice. Recording continues on a linear timeline rather than wrapping at the loop boundary. Stop playback/recording before importing, processing, switching workspaces or making structural edits.

## Arrange and mix

- Click a track or clip to highlight it. Drag clips horizontally to change timing or vertically between tracks. Translucent waveforms and **MIX ×N** strips show overlapping clips within a lane.
- Click a track name to rename it. Use a clip's pencil to rename that recording. **New track** adds an empty destination.
- Select a clip to enter an exact start time or split it at the stopped playhead. **Snap 0.1 s** helps alignment; hold Alt while dragging to bypass snap.
- Adjust **Vol**, **Pan**, **M** (mute) and **S** (solo). Lower levels if several tracks clip when mixed.
- Check tracks to combine, enter a name and choose **Combine selected**. The new track preserves timing and gaps; originals remain and are muted.
- To add drive/EQ after recording, click a track and use **Recorded audio effects → Create processed copy**. The original audio remains unchanged and its track is muted. Mute the copy and unmute the source to compare.
- Use a track's download button for its WAV, or **Export mix** for a stereo WAV of the arrangement.

## Save and back up

Workspace changes save automatically. Reopen a workspace from the sidebar/start screen after launching the app. Audio monitoring never resumes automatically.

On macOS, back up the complete workspace folder under:

```text
~/Library/Application Support/studio.kuerdas.desktop/Workspaces/<id>/
```

Keep `workspace.json`, `session.json` and all managed audio files together. Exporting a mix does not create an editable project backup. Removing a track retains its managed source WAV.

## Current limits

This is an early recording build: 16 tracks, 128 clips, approximately 512 MB of decoded session audio, a one-hour timeline, and ten-minute takes/imports/combined spans. Overdubs do not yet compensate physical device latency; drag or numerically shift a take if alignment needs correction. Undo/redo, automatic crash-take recovery and portable project packaging remain future work.

If the pedal is missing, reconnect USB, refresh Audio settings and reselect input/output. Confirm 44.1 kHz for Sinco-Audio. A stopped stream needs an explicit restart. See [testing](testing.md), [recording internals and limits](recording.md) and the [hardware validation sheet](hardware-validation.md) for more detail.
