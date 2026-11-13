//! Versioned session contract. Disk persistence and migrations arrive in M6.
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub schema_version: u32,
    pub name: String,
    pub sample_rate: u32,
    pub tempo: f64,
    pub tracks: Vec<Track>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub name: String,
    pub gain: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub clips: Vec<Clip>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub audio_path: String,
    pub start_frame: u64,
    pub source_offset: u64,
    pub length_frames: u64,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            schema_version: 1,
            name: "Untitled session".into(),
            sample_rate: 48_000,
            tempo: 120.0,
            tracks: vec![],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_roundtrip_preserves_sample_positions() {
        let mut session = Session::default();
        session.tracks.push(Track {
            id: "guitar".into(),
            name: "Guitar".into(),
            gain: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            clips: vec![Clip {
                id: "take".into(),
                audio_path: "audio/take.wav".into(),
                start_frame: 96000,
                source_offset: 123,
                length_frames: 48000,
            }],
        });
        let restored: Session =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(restored.tracks[0].clips[0].start_frame, 96000);
        assert_eq!(restored.tracks[0].clips[0].source_offset, 123);
    }
}
