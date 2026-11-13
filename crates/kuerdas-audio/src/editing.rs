use super::*;
fn name(value: &str) -> Result<String, String> {
    let name: String = value.trim().chars().take(80).collect();
    if name.is_empty() {
        Err("Enter a name".into())
    } else {
        Ok(name)
    }
}
pub(super) fn blank_info(label: String) -> TrackInfo {
    TrackInfo {
        id: media::unique_id(),
        name: label,
        kind: "audio".into(),
        file: String::new(),
        start_frame: 0,
        frames: 0,
        channels: 1,
        gain: 1.0,
        pan: 0.0,
        mute: false,
        solo: false,
        tone: String::new(),
        clips: vec![],
        peaks: vec![],
    }
}
impl Engine {
    pub(super) fn editable(&self) -> Result<(), String> {
        if self.storage.as_os_str().is_empty() {
            return Err("Create or open a named workspace first".into());
        }
        if self.streams.is_some() {
            Err("Stop playback/recording before editing tracks".into())
        } else {
            Ok(())
        }
    }
    pub(super) fn draft(&self) -> Vec<AudioTrack> {
        self.tracks.iter().map(AudioTrack::detached).collect()
    }
    pub(super) fn commit_tracks(&mut self, mut draft: Vec<AudioTrack>) -> Result<(), String> {
        self.editable()?;
        if draft.len() > MAX_TRACKS
            || draft.iter().map(|t| t.clips.len()).sum::<usize>() > MAX_CLIPS
        {
            return Err("Session limit: 16 tracks and 128 clips".into());
        }
        let samples: usize = draft
            .iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.samples.len())
            .sum();
        if samples > MAX_SAMPLES {
            return Err("Session exceeds the 512 MB audio-cache limit".into());
        }
        for track in &mut draft {
            for clip in &track.clips {
                if clip
                    .info
                    .start_frame
                    .checked_add(clip.info.frames)
                    .is_none_or(|end| end > self.sample_rate as u64 * 3600)
                {
                    return Err("Clips must fit within the one-hour timeline".into());
                }
            }
            track.refresh();
        }
        let old = std::mem::replace(&mut self.tracks, draft);
        if let Err(error) = self.save() {
            self.tracks = old;
            return Err(error);
        }
        self.revision += 1;
        Ok(())
    }
    pub fn create_track(&mut self, label: &str, rate: u32) -> Result<String, String> {
        self.editable()?;
        let info = blank_info(name(label)?);
        let id = info.id.clone();
        let mut draft = self.draft();
        draft.push(AudioTrack::from_clips(info, vec![]));
        if ![44100, 48000].contains(&rate) {
            return Err("Invalid session rate".into());
        }
        let old = self.sample_rate;
        if !self.has_audio() {
            self.sample_rate = rate;
        }
        if let Err(e) = self.commit_tracks(draft) {
            self.sample_rate = old;
            return Err(e);
        }
        Ok(id)
    }
    pub fn rename_track(&mut self, id: &str, label: &str) -> Result<(), String> {
        self.editable()?;
        let mut draft = self.draft();
        draft
            .iter_mut()
            .find(|t| t.info.id == id)
            .ok_or("Track not found")?
            .info
            .name = name(label)?;
        self.commit_tracks(draft)
    }
    pub fn rename_clip(&mut self, id: &str, label: &str) -> Result<(), String> {
        self.editable()?;
        let mut draft = self.draft();
        draft
            .iter_mut()
            .flat_map(|t| &mut t.clips)
            .find(|c| c.info.id == id)
            .ok_or("Clip not found")?
            .info
            .name = name(label)?;
        self.commit_tracks(draft)
    }
    pub fn move_clip(&mut self, id: &str, target: &str, frame: u64) -> Result<(), String> {
        self.editable()?;
        let mut draft = self.draft();
        let destination = draft
            .iter()
            .position(|t| t.info.id == target)
            .ok_or("Destination track not found")?;
        let (source, index) = draft
            .iter()
            .enumerate()
            .find_map(|(row, t)| {
                t.clips
                    .iter()
                    .position(|c| c.info.id == id)
                    .map(|index| (row, index))
            })
            .ok_or("Clip not found")?;
        let mut clip = draft[source].clips.remove(index);
        clip.info.start_frame = frame;
        draft[destination].clips.push(clip);
        draft[destination].clips.sort_by_key(|c| c.info.start_frame);
        self.commit_tracks(draft)
    }
    pub fn split_clip(&mut self, id: &str, frame: u64) -> Result<(), String> {
        self.editable()?;
        let mut draft = self.draft();
        let track = draft
            .iter_mut()
            .find(|t| t.clips.iter().any(|c| c.info.id == id))
            .ok_or("Clip not found")?;
        let index = track.clips.iter().position(|c| c.info.id == id).unwrap();
        let original = &mut track.clips[index];
        let left = frame
            .checked_sub(original.info.start_frame)
            .filter(|n| *n > 0 && *n < original.info.frames)
            .ok_or("Place the playhead inside the selected clip")?;
        let mut right = original.clone();
        right.info.id = media::unique_id();
        right.info.name = format!("{} (2)", right.info.name);
        right.info.start_frame = frame;
        right.info.source_offset += left;
        right.info.frames -= left;
        original.info.frames = left;
        for clip in [&mut *original, &mut right] {
            let start = clip.info.source_offset as usize * clip.info.channels as usize;
            let end = start + clip.info.frames as usize * clip.info.channels as usize;
            clip.info.peaks = media::peaks(&clip.samples[start..end], clip.info.channels as usize);
        }
        track.clips.insert(index + 1, right);
        self.commit_tracks(draft)
    }
    pub fn import_many(&mut self, paths: &[PathBuf], rate: u32) -> Result<(), String> {
        self.editable()?;
        if paths.is_empty() {
            return Ok(());
        }
        if self.tracks.len() + paths.len() > MAX_TRACKS
            || self.clip_count() + paths.len() > MAX_CLIPS
        {
            return Err("Import exceeds the session track/clip limit".into());
        }
        if ![44100, 48000].contains(&rate) {
            return Err("Invalid session rate".into());
        }
        let target = if self.has_audio() {
            self.sample_rate
        } else {
            rate
        };
        let mut draft = self.draft();
        let mut total = self.sample_count();
        for path in paths {
            let (data, channels) = media::decode_audio(path, target)?;
            total += data.len();
            if total > MAX_SAMPLES {
                return Err("Imports exceed the 512 MB session cache".into());
            }
            let mut info = blank_info(name(
                &path.file_stem().unwrap_or_default().to_string_lossy(),
            )?);
            info.kind = "backing".into();
            info.file = format!("import-{}.wav", info.id);
            info.channels = channels;
            info.gain = 0.6;
            media::write_wav(&self.storage.join(&info.file), target, channels, &data)?;
            draft.push(AudioTrack::new(info, data));
        }
        let old = self.sample_rate;
        self.sample_rate = target;
        if let Err(e) = self.commit_tracks(draft) {
            self.sample_rate = old;
            return Err(e);
        }
        Ok(())
    }
    pub(super) fn selected_audio(&self, ids: &[String]) -> Result<Vec<AudioTrack>, String> {
        if ids.is_empty() {
            return Err("Select tracks to combine".into());
        }
        let mut selected = Vec::new();
        for id in ids {
            if selected.iter().any(|t: &AudioTrack| &t.info.id == id) {
                continue;
            }
            let track = self
                .tracks
                .iter()
                .find(|t| &t.info.id == id)
                .ok_or("Selected track not found")?;
            let detached = track.detached();
            detached.controls.mute.store(false, Ordering::Relaxed);
            detached.controls.solo.store(false, Ordering::Relaxed);
            selected.push(detached);
        }
        Ok(selected)
    }
    pub fn combine_tracks(&mut self, ids: &[String], label: &str) -> Result<String, String> {
        self.editable()?;
        let selected = self.selected_audio(ids)?;
        if selected.iter().map(|t| t.clips.len()).sum::<usize>() < 2 {
            return Err("Select at least two audio clips to combine".into());
        }
        if self.tracks.len() >= MAX_TRACKS {
            return Err("Create room for the combined track (maximum 16 tracks)".into());
        }
        let start = selected
            .iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.info.start_frame)
            .min()
            .unwrap();
        let end = selected
            .iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.info.start_frame + c.info.frames)
            .max()
            .unwrap();
        let frames = end - start;
        if frames > self.sample_rate as u64 * 600 {
            return Err("Combined selection must span ten minutes or less".into());
        }
        if self.sample_count() + frames as usize * 2 > MAX_SAMPLES {
            return Err("Combined audio exceeds session memory limit".into());
        }
        let mut info = blank_info(name(label)?);
        let id = info.id.clone();
        info.kind = "combined".into();
        info.file = format!("combined-{id}.wav");
        info.channels = 2;
        info.start_frame = start;
        info.tone = "Combined from selected tracks (source volume/pan applied)".into();
        let controls = mix_controls(&selected);
        let mut samples = Vec::with_capacity(frames as usize * 2);
        for frame in start..end {
            let sample = mix_at(&selected, &controls, frame);
            samples.extend(sample.map(|x| x.clamp(-1.0, 1.0)));
        }
        media::write_wav(
            &self.storage.join(&info.file),
            self.sample_rate,
            2,
            &samples,
        )?;
        let mut draft = self.draft();
        for track in &mut draft {
            if ids.contains(&track.info.id) {
                track.controls.mute.store(true, Ordering::Relaxed);
                track.controls.solo.store(false, Ordering::Relaxed);
            }
        }
        draft.push(AudioTrack::new(info, samples));
        self.commit_tracks(draft)?;
        Ok(id)
    }
    pub fn export_track(&self, id: &str, path: &Path) -> Result<(), String> {
        self.editable()?;
        let tracks = self.selected_audio(&[id.to_string()])?;
        let end = tracks[0].info.start_frame + tracks[0].info.frames;
        if end == 0 {
            return Err("This track has no audio".into());
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
        let controls = mix_controls(&tracks);
        for frame in 0..end {
            for sample in mix_at(&tracks, &controls, frame) {
                writer
                    .write_sample(sample.clamp(-1.0, 1.0))
                    .map_err(|e| e.to_string())?;
            }
        }
        writer.finalize().map_err(|e| e.to_string())?;
        std::fs::rename(temporary, path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("kuerdas-edits-{}", media::unique_id()))
    }
    fn imported() -> (Engine, PathBuf) {
        let root = root();
        std::fs::create_dir_all(&root).unwrap();
        let a = root.join("a.wav");
        let b = root.join("b.wav");
        media::write_wav(&a, 44100, 1, &[0.1, 0.2, 0.3, 0.4]).unwrap();
        media::write_wav(&b, 44100, 1, &[0.5, 0.6]).unwrap();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        engine.import_many(&[a, b], 44100).unwrap();
        for id in engine
            .tracks
            .iter()
            .map(|t| t.info.id.clone())
            .collect::<Vec<_>>()
        {
            engine.update_track(&id, 1.0, 0.0, false, false).unwrap();
        }
        (engine, root)
    }
    #[test]
    fn clips_move_between_tracks_split_and_reopen_without_rewriting_sources() {
        let (mut engine, root) = imported();
        let source = engine.tracks[0].info.id.clone();
        let clip = engine.tracks[0].clips[0].info.id.clone();
        let bytes = std::fs::read(root.join(&engine.tracks[0].clips[0].info.file)).unwrap();
        let destination = engine.create_track("Lead", 44100).unwrap();
        engine.rename_track(&source, "Rhythm").unwrap();
        engine.rename_clip(&clip, "Verse").unwrap();
        engine.move_clip(&clip, &destination, 10).unwrap();
        engine.split_clip(&clip, 12).unwrap();
        assert!(engine.tracks[0].clips.is_empty());
        let track = engine
            .tracks
            .iter()
            .find(|t| t.info.id == destination)
            .unwrap();
        assert_eq!(track.clips.len(), 2);
        assert_eq!(track.clips[1].info.source_offset, 2);
        assert_eq!(track.stereo_at(10), [0.1; 2]);
        assert_eq!(track.stereo_at(12), [0.3; 2]);
        assert_eq!(
            std::fs::read(root.join(&track.clips[0].info.file)).unwrap(),
            bytes
        );
        drop(engine);
        let mut reopened = Engine::default();
        reopened.configure_storage(root.clone()).unwrap();
        let track = reopened
            .tracks
            .iter()
            .find(|t| t.info.id == destination)
            .unwrap();
        assert_eq!(track.clips.len(), 2);
        assert_eq!(track.clips[0].info.name, "Verse");
        assert_eq!(track.stereo_at(13), [0.4; 2]);
        assert_eq!(reopened.tracks[0].info.name, "Rhythm");
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn combine_renders_positions_and_retains_muted_originals() {
        let (mut engine, root) = imported();
        let ids: Vec<_> = engine.tracks.iter().map(|t| t.info.id.clone()).collect();
        let a = engine.tracks[0].clips[0].info.id.clone();
        let b = engine.tracks[1].clips[0].info.id.clone();
        engine.move_clip(&a, &ids[0], 10).unwrap();
        engine.move_clip(&b, &ids[1], 12).unwrap();
        let combined = engine.combine_tracks(&ids, "Combined rhythm").unwrap();
        assert_eq!(engine.tracks.len(), 3);
        let track = engine
            .tracks
            .iter()
            .find(|t| t.info.id == combined)
            .unwrap();
        assert_eq!(track.info.start_frame, 10);
        assert_eq!(track.info.frames, 4);
        assert!((track.stereo_at(12)[0] - 0.8).abs() < 1e-6);
        assert!((track.stereo_at(13)[0] - 1.0).abs() < 1e-6);
        for original in &engine.tracks[..2] {
            assert!(original.controls.mute.load(Ordering::Relaxed));
            assert!(root.join(&original.clips[0].info.file).exists());
        }
        assert!((mix_at(&engine.tracks, &mix_controls(&engine.tracks), 12)[0] - 0.8).abs() < 1e-6);
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn legacy_session_is_backed_up_and_migrated_before_editing() {
        let root = root();
        std::fs::create_dir_all(&root).unwrap();
        media::write_wav(&root.join("legacy.wav"), 44100, 1, &[0.1, 0.2, 0.3, 0.4]).unwrap();
        let legacy=serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"sampleRate":44100,"tracks":[{"id":"old","name":"Original take","kind":"take","file":"legacy.wav","startFrame":8,"frames":4,"channels":1,"gain":1.0,"pan":0.0,"mute":false,"solo":false,"tone":"KPT Pro"}]})).unwrap();
        std::fs::write(root.join("session.json"), &legacy).unwrap();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        assert_eq!(
            std::fs::read(root.join("session-v1-backup.json")).unwrap(),
            legacy
        );
        assert_eq!(engine.tracks[0].clips.len(), 1);
        assert_eq!(engine.tracks[0].stereo_at(8), [0.1; 2]);
        engine.rename_track("old", "Renamed").unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("session.json")).unwrap()).unwrap();
        assert_eq!(json["schemaVersion"], 2);
        drop(engine);
        let mut reopened = Engine::default();
        reopened.configure_storage(root.clone()).unwrap();
        assert_eq!(reopened.tracks[0].info.name, "Renamed");
        assert_eq!(
            std::fs::read(root.join("session-v1-backup.json")).unwrap(),
            legacy
        );
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_moves_do_not_change_current_session() {
        let (mut engine, root) = imported();
        let id = engine.tracks[0].clips[0].info.id.clone();
        let track = engine.tracks[0].info.id.clone();
        let before = std::fs::read(root.join("session.json")).unwrap();
        assert!(engine.move_clip(&id, &track, 44100 * 3600 + 1).is_err());
        assert_eq!(engine.tracks[0].clips[0].info.start_frame, 0);
        assert_eq!(std::fs::read(root.join("session.json")).unwrap(), before);
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn new_take_can_be_appended_to_an_existing_track() {
        let root = root();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        let id = engine.create_track("Rhythm", 44100).unwrap();
        for start in [0, 1000] {
            let (recorder, mut writer) = Recorder::new(
                root.join(format!("take-{start}.wav")),
                44100,
                start,
                "KPT Pro".into(),
            )
            .unwrap();
            for _ in 0..500 {
                writer.push(0.1).unwrap();
            }
            drop(writer);
            engine.recorder = Some(recorder);
            engine.recording_target = Some(id.clone());
            engine.stop();
            assert!(engine.last_error.is_none());
        }
        assert_eq!(engine.tracks.len(), 1);
        assert_eq!(engine.tracks[0].clips.len(), 2);
        assert_eq!(engine.duration(), 1500);
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn common_format_uploads_decode_and_resample() {
        let root = root();
        let mut engine = Engine::default();
        engine.configure_storage(root.clone()).unwrap();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let paths =
            ["tone.mp3", "tone.flac", "tone.m4a", "tone.ogg"].map(|name| fixtures.join(name));
        engine.import_many(&paths, 48000).unwrap();
        assert_eq!(engine.tracks.len(), 4);
        for track in &engine.tracks {
            assert!(track.info.frames > 1000 && track.info.frames < 48000);
            assert!(track.clips[0].samples.iter().any(|s| s.abs() > 0.01));
        }
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
}
