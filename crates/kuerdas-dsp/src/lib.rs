//! Allocation-free mono guitar processing. Params cross threads via atomics.
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub struct Parameters {
    pub drive: AtomicU32,
    pub output: AtomicU32,
    pub gate: AtomicU32,
    pub bypass: AtomicBool,
    bass: AtomicU32,
    treble: AtomicU32,
    delay_ms: AtomicU32,
    delay_mix: AtomicU32,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            drive: AtomicU32::new(3.0f32.to_bits()),
            output: AtomicU32::new(0.25f32.to_bits()),
            gate: AtomicU32::new(0.005f32.to_bits()),
            bypass: AtomicBool::new(false),
            bass: AtomicU32::new(0f32.to_bits()),
            treble: AtomicU32::new(0f32.to_bits()),
            delay_ms: AtomicU32::new(320f32.to_bits()),
            delay_mix: AtomicU32::new(0f32.to_bits()),
        }
    }
}
impl Parameters {
    pub fn update_color(
        &self,
        bass: f32,
        treble: f32,
        delay_ms: f32,
        delay_mix: f32,
    ) -> Result<(), String> {
        if [bass, treble, delay_ms, delay_mix]
            .iter()
            .any(|p| !p.is_finite())
        {
            return Err("Tone controls must be finite".into());
        }
        self.bass
            .store(bass.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
        self.treble
            .store(treble.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
        self.delay_ms
            .store(delay_ms.clamp(50.0, 1000.0).to_bits(), Ordering::Relaxed);
        self.delay_mix
            .store(delay_mix.clamp(0.0, 0.6).to_bits(), Ordering::Relaxed);
        Ok(())
    }
    pub fn update(&self, drive: f32, output: f32, gate: f32, bypass: bool) -> Result<(), String> {
        if !drive.is_finite() || !output.is_finite() || !gate.is_finite() {
            return Err("Parameters must be finite".into());
        }
        self.drive
            .store(drive.clamp(1.0, 20.0).to_bits(), Ordering::Relaxed);
        self.output
            .store(output.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        self.gate
            .store(gate.clamp(0.0, 0.1).to_bits(), Ordering::Relaxed);
        self.bypass.store(bypass, Ordering::Relaxed);
        Ok(())
    }
}
pub struct GuitarRig {
    gain: f32,
    level: f32,
    gate_gain: f32,
    envelope: f32,
    smoothing: f32,
    envelope_release: f32,
    gate_smoothing: f32,
    delay: Vec<f32>,
    delay_index: usize,
    rate: f32,
    low: f32,
    high_low: f32,
    low_coeff: f32,
    high_coeff: f32,
    bass: f32,
    treble: f32,
    mix: f32,
}
impl Default for GuitarRig {
    fn default() -> Self {
        Self::for_sample_rate(48_000)
    }
}
impl GuitarRig {
    /// Coefficients are prepared on the control thread, never in the callback.
    pub fn for_sample_rate(sample_rate: u32) -> Self {
        let ratio = 48_000.0 / sample_rate.max(1) as f32;
        Self {
            gain: 3.0,
            level: 0.25,
            gate_gain: 0.0,
            envelope: 0.0,
            smoothing: 1.0 - 0.998f32.powf(ratio),
            envelope_release: 0.9995f32.powf(ratio),
            gate_smoothing: 1.0 - 0.995f32.powf(ratio),
            delay: vec![0.0; sample_rate.max(1) as usize + 1],
            delay_index: 0,
            rate: sample_rate.max(1) as f32,
            low: 0.0,
            high_low: 0.0,
            low_coeff: 1.0
                - (-2.0 * std::f32::consts::PI * 250.0 / sample_rate.max(1) as f32).exp(),
            high_coeff: 1.0
                - (-2.0 * std::f32::consts::PI * 3000.0 / sample_rate.max(1) as f32).exp(),
            bass: 0.0,
            treble: 0.0,
            mix: 0.0,
        }
    }
    pub fn process(&mut self, input: f32, params: &Parameters) -> f32 {
        let input = if input.is_finite() {
            input.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        // One-pole smoothing avoids abrupt gain and gate changes. Coefficients preserve time constants across rates.
        self.gain +=
            self.smoothing * (f32::from_bits(params.drive.load(Ordering::Relaxed)) - self.gain);
        self.level +=
            self.smoothing * (f32::from_bits(params.output.load(Ordering::Relaxed)) - self.level);
        self.envelope = input.abs().max(self.envelope * self.envelope_release);
        let threshold = f32::from_bits(params.gate.load(Ordering::Relaxed));
        let target = if self.envelope >= threshold { 1.0 } else { 0.0 };
        self.gate_gain += self.gate_smoothing * (target - self.gate_gain);
        let wet = if params.bypass.load(Ordering::Relaxed) {
            input
        } else {
            (input * self.gate_gain * self.gain).tanh()
        };
        if params.bypass.load(Ordering::Relaxed) {
            return (input * self.level).clamp(-1.0, 1.0);
        }
        self.bass +=
            self.smoothing * (f32::from_bits(params.bass.load(Ordering::Relaxed)) - self.bass);
        self.treble +=
            self.smoothing * (f32::from_bits(params.treble.load(Ordering::Relaxed)) - self.treble);
        self.mix +=
            self.smoothing * (f32::from_bits(params.delay_mix.load(Ordering::Relaxed)) - self.mix);
        self.low += self.low_coeff * (wet - self.low);
        self.high_low += self.high_coeff * (wet - self.high_low);
        let colored = wet + self.low * self.bass + (wet - self.high_low) * self.treble;
        let offset =
            (f32::from_bits(params.delay_ms.load(Ordering::Relaxed)) * self.rate / 1000.0) as usize;
        let read = (self.delay_index + self.delay.len() - offset.min(self.delay.len() - 1))
            % self.delay.len();
        let delayed = self.delay[read];
        self.delay[self.delay_index] = (colored + delayed * 0.3).clamp(-2.0, 2.0);
        self.delay_index = (self.delay_index + 1) % self.delay.len();
        ((colored * (1.0 - self.mix) + delayed * self.mix) * self.level).clamp(-1.0, 1.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_is_finite_and_bounded() {
        let p = Parameters::default();
        let mut rig = GuitarRig::default();
        for x in [f32::NAN, f32::INFINITY, -100.0, 100.0, 0.0] {
            for _ in 0..1000 {
                let y = rig.process(x, &p);
                assert!(y.is_finite() && y.abs() <= 1.0);
            }
        }
    }
    #[test]
    fn bypass_preserves_polarity_and_applies_output_gain() {
        let p = Parameters::default();
        p.update(20.0, 0.5, 0.1, true).unwrap();
        let mut rig = GuitarRig::default();
        for _ in 0..10000 {
            rig.process(-0.5, &p);
        }
        assert!((rig.process(-0.5, &p) + 0.25).abs() < 0.001);
    }
    #[test]
    fn smoothing_time_matches_at_both_hardware_rates() {
        let p = Parameters::default();
        p.update(3.0, 0.8, 0.005, true).unwrap();
        let mut outputs = Vec::new();
        for rate in [44_100, 48_000] {
            let mut rig = GuitarRig::for_sample_rate(rate);
            for _ in 0..(rate / 100) {
                rig.process(0.5, &p);
            }
            outputs.push(rig.process(0.5, &p));
        }
        assert!((outputs[0] - outputs[1]).abs() < 0.0001);
    }
    #[test]
    fn delay_echo_occurs_at_selected_time_and_stays_finite() {
        for rate in [44100, 48000] {
            let p = Parameters::default();
            p.update(1.0, 0.5, 0.0, false).unwrap();
            p.update_color(0.0, 0.0, 50.0, 0.5).unwrap();
            let mut rig = GuitarRig::for_sample_rate(rate);
            rig.process(0.5, &p);
            for _ in 1..rate / 20 {
                assert_eq!(rig.process(0.0, &p), 0.0);
            }
            assert!(rig.process(0.0, &p) > 0.0);
            for _ in 0..rate {
                let output = rig.process(0.0, &p);
                assert!(output.is_finite() && output.abs() <= 1.0);
            }
        }
    }
    #[test]
    fn invalid_controls_are_rejected() {
        assert!(Parameters::default()
            .update(f32::NAN, 1.0, 0.0, false)
            .is_err());
    }
}
