# Recording, backing tracks and workspaces

## Use the desktop build

1. On the start screen, enter a custom workspace name and click **Create workspace**, or open a saved workspace. Earlier recordings appear as **Recovered recordings**; choose **Name & open** to preserve them in a named workspace. The sidebar only manages workspaces.
2. Connect KPT Pro and open **Audio settings**. Select **KPT Pro · Sinco-Audio**, the correct input channel and your output. The observed model uses 44,100 Hz. Start with headphones and low physical volume.
3. Select your tone on the physical KPT Pro. The live signal bypasses software effects; **Input trim** controls capture/monitoring level. Choose either pedal direct monitoring or app monitoring to avoid doubled sound.
4. Choose **Upload audio**, select one or multiple mono/stereo WAV, MP3, M4A (AAC/ALAC), FLAC or Ogg files, or drop files into the native app. Each file can be up to ten minutes. Idle monitoring is stopped for structural edits; stop playback/recording first. Imports become managed float WAV copies at the session rate off the audio callback.
5. Press **Play** to audition. **Loop** repeats the existing arrangement for practice. Volume/pan/mute/solo apply independently to each track.
6. Stop and rewind, or click the stopped timeline to choose a position. Choose **Record to → New track per take**, or choose an existing track / click its **R** button. Press **Record** to create a mono guitar clip while existing tracks play. Recording does not wrap at the loop boundary; it continues on a linear timeline.
7. Press **Stop** to drain the writer and finalize WAV. A waveform and new take track appear. Rewind to audition, or record another overdub. Mute/solo existing takes as needed.
8. Export any track or export a stereo WAV mix. The workspace is saved automatically; choose it from the start screen/sidebar on the next launch.

## What is recorded

The KPT Pro input callback processes the selected mono USB channel and sends it directly to the recording writer, independently of the output-device clock and monitoring buffer. A separate queue supplies that processed signal to the output callback, where backing tracks and previous takes are mixed. The app's playback mix is therefore not intentionally printed into a new take. Hardware USB-loopback/mix settings can still feed computer playback back into the input; verify/disable those on KPT Pro if the backing track appears in your take. This has not been characterized for every firmware version.

Live capture bypasses the software gate/drive/EQ/delay and preserves input trim. Stereo KPT input recording is not yet implemented; choose the mono input channel carrying your guitar.

To add effects afterward, click a track, adjust **Recorded audio effects**, then **Create processed copy**. Drive, bass, treble and output are rendered off the real-time callback into a new stereo WAV. The copy retains the source timeline position and volume/pan, and the original track is kept and muted. Unmute it and mute the copy to compare. Stop playback/recording before processing. This is basic drive/EQ, not a cabinet/amp model or hardware patch recreation.

## Audio and disk safety

- CPAL output frames drive playback; captured input frames drive recording position. React displays an atomic position; browser timers never schedule playback or recording.
- A preallocated two-second mono SPSC queue connects callback to the `hound` writer worker. No disk, blocking lock, IPC, allocation or thread join happens in the audio callback.
- On Stop, streams are dropped before signaling the writer to drain/finalize, so the producer cannot race finalization. Normal app quit finalizes a take too; takes can append to an existing track without replacing its clips.
- Recording-writer queue overflow and writer/stream faults invalidate the take; the UI reports the retained file path rather than adding a successful track with hidden sample loss. A ten-minute take limit is explicit. Incomplete files need manual inspection; automatic recovery is not implemented.
- Finalized media are immutable. Session metadata uses temp-write, flush/sync and rename. Removing a track only removes its session reference; it does not delete the WAV.
- Monitoring queue underruns/overruns are counted but do not stop or corrupt the independently captured take. They are not reported as device disconnections.
- Input/output hardware clocks can drift. Prefer the same physical interface for input/output, and monitor sample under/overrun counters. Adaptive clock correction remains future work.

## Storage and limits

Named workspaces live under macOS app data `~/Library/Application Support/studio.kuerdas.desktop/Workspaces/<id>/`. `workspace.json` holds the custom name; `session.json` references managed WAV files by basename. Workspace names never become filesystem paths. Starting the app does not open or monitor audio automatically. Create/open a workspace before importing, editing or recording; switching is disabled during recording/playback and stops idle monitoring.

On first upgrade, a nonempty legacy `Sessions/current/` is copied to a recovered workspace once. Its original directory and files are retained untouched. The recovered copy must be named before opening. Schema-v1 sessions retain `session-v1-backup.json` before their first schema-v2 edit. Imports never modify their external source. Preserve the whole workspace folder when making manual backups; portable project packaging remains future work.

Session rate locks after the first imported/recorded track. New imports are converted to that rate using basic linear interpolation. This is an initial resampler, not mastering-grade conversion. Up to 16 tracks, 128 clips and 128 million decoded float samples (~512 MB) are cached in memory; each input file and take is limited to ten minutes. Streaming long backing files is future work. Waveforms use bounded worker-generated peak summaries rather than original PCM over IPC.

Per-track pan uses linear stereo balance (mono is duplicated at center). Mute takes precedence over solo; if any track is soloed, non-solo tracks are silent. Live mixer changes ramp across a callback block. Playback sums in float and clamps final stereo output to ±1; lower track gains to leave headroom. Export uses current gains/pan/mute/solo and starts at frame zero, including leading offsets, with no live input and no loop repetition. Mix exports are finalized to a temporary file before replacing the chosen destination; individual-track export renders all its clips with volume/pan and timeline offsets into stereo WAV (independent of mute/solo).

## Arrange, rename and combine

- Click a track name to rename it; press Enter or leave the field to save, Escape to cancel.
- Click a track or clip for a visible selection highlight. Overlapping waveforms are translucent; a **MIX ×N** strip identifies simultaneous clips within a lane.
- Click a clip to select it. Drag horizontally to change its start time or vertically to move it into another track. Source WAV data stay unchanged. Snap rounds to 0.1 seconds; Alt-drag temporarily disables it. Arrow keys nudge a focused clip by 0.1 seconds, Shift+arrow by one second.
- The selected-clip inspector offers an exact destination track and numeric start time. Use zoom controls for finer movement or longer views. Starts cannot be negative; all clips must fit within the one-hour timeline.
- Use the pencil on a clip (or double-click its name) to rename a recording independently from its track name.
- Click the ruler/background to place the stopped playhead inside a clip, then **Split at playhead**. Both resulting clips retain source offsets into the same immutable audio file.
- Check the tracks whose clips should be combined (a single track containing multiple clips works too), enter a name and click **Combine selected**. Kuerdas renders all selected clips at their current positions using each track's volume/pan. Selection rendering ignores existing mute/solo states. The new stereo track starts at the earliest selected clip; silent gaps are preserved. Original tracks/files stay in place and are muted to prevent doubled playback. Unmute them to compare or continue editing.
- Combining needs room for one additional track and a selection span of ten minutes or less. Rendered output is clamped to ±1; keep track levels below clipping.
- Structural edits are serialized off the real-time callback and saved transactionally. On save failure, the in-memory layout is restored. Automatic undo/redo is not implemented yet.

## Known timing limits

Overdubs use engine sample positions but **do not yet compensate physical input/output latency**, queue priming or independent-device drift. Newly recorded takes can need alignment adjustments; clip dragging or the numeric Start field can correct alignment manually. Do not describe this as sample-aligned production overdubbing until loopback measurement and compensation are implemented. Live listening, long recording, disconnect, disk-full and sleep/wake checks remain required before release.

## Tests

Native fixtures cover complete writer drain/finalization, error-on-sample-loss, WAV resampling/channel preservation, sample offsets, path validation, record-stop track creation/tone metadata, current-session reopen, mute/solo/pan precedence and deterministic stereo mix export. Hardware acceptance remains in hardware-validation.md.
