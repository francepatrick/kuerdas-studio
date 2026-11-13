# Hardware validation · manual evidence

Status: **not performed**. Complete this with actual hardware; no compatibility or latency result is assumed.

## Environment

| Field                             | Observation               |
| --------------------------------- | ------------------------- |
| Date / tester / commit            | Pending                   |
| Mac model / chip / RAM            | Pending                   |
| macOS version                     | Pending                   |
| Interface model / firmware        | KPT Pro required; pending |
| USB connection / hub              | Pending                   |
| CPAL device identifier/name       | Pending                   |
| Input/output channels             | Pending                   |
| Reported sample formats/rates     | Pending                   |
| Dry/bypass USB routing available? | Pending                   |
| Direct monitor disabled?          | Pending                   |
| Output/headphone destination      | Pending                   |

## Smoke test

- [ ] Desktop launches with microphone usage description.
- [ ] Microphone permission allowed and denied paths tested.
- [ ] KPT Pro enumerates with actual configs; verify advertised name and USB vendor/product identity; unsupported/generic names stay blocked.
- [ ] No KPT Pro: connection-required message and monitoring disabled.
- [ ] Built-in/other input rejected by native start command, even with KPT Pro connected.
- [ ] Reconnecting KPT Pro updates availability without restarting the app; monitoring remains off.
- [ ] Explicit input/output selection; each valid mono input channel verified.
- [ ] Unsupported config/missing device produces actionable error, no fallback output.
- [ ] Monitoring starts only on click and stops immediately on request.
- [ ] Gate/drive/output and effects bypass audibly match behavior.
- [ ] Input/output meters follow the real signal and settle at silence.
- [ ] Restart app: preferences retained, monitoring remains off.

## Latency matrix

Use physical loopback/correlation, direct monitoring disabled, same known impulse and sample rate for every run. Record median and p95 over repeated trials. Requested buffer duration is not round-trip latency.

| Input → output | Rate  | Requested frames | Observed callback frames | Median RTL / p95 | Under/overrun deltas | Result  |
| -------------- | ----- | ---------------- | ------------------------ | ---------------- | -------------------- | ------- |
| Pending        | 48000 | 64               | Pending                  | Pending          | Pending              | Pending |
| Pending        | 48000 | 128              | Pending                  | Pending          | Pending              | Pending |
| Pending        | 48000 | 256              | Pending                  | Pending          | Pending              | Pending |

Target: ≤12 ms measured round-trip with starter rig on reference hardware; document hardware exceptions. Current UI does not report observed callback frames; capture via a non-real-time diagnostics extension/profiler when completing this test.

## Reliability / faults

- [ ] Same-interface 30-minute live monitoring: stable latency and no steady-state dropped samples.
- [ ] Separate-device 30-minute run: quantify drift and queue faults; do not mark solved without adaptive/synchronized routing.
- [ ] Unplug input/output during monitoring: silence, stopped state and useful error, no crash.
- [ ] Reconnect, refresh, explicitly restart: signal returns without volume spike.
- [ ] Sleep/wake and rate change: document current result and defects.
- [ ] Busy CPU test: record callback deadline misses without callback logging.

## Recording/export hardware gates

The initial recording implementation is available; these hardware/fault gates are not yet completed. M5–M7 require long takes, writer backpressure, disk-full/denied writes, crash recovery, overdub alignment, moved project reopen, reference mixdown, signed package and clean-Mac permission verification.

## Findings / blockers

Native discovery on 2026-11-13 (user reports connected KPT Pro):

- Audio name: **Sinco-Audio**; manufacturer from macOS: **Jieli Technology**.
- Separate input/output CoreAudio endpoints, 2 channels each, float32.
- Both endpoints report **44,100 Hz only**, with 14–4096 frame buffer range.
- CoreAudio UID lineage: `AppleUSBAudioEngine:Jieli Technology:USB Composite Device:<serial>:<endpoint>`.
- IORegistry reports USB vendor `0x3654`, product `0x7756`, product label `USB Composite Device`. IDs have not been established as exclusive to KPT Pro, so VID/PID alone is not used as a global allowlist.
- Detection updated to match exact Sinco-Audio alias plus native USB UID lineage. 44.1 kHz engine support added.
- Live guitar sound, latency, dry routing, disconnect and long-running monitoring remain unverified.

### Recording startup fix · 2026-11-13

The initial recording build stopped at about 5 ms when the monitoring queue overflowed; this was incorrectly reported as a device disconnect. Recording now captures input directly to its own writer queue, independent of output progress. Regression fixtures cover a saturated monitor queue with no output callbacks, exact recorded sample count/channel data, and genuine writer backpressure.

After rebuilding/reopening the native app, the actual KPT Pro / Sinco-Audio input at 44,100 Hz, 128 requested frames, with MacBook Pro Speakers output showed a saved **12.701-second take** and a subsequent recording continuing past **17 seconds**, despite 908 monitoring overrun samples. No device-failure banner was present. The user was actively testing; ongoing recording was left undisturbed. This verifies startup/continued capture and take creation, not listening quality, latency compensation or a sustained stress test.
