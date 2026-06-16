use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rustfft::{num_complex::Complex, FftPlanner};

// Display 48 bands instead of 64 — gives each band more FFT bins,
// eliminating the blocky identical-bar problem in the low frequencies.
pub const NUM_BANDS: usize = 48;
const FFT_SIZE: usize = 2048;

// Frequency range to display. Starting at 80Hz cuts the sub-bass mud
// where log-scale bands are too narrow to be meaningful visually.
// 20kHz is the top of human hearing.
const FREQ_MIN: f32 = 80.0;
const FREQ_MAX: f32 = 20000.0;
const SAMPLE_RATE: f32 = 44100.0;

pub struct SpectrumAnalyzer {
    planner: FftPlanner<f32>,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    // Per-band peak for normalisation
    peak_hold: [f32; NUM_BANDS],
    // Per-band smoothed output — CAVA-style attack/decay
    smoothed: [f32; NUM_BANDS],
}

impl SpectrumAnalyzer {
    pub fn new(sample_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        SpectrumAnalyzer {
            planner: FftPlanner::new(),
            sample_buffer,
            peak_hold: [1e-6; NUM_BANDS],
            smoothed: [0.0; NUM_BANDS],
        }
    }

    pub fn compute(&mut self) -> [f32; NUM_BANDS] {
        let samples: Vec<f32> = {
            let buf = self.sample_buffer.lock().unwrap();
            if buf.len() < FFT_SIZE {
                return self.smoothed;
            }
            buf.iter().rev().take(FFT_SIZE).cloned().collect()
        };

        // Normalise raw samples so volume knob doesn't affect visualiser height.
        // Find the peak sample magnitude and scale everything to [-1, 1].
        let peak = samples.iter().cloned().map(f32::abs).fold(1e-6f32, f32::max);
        let samples: Vec<f32> = samples.iter().map(|s| s / peak).collect();

        let fft = self.planner.plan_fft_forward(FFT_SIZE);

        // Hann window to reduce spectral leakage
        let mut input: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let window = 0.5
                    * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32
                            / (FFT_SIZE - 1) as f32)
                            .cos());
                Complex { re: s * window, im: 0.0 }
            })
            .collect();

        fft.process(&mut input);

        let half = FFT_SIZE / 2;
        let magnitudes: Vec<f32> = input[..half]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt() / FFT_SIZE as f32)
            .collect();

        // Map frequency Hz to FFT bin index
        let hz_to_bin = |hz: f32| -> usize {
            ((hz / SAMPLE_RATE) * FFT_SIZE as f32) as usize
        };

        // Aggregate into NUM_BANDS on a log scale between FREQ_MIN and FREQ_MAX
        let mut bands = [0.0f32; NUM_BANDS];
        let log_min = FREQ_MIN.log2();
        let log_max = FREQ_MAX.log2();

        for (i, band) in bands.iter_mut().enumerate() {
            let lo_hz = 2f32.powf(log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32);
            let hi_hz = 2f32.powf(log_min + (log_max - log_min) * (i + 1) as f32 / NUM_BANDS as f32);

            let idx_lo = hz_to_bin(lo_hz).clamp(0, half - 1);
            let idx_hi = hz_to_bin(hi_hz).clamp(idx_lo + 1, half);

            let sum: f32 = magnitudes[idx_lo..idx_hi].iter().sum();
            let count = (idx_hi - idx_lo).max(1) as f32;
            *band = sum / count;
        }

        // Per-band peak normalisation with slow decay.
        // Each band normalises against its own recent peak so no frequency
        // range permanently dominates the display.
        const PEAK_DECAY: f32 = 0.995;
        const PEAK_FLOOR: f32 = 1e-6;
        // Cap at 0.85 so bars never fully fill — gives visual headroom.
        const AMPLITUDE_CAP: f32 = 0.85;

        for (i, band) in bands.iter_mut().enumerate() {
            self.peak_hold[i] = (self.peak_hold[i] * PEAK_DECAY).max(PEAK_FLOOR);
            if *band > self.peak_hold[i] {
                self.peak_hold[i] = *band;
            }
            *band = (*band / self.peak_hold[i]).clamp(0.0, AMPLITUDE_CAP);
        }

        // CAVA-style smoothing: fast attack, slow decay.
        // When signal rises, snap up quickly (attack = 0.6).
        // When signal falls, glide down slowly (decay = 0.12).
        // This gives the fluid organic movement of a good visualiser.
        const ATTACK: f32 = 0.6;
        const DECAY: f32  = 0.12;

        for (i, band) in bands.iter().enumerate() {
            let prev = self.smoothed[i];
            self.smoothed[i] = if *band > prev {
                prev + (*band - prev) * ATTACK
            } else {
                prev + (*band - prev) * DECAY
            };
        }

        self.smoothed
    }
}
