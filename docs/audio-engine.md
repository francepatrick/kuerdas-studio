# Audio engine contract

## First signal path

Selected CPAL input channel → input-owned guitar DSP → dedicated recording-writer queue and separate monitoring queue → output callback mixes monitored guitar with session tracks. Disk recording never depends on output callback progress.

The monitoring queue is allocated at stream start with capacity `4 × requested buffer frames`; no artificial zeros are recorded. This bounds queued latency but does not guarantee a specific round-trip latency. Overflow discards incoming samples and increments the overrun sample counter. Underflow writes silence and increments the underrun sample counter. These are **sample counts**, not callback/glitch counts. Counters reset on a successful configuration/setup attempt. Meters report peak amplitude since the previous status read.

## IPC

| Command            | Request                                                               | Result                                                            |
| ------------------ | --------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `list_devices`     | none                                                                  | Device identifiers, names, directions, channel/rate/format ranges |
| `start_monitoring` | `config: {inputId, outputId, inputChannel, sampleRate, bufferFrames}` | Success only after both streams start                             |
| `stop_monitoring`  | none                                                                  | Streams dropped; safe when stopped                                |
| `set_parameters`   | `drive, output, gate, bypass`                                         | Validated atomic control updates                                  |
| `engine_status`    | none                                                                  | Running, peak levels, sample under/overruns, stream failure       |

KPT Pro must be the selected input, validated natively before opening streams. A native watchdog checks the selected input every second and tears down streams if it disappears, even without UI polling. The UI refreshes discovery every two seconds and never auto-restarts monitoring.

Channels are zero-based over IPC, one-based in the interface. Start always drops an existing stream pair first. Any startup failure leaves the engine stopped. Float32 at 44,100 or 48,000 Hz and 64/128/256 requested frames are the initial supported combinations; unsupported hardware is rejected. Buffer ranges are checked when CPAL reports them, with final validation by stream creation. Parameters reject non-finite numbers and clamp drive to 1–20, output to 0–1 and gate to 0–0.1.

Effects bypass skips the gate and distortion while preserving output gain. The gate uses an amplitude envelope and smoothed opening/closing; drive and output controls use one-pole smoothing to reduce clicks. This starter distortion has no oversampling and can alias at high drive. Amp modeling, anti-aliasing, EQ and convolution need separate implementation and listening/CPU validation.

## Observed KPT Pro USB configuration

On this machine the connected device advertises `Sinco-Audio`, with a CoreAudio Jieli Technology / USB Composite Device UID. It exposes separate stereo input/output endpoints, float32, with **44,100 Hz as the only supported rate**. Discovery matches both the alias and USB UID lineage. UI chooses a common supported rate; 48 kHz is not forced. Gate and gain smoothing are scaled to preserve timing at both supported engine rates. Other hardware revisions may differ; inspect actual ranges before starting.

## Latency and clocks

At 48 kHz, 64/128/256 frames represent 1.33/2.67/5.33 ms **per buffer**. These values are not end-to-end latency: input/output buffering, queue occupancy, conversion and driver safety offsets add time. Measure physical round-trip latency with a loopback, disable hardware direct monitoring, and record median/p95 plus test configuration. Aim for a measured round-trip ≤12 ms for the v0.1 default rig, subject to hardware feasibility; document exceptions rather than silently relaxing this criterion.

The initial same-rate queue does not reconcile independent hardware clocks. A sustained test with separate devices may underrun or overrun even at nominally identical rates. Adaptive sample-rate conversion or a synchronized duplex backend is required before promising reliable independent-device monitoring. Queue correction must not introduce unbounded latency.

## Recording implementation (initial M5 slice)

The initial implementation uses a preallocated two-second mono SPSC sample queue and a background hound writer. The input callback sends processed samples directly to the writer queue, before any monitoring/output mixing; output queue faults cannot invalidate disk capture. See recording.md for current implementation details. A future block pool will improve batching. If the writer falls behind, surface a recording fault; never report a complete take after dropping samples. A background `hound` writer finalizes WAV headers on Stop; session metadata is atomically saved after a successful take. A future recovery journal will reconstruct incomplete takes; this is not implemented. Disk-full, permission errors and interrupted writes must be tested.

## Verification

Rust unit tests cover non-finite input, output bounds, bypass gain and invalid controls. Hardware verification is recorded in [hardware-validation.md](hardware-validation.md). Before declaring v0.1 ready, include channel routing, configuration failures, parameter changes while playing, 30-minute stability and device-disconnect behavior. CI compilation cannot prove callback deadlines or audible DSP quality.
