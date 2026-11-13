use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};
pub struct Recorder {
    done: Arc<AtomicBool>,
    pub failed: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<u64, String>>>,
    pub path: PathBuf,
    pub start_frame: u64,
    pub tone: String,
}
impl Recorder {
    pub fn new(
        path: PathBuf,
        rate: u32,
        start_frame: u64,
        tone: String,
    ) -> Result<(Self, rtrb::Producer<f32>), String> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let mut writer = hound::WavWriter::new(
            std::io::BufWriter::new(file),
            hound::WavSpec {
                channels: 1,
                sample_rate: rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .map_err(|e| e.to_string())?;
        let (producer, mut consumer) = rtrb::RingBuffer::<f32>::new(rate as usize * 2);
        let done = Arc::new(AtomicBool::new(false));
        let failed = Arc::new(AtomicBool::new(false));
        let finished = done.clone();
        let fault = failed.clone();
        let worker = std::thread::spawn(move || {
            let mut samples = 0;
            loop {
                // Recheck after a failed pop: producer may publish before done is set.
                let is_done = finished.load(Ordering::Acquire);
                match consumer.pop() {
                    Ok(sample) => {
                        if let Err(e) = writer.write_sample(sample) {
                            fault.store(true, Ordering::Relaxed);
                            return Err(e.to_string());
                        }
                        samples += 1;
                    }
                    Err(_) if is_done => break,
                    Err(_) => std::thread::sleep(std::time::Duration::from_millis(2)),
                }
            }
            writer.finalize().map_err(|e| {
                fault.store(true, Ordering::Relaxed);
                e.to_string()
            })?;
            if fault.load(Ordering::Relaxed) {
                Err("Recording lost samples or reached its ten-minute limit; incomplete take was retained on disk".into())
            } else {
                Ok(samples)
            }
        });
        Ok((
            Self {
                done,
                failed,
                worker: Some(worker),
                path,
                start_frame,
                tone,
            },
            producer,
        ))
    }
    // Call only after dropping the streams (and therefore the producer).
    pub fn finish(mut self) -> Result<u64, String> {
        self.done.store(true, Ordering::Release);
        self.worker
            .take()
            .unwrap()
            .join()
            .map_err(|_| "Recording writer failed".to_string())?
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writer_finalizes_all_queued_samples() {
        let path =
            std::env::temp_dir().join(format!("kuerdas-test-{}.wav", crate::media::unique_id()));
        let (recorder, mut producer) =
            Recorder::new(path.clone(), 44100, 0, "KPT Pro".into()).unwrap();
        for i in 0..1000 {
            producer.push(i as f32 / 1000.0).unwrap();
        }
        drop(producer);
        assert_eq!(recorder.finish().unwrap(), 1000);
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.duration(), 1000);
        assert_eq!(reader.spec().sample_rate, 44100);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn sample_loss_is_not_reported_as_success() {
        let path =
            std::env::temp_dir().join(format!("kuerdas-fault-{}.wav", crate::media::unique_id()));
        let (recorder, producer) = Recorder::new(path.clone(), 44100, 0, String::new()).unwrap();
        recorder.failed.store(true, Ordering::Relaxed);
        drop(producer);
        assert!(recorder.finish().is_err());
        std::fs::remove_file(path).unwrap();
    }
}
