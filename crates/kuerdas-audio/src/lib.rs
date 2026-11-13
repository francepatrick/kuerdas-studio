//! Native transport: no locks, allocation, files, dialogs, or IPC in callbacks.
mod editing;
mod media;
mod processing;
mod recording;
mod workspaces;
use cpal::traits::{DeviceTrait, StreamTrait};
use kuerdas_dsp::{GuitarRig, Parameters};
use media::{
    AudioClip, AudioTrack, ClipInfo, Manifest, TrackInfo, MAX_CLIPS, MAX_SAMPLES, MAX_TRACKS,
};
pub use media::{SessionView, TrackView};
pub use processing::RecordedEffects;
use recording::Recorder;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};
pub use workspaces::{WorkspaceCatalog, WorkspaceInfo, WorkspaceView};

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StartConfig {
    pub input_id: String,
    pub output_id: String,
    pub input_channel: u16,
    pub sample_rate: u32,
    pub buffer_frames: u32,
}
#[derive(Default)]
struct Diagnostics {
    input_peak: AtomicU32,
    output_peak: AtomicU32,
    underruns: AtomicU64,
    overruns: AtomicU64,
    failed: AtomicBool,
}
#[derive(Default)]
struct Transport {
    playing: AtomicBool,
    recording: AtomicBool,
    monitoring: AtomicBool,
    looping: AtomicBool,
    position: AtomicU64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
    pub input_peak: f32,
    pub output_peak: f32,
    pub underruns: u64,
    pub overruns: u64,
    pub device_error: bool,
    pub playing: bool,
    pub recording: bool,
    pub monitoring: bool,
    pub looping: bool,
    pub position_frames: u64,
    pub duration_frames: u64,
    pub session_revision: u64,
    pub error: Option<String>,
}
#[derive(Default)]
pub struct Engine {
    streams: Option<(cpal::Stream, cpal::Stream)>,
    pub params: Arc<Parameters>,
    diagnostics: Arc<Diagnostics>,
    active_config: Option<StartConfig>,
    transport: Arc<Transport>,
    recorder: Option<Recorder>,
    recording_target: Option<String>,
    tracks: Vec<AudioTrack>,
    storage: PathBuf,
    sample_rate: u32,
    revision: u64,
    last_error: Option<String>,
}
impl Engine {
    pub fn configure_storage(&mut self, root: PathBuf) -> Result<(), String> {
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        self.storage = root;
        let path = self.storage.join("session.json");
        if path.exists() {
            let manifest: Manifest =
                serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                    .map_err(|e| format!("Session metadata needs recovery: {e}"))?;
            if ![1, 2].contains(&manifest.schema_version)
                || ![44100, 48000].contains(&manifest.sample_rate)
                || manifest.tracks.len() > MAX_TRACKS
            {
                return Err("Unsupported session version or configuration".into());
            }
            if manifest.schema_version == 1 {
                let backup = self.storage.join("session-v1-backup.json");
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(backup)
                {
                    Ok(mut file) => {
                        use std::io::Write;
                        file.write_all(&std::fs::read(&path).map_err(|e| e.to_string())?)
                            .map_err(|e| e.to_string())?;
                        file.sync_all().map_err(|e| e.to_string())?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e.to_string()),
                }
            }
            self.sample_rate = manifest.sample_rate;
            for info in manifest.tracks {
                if !info.gain.is_finite()
                    || !info.pan.is_finite()
                    || !(0.0..=2.0).contains(&info.gain)
                    || !(-1.0..=1.0).contains(&info.pan)
                {
                    return Err("Invalid saved track controls".into());
                }
                let regions = if info.clips.is_empty() && !info.file.is_empty() {
                    vec![ClipInfo {
                        id: media::unique_id(),
                        name: info.name.clone(),
                        file: info.file.clone(),
                        start_frame: info.start_frame,
                        frames: info.frames,
                        channels: info.channels,
                        tone: info.tone.clone(),
                        source_offset: 0,
                        peaks: vec![],
                    }]
                } else {
                    info.clips.clone()
                };
                let mut clips = Vec::new();
                let mut loaded = 0usize;
                for region in regions {
                    if region
                        .start_frame
                        .checked_add(region.frames)
                        .is_none_or(|end| end > self.sample_rate as u64 * 3600)
                    {
                        return Err("Clip lies outside the one-hour timeline".into());
                    }
                    let (samples, channels) = media::decode_wav(
                        &media::managed_path(&self.storage, &region.file)?,
                        self.sample_rate,
                    )?;
                    if channels != region.channels
                        || self.sample_count() + loaded + samples.len() > MAX_SAMPLES
                    {
                        return Err("Invalid channels or session memory limit exceeded".into());
                    }
                    loaded += samples.len();
                    clips.push(AudioClip::new(region, samples)?);
                }
                if self.clip_count() + clips.len() > MAX_CLIPS {
                    return Err("Session exceeds 128 clips".into());
                }
                self.tracks.push(AudioTrack::from_clips(info, clips));
            }
        }
        self.revision += 1;
        Ok(())
    }
    fn sample_count(&self) -> usize {
        self.tracks
            .iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.samples.len())
            .sum()
    }
    fn clip_count(&self) -> usize {
        self.tracks.iter().map(|t| t.clips.len()).sum()
    }
    fn has_audio(&self) -> bool {
        self.tracks.iter().any(|t| !t.clips.is_empty())
    }
    fn duration(&self) -> u64 {
        self.tracks
            .iter()
            .map(|t| t.info.start_frame.saturating_add(t.info.frames))
            .max()
            .unwrap_or(0)
    }
    fn save(&self) -> Result<(), String> {
        if self.storage.as_os_str().is_empty() {
            return Err("Session storage is not ready".into());
        }
        let manifest = Manifest {
            schema_version: 2,
            sample_rate: if self.sample_rate == 0 {
                44100
            } else {
                self.sample_rate
            },
            tracks: self
                .tracks
                .iter()
                .map(|t| t.controls.snapshot(&t.info))
                .collect(),
        };
        let pending = self.storage.join("session.json.tmp");
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&pending).map_err(|e| e.to_string())?;
            file.write_all(&serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        std::fs::rename(pending, self.storage.join("session.json")).map_err(|e| e.to_string())
    }
    pub fn session(&self) -> SessionView {
        SessionView {
            sample_rate: self.sample_rate,
            tracks: self.tracks.iter().map(|t| t.view()).collect(),
            storage_path: self.storage.display().to_string(),
        }
    }
    pub fn stop(&mut self) {
        self.streams = None;
        self.active_config = None;
        self.transport.playing.store(false, Ordering::Relaxed);
        self.transport.recording.store(false, Ordering::Relaxed);
        self.transport.monitoring.store(false, Ordering::Relaxed);
        let destination = self.recording_target.take();
        if let Some(recorder) = self.recorder.take() {
            let path = recorder.path.clone();
            let start = recorder.start_frame;
            let tone = recorder.tone.clone();
            if self.diagnostics.failed.load(Ordering::Relaxed) {
                recorder.failed.store(true, Ordering::Relaxed);
            }
            let result = recorder.finish().and_then(|samples| {
                if samples == 0 {
                    return Err("Take was empty; no track was added".into());
                }
                let (data, channels) = media::decode_wav(&path, self.sample_rate)?;
                if self.sample_count() + data.len() > MAX_SAMPLES {
                    return Err("Session memory limit reached; take retained on disk".into());
                }
                let info = TrackInfo {
                    id: media::unique_id(),
                    name: format!(
                        "Guitar take {}",
                        self.tracks.iter().filter(|t| t.info.kind == "take").count() + 1
                    ),
                    kind: "take".into(),
                    file: path.file_name().unwrap().to_string_lossy().into_owned(),
                    start_frame: start,
                    frames: samples,
                    channels,
                    gain: 1.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    tone,
                    clips: vec![],
                    peaks: vec![],
                };
                let take = AudioTrack::new(info, data);
                let mut draft: Vec<_> = self.tracks.iter().map(AudioTrack::detached).collect();
                if let Some(id) = &destination {
                    let track = draft
                        .iter_mut()
                        .find(|t| &t.info.id == id)
                        .ok_or("Recording destination missing; take retained on disk")?;
                    track.clips.extend(take.clips);
                    track.refresh();
                } else {
                    draft.push(take);
                }
                self.commit_tracks(draft)
            });
            if let Err(e) = result {
                self.last_error = Some(format!("{e}. Audio file: {}", path.display()));
            }
        }
    }
    pub fn check_connection(&mut self) {
        let lost = self.active_config.as_ref().is_some_and(|c| {
            kuerdas_devices::find(&c.input_id, true)
                .and_then(|d| kuerdas_devices::require_kpt_pro(&d))
                .is_err()
        });
        if lost {
            self.diagnostics.failed.store(true, Ordering::Relaxed);
        }
        if lost
            || self.diagnostics.failed.load(Ordering::Relaxed)
            || self
                .recorder
                .as_ref()
                .is_some_and(|r| r.failed.load(Ordering::Relaxed))
            || (self.recorder.is_some() && !self.transport.recording.load(Ordering::Relaxed))
        {
            self.stop();
        }
        if self.streams.is_some()
            && !self.transport.playing.load(Ordering::Relaxed)
            && !self.transport.monitoring.load(Ordering::Relaxed)
        {
            self.stop();
        }
    }
    pub fn status(&mut self) -> Status {
        if self.diagnostics.failed.load(Ordering::Relaxed) {
            self.stop();
        }
        Status {
            running: self.streams.is_some(),
            input_peak: f32::from_bits(self.diagnostics.input_peak.swap(0, Ordering::Relaxed)),
            output_peak: f32::from_bits(self.diagnostics.output_peak.swap(0, Ordering::Relaxed)),
            underruns: self.diagnostics.underruns.load(Ordering::Relaxed),
            overruns: self.diagnostics.overruns.load(Ordering::Relaxed),
            device_error: self.diagnostics.failed.load(Ordering::Relaxed),
            playing: self.transport.playing.load(Ordering::Relaxed),
            recording: self.transport.recording.load(Ordering::Relaxed),
            monitoring: self.transport.monitoring.load(Ordering::Relaxed),
            looping: self.transport.looping.load(Ordering::Relaxed),
            position_frames: self.transport.position.load(Ordering::Relaxed),
            duration_frames: self.duration(),
            session_revision: self.revision,
            error: self.last_error.take(),
        }
    }
    fn validate_rate(&self, config: &StartConfig) -> Result<(), String> {
        if self.storage.as_os_str().is_empty() {
            return Err("Create or open a named workspace before starting audio".into());
        }
        if ![44100, 48000].contains(&config.sample_rate)
            || ![64, 128, 256].contains(&config.buffer_frames)
        {
            return Err("Use 44.1/48 kHz and 64/128/256 frames".into());
        }
        if self.has_audio() && config.sample_rate != self.sample_rate {
            return Err(format!(
                "This session uses {} Hz. Select compatible devices at that rate.",
                self.sample_rate
            ));
        }
        Ok(())
    }
    pub fn start(&mut self, config: StartConfig) -> Result<(), String> {
        self.validate_rate(&config)?;
        if self.active_config.as_ref() == Some(&config) && self.streams.is_some() {
            self.transport.monitoring.store(true, Ordering::Relaxed);
            return Ok(());
        }
        self.stop();
        self.sample_rate = config.sample_rate;
        self.start_streams(config, None)?;
        self.transport.monitoring.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub fn stop_monitoring(&mut self) {
        self.transport.monitoring.store(false, Ordering::Relaxed);
        if !self.transport.playing.load(Ordering::Relaxed) {
            self.stop();
        }
    }
    pub fn play(&mut self, config: StartConfig) -> Result<(), String> {
        if self.recorder.is_some() {
            return Err("Stop recording before playback".into());
        }
        if !self.has_audio() {
            return Err("Import a backing track or record a take first".into());
        }
        self.validate_rate(&config)?;
        self.stop();
        if self.transport.position.load(Ordering::Relaxed) >= self.duration() {
            self.transport.position.store(0, Ordering::Relaxed);
        }
        self.start_streams(config, None)?;
        self.transport.playing.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub fn record(&mut self, config: StartConfig, tone: String) -> Result<(), String> {
        self.record_into(config, tone, None)
    }
    pub fn record_into(
        &mut self,
        config: StartConfig,
        tone: String,
        target: Option<String>,
    ) -> Result<(), String> {
        if self.recorder.is_some() {
            return Err("Already recording".into());
        }
        if target
            .as_ref()
            .is_some_and(|id| !self.tracks.iter().any(|t| &t.info.id == id))
        {
            return Err("Recording destination no longer exists".into());
        }
        if self.clip_count() >= MAX_CLIPS {
            return Err("Maximum 128 clips per session".into());
        }
        if target.is_none() && self.tracks.len() >= MAX_TRACKS {
            return Err("Maximum 16 tracks per session; select an existing recording track".into());
        }
        self.validate_rate(&config)?;
        self.stop();
        self.sample_rate = config.sample_rate;
        if self.storage.as_os_str().is_empty() {
            return Err("Session storage is not ready".into());
        }
        let path = self
            .storage
            .join(format!("take-{}.wav", media::unique_id()));
        let (recorder, producer) = Recorder::new(
            path,
            self.sample_rate,
            self.transport.position.load(Ordering::Relaxed),
            tone,
        )?;
        let fault = recorder.failed.clone();
        self.recorder = Some(recorder);
        self.recording_target = target;
        if let Err(e) = self.start_streams(config, Some((producer, fault))) {
            self.stop();
            return Err(e);
        }
        self.transport.monitoring.store(true, Ordering::Relaxed);
        self.transport.recording.store(true, Ordering::Relaxed);
        self.transport.playing.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub fn seek(&mut self, frame: u64) -> Result<(), String> {
        if self.transport.playing.load(Ordering::Relaxed) {
            return Err("Stop transport before seeking".into());
        }
        self.transport
            .position
            .store(frame.min(self.duration()), Ordering::Relaxed);
        Ok(())
    }
    pub fn set_loop(&self, enabled: bool) -> Result<(), String> {
        if self.recorder.is_some() {
            return Err("Loop changes are disabled during recording".into());
        }
        self.transport.looping.store(enabled, Ordering::Relaxed);
        Ok(())
    }
    pub fn import(&mut self, path: &Path, rate: u32) -> Result<(), String> {
        self.import_many(&[path.to_path_buf()], rate)
    }
    pub fn update_track(
        &mut self,
        id: &str,
        gain: f32,
        pan: f32,
        mute: bool,
        solo: bool,
    ) -> Result<(), String> {
        if !gain.is_finite() || !pan.is_finite() {
            return Err("Track controls must be finite".into());
        }
        let track = self
            .tracks
            .iter()
            .find(|t| t.info.id == id)
            .ok_or("Track not found")?;
        track
            .controls
            .gain
            .store(gain.clamp(0.0, 2.0).to_bits(), Ordering::Relaxed);
        track
            .controls
            .pan
            .store(pan.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
        track.controls.mute.store(mute, Ordering::Relaxed);
        track.controls.solo.store(solo, Ordering::Relaxed);
        self.save()
    }
    pub fn remove_track(&mut self, id: &str) -> Result<(), String> {
        if !self.tracks.iter().any(|t| t.info.id == id) {
            return Err("Track not found".into());
        }
        let draft = self
            .tracks
            .iter()
            .filter(|t| t.info.id != id)
            .map(AudioTrack::detached)
            .collect();
        self.commit_tracks(draft)
    }
    pub fn export_mix(&self, path: &Path) -> Result<(), String> {
        if self.streams.is_some() {
            return Err("Stop audio before exporting a mix".into());
        }
        let frames = self.duration();
        if frames == 0 {
            return Err("No audio to export".into());
        }
        let temporary = path.with_file_name(format!(".kuerdas-export-{}.wav", media::unique_id()));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        let mut writer = hound::WavWriter::new(
            std::io::BufWriter::new(file),
            hound::WavSpec {
                channels: 2,
                sample_rate: self.sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .map_err(|e| e.to_string())?;
        let controls = mix_controls(&self.tracks);
        for frame in 0..frames {
            let sample = mix_at(&self.tracks, &controls, frame);
            for channel in sample {
                writer
                    .write_sample(channel.clamp(-1.0, 1.0))
                    .map_err(|e| e.to_string())?;
            }
        }
        writer.finalize().map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, path).map_err(|e| e.to_string())
    }
    fn start_streams(
        &mut self,
        request: StartConfig,
        mut recording: Option<(rtrb::Producer<f32>, Arc<AtomicBool>)>,
    ) -> Result<(), String> {
        self.params.bypass.store(true, Ordering::Relaxed);
        let input = kuerdas_devices::find(&request.input_id, true)?;
        kuerdas_devices::require_kpt_pro(&input)?;
        let output = kuerdas_devices::find(&request.output_id, false)?;
        let in_config = choose_config(&input, true, &request)?;
        let out_config = choose_config(&output, false, &request)?;
        let (producer, mut consumer) =
            rtrb::RingBuffer::<f32>::new(request.buffer_frames as usize * 4);
        let diag = Arc::new(Diagnostics::default());
        self.diagnostics = diag.clone();
        let in_error = diag.clone();
        let mut capture = CapturePath {
            monitor: producer,
            recording: recording.take(),
            params: self.params.clone(),
            transport: self.transport.clone(),
            diagnostics: diag.clone(),
            rig: GuitarRig::for_sample_rate(request.sample_rate),
            channels: in_config.channels as usize,
            channel: request.input_channel as usize,
            max_record_frames: (request.sample_rate as u64 * 600)
                .min(MAX_SAMPLES.saturating_sub(self.sample_count()) as u64),
            recorded: 0,
        };
        let input_stream = input
            .build_input_stream(
                &in_config,
                move |data: &[f32], _| capture.process(data),
                move |_| {
                    in_error.failed.store(true, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|e| e.to_string())?;
        let out_diag = diag.clone();
        let out_error = diag;
        let transport = self.transport.clone();
        let out_channels = out_config.channels as usize;
        // The input callback owns guitar DSP and recording. This callback only
        // consumes monitored samples and renders the backing/session mix.
        let tracks = self.tracks.clone();
        let duration = self.duration();
        let mut smoothed_controls = [[0.0f32; 2]; MAX_TRACKS];
        let mut playback_position = transport.position.load(Ordering::Relaxed);
        let output_stream = output
            .build_output_stream(
                &out_config,
                move |data: &mut [f32], _| {
                    let mut peak = 0.0f32;
                    let target_controls = mix_controls(&tracks);
                    let block_frames = (data.len() / out_channels).max(1) as f32;
                    let mut steps = [[0.0f32; 2]; MAX_TRACKS];
                    for i in 0..tracks.len() {
                        for ch in 0..2 {
                            steps[i][ch] =
                                (target_controls[i][ch] - smoothed_controls[i][ch]) / block_frames;
                        }
                    }
                    let mut position = playback_position;
                    for frame in data.chunks_exact_mut(out_channels) {
                        for i in 0..tracks.len() {
                            for ch in 0..2 {
                                smoothed_controls[i][ch] += steps[i][ch];
                            }
                        }
                        let failed = out_diag.failed.load(Ordering::Relaxed);
                        let rec = transport.recording.load(Ordering::Relaxed);
                        let guitar = if failed {
                            0.0
                        } else {
                            match consumer.pop() {
                                Ok(sample) => sample,
                                Err(_) => {
                                    out_diag.underruns.fetch_add(1, Ordering::Relaxed);
                                    0.0
                                }
                            }
                        };
                        let mut mixed = [0.0f32; 2];
                        let playing = transport.playing.load(Ordering::Relaxed);
                        if playing && !failed {
                            if !rec && duration > 0 && position >= duration {
                                if transport.looping.load(Ordering::Relaxed) {
                                    position = 0;
                                } else {
                                    transport.playing.store(false, Ordering::Relaxed);
                                }
                            }
                            if transport.playing.load(Ordering::Relaxed) {
                                mixed = mix_at(&tracks, &smoothed_controls, position);
                                position = position.saturating_add(1);
                            }
                        }
                        if transport.monitoring.load(Ordering::Relaxed) && !failed {
                            mixed[0] += guitar;
                            mixed[1] += guitar;
                        }
                        if failed {
                            mixed = [0.0; 2];
                        }
                        mixed[0] = mixed[0].clamp(-1.0, 1.0);
                        mixed[1] = mixed[1].clamp(-1.0, 1.0);
                        if out_channels == 1 {
                            frame[0] = (mixed[0] + mixed[1]) * 0.5;
                        } else {
                            frame[0] = mixed[0];
                            frame[1] = mixed[1];
                            for ch in frame.iter_mut().skip(2) {
                                *ch = 0.0;
                            }
                        }
                        peak = peak.max(mixed[0].abs()).max(mixed[1].abs());
                    }
                    playback_position = position;
                    // While recording, capture (not the output device clock)
                    // publishes the number of samples actually written to WAV.
                    if !transport.recording.load(Ordering::Relaxed) {
                        transport.position.store(position, Ordering::Relaxed);
                    }
                    out_diag
                        .output_peak
                        .fetch_max(peak.to_bits(), Ordering::Relaxed);
                },
                move |_| {
                    out_error.failed.store(true, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|e| e.to_string())?;
        input_stream.play().map_err(|e| e.to_string())?;
        output_stream.play().map_err(|e| e.to_string())?;
        self.active_config = Some(request);
        self.streams = Some((input_stream, output_stream));
        Ok(())
    }
}
/// Input-owned signal path. Monitor queue saturation must never lose disk
/// samples or masquerade as a device disconnection.
struct CapturePath {
    monitor: rtrb::Producer<f32>,
    recording: Option<(rtrb::Producer<f32>, Arc<AtomicBool>)>,
    params: Arc<Parameters>,
    transport: Arc<Transport>,
    diagnostics: Arc<Diagnostics>,
    rig: GuitarRig,
    channels: usize,
    channel: usize,
    max_record_frames: u64,
    recorded: u64,
}
impl CapturePath {
    fn process(&mut self, data: &[f32]) {
        let mut peak = 0.0f32;
        let mut captured = 0u64;
        for frame in data.chunks_exact(self.channels) {
            let raw = frame[self.channel];
            let input = if raw.is_finite() {
                raw.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            peak = peak.max(input.abs());
            let guitar = self.rig.process(input, &self.params);
            // Record first, independently of the output/monitor buffer.
            if self.transport.recording.load(Ordering::Relaxed) {
                if let Some((writer, fault)) = &mut self.recording {
                    if self.recorded >= self.max_record_frames || writer.push(guitar).is_err() {
                        fault.store(true, Ordering::Relaxed);
                        self.transport.recording.store(false, Ordering::Relaxed);
                        self.transport.playing.store(false, Ordering::Relaxed);
                    } else {
                        self.recorded += 1;
                        captured += 1;
                    }
                }
            }
            if self.monitor.push(guitar).is_err() {
                self.diagnostics.overruns.fetch_add(1, Ordering::Relaxed);
            }
        }
        if captured > 0 {
            self.transport
                .position
                .fetch_add(captured, Ordering::Relaxed);
        }
        self.diagnostics
            .input_peak
            .fetch_max(peak.to_bits(), Ordering::Relaxed);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}
fn mix_controls(tracks: &[AudioTrack]) -> [[f32; 2]; MAX_TRACKS] {
    let mut result = [[0.0; 2]; MAX_TRACKS];
    let any_solo = tracks
        .iter()
        .any(|t| t.controls.solo.load(Ordering::Relaxed));
    for (i, track) in tracks.iter().enumerate() {
        if track.controls.mute.load(Ordering::Relaxed)
            || (any_solo && !track.controls.solo.load(Ordering::Relaxed))
        {
            continue;
        }
        let gain = f32::from_bits(track.controls.gain.load(Ordering::Relaxed));
        let pan = f32::from_bits(track.controls.pan.load(Ordering::Relaxed));
        result[i] = [gain * (1.0 - pan.max(0.0)), gain * (1.0 + pan.min(0.0))];
    }
    result
}
fn mix_at(tracks: &[AudioTrack], controls: &[[f32; 2]; MAX_TRACKS], frame: u64) -> [f32; 2] {
    let mut result = [0.0; 2];
    for (i, track) in tracks.iter().enumerate() {
        let sample = track.stereo_at(frame);
        result[0] += sample[0] * controls[i][0];
        result[1] += sample[1] * controls[i][1];
    }
    result
}
fn choose_config(
    device: &cpal::Device,
    input: bool,
    request: &StartConfig,
) -> Result<cpal::StreamConfig, String> {
    let configs: Vec<_> = if input {
        device.supported_input_configs().map(|v| v.collect())
    } else {
        device.supported_output_configs().map(|v| v.collect())
    }
    .map_err(|e| e.to_string())?;
    let supported = configs
        .into_iter()
        .filter(|c| {
            c.sample_format() == cpal::SampleFormat::F32
                && c.min_sample_rate() <= request.sample_rate
                && c.max_sample_rate() >= request.sample_rate
                && (!input || c.channels() > request.input_channel)
        })
        .min_by_key(|c| c.channels())
        .ok_or("Device does not support float32 at the selected rate. Choose a rate shared by input and output.")?;
    if let cpal::SupportedBufferSize::Range { min, max } = supported.buffer_size() {
        if request.buffer_frames < *min || request.buffer_frames > *max {
            return Err(format!(
                "Device requires buffers between {min} and {max} frames"
            ));
        }
    }
    let mut config = supported.with_sample_rate(request.sample_rate).config();
    config.buffer_size = cpal::BufferSize::Fixed(request.buffer_frames);
    Ok(config)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("kuerdas-session-{}", media::unique_id()))
    }
    #[test]
    fn recording_survives_a_full_monitor_queue_without_output_callbacks() {
        let root = temp_root();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        engine.sample_rate = 44100;
        let (recorder, writer) = Recorder::new(
            root.join("take-clock-test.wav"),
            44100,
            22050,
            "KPT Pro direct".into(),
        )
        .unwrap();
        let fault = recorder.failed.clone();
        let (monitor, _idle_output) = rtrb::RingBuffer::new(4);
        let transport = Arc::new(Transport::default());
        transport.position.store(22050, Ordering::Relaxed);
        transport.recording.store(true, Ordering::Relaxed);
        transport.playing.store(true, Ordering::Relaxed);
        let diagnostics = Arc::new(Diagnostics::default());
        let params = Arc::new(Parameters::default());
        params.update(1.0, 0.25, 0.0, true).unwrap();
        let mut capture = CapturePath {
            monitor,
            recording: Some((writer, fault.clone())),
            params,
            transport: transport.clone(),
            diagnostics: diagnostics.clone(),
            rig: GuitarRig::for_sample_rate(44100),
            channels: 2,
            channel: 1,
            max_record_frames: 44100,
            recorded: 0,
        };
        let samples: Vec<_> = (0..2048).flat_map(|_| [0.2f32, 0.4]).collect();
        capture.process(&samples);
        assert_eq!(diagnostics.overruns.load(Ordering::Relaxed), 2044);
        assert!(!diagnostics.failed.load(Ordering::Relaxed));
        assert!(!fault.load(Ordering::Relaxed));
        assert!(transport.recording.load(Ordering::Relaxed));
        assert_eq!(transport.position.load(Ordering::Relaxed), 22050 + 2048);
        drop(capture);
        engine.recorder = Some(recorder);
        engine.diagnostics = diagnostics;
        engine.transport = transport;
        engine.stop();
        assert_eq!(engine.tracks.len(), 1);
        assert_eq!(engine.tracks[0].info.frames, 2048);
        assert!(engine.tracks[0].clips[0]
            .samples
            .iter()
            .all(|sample| (*sample - 0.1).abs() < 1e-6));
        assert!(!engine.status().device_error);
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn writer_backpressure_is_a_recording_fault_not_a_device_disconnect() {
        let (monitor, _output) = rtrb::RingBuffer::new(4);
        let (writer, _stalled_disk) = rtrb::RingBuffer::new(2);
        let fault = Arc::new(AtomicBool::new(false));
        let transport = Arc::new(Transport::default());
        transport.recording.store(true, Ordering::Relaxed);
        transport.playing.store(true, Ordering::Relaxed);
        let diagnostics = Arc::new(Diagnostics::default());
        let mut capture = CapturePath {
            monitor,
            recording: Some((writer, fault.clone())),
            params: Arc::new(Parameters::default()),
            transport: transport.clone(),
            diagnostics: diagnostics.clone(),
            rig: GuitarRig::for_sample_rate(44100),
            channels: 1,
            channel: 0,
            max_record_frames: 44100,
            recorded: 0,
        };
        capture.process(&[0.1; 8]);
        assert!(fault.load(Ordering::Relaxed));
        assert!(!transport.recording.load(Ordering::Relaxed));
        assert!(!diagnostics.failed.load(Ordering::Relaxed));
        assert_eq!(capture.recorded, 2);
    }
    #[test]
    fn backing_import_save_reopen_and_mix_export() {
        let root = temp_root();
        let source = root.with_extension("wav");
        let mix = root.with_extension("mix.wav");
        let data: Vec<_> = (0..480).flat_map(|_| [0.5f32, -0.25]).collect();
        media::write_wav(&source, 48000, 2, &data).unwrap();
        {
            let mut engine = Engine::default();
            engine.configure_storage(root.clone()).unwrap();
            engine.import(&source, 44100).unwrap();
            assert_eq!(engine.tracks[0].info.frames, 441);
            let id = engine.tracks[0].info.id.clone();
            engine.update_track(&id, 0.5, 0.0, false, false).unwrap();
            engine.export_mix(&mix).unwrap();
        }
        let (mixed, ch) = media::decode_wav(&mix, 44100).unwrap();
        assert_eq!(ch, 2);
        assert_eq!(mixed.len(), 882);
        assert!((mixed[0] - 0.25).abs() < 1e-6);
        assert!((mixed[1] + 0.125).abs() < 1e-6);
        {
            let mut engine = Engine::default();
            engine.configure_storage(root.clone()).unwrap();
            assert_eq!(engine.tracks.len(), 1);
            assert_eq!(engine.sample_rate, 44100);
            assert_eq!(
                engine.tracks[0]
                    .controls
                    .snapshot(&engine.tracks[0].info)
                    .gain,
                0.5
            );
        }
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(source).unwrap();
        std::fs::remove_file(mix).unwrap();
    }
    #[test]
    fn mute_solo_and_pan_have_defined_precedence() {
        fn track(id: &str) -> AudioTrack {
            AudioTrack::new(
                TrackInfo {
                    id: id.into(),
                    name: id.into(),
                    kind: "take".into(),
                    file: format!("{id}.wav"),
                    start_frame: 0,
                    frames: 1,
                    channels: 1,
                    gain: 1.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    tone: String::new(),
                    clips: vec![],
                    peaks: vec![],
                },
                vec![0.25],
            )
        }
        let tracks = vec![track("a"), track("b")];
        tracks[0].controls.solo.store(true, Ordering::Relaxed);
        tracks[0]
            .controls
            .pan
            .store(1f32.to_bits(), Ordering::Relaxed);
        assert_eq!(mix_at(&tracks, &mix_controls(&tracks), 0), [0.0, 0.25]);
        tracks[0].controls.mute.store(true, Ordering::Relaxed);
        assert_eq!(mix_at(&tracks, &mix_controls(&tracks), 0), [0.0, 0.0]);
    }
    #[test]
    fn record_stop_finalizes_take_and_restores_its_tone_metadata() {
        let root = temp_root();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        engine.sample_rate = 44100;
        let path = root.join("take-test.wav");
        let (recorder, mut producer) =
            Recorder::new(path, 44100, 22050, "My KPT Pro / hardware".into()).unwrap();
        for _ in 0..1000 {
            producer.push(0.1).unwrap();
        }
        drop(producer);
        engine.recorder = Some(recorder);
        engine.stop();
        assert_eq!(engine.tracks.len(), 1);
        assert_eq!(engine.tracks[0].info.start_frame, 22050);
        assert_eq!(engine.tracks[0].info.tone, "My KPT Pro / hardware");
        assert_eq!(engine.tracks[0].info.frames, 1000);
        assert!(engine.last_error.is_none());
        drop(engine);
        let mut restored = Engine::default();
        restored.configure_storage(root.clone()).unwrap();
        assert_eq!(restored.tracks[0].info.start_frame, 22050);
        drop(restored);
        std::fs::remove_dir_all(root).unwrap();
    }
}
