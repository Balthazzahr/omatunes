use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rustfft::{num_complex::Complex, FftPlanner};

pub const NUM_BANDS: usize = 144;
const FFT_SIZE: usize = 2048;

const FREQ_MIN: f32 = 80.0;
const FREQ_MAX: f32 = 20000.0;
const SAMPLE_RATE: f32 = 48000.0;

pub struct SpectrumAnalyzer {
    planner: FftPlanner<f32>,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    peak_hold: [f32; NUM_BANDS],
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

        let peak = samples.iter().cloned().map(f32::abs).fold(1e-6f32, f32::max);
        let samples: Vec<f32> = samples.iter().map(|s| s / peak).collect();

        let fft = self.planner.plan_fft_forward(FFT_SIZE);

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

        let hz_to_bin = |hz: f32| -> usize {
            ((hz / SAMPLE_RATE) * FFT_SIZE as f32) as usize
        };

        let mut bands = [0.0f32; NUM_BANDS];
        let log_min = FREQ_MIN.log2();
        let log_max = FREQ_MAX.log2();

        let mut last_idx_lo = 0;
        for (i, band) in bands.iter_mut().enumerate() {
            let lo_hz = 2f32.powf(log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32);
            let hi_hz = 2f32.powf(log_min + (log_max - log_min) * (i + 1) as f32 / NUM_BANDS as f32);

            let mut idx_lo = hz_to_bin(lo_hz).clamp(0, half - 1);
            
            if i > 0 && idx_lo <= last_idx_lo {
                idx_lo = (last_idx_lo + 1).clamp(0, half - 1);
            }
            last_idx_lo = idx_lo;

            let idx_hi = hz_to_bin(hi_hz).clamp(idx_lo + 1, half);

            let sum: f32 = magnitudes[idx_lo..idx_hi].iter().sum();
            let count = (idx_hi - idx_lo).max(1) as f32;
            *band = sum / count;
        }

        const PEAK_DECAY: f32 = 0.995;
        const PEAK_FLOOR: f32 = 1e-6;
        const AMPLITUDE_CAP: f32 = 1.0;
        const MIN_VISUAL_THRESHOLD: f32 = 0.002;

        for (i, band) in bands.iter_mut().enumerate() {
            self.peak_hold[i] = (self.peak_hold[i] * PEAK_DECAY).max(PEAK_FLOOR);
            if *band > self.peak_hold[i] {
                self.peak_hold[i] = *band;
            }
            
            let raw_energy = *band;
            *band = (raw_energy / self.peak_hold[i]).clamp(0.0, AMPLITUDE_CAP);
            
            if raw_energy < MIN_VISUAL_THRESHOLD {
                let scaling_factor = raw_energy / MIN_VISUAL_THRESHOLD;
                *band *= scaling_factor;
            }
        }

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
