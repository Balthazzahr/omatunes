use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, LineCap};
use iced::{Color, Element, Length, Point, Rectangle, Size};

use crate::app::Message;
use crate::audio::spectrum::NUM_BANDS;
use crate::ui::theme;

pub struct SpectrumView<'a> {
    pub bands: &'a [f32; NUM_BANDS],
    pub history: &'a std::collections::VecDeque<[f32; NUM_BANDS]>,
    pub mode: usize,
    pub tick: u32,
    pub sensitivity: f32,
    pub decay: f32,
    pub color_shift: f32,
    pub bar_count: usize,
    pub bg_mode: usize,
    pub bg_color: &'a str,
    pub aurora_preset: usize,
    pub depth_warp_speed: f32,
    pub kaleidoscope_axes: usize,
}

impl<'a, Message> canvas::Program<Message> for SpectrumView<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::TRANSPARENT,
        );

        // Render historical ghost frames ONLY for modes where ghosting is active (modes other than 0 and 1)
        if self.mode != 0 && self.mode != 1 {
            let hist_len = self.history.len();
            if hist_len > 0 {
                for (idx, hist_bands) in self.history.iter().enumerate() {
                    let age = hist_len - idx;
                    let age_factor = age as f32 / (hist_len as f32 + 1.0);
                    let alpha = (1.0 - age_factor * 0.85) * (1.0 - self.decay * 0.65).clamp(0.15, 0.95);
                    let shift = age as f32 * self.color_shift * 0.7;

                    if alpha > 0.01 {
                        self.render_ghost_mode(&mut frame, bounds, hist_bands, age, age_factor, alpha, shift, self.tick.saturating_sub(age as u32));
                    }
                }
            }
        }

        // Render live current frame
        self.render_mode(&mut frame, bounds, self.bands, 1.0, 0.0, self.tick);

        vec![frame.into_geometry()]
    }
}

impl<'a> SpectrumView<'a> {
    fn render_mode(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        match self.mode {
            0 => self.draw_mirrored_bars_with_peaks(frame, bounds, bands, alpha, shift, tick),
            1 => self.draw_radial_pulse_expanding_waves(frame, bounds, bands, alpha, shift, tick),
            2 => self.draw_liquid_ribbon(frame, bounds, bands, alpha, shift, tick, 0, 0.0),
            3 => self.draw_particle_constellation_extended(frame, bounds, bands, alpha, shift, tick, 0, 0.0),
            4 => self.draw_depth_tunnel(frame, bounds, bands, alpha, shift, tick, 0, 0.0),
            5 => self.draw_3d_wireframe_grid(frame, bounds, bands, alpha, shift, tick),
            6 => self.draw_kaleidoscope(frame, bounds, bands, alpha, shift, tick),
            7 => self.draw_cosmic_aurora(frame, bounds, bands, alpha, shift, tick),
            8 => self.draw_synthwave_horizon(frame, bounds, bands, alpha, shift, tick),
            _ => self.draw_mirrored_bars_with_peaks(frame, bounds, bands, alpha, shift, tick),
        }
    }

    fn render_ghost_mode(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], age: usize, age_factor: f32, alpha: f32, shift: f32, tick: u32) {
        match self.mode {
            2 => self.draw_liquid_ribbon(frame, bounds, bands, alpha, shift, tick, age, age_factor),
            3 => self.draw_particle_constellation_extended(frame, bounds, bands, alpha, shift, tick, age, age_factor),
            4 => self.draw_depth_tunnel(frame, bounds, bands, alpha, shift, tick, age, age_factor),
            5 => self.draw_3d_wireframe_grid(frame, bounds, bands, alpha, shift, tick),
            6 => self.draw_kaleidoscope(frame, bounds, bands, alpha, shift, tick),
            7 => self.draw_cosmic_aurora(frame, bounds, bands, alpha, shift, tick),
            8 => self.draw_synthwave_horizon(frame, bounds, bands, alpha, shift, tick),
            _ => {},
        }
    }

    // Mode 0: Live Mirrored Spectrograph Bars with Downsampling & Gravity Bouncing Peak Limiters
    fn draw_mirrored_bars_with_peaks(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let num_bars = self.bar_count.clamp(10, NUM_BANDS);
        let bar_width = (width / num_bars as f32) - 1.0;
        let gap = 1.0;
        let cy = height / 2.0;
        let tick_f = tick as f32;

        let bands_per_bar = (NUM_BANDS as f32 / num_bars as f32).max(1.0);

        for i in 0..num_bars {
            let start_band = (i as f32 * bands_per_bar) as usize;
            let end_band = ((i as f32 + 1.0) * bands_per_bar) as usize;
            let slice = &bands[start_band.min(NUM_BANDS - 1)..end_band.min(NUM_BANDS).max(start_band + 1)];
            let raw_amp = slice.iter().sum::<f32>() / slice.len() as f32;

            let amp = (raw_amp * self.sensitivity).clamp(0.0, 1.0);
            let x = i as f32 * (bar_width + gap);
            let half_h = (amp * height * 0.48).max(1.0);
            let y = cy - half_h;
            let bar_height = half_h * 2.0;

            let base_color = theme::spectrum_bar_color(amp);
            let color = apply_ghost_style(base_color, alpha, shift);

            // Live spectrograph bar
            let path = Path::rectangle(
                Point::new(x, y),
                Size::new(bar_width.max(1.0), bar_height),
            );
            frame.fill(&path, color);

            // Floating peak limiter with realistic gravity bounce off rising spectrograph bars
            let mut peak_amp = amp;
            let mut peak_vel = 0.0_f32;

            // Trace history to compute peak trajectory with gravity acceleration and restitution bounce
            for (hist_idx, hist_bands) in self.history.iter().enumerate().take(16) {
                let h_slice = &hist_bands[start_band.min(NUM_BANDS - 1)..end_band.min(NUM_BANDS).max(start_band + 1)];
                let h_amp = (h_slice.iter().sum::<f32>() / h_slice.len() as f32 * self.sensitivity).clamp(0.0, 1.0);
                
                // Gravity acceleration over history frames
                let dt = (hist_idx + 1) as f32 * 0.05;
                let gravity_fall = 0.5 * 9.8 * dt * dt * 0.08;
                let calculated_peak = (h_amp - gravity_fall).max(0.0);

                if calculated_peak > peak_amp {
                    peak_amp = calculated_peak;
                }
            }

            // Add subtle sine oscillation bounce when peak touches rising bar surface
            let bounce_mod = if (peak_amp - amp).abs() < 0.02 { (tick_f * 0.3 + i as f32).sin() * 0.015 } else { 0.0 };
            let final_peak_amp = (peak_amp + bounce_mod).max(amp);

            let peak_half_h = (final_peak_amp * height * 0.48).max(half_h);
            let cap_y_top = cy - peak_half_h - 2.5;
            let cap_y_bot = cy + peak_half_h + 0.5;

            let cap_color = apply_ghost_style(theme::accent(), alpha * 0.95, shift + 0.1);
            let cap_top = Path::rectangle(Point::new(x, cap_y_top), Size::new(bar_width.max(1.0), 2.5));
            let cap_bot = Path::rectangle(Point::new(x, cap_y_bot), Size::new(bar_width.max(1.0), 2.5));

            frame.fill(&cap_top, cap_color);
            frame.fill(&cap_bot, cap_color);
        }
    }

    // Mode 1: Radial Pulse with Clean Spike-Triggered Infinite Shockwaves & Radial Bass Sparks
    fn draw_radial_pulse_expanding_waves(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.85).max(20.0);
        let base_r = max_r * 0.20;
        let tick_f = tick as f32;

        let bass_energy = (bands[..10].iter().sum::<f32>() / 10.0 * self.sensitivity).clamp(0.0, 1.0);
        let low_mid_energy = (bands[10..35].iter().sum::<f32>() / 25.0 * self.sensitivity).clamp(0.0, 1.0);
        let high_mid_energy = (bands[35..75].iter().sum::<f32>() / 40.0 * self.sensitivity).clamp(0.0, 1.0);
        let treble_energy = (bands[75..144].iter().sum::<f32>() / 69.0 * self.sensitivity).clamp(0.0, 1.0);

        let group_energies = [
            (bass_energy, theme::accent(), 0.0_f32),
            (low_mid_energy, theme::spectrum_bar_color(0.85), 0.2_f32),
            (high_mid_energy, theme::spectrum_bar_color(0.55), 0.4_f32),
            (treble_energy, theme::spectrum_bar_color(0.35), 0.6_f32),
        ];

        // 1. Render expanding shockwave circles launched into infinity ONLY on genuine energy spikes
        let hist_len = self.history.len();
        if hist_len > 0 {
            for (group_idx, &(energy, color, color_offset)) in group_energies.iter().enumerate() {
                // High threshold trigger so rings only spawn on spikes rather than continuously
                if energy > 0.45 {
                    for (idx, hist_bands) in self.history.iter().enumerate() {
                        let age = hist_len - idx;
                        let range_energy = match group_idx {
                            0 => hist_bands[..10].iter().sum::<f32>() / 10.0,
                            1 => hist_bands[10..35].iter().sum::<f32>() / 25.0,
                            2 => hist_bands[35..75].iter().sum::<f32>() / 40.0,
                            _ => hist_bands[75..144].iter().sum::<f32>() / 69.0,
                        } * self.sensitivity;

                        if range_energy > 0.50 {
                            let ring_r = base_r + (age as f32 * 24.0 * (1.0 + group_idx as f32 * 0.2));
                            let opacity = (1.0 - (ring_r / (cx.max(cy) * 1.3))).clamp(0.0, 0.90) * alpha;
                            if opacity > 0.02 {
                                let ring_path = Path::circle(Point::new(cx, cy), ring_r);
                                let ring_color = apply_ghost_style(color, opacity, shift + color_offset);
                                frame.stroke(
                                    &ring_path,
                                    Stroke::default().with_color(ring_color).with_width(2.0 + range_energy * 3.0),
                                );
                            }
                        }
                    }
                }
            }
        }

        // 2. Radial Bass Spark Explosions shooting out on bass hits
        if bass_energy > 0.6 {
            let spark_count = 12;
            for s in 0..spark_count {
                let spark_angle = (s as f32 / spark_count as f32) * std::f32::consts::TAU + (tick_f * 0.05);
                let spark_dist = base_r + bass_energy * max_r * 0.65;
                let sx = cx + spark_dist * spark_angle.cos();
                let sy = cy + spark_dist * spark_angle.sin();

                let spark_path = Path::circle(Point::new(sx, sy), 2.5 + bass_energy * 3.5);
                let spark_color = apply_ghost_style(theme::accent(), alpha * 0.85, shift + s as f32 * 0.05);
                frame.fill(&spark_path, spark_color);
            }
        }

        // 3. Radial Spectrum Spokes emanating cleanly from base_r
        let num = NUM_BANDS;
        let core_r = base_r;
        for (i, &raw_amp) in bands.iter().enumerate() {
            let amp = (raw_amp * self.sensitivity).clamp(0.0, 1.0);
            let angle = (i as f32 / num as f32) * std::f32::consts::TAU - (std::f32::consts::FRAC_PI_2);
            let spoke_len = amp * (max_r - core_r);

            let inner_x = cx + core_r * angle.cos();
            let inner_y = cy + core_r * angle.sin();

            let outer_x = cx + (core_r + spoke_len) * angle.cos();
            let outer_y = cy + (core_r + spoke_len) * angle.sin();

            let spoke_color = apply_ghost_style(theme::spectrum_bar_color(amp), alpha, shift);
            let path = Path::line(Point::new(inner_x, inner_y), Point::new(outer_x, outer_y));
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(spoke_color)
                    .with_width(2.0)
                    .with_line_cap(LineCap::Round),
            );
        }
    }

    // Mode 2: Liquid Rainbow Ribbon Stream
    fn draw_liquid_ribbon(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32, age: usize, age_factor: f32) {
        let width = bounds.width;
        let height = bounds.height;
        let cy = height / 2.0;
        let tick_f = tick as f32;

        let layers = [
            (0.60, 0.025, theme::accent(), 3.5),
            (0.40, 0.040, theme::spectrum_bar_color(0.85), 2.5),
            (0.25, 0.060, theme::spectrum_bar_color(0.50), 1.8),
        ];

        let num_points = 64;
        let step_x = width / (num_points - 1) as f32;
        let y_drift = if age > 0 { (age as f32 * 5.0) * (if age % 2 == 0 { 1.0 } else { -1.0 }) } else { 0.0 };

        for (l_idx, &(amp_scale, freq_scale, color, stroke_w)) in layers.iter().enumerate() {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let phase = (tick_f * 0.04 * (l_idx + 1) as f32) - (age_factor * 0.8);

            for i in 0..num_points {
                let x = i as f32 * step_x;
                let band_idx = ((i as f32 / num_points as f32) * NUM_BANDS as f32) as usize % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

                let sine = ((i as f32 * freq_scale * 10.0) + phase).sin();
                let wave_y = cy + y_drift + (sine * amp * height * 0.4 * amp_scale);

                if i == 0 {
                    builder.move_to(Point::new(x, wave_y));
                } else {
                    builder.line_to(Point::new(x, wave_y));
                }
            }

            let path = builder.build();
            let rainbow_shift = shift + (l_idx as f32 * 0.4) + (age as f32 * 0.35);
            let wave_color = apply_ghost_style(color, alpha * (0.85 - (l_idx as f32 * 0.2)), rainbow_shift);

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(wave_color)
                    .with_width(if age > 0 { stroke_w * 0.85 } else { stroke_w })
                    .with_line_cap(LineCap::Round),
            );
        }
    }

    // Mode 3: Extended Multi-Loop Lissajous Constellation Web
    fn draw_particle_constellation_extended(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32, age: usize, age_factor: f32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let tick_f = tick as f32;
        let bass = (bands[..8].iter().sum::<f32>() / 8.0 * self.sensitivity).clamp(0.0, 1.0);

        let num_particles = 80;
        let mut points: Vec<(Point, f32, Color)> = Vec::with_capacity(num_particles);

        for i in 0..num_particles {
            let band_idx = (i * 2) % NUM_BANDS;
            let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

            // 3x Lissajous winding loops across full canvas
            let t = (i as f32 / num_particles as f32) * std::f32::consts::TAU * 3.0 + (tick_f * 0.008) - (age_factor * 0.2);

            let scale_x = bounds.width * 0.42 + amp * 40.0;
            let scale_y = bounds.height * 0.38 + bass * 30.0;

            let px = cx + scale_x * (t * 0.6).sin() * (t * 0.4 + tick_f * 0.003).cos();
            let py = cy + scale_y * (t * 0.8).cos() + (amp * 25.0 * (t * 4.0).sin());

            let particle_shift = shift + (i as f32 * 0.03);
            let color = apply_ghost_style(theme::spectrum_bar_color(amp), alpha, particle_shift);
            points.push((Point::new(px, py), amp, color));
        }

        // Intertwined web connections with long reach (120px reach threshold)
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let (p1, a1, c1) = points[i];
                let (p2, a2, _) = points[j];

                let dx = p1.x - p2.x;
                let dy = p1.y - p2.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 14400.0 {
                    let line_alpha = (1.0 - (dist_sq.sqrt() / 120.0)) * ((a1 + a2) * 0.5) * 0.55 * alpha;
                    if line_alpha > 0.02 {
                        let line_path = Path::line(p1, p2);
                        frame.stroke(
                            &line_path,
                            Stroke::default()
                                .with_color(Color { a: line_alpha, ..c1 })
                                .with_width(if age > 0 { 0.8 } else { 1.1 }),
                        );
                    }
                }
            }
        }

        // Draw particle nodes
        for (pt, amp, color) in points {
            let radius = if age > 0 { (1.8 + amp * 3.0) * (1.0 - age_factor * 0.4) } else { 2.5 + amp * 4.5 };
        let particle_path = Path::circle(pt, radius.max(1.0));
            frame.fill(&particle_path, color);
        }
    }

    // Mode 4: Hyperdrive Warp Depth Tunnel Flight
    fn draw_depth_tunnel(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32, age: usize, age_factor: f32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.90).max(20.0);
        let tick_f = tick as f32;

        let ring_count = 9;
        let num_vertices = 28;

        // Continuous forward warp motion offset based on tick
        let warp_phase = (tick_f * 0.035) % 1.0;

        let mut ring_pts: Vec<Vec<Point>> = Vec::with_capacity(ring_count);

        for k in 0..ring_count {
            // Forward z-motion: rings move continuously outward from depth 0.0 to 1.0
            let depth = ((k as f32 + warp_phase) / ring_count as f32) % 1.0;
            if depth < 0.05 { continue; } // Skip rings spawning right at center point

            let scale_r = base_perspective(depth) * max_r * (1.0 - age_factor * 0.4);
            let ring_alpha = (depth * 0.85) * alpha;

            let mut builder = iced::widget::canvas::path::Builder::new();
            let mut current_ring_pts = Vec::with_capacity(num_vertices);

            for v in 0..num_vertices {
                let angle = (v as f32 / num_vertices as f32) * std::f32::consts::TAU + (tick_f * 0.008 * (k as f32 + 1.0));
                let band_idx = (v * (NUM_BANDS / num_vertices)) % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

                let r_mod = scale_r + (amp * 26.0 * depth);
                let vx = cx + r_mod * angle.cos();
                let vy = cy + r_mod * angle.sin();

                let pt = Point::new(vx, vy);
                current_ring_pts.push(pt);

                if v == 0 {
                    builder.move_to(pt);
                } else {
                    builder.line_to(pt);
                }
            }
            builder.close();

            let path = builder.build();
            let avg_amp = bands[k * 12 % NUM_BANDS] * self.sensitivity;
            let base_color = theme::spectrum_bar_color(avg_amp);
            let color = apply_ghost_style(base_color, ring_alpha, shift + (k as f32 * 0.12));

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(color)
                    .with_width(if age > 0 { 1.0 } else { 1.2 + depth * 2.2 }),
            );

            ring_pts.push(current_ring_pts);
        }

        if ring_pts.len() > 1 {
            for k in 0..(ring_pts.len() - 1) {
                let outer_pts = &ring_pts[k + 1];
                let inner_pts = &ring_pts[k];

                let waterfall_shift = shift + (k as f32 * 0.18);
                let waterfall_color = apply_ghost_style(theme::accent(), alpha * 0.35, waterfall_shift);

                for v in (0..num_vertices).step_by(2) {
                    if v < outer_pts.len() && v < inner_pts.len() {
                        let p1 = outer_pts[v];
                        let p2 = inner_pts[v];
                        let line_path = Path::line(p1, p2);
                        frame.stroke(
                            &line_path,
                            Stroke::default()
                                .with_color(waterfall_color)
                                .with_width(1.0),
                        );
                    }
                }
            }
        }
    }

    // Mode 5: NEW - 3D Wireframe Depth Grid
    fn draw_3d_wireframe_grid(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, _tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let horizon_y = height * 0.45;

        let num_cols = 28;
        let num_rows = 14;

        for r in 0..num_rows {
            let row_t = (r as f32 + 1.0) / num_rows as f32;
            let y = horizon_y + (row_t * row_t) * (height - horizon_y);

            let mut builder = iced::widget::canvas::path::Builder::new();
            for c in 0..num_cols {
                let col_t = c as f32 / (num_cols - 1) as f32;
                let x_spread = (col_t - 0.5) * width * (0.2 + row_t * 1.1);
                let x = width * 0.5 + x_spread;

                let band_idx = (c * (NUM_BANDS / num_cols)) % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);
                let ridge_y = y - (amp * 40.0 * row_t);

                if c == 0 {
                    builder.move_to(Point::new(x, ridge_y));
                } else {
                    builder.line_to(Point::new(x, ridge_y));
                }
            }

            let line_color = apply_ghost_style(theme::accent(), alpha * row_t, shift + (r as f32 * 0.08));
            frame.stroke(&builder.build(), Stroke::default().with_color(line_color).with_width(1.2 * row_t + 0.5));
        }
    }

    // Mode 6: NEW - Kaleidoscope / Sacred Geometry Mirror
    fn draw_kaleidoscope(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = cx.min(cy) * 0.85;
        let num_axes = 8;
        let tick_f = tick as f32;

        for axis in 0..num_axes {
            let axis_angle = (axis as f32 / num_axes as f32) * std::f32::consts::TAU + (tick_f * 0.005);

            for (i, &raw_amp) in bands.iter().step_by(3).enumerate() {
                let amp = (raw_amp * self.sensitivity).clamp(0.0, 1.0);
                if amp < 0.05 { continue; }

                let r = (i as f32 / (NUM_BANDS / 3) as f32) * max_r;
                let offset_angle = axis_angle + (amp * 0.3 * (if i % 2 == 0 { 1.0 } else { -1.0 }));

                let px = cx + r * offset_angle.cos();
                let py = cy + r * offset_angle.sin();

                let color = apply_ghost_style(theme::spectrum_bar_color(amp), alpha, shift + (axis as f32 * 0.12));
                let poly = Path::circle(Point::new(px, py), 2.0 + amp * 5.0);
                frame.fill(&poly, color);
            }
        }
    }

    // Mode 7: Cosmic Aurora Borealis & Draped Plasma Curtains
    fn draw_cosmic_aurora(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let tick_f = tick as f32;

        let num_curtains = 6;
        let num_points = 32;

        for c in 0..num_curtains {
            let curtain_t = (c as f32 + 1.0) / num_curtains as f32;
            let curtain_x_start = (c as f32 / num_curtains as f32) * width;
            let curtain_width = width * 0.35;

            // Plasma curtain color palette (Polar Emerald #00ff88 & Aurora Magenta #ff00c8)
            let base_aurora_color = if c % 2 == 0 {
                Color::from_rgb(0.0, 0.95, 0.55) // Emerald
            } else {
                Color::from_rgb(0.95, 0.10, 0.75) // Magenta
            };

            for i in 0..num_points {
                let pt_t = i as f32 / num_points as f32;
                let band_idx = (c * 20 + i * 2) % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

                let x = curtain_x_start + pt_t * curtain_width + ((tick_f * 0.02 + c as f32).sin() * 20.0);
                let top_y = height * 0.05 + ((pt_t * 5.0 + tick_f * 0.03).cos() * 15.0);
                let curtain_h = height * 0.55 + amp * 120.0;
                let bot_y = top_y + curtain_h;

                let curtain_alpha = (0.30 + amp * 0.55) * alpha * (1.0 - (c as f32 * 0.1));
                let line_color = apply_ghost_style(base_aurora_color, curtain_alpha, shift + c as f32 * 0.15);

                let ray = Path::line(Point::new(x, top_y), Point::new(x, bot_y));
                frame.stroke(
                    &ray,
                    Stroke::default()
                        .with_color(line_color)
                        .with_width(2.5 + amp * 3.0),
                );
            }
        }
    }

    // Mode 8: Retro Synthwave Horizon (Neon Fluorescence, Clipped Sun, Mountains, & Forward Grid)
    fn draw_synthwave_horizon(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, _shift: f32, tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let horizon_y = height * 0.55;
        let cx = width / 2.0;
        let tick_f = tick as f32;

        let bass = (bands[..10].iter().sum::<f32>() / 10.0 * self.sensitivity).clamp(0.0, 1.0);
        let mid = (bands[10..50].iter().sum::<f32>() / 40.0 * self.sensitivity).clamp(0.0, 1.0);

        // Neon Fluorescence Synthwave Color Palette
        let neon_magenta = Color::from_rgb(1.0, 0.0, 0.50); // #ff007f
        let neon_cyan = Color::from_rgb(0.0, 0.95, 1.0);     // #00f3ff
        let neon_purple = Color::from_rgb(0.60, 0.0, 1.0);   // #9d00ff
        let sunset_yellow = Color::from_rgb(1.0, 0.80, 0.0); // #ffcc00

        // 1. Synthwave Pulsing Sky Skygradient
        let sky_color = Color { a: 0.15 + mid * 0.10, ..neon_purple };
        let sky_path = Path::rectangle(Point::ORIGIN, Size::new(width, horizon_y));
        frame.fill(&sky_path, sky_color);

        // 2. Synthwave Sunset Sun (CLIPPED STRICTLY ABOVE HORIZON)
        let sun_r = (height * 0.22) + bass * 18.0;
        let sun_cy = horizon_y - 15.0;

        // Render sun circles with horizontal blind cuts
        for r_step in (0..=20).rev() {
            let step_r = sun_r * (r_step as f32 / 20.0);
            let y_pos = sun_cy - step_r;
            
            // Only draw portions strictly above horizon line
            if y_pos < horizon_y {
                let sun_path = Path::circle(Point::new(cx, sun_cy), step_r);
                let sun_blend = r_step as f32 / 20.0;
                let c_r = neon_magenta.r * (1.0 - sun_blend) + sunset_yellow.r * sun_blend;
                let c_g = neon_magenta.g * (1.0 - sun_blend) + sunset_yellow.g * sun_blend;
                let c_b = neon_magenta.b * (1.0 - sun_blend) + sunset_yellow.b * sun_blend;
                
                frame.fill(&sun_path, Color { r: c_r, g: c_g, b: c_b, a: alpha * 0.75 });
            }
        }

        // Horizontal Blind Cut lines across the sun
        let cut_count = 6;
        for c in 0..cut_count {
            let cut_y = sun_cy - sun_r * 0.6 + (c as f32 * 12.0);
            if cut_y < horizon_y && cut_y > (sun_cy - sun_r) {
                let cut_line = Path::line(Point::new(cx - sun_r, cut_y), Point::new(cx + sun_r, cut_y));
                frame.stroke(&cut_line, Stroke::default().with_color(Color::BLACK).with_width(2.5));
            }
        }

        // 3. Horizon Mountain Silhouettes (Pulsing with mid audio frequencies)
        let mut mountain_builder = iced::widget::canvas::path::Builder::new();
        mountain_builder.move_to(Point::new(0.0, horizon_y));

        let mtn_pts = 16;
        for m in 0..=mtn_pts {
            let mx = (m as f32 / mtn_pts as f32) * width;
            let m_amp = bands[(m * 8) % NUM_BANDS] * self.sensitivity;
            let mtn_h = 25.0 + (m % 3) as f32 * 15.0 + (m_amp * 30.0) + (mid * 15.0);
            mountain_builder.line_to(Point::new(mx, horizon_y - mtn_h));
        }
        mountain_builder.line_to(Point::new(width, horizon_y));
        mountain_builder.close();

        let mtn_color = Color::from_rgb(0.08, 0.04, 0.15);
        frame.fill(&mountain_builder.build(), mtn_color);

        // 4. Horizon Equalizer Spectrum Rays
        let eq_count = 32;
        let eq_step = width / eq_count as f32;
        for e in 0..eq_count {
            let ex = e as f32 * eq_step;
            let e_amp = (bands[(e * 4) % NUM_BANDS] * self.sensitivity).clamp(0.0, 1.0);
            let bar_h = e_amp * 45.0;

            let eq_line = Path::line(Point::new(ex, horizon_y), Point::new(ex, horizon_y - bar_h));
            frame.stroke(&eq_line, Stroke::default().with_color(neon_cyan).with_width(2.0));
        }

        // 5. Perspective Horizon Ground Floor Grid with Forward Scrolling Motion
        let num_lines = 16;
        for i in 0..num_lines {
            let t = i as f32 / (num_lines - 1) as f32;
            let start_x = cx + (t - 0.5) * width * 0.15;
            let end_x = cx + (t - 0.5) * width * 1.6;

            let line_path = Path::line(Point::new(start_x, horizon_y), Point::new(end_x, height));
            frame.stroke(&line_path, Stroke::default().with_color(neon_magenta).with_width(1.3));
        }

        // Horizontal forward moving grid lines
        let num_h_lines = 10;
        let scroll_phase = (tick_f * 0.03 * (1.0 + bass * 0.5)) % 1.0;

        for h in 0..num_h_lines {
            let row_t = ((h as f32 + scroll_phase) / num_h_lines as f32) % 1.0;
            let hy = horizon_y + (row_t * row_t) * (height - horizon_y);

            let row_line = Path::line(Point::new(0.0, hy), Point::new(width, hy));
            let h_alpha = row_t * alpha;
            frame.stroke(&row_line, Stroke::default().with_color(Color { a: h_alpha, ..neon_cyan }).with_width(1.0 + row_t * 1.5));
        }
    }
}

fn base_perspective(depth: f32) -> f32 {
    depth * depth
}

fn apply_ghost_style(c: Color, alpha: f32, shift: f32) -> Color {
    let mut color = c;
    if shift > 0.01 {
        let s = (shift * 0.25) % 1.0;
        let r = c.r * (1.0 - s) + c.g * s;
        let g = c.g * (1.0 - s) + c.b * s;
        let b = c.b * (1.0 - s) + c.r * s;
        color = Color { r, g, b, a: c.a };
    }
    Color {
        a: (color.a * alpha).clamp(0.0, 1.0),
        ..color
    }
}

pub fn view<'a>(
    bands: &'a [f32; NUM_BANDS],
    history: &'a std::collections::VecDeque<[f32; NUM_BANDS]>,
    mode: usize,
    tick: u32,
    sensitivity: f32,
    decay: f32,
    color_shift: f32,
    bar_count: usize,
    bg_mode: usize,
    bg_color: &'a str,
    aurora_preset: usize,
    depth_warp_speed: f32,
    kaleidoscope_axes: usize,
) -> Element<'a, Message> {
    Canvas::new(SpectrumView {
        bands,
        history,
        mode,
        tick,
        sensitivity,
        decay,
        color_shift,
        bar_count,
        bg_mode,
        bg_color,
        aurora_preset,
        depth_warp_speed,
        kaleidoscope_axes,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
