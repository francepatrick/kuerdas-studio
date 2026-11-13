use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};
pub const MAX_TRACKS: usize = 16;
pub const MAX_CLIPS: usize = 128;
pub const MAX_SAMPLES: usize = 128_000_000;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipInfo {
    pub id: String,
    pub name: String,
    pub file: String,
    pub start_frame: u64,
    pub frames: u64,
    pub channels: u16,
    pub tone: String,
    #[serde(default)]
    pub source_offset: u64,
    #[serde(skip)]
    pub peaks: Vec<f32>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    // Legacy single-region fields are derived for older UI/debugging tools.
    pub file: String,
    pub start_frame: u64,
    pub frames: u64,
    pub channels: u16,
    pub gain: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub tone: String,
    #[serde(default)]
    pub clips: Vec<ClipInfo>,
    #[serde(skip)]
    pub peaks: Vec<f32>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    #[serde(flatten)]
    pub info: TrackInfo,
    pub waveform: Vec<f32>,
    pub waveforms: HashMap<String, Vec<f32>>,
}
#[derive(Default)]
pub struct TrackControls {
    pub gain: AtomicU32,
    pub pan: AtomicU32,
    pub mute: AtomicBool,
    pub solo: AtomicBool,
}
impl TrackControls {
    pub fn from_info(info: &TrackInfo) -> Self {
        Self {
            gain: AtomicU32::new(info.gain.to_bits()),
            pan: AtomicU32::new(info.pan.to_bits()),
            mute: AtomicBool::new(info.mute),
            solo: AtomicBool::new(info.solo),
        }
    }
    pub fn snapshot(&self, info: &TrackInfo) -> TrackInfo {
        let mut result = info.clone();
        result.gain = f32::from_bits(self.gain.load(Ordering::Relaxed));
        result.pan = f32::from_bits(self.pan.load(Ordering::Relaxed));
        result.mute = self.mute.load(Ordering::Relaxed);
        result.solo = self.solo.load(Ordering::Relaxed);
        result
    }
}
#[derive(Clone)]
pub struct AudioClip {
    pub info: ClipInfo,
    pub samples: Arc<Vec<f32>>,
}
impl AudioClip {
    pub fn new(mut info: ClipInfo, samples: Vec<f32>) -> Result<Self, String> {
        let available = (samples.len() / info.channels.max(1) as usize) as u64;
        if ![1, 2].contains(&info.channels) || info.source_offset > available {
            return Err("Invalid clip channel count or source offset".into());
        }
        if info.frames == 0 {
            info.frames = available - info.source_offset;
        }
        if info.frames == 0
            || info
                .source_offset
                .checked_add(info.frames)
                .is_none_or(|end| end > available)
        {
            return Err("Clip extends beyond its audio source".into());
        }
        let start = info.source_offset as usize * info.channels as usize;
        let end = start + info.frames as usize * info.channels as usize;
        info.peaks = peaks(&samples[start..end], info.channels as usize);
        Ok(Self {
            info,
            samples: Arc::new(samples),
        })
    }
    pub fn stereo_at(&self, position: u64) -> [f32; 2] {
        let Some(frame) = position
            .checked_sub(self.info.start_frame)
            .filter(|f| *f < self.info.frames)
        else {
            return [0.0; 2];
        };
        let i = (self.info.source_offset + frame) as usize * self.info.channels as usize;
        let left = self.samples[i];
        [
            left,
            if self.info.channels == 2 {
                self.samples[i + 1]
            } else {
                left
            },
        ]
    }
}
#[derive(Clone)]
pub struct AudioTrack {
    pub info: TrackInfo,
    pub clips: Vec<AudioClip>,
    pub controls: Arc<TrackControls>,
}
impl AudioTrack {
    pub fn new(info: TrackInfo, samples: Vec<f32>) -> Self {
        let clip = AudioClip::new(
            ClipInfo {
                id: unique_id(),
                name: info.name.clone(),
                file: info.file.clone(),
                start_frame: info.start_frame,
                frames: 0,
                channels: info.channels,
                tone: info.tone.clone(),
                source_offset: 0,
                peaks: vec![],
            },
            samples,
        )
        .expect("validated audio source");
        Self::from_clips(info, vec![clip])
    }
    pub fn from_clips(info: TrackInfo, clips: Vec<AudioClip>) -> Self {
        let controls = Arc::new(TrackControls::from_info(&info));
        let mut track = Self {
            info,
            clips,
            controls,
        };
        track.refresh();
        track
    }
    pub fn refresh(&mut self) {
        self.info.clips = self.clips.iter().map(|c| c.info.clone()).collect();
        self.info.start_frame = self
            .clips
            .iter()
            .map(|c| c.info.start_frame)
            .min()
            .unwrap_or(0);
        let end = self
            .clips
            .iter()
            .map(|c| c.info.start_frame.saturating_add(c.info.frames))
            .max()
            .unwrap_or(0);
        self.info.frames = end.saturating_sub(self.info.start_frame);
        self.info.file = self
            .clips
            .first()
            .map(|c| c.info.file.clone())
            .unwrap_or_default();
        self.info.channels = if self.clips.iter().any(|c| c.info.channels == 2) {
            2
        } else {
            1
        };
        self.info.peaks = self
            .clips
            .first()
            .map(|c| c.info.peaks.clone())
            .unwrap_or_default();
    }
    pub fn detached(&self) -> Self {
        Self::from_clips(self.controls.snapshot(&self.info), self.clips.clone())
    }
    pub fn view(&self) -> TrackView {
        TrackView {
            info: self.controls.snapshot(&self.info),
            waveform: self.info.peaks.clone(),
            waveforms: self
                .clips
                .iter()
                .map(|c| (c.info.id.clone(), c.info.peaks.clone()))
                .collect(),
        }
    }
    pub fn stereo_at(&self, position: u64) -> [f32; 2] {
        let mut out = [0.0; 2];
        for clip in &self.clips {
            let sample = clip.stereo_at(position);
            out[0] += sample[0];
            out[1] += sample[1];
        }
        out
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub sample_rate: u32,
    pub tracks: Vec<TrackInfo>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub sample_rate: u32,
    pub tracks: Vec<TrackView>,
    pub storage_path: String,
}
pub fn unique_id() -> String {
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}
pub fn managed_path(root: &Path, file: &str) -> Result<PathBuf, String> {
    if Path::new(file).components().count() != 1
        || !matches!(
            Path::new(file).components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err("Invalid session media reference".into());
    }
    Ok(root.join(file))
}
pub fn decode_wav(path: &Path, target_rate: u32) -> Result<(Vec<f32>, u16), String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 512_000_000 {
        return Err("WAV exceeds the 512 MB import limit".into());
    }
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    if ![1, 2].contains(&spec.channels)
        || !(8_000..=192_000).contains(&spec.sample_rate)
        || reader.duration() == 0
        || reader.duration() as u64 > spec.sample_rate as u64 * 600
    {
        return Err("Use a non-empty mono/stereo WAV up to ten minutes at 8–192 kHz".into());
    }
    let data: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| {
                s.map(|v| {
                    if v.is_finite() {
                        v.clamp(-1.0, 1.0)
                    } else {
                        0.0
                    }
                })
            })
            .collect::<Result<_, _>>(),
        hound::SampleFormat::Int => {
            if ![8, 16, 24, 32].contains(&spec.bits_per_sample) {
                return Err("Unsupported PCM bit depth".into());
            }
            let scale = 2f32.powi(spec.bits_per_sample as i32 - 1);
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| (v as f32 / scale).clamp(-1.0, 1.0)))
                .collect::<Result<_, _>>()
        }
    }
    .map_err(|e| e.to_string())?;
    Ok((
        resample(&data, spec.channels as usize, spec.sample_rate, target_rate)?,
        spec.channels,
    ))
}
pub fn resample(data: &[f32], channels: usize, from: u32, to: u32) -> Result<Vec<f32>, String> {
    if ![1, 2].contains(&channels) || from == 0 || to == 0 || data.len() < channels {
        return Err("Invalid or empty audio for rate conversion".into());
    }
    let frames = data.len() / channels;
    let target_frames = ((frames as u64 * to as u64 + from as u64 / 2) / from as u64) as usize;
    if target_frames == 0 {
        return Err("Audio is too short at the session rate".into());
    }
    if target_frames * channels > MAX_SAMPLES {
        return Err("Decoded audio exceeds session memory limit".into());
    }
    if from == to {
        return Ok(data.to_vec());
    }
    let mut result = Vec::with_capacity(target_frames * channels);
    for frame in 0..target_frames {
        let position = frame as f64 * from as f64 / to as f64;
        let a = (position.floor() as usize).min(frames - 1);
        let b = (a + 1).min(frames - 1);
        let mix = (position - a as f64) as f32;
        for ch in 0..channels {
            result.push(data[a * channels + ch] * (1.0 - mix) + data[b * channels + ch] * mix);
        }
    }
    Ok(result)
}
pub fn write_wav(path: &Path, rate: u32, channels: u16, data: &[f32]) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let mut writer = hound::WavWriter::new(
        std::io::BufWriter::new(file),
        hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .map_err(|e| e.to_string())?;
    for sample in data {
        writer.write_sample(*sample).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())
}
pub fn peaks(data: &[f32], channels: usize) -> Vec<f32> {
    let frames = data.len() / channels;
    let stride = frames.div_ceil(1024).max(1);
    data.chunks(stride * channels)
        .map(|chunk| chunk.iter().fold(0.0f32, |peak, s| peak.max(s.abs())))
        .collect()
}
/// Decode user-selected common audio formats on the control worker. Stored
/// media always becomes float WAV, so session reopen needs no external codec.
pub fn decode_audio(path: &Path, target_rate: u32) -> Result<(Vec<f32>, u16), String> {
    if path
        .extension()
        .is_some_and(|v| v.eq_ignore_ascii_case("wav"))
    {
        return decode_wav(path, target_rate);
    }
    use symphonia::core::{
        audio::SampleBuffer, errors::Error, formats::FormatOptions, io::MediaSourceStream,
        probe::Hint,
    };
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 512_000_000 {
        return Err("Audio file exceeds 512 MB".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|v| v.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
            &Default::default(),
        )
        .map_err(|e| format!("Unsupported audio file: {e}"))?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("No audio stream found")?;
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    let mut spec = None;
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;
        let current = *decoded.spec();
        let channels = current.channels.count();
        if !(8000..=192000).contains(&current.rate) || ![1, 2].contains(&channels) {
            return Err("Use mono/stereo audio at 8–192 kHz".into());
        }
        if spec.is_some_and(|old| old != current) {
            return Err("Audio format changes within the file are not supported".into());
        }
        spec = Some(current);
        let count = decoded.frames() * channels;
        if data.len() + count > MAX_SAMPLES
            || ((data.len() + count) / channels) as u64 > current.rate as u64 * 600
        {
            return Err("Import exceeds ten minutes or the decoded-audio memory limit".into());
        }
        let mut converted = SampleBuffer::<f32>::new(decoded.capacity() as u64, current);
        converted.copy_interleaved_ref(decoded);
        data.extend(converted.samples().iter().map(|v| {
            if v.is_finite() {
                v.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        }));
    }
    let spec = spec.ok_or("Audio file is empty")?;
    let channels = spec.channels.count() as u16;
    if data.is_empty() {
        return Err("Audio file is empty".into());
    }
    if spec.rate == target_rate {
        Ok((data, channels))
    } else {
        Ok((
            resample(&data, channels as usize, spec.rate, target_rate)?,
            channels,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resampling_keeps_stereo_and_duration() {
        let x: Vec<_> = (0..480).flat_map(|_| [0.25, -0.5]).collect();
        let y = resample(&x, 2, 48000, 44100).unwrap();
        assert_eq!(y.len(), 882);
        assert!((y[0] - 0.25).abs() < 1e-6);
        assert!((y[1] + 0.5).abs() < 1e-6);
    }
    #[test]
    fn track_offsets_and_boundaries_are_sample_accurate() {
        let info = TrackInfo {
            id: "a".into(),
            name: "a".into(),
            kind: "take".into(),
            file: "a.wav".into(),
            start_frame: 10,
            frames: 0,
            channels: 1,
            gain: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            tone: String::new(),
            clips: vec![],
            peaks: vec![],
        };
        let track = AudioTrack::new(info, vec![0.3, 0.6]);
        assert_eq!(track.stereo_at(9), [0.0; 2]);
        assert_eq!(track.stereo_at(10), [0.3; 2]);
        assert_eq!(track.stereo_at(12), [0.0; 2]);
    }
    #[test]
    fn media_paths_cannot_escape_session() {
        for path in ["../a.wav", "/tmp/a.wav", "a/b.wav", ".."] {
            assert!(managed_path(Path::new("/session"), path).is_err());
        }
    }
}
