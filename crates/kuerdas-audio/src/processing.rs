use super::*;
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedEffects {
    pub drive: f32,
    pub bass: f32,
    pub treble: f32,
    pub level: f32,
}
impl Engine {
    pub fn process_recorded_track(
        &mut self,
        id: &str,
        settings: RecordedEffects,
    ) -> Result<String, String> {
        self.editable()?;
        if [
            settings.drive,
            settings.bass,
            settings.treble,
            settings.level,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            return Err("Recorded-audio effects must be finite".into());
        }
        if self.tracks.len() >= MAX_TRACKS {
            return Err("Make room for the processed copy (maximum 16 tracks)".into());
        }
        let selected = self.selected_audio(&[id.into()])?;
        let source = &selected[0];
        if source.clips.is_empty() {
            return Err("Select a track with recorded audio".into());
        }
        let start = source.info.start_frame;
        let frames = source.info.frames;
        if frames > self.sample_rate as u64 * 600
            || self.sample_count() + frames as usize * 2 > MAX_SAMPLES
        {
            return Err("Processed track exceeds the audio-cache or ten-minute limit".into());
        }
        let params = Parameters::default();
        params.update(settings.drive, settings.level, 0.0, false)?;
        params.update_color(settings.bass, settings.treble, 320.0, 0.0)?;
        let mut left = GuitarRig::for_sample_rate(self.sample_rate);
        let mut right = GuitarRig::for_sample_rate(self.sample_rate);
        let controls = mix_controls(&selected);
        let mut audio = Vec::with_capacity(frames as usize * 2);
        for frame in start..start + frames {
            let sample = mix_at(&selected, &controls, frame);
            audio.push(left.process(sample[0], &params));
            audio.push(right.process(sample[1], &params));
        }
        let mut info = editing::blank_info(format!("{} · processed", source.info.name));
        let new_id = info.id.clone();
        info.kind = "processed".into();
        info.file = format!("processed-{new_id}.wav");
        info.start_frame = start;
        info.channels = 2;
        info.tone = format!(
            "Recorded-audio drive/EQ: drive {}, bass {}, treble {}, level {}",
            settings.drive, settings.bass, settings.treble, settings.level
        );
        media::write_wav(&self.storage.join(&info.file), self.sample_rate, 2, &audio)?;
        let mut draft = self.draft();
        if let Some(original) = draft.iter_mut().find(|t| t.info.id == id) {
            original.controls.mute.store(true, Ordering::Relaxed);
            original.controls.solo.store(false, Ordering::Relaxed);
        }
        draft.push(AudioTrack::new(info, audio));
        self.commit_tracks(draft)?;
        Ok(new_id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effects_make_a_processed_copy_without_rewriting_the_recording() {
        let root = std::env::temp_dir().join(format!("kuerdas-fx-{}", media::unique_id()));
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        let source = root.join("source.wav");
        media::write_wav(&source, 44100, 1, &vec![0.15; 4410]).unwrap();
        engine.import(&source, 44100).unwrap();
        let original = engine.tracks[0].info.id.clone();
        let original_path = root.join(&engine.tracks[0].info.file);
        let bytes = std::fs::read(&original_path).unwrap();
        let processed = engine
            .process_recorded_track(
                &original,
                RecordedEffects {
                    drive: 8.0,
                    bass: 0.1,
                    treble: -0.2,
                    level: 0.5,
                },
            )
            .unwrap();
        assert_eq!(engine.tracks.len(), 2);
        assert!(engine.tracks[0].controls.mute.load(Ordering::Relaxed));
        let track = engine
            .tracks
            .iter()
            .find(|t| t.info.id == processed)
            .unwrap();
        assert_eq!(track.info.frames, 4410);
        assert!(track.clips[0]
            .samples
            .iter()
            .all(|s| s.is_finite() && s.abs() <= 1.0));
        assert_eq!(std::fs::read(original_path).unwrap(), bytes);
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
}
