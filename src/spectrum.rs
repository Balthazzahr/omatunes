use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rustfft::{num_complex::Complex, FftPlanner};

pub const NUM_BANDS: usize = 64;
const FFT_SIZE: usize = 2048;

pub struct SpectrumAnalyzer {
    planner: FftPlanner<f32>,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    peak_hold: [f32; NUM_BANDS],
}

impl SpectrumAnalyzer {
    pub fn new(sample_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        SpectrumAnalyzer {
            planner: FftPlanner::new(),
            sample_buffer,
            peak_hold: [1e-6; NUM_BANDS],
        }
    }

    /// Retorna amplitudes normalizadas [0.0, 1.0] para cada uma das NUM_BANDS bandas.
    pub fn compute(&mut self) -> [f32; NUM_BANDS] {
        let samples: Vec<f32> = {
            let buf = self.sample_buffer.lock().unwrap();
            if buf.len() < FFT_SIZE {
                return [0.0; NUM_BANDS];
            }
            buf.iter().rev().take(FFT_SIZE).cloned().collect()
        };

        let fft = self.planner.plan_fft_forward(FFT_SIZE);

        // Janela de Hann para reduzir leakage espectral
        let mut input: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos());
                Complex { re: s * window, im: 0.0 }
            })
            .collect();

        fft.process(&mut input);

        // Apenas metade positiva do espectro
        let half = FFT_SIZE / 2;
        let magnitudes: Vec<f32> = input[..half]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt() / FFT_SIZE as f32)
            .collect();

        // Agregar em NUM_BANDS bandas em escala logarítmica
        let mut bands = [0.0f32; NUM_BANDS];
        let log_min = (1f32).log2();
        let log_max = (half as f32).log2();

        for (i, band) in bands.iter_mut().enumerate() {
            let lo = log_min + (log_max - log_min) * i as f32 / NUM_BANDS as f32;
            let hi = log_min + (log_max - log_min) * (i + 1) as f32 / NUM_BANDS as f32;

            let idx_lo = (2f32.powf(lo) as usize).clamp(0, half - 1);
            let idx_hi = (2f32.powf(hi) as usize).clamp(idx_lo + 1, half);

            let sum: f32 = magnitudes[idx_lo..idx_hi].iter().sum();
            let count = (idx_hi - idx_lo).max(1) as f32;
            *band = sum / count;
        }

        // Per-band peak normalisation with slow decay.
        // Each band normalises against its own recent peak so bass cannot
        // permanently dominate mids and highs.
        const PEAK_DECAY: f32 = 0.995;
        const PEAK_FLOOR: f32 = 1e-6;

        for (i, band) in bands.iter_mut().enumerate() {
            self.peak_hold[i] = (self.peak_hold[i] * PEAK_DECAY).max(PEAK_FLOOR);
            if *band > self.peak_hold[i] {
                self.peak_hold[i] = *band;
            }
            *band = (*band / self.peak_hold[i]).clamp(0.0, 1.0);
        }

        bands
    }
}
