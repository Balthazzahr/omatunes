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

    // Mode 0: Live Mirrored Spectrograph Bars with Falling Peak Markers (No Ghosting)
    fn draw_mirrored_bars_with_peaks(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, _tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let bar_width = (width / NUM_BANDS as f32) - 1.0;
        let gap = 1.0;
        let cy = height / 2.0;

        for (i, &raw_amp) in bands.iter().enumerate() {
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

            // Peak reference marker calculation (falling caps under gravity simulation)
            let mut hist_max_amp = amp;
            for (hist_idx, hist_bands) in self.history.iter().enumerate().take(12) {
                let h_amp = (hist_bands[i] * self.sensitivity).clamp(0.0, 1.0);
                let decay_penalty = hist_idx as f32 * 0.04;
                let decayed = (h_amp - decay_penalty).max(0.0);
                if decayed > hist_max_amp {
                    hist_max_amp = decayed;
                }
            }

            let peak_half_h = (hist_max_amp * height * 0.48).max(half_h);
            let cap_y_top = cy - peak_half_h - 2.0;
            let cap_y_bot = cy + peak_half_h;

            let cap_color = apply_ghost_style(theme::accent(), alpha * 0.9, shift + 0.1);
            let cap_top = Path::rectangle(Point::new(x, cap_y_top), Size::new(bar_width.max(1.0), 2.5));
            let cap_bot = Path::rectangle(Point::new(x, cap_y_bot), Size::new(bar_width.max(1.0), 2.5));

            frame.fill(&cap_top, cap_color);
            frame.fill(&cap_bot, cap_color);

            // High peak bloom glow
            if amp > 0.75 {
                let glow_path = Path::rectangle(Point::new(x - 1.0, y - 2.0), Size::new(bar_width.max(1.0) + 2.0, bar_height + 4.0));
                let mut glow_color = color;
                glow_color.a = 0.25;
                frame.stroke(&glow_path, Stroke::default().with_color(glow_color).with_width(1.5));
            }
        }
    }

    // Mode 1: Radial Pulse with 4-Group Expanding Circles into Infinity & Music-Reactive Core
    fn draw_radial_pulse_expanding_waves(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.85).max(20.0);
        let base_r = max_r * 0.22;
        let tick_f = tick as f32;

        // 4 Frequency Band Energy Groups (Bass, Low-Mid, High-Mid, Treble)
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

        // Render expanding shockwave circles launched into infinity based on group spikes
        let hist_len = self.history.len();
        if hist_len > 0 {
            for (group_idx, &(energy, color, color_offset)) in group_energies.iter().enumerate() {
                if energy > 0.2 {
                    for (idx, hist_bands) in self.history.iter().enumerate() {
                        let age = hist_len - idx;
                        let range_energy = match group_idx {
                            0 => hist_bands[..10].iter().sum::<f32>() / 10.0,
                            1 => hist_bands[10..35].iter().sum::<f32>() / 25.0,
                            2 => hist_bands[35..75].iter().sum::<f32>() / 40.0,
                            _ => hist_bands[75..144].iter().sum::<f32>() / 69.0,
                        } * self.sensitivity;

                        if range_energy > 0.3 {
                            let ring_r = base_r + (age as f32 * 18.0 * (1.0 + group_idx as f32 * 0.25));
                            let opacity = (1.0 - (ring_r / (cx.max(cy) * 1.2))).clamp(0.0, 0.85) * alpha;
                            if opacity > 0.02 {
                                let ring_path = Path::circle(Point::new(cx, cy), ring_r);
                                let ring_color = apply_ghost_style(color, opacity, shift + color_offset);
                                frame.stroke(
                                    &ring_path,
                                    Stroke::default().with_color(ring_color).with_width(1.5 + range_energy * 2.0),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Dynamic Inner Core: Reactive Flower / Star Polygon
        let core_r = base_r + bass_energy * 14.0;
        let num_petals = 8;
        let mut core_builder = iced::widget::canvas::path::Builder::new();

        for p in 0..(num_petals * 2) {
            let angle = (p as f32 / (num_petals * 2) as f32) * std::f32::consts::TAU + (tick_f * 0.02);
            let petal_mod = if p % 2 == 0 { core_r * (1.0 + low_mid_energy * 0.35) } else { core_r * 0.65 };
            let px = cx + petal_mod * angle.cos();
            let py = cy + petal_mod * angle.sin();

            if p == 0 {
                core_builder.move_to(Point::new(px, py));
            } else {
                core_builder.line_to(Point::new(px, py));
            }
        }
        core_builder.close();
        let core_path = core_builder.build();

        let core_color = apply_ghost_style(theme::accent(), alpha * 0.6, shift);
        frame.fill(&core_path, Color { a: core_color.a * 0.35, ..core_color });
        frame.stroke(&core_path, Stroke::default().with_color(core_color).with_width(2.0));

        // Radial Spectrum Spokes
        let num = NUM_BANDS;
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

    // Mode 3: Extended Lissajous Organic Constellation (Longer, Dynamic Curve)
    fn draw_particle_constellation_extended(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32, age: usize, age_factor: f32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let tick_f = tick as f32;
        let bass = (bands[..8].iter().sum::<f32>() / 8.0 * self.sensitivity).clamp(0.0, 1.0);

        let num_particles = 64;
        let mut points: Vec<(Point, f32, Color)> = Vec::with_capacity(num_particles);

        for i in 0..num_particles {
            let band_idx = (i * 2) % NUM_BANDS;
            let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

            let t = (i as f32 / num_particles as f32) * std::f32::consts::TAU * 2.0 + (tick_f * 0.006) - (age_factor * 0.2);

            // Extended organic Lissajous curve parameters
            let scale_x = bounds.width * 0.38 + amp * 35.0;
            let scale_y = bounds.height * 0.35 + bass * 25.0;

            let px = cx + scale_x * (t * 0.5).sin() * (t * 0.3 + tick_f * 0.002).cos();
            let py = cy + scale_y * (t * 0.7).cos() + (amp * 20.0 * (t * 3.0).sin());

            let particle_shift = shift + (i as f32 * 0.04);
            let color = apply_ghost_style(theme::spectrum_bar_color(amp), alpha, particle_shift);
            points.push((Point::new(px, py), amp, color));
        }

        // Draw connecting constellation lines
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let (p1, a1, c1) = points[i];
                let (p2, a2, _) = points[j];

                let dx = p1.x - p2.x;
                let dy = p1.y - p2.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 4900.0 {
                    let line_alpha = (1.0 - (dist_sq.sqrt() / 70.0)) * ((a1 + a2) * 0.5) * 0.65 * alpha;
                    if line_alpha > 0.02 {
                        let line_path = Path::line(p1, p2);
                        frame.stroke(
                            &line_path,
                            Stroke::default()
                                .with_color(Color { a: line_alpha, ..c1 })
                                .with_width(if age > 0 { 0.9 } else { 1.2 }),
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

    // Mode 4: Hyperdrive Depth Tunnel Waterfall
    fn draw_depth_tunnel(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32, age: usize, age_factor: f32) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.85).max(20.0);
        let tick_f = tick as f32;

        let ring_count = 7;
        let num_vertices = 24;
        let tunnel_contraction = 1.0 - (age_factor * 0.60);

        let mut ring_pts: Vec<Vec<Point>> = Vec::with_capacity(ring_count);

        for k in 0..ring_count {
            let depth = (k as f32 + 1.0) / ring_count as f32;
            let scale_r = base_perspective(depth) * max_r * tunnel_contraction;
            let ring_alpha = (0.25 + (depth * 0.75)) * alpha;

            let mut builder = iced::widget::canvas::path::Builder::new();
            let mut current_ring_pts = Vec::with_capacity(num_vertices);

            for v in 0..num_vertices {
                let angle = (v as f32 / num_vertices as f32) * std::f32::consts::TAU + (tick_f * 0.01 * (k as f32 + 1.0)) - (age_factor * 0.1);
                let band_idx = (v * (NUM_BANDS / num_vertices)) % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

                let r_mod = scale_r + (amp * 22.0 * depth * tunnel_contraction);
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
            let avg_amp = bands[k * 15 % NUM_BANDS] * self.sensitivity;
            let base_color = theme::spectrum_bar_color(avg_amp);
            let color = apply_ghost_style(base_color, ring_alpha, shift + (k as f32 * 0.15));

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(color)
                    .with_width(if age > 0 { 1.2 } else { 1.8 + depth * 1.5 }),
            );

            ring_pts.push(current_ring_pts);
        }

        if ring_pts.len() > 1 {
            for k in 0..(ring_pts.len() - 1) {
                let outer_pts = &ring_pts[k + 1];
                let inner_pts = &ring_pts[k];

                let waterfall_shift = shift + (k as f32 * 0.2) + (age as f32 * 0.25);
                let waterfall_color = apply_ghost_style(theme::accent(), alpha * 0.45, waterfall_shift);

                for v in (0..num_vertices).step_by(2) {
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

    // Mode 7: NEW - Cosmic Aurora Borealis & Flowing Waves
    fn draw_cosmic_aurora(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let tick_f = tick as f32;

        let num_waves = 4;
        for w in 0..num_waves {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let base_y = height * (0.3 + w as f32 * 0.15);

            for i in 0..40 {
                let x = i as f32 * (width / 39.0);
                let band_idx = (i * 3 + w * 10) % NUM_BANDS;
                let amp = (bands[band_idx] * self.sensitivity).clamp(0.0, 1.0);

                let wave_y = base_y + ((x * 0.01 + tick_f * 0.03 + w as f32).sin() * 25.0) - (amp * 45.0);

                if i == 0 {
                    builder.move_to(Point::new(x, wave_y));
                } else {
                    builder.line_to(Point::new(x, wave_y));
                }
            }

            let color = apply_ghost_style(theme::spectrum_bar_color(0.7 - w as f32 * 0.15), alpha * 0.75, shift + w as f32 * 0.25);
            frame.stroke(&builder.build(), Stroke::default().with_color(color).with_width(3.0 - w as f32 * 0.5));
        }
    }

    // Mode 8: NEW - Retro Synthwave Horizon
    fn draw_synthwave_horizon(&self, frame: &mut Frame, bounds: Rectangle, bands: &[f32; NUM_BANDS], alpha: f32, shift: f32, _tick: u32) {
        let width = bounds.width;
        let height = bounds.height;
        let horizon_y = height * 0.55;
        let cx = width / 2.0;

        let bass = (bands[..8].iter().sum::<f32>() / 8.0 * self.sensitivity).clamp(0.0, 1.0);

        // Synthwave Sunset Sun
        let sun_r = (height * 0.22) + bass * 15.0;
        let sun_path = Path::circle(Point::new(cx, horizon_y - 10.0), sun_r);
        let sun_color = apply_ghost_style(theme::accent(), alpha * 0.85, shift);
        frame.fill(&sun_path, Color { a: sun_color.a * 0.45, ..sun_color });

        // Perspective Horizon Grid Lines
        let num_lines = 12;
        for i in 0..num_lines {
            let t = i as f32 / num_lines as f32;
            let start_x = cx + (t - 0.5) * width * 0.2;
            let end_x = cx + (t - 0.5) * width * 1.5;

            let line_path = Path::line(Point::new(start_x, horizon_y), Point::new(end_x, height));
            frame.stroke(&line_path, Stroke::default().with_color(sun_color).with_width(1.2));
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
) -> Element<'a, Message> {
    Canvas::new(SpectrumView {
        bands,
        history,
        mode,
        tick,
        sensitivity,
        decay,
        color_shift,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
