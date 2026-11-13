# Kuerdas Studio

A desktop recording studio for guitarists using KPT Pro, built for macOS Apple Silicon.

Kuerdas helps you turn a guitar idea into a recording: play through your pedal, record takes over a backing track, arrange them in a timeline and export your mix. Your work stays in named local workspaces on your Mac.

## Our goal

Make recording guitar straightforward, from the first take to a finished mix. KPT Pro supplies the guitar tone; Kuerdas provides the workspace to capture, arrange and mix it. You choose sounds on the pedal and can add drive and tone shaping to copies of recorded tracks afterward.

The longer-term direction is a fuller music-production environment. The current focus is reliable guitar recording, backing tracks, arrangement and mixing. See the [roadmap](docs/roadmap.md) for future plans and acceptance criteria.

## What you can do

- **Organize recordings:** create, rename and reopen workspaces with separate tracks and audio.
- **Record your guitar:** capture KPT Pro over USB into a new track or an existing lane.
- **Play with backing tracks:** upload WAV, MP3, M4A, FLAC or Ogg files and use playback, loop and overdubbing.
- **Arrange your takes:** drag clips in time or between tracks, rename, split and combine them. Highlights and translucent overlaps help you inspect the arrangement.
- **Build your mix:** adjust track volume, pan, mute and solo, then export individual tracks or a stereo WAV mix.
- **Process recorded audio:** add drive/EQ to a new copy while keeping the original take.

## Start recording

Use the desktop app on macOS Apple Silicon with KPT Pro connected by USB. The observed unit appears as **KPT Pro · Sinco-Audio** and uses **44.1 kHz**. Headphones are recommended for monitoring.

Follow the [recording quick start](docs/quick-start.md) to create a workspace, connect the pedal and record your first take. Existing recordings from the earlier build are preserved under **Recovered recordings**; name that workspace to open them.

## Project status

Kuerdas is an early recording preview. Workspaces save locally, imports use managed copies, and editing keeps source recordings intact. Back up the complete workspace folder to preserve an editable session; a mix export is only the rendered audio.

Current limits include 16 tracks, 128 clips, approximately 512 MB of decoded audio, a one-hour timeline and ten-minute takes/imports/combined spans. Latency compensation, undo/redo, automatic crash-take recovery, portable project packaging and release signing remain future work. Sustained hardware and latency testing is still required. See [recording details](docs/recording.md).

## Documentation

| Guide                                              | Purpose                                                            |
| -------------------------------------------------- | ------------------------------------------------------------------ |
| [Recording quick start](docs/quick-start.md)       | Workspaces, recording, backing tracks, editing, mixing and backups |
| [Run locally](docs/run-local.md)                   | Prerequisites, desktop launch and browser preview                  |
| [Development](docs/development.md)                 | Stack, repository structure, checks and packaging                  |
| [Architecture](docs/architecture.md)               | Engine boundaries, threading and persistence                       |
| [Recording details](docs/recording.md)             | Storage, media formats, processing and limitations                 |
| [Testing](docs/testing.md)                         | Automated checks and manual workflows                              |
| [Hardware validation](docs/hardware-validation.md) | KPT Pro compatibility and reliability checks                       |
| [Roadmap](docs/roadmap.md)                         | Longer-term development goals and milestones                       |
| [Session notes](docs/session-notes.md)             | Implementation decisions, evidence and open work                   |
