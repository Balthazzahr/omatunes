use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, LineCap};
use iced::{Color, Element, Length, Point, Rectangle, Size};

use crate::app::Message;
use crate::audio::spectrum::NUM_BANDS;
use crate::ui::theme;

pub struct SpectrumView {
    pub bands: [f32; NUM_BANDS],
    pub mode: usize,
    pub tick: u32,
}

impl<Message> canvas::Program<Message> for SpectrumView {
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

        match self.mode {
            0 => self.draw_mirrored_bars(&mut frame, bounds),
            1 => self.draw_radial_pulse(&mut frame, bounds),
            2 => self.draw_liquid_ribbon(&mut frame, bounds),
            3 => self.draw_particle_constellation(&mut frame, bounds),
            4 => self.draw_depth_tunnel(&mut frame, bounds),
            _ => self.draw_mirrored_bars(&mut frame, bounds),
        }

        vec![frame.into_geometry()]
    }
}

impl SpectrumView {
    // Mode 0: Mirrored Spectrograph Bars
    fn draw_mirrored_bars(&self, frame: &mut Frame, bounds: Rectangle) {
        let width = bounds.width;
        let height = bounds.height;
        let bar_width = (width / NUM_BANDS as f32) - 1.0;
        let gap = 1.0;
        let cy = height / 2.0;

        for (i, &amplitude) in self.bands.iter().enumerate() {
            let x = i as f32 * (bar_width + gap);
            let half_h = (amplitude * height * 0.48).max(1.0);
            let y = cy - half_h;
            let bar_height = half_h * 2.0;

            let color = theme::spectrum_bar_color(amplitude);

            let path = Path::rectangle(
                Point::new(x, y),
                Size::new(bar_width.max(1.0), bar_height),
            );
            frame.fill(&path, color);
        }
    }

    // Mode 1: Radial Pulse & Orbital Spectrum
    fn draw_radial_pulse(&self, frame: &mut Frame, bounds: Rectangle) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.85).max(20.0);
        let base_r = max_r * 0.25;

        // Calculate bass kick energy from lowest 10 bands
        let bass_energy = (self.bands[..10].iter().sum::<f32>() / 10.0).clamp(0.0, 1.0);
        let core_r = base_r + bass_energy * 18.0;

        // Draw inner glowing core
        let core_color = theme::accent();
        let core_path = Path::circle(Point::new(cx, cy), core_r);
        frame.fill(&core_path, Color { a: 0.25, ..core_color });
        frame.stroke(
            &core_path,
            Stroke::default().with_color(core_color).with_width(2.0),
        );

        // Draw 144 radial spokes
        let num = NUM_BANDS;
        for (i, &amp) in self.bands.iter().enumerate() {
            let angle = (i as f32 / num as f32) * std::f32::consts::TAU - (std::f32::consts::FRAC_PI_2);
            let spoke_len = amp * (max_r - core_r);

            let inner_x = cx + core_r * angle.cos();
            let inner_y = cy + core_r * angle.sin();

            let outer_x = cx + (core_r + spoke_len) * angle.cos();
            let outer_y = cy + (core_r + spoke_len) * angle.sin();

            let spoke_color = theme::spectrum_bar_color(amp);
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

    // Mode 2: Liquid Silk Waveform Ribbon
    fn draw_liquid_ribbon(&self, frame: &mut Frame, bounds: Rectangle) {
        let width = bounds.width;
        let height = bounds.height;
        let cy = height / 2.0;
        let tick = self.tick as f32;

        // We render 3 overlapping fluid wave layers
        let layers = [
            (0.60, 0.025, theme::accent(), 3.5),
            (0.40, 0.040, theme::spectrum_bar_color(0.85), 2.5),
            (0.25, 0.060, theme::spectrum_bar_color(0.50), 1.8),
        ];

        let num_points = 64;
        let step_x = width / (num_points - 1) as f32;

        for (l_idx, &(amp_scale, freq_scale, color, stroke_w)) in layers.iter().enumerate() {
            let mut builder = iced::widget::canvas::path::Builder::new();
            let phase = tick * 0.04 * (l_idx + 1) as f32;

            for i in 0..num_points {
                let x = i as f32 * step_x;
                let band_idx = ((i as f32 / num_points as f32) * NUM_BANDS as f32) as usize % NUM_BANDS;
                let amp = self.bands[band_idx];

                let sine = ((i as f32 * freq_scale * 10.0) + phase).sin();
                let wave_y = cy + (sine * amp * height * 0.4 * amp_scale);

                if i == 0 {
                    builder.move_to(Point::new(x, wave_y));
                } else {
                    builder.line_to(Point::new(x, wave_y));
                }
            }

            let path = builder.build();
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color { a: 0.85 - (l_idx as f32 * 0.2), ..color })
                    .with_width(stroke_w)
                    .with_line_cap(LineCap::Round),
            );
        }
    }

    // Mode 3: Particle Constellation Starburst
    fn draw_particle_constellation(&self, frame: &mut Frame, bounds: Rectangle) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let tick = self.tick as f32;
        let bass = (self.bands[..8].iter().sum::<f32>() / 8.0).clamp(0.0, 1.0);

        let num_particles = 48;
        let mut points: Vec<(Point, f32, Color)> = Vec::with_capacity(num_particles);

        for i in 0..num_particles {
            let band_idx = (i * 3) % NUM_BANDS;
            let amp = self.bands[band_idx];

            let base_angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU;
            let orbit_angle = base_angle + tick * 0.008;

            let dist = 25.0 + (i as f32 * 2.2) + amp * 55.0 + bass * 20.0;
            let px = cx + dist * orbit_angle.cos();
            let py = cy + dist * orbit_angle.sin();

            let color = theme::spectrum_bar_color(amp);
            points.push((Point::new(px, py), amp, color));
        }

        // Draw connecting constellation lines between nearby nodes
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let (p1, a1, c1) = points[i];
                let (p2, a2, _) = points[j];

                let dx = p1.x - p2.x;
                let dy = p1.y - p2.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 3600.0 { // 60px radius
                    let alpha = (1.0 - (dist_sq.sqrt() / 60.0)) * ((a1 + a2) * 0.5) * 0.6;
                    if alpha > 0.05 {
                        let line_path = Path::line(p1, p2);
                        frame.stroke(
                            &line_path,
                            Stroke::default()
                                .with_color(Color { a: alpha, ..c1 })
                                .with_width(1.2),
                        );
                    }
                }
            }
        }

        // Draw glowing particle nodes
        for (pt, amp, color) in points {
            let radius = 2.5 + amp * 4.5;
            let particle_path = Path::circle(pt, radius);
            frame.fill(&particle_path, color);
        }
    }

    // Mode 4: Hyperdrive Tunnel / Depth Rings
    fn draw_depth_tunnel(&self, frame: &mut Frame, bounds: Rectangle) {
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let max_r = (cx.min(cy) * 0.85).max(20.0);
        let tick = self.tick as f32;

        let ring_count = 7;
        let num_vertices = 24;

        for k in 0..ring_count {
            let depth = (k as f32 + 1.0) / ring_count as f32;
            let scale_r = base_perspective(depth) * max_r;
            let alpha = 0.2 + (depth * 0.75);

            let mut builder = iced::widget::canvas::path::Builder::new();

            for v in 0..num_vertices {
                let angle = (v as f32 / num_vertices as f32) * std::f32::consts::TAU + (tick * 0.01 * (k as f32 + 1.0));
                let band_idx = (v * (NUM_BANDS / num_vertices)) % NUM_BANDS;
                let amp = self.bands[band_idx];

                let r_mod = scale_r + (amp * 20.0 * depth);
                let vx = cx + r_mod * angle.cos();
                let vy = cy + r_mod * angle.sin();

                if v == 0 {
                    builder.move_to(Point::new(vx, vy));
                } else {
                    builder.line_to(Point::new(vx, vy));
                }
            }
            builder.close();

            let path = builder.build();
            let avg_amp = self.bands[k * 15 % NUM_BANDS];
            let color = theme::spectrum_bar_color(avg_amp);

            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color { a: alpha, ..color })
                    .with_width(1.5 + depth * 1.5),
            );
        }
    }
}

fn base_perspective(depth: f32) -> f32 {
    depth * depth
}

pub fn view(bands: [f32; NUM_BANDS], mode: usize, tick: u32) -> Element<'static, Message> {
    Canvas::new(SpectrumView { bands, mode, tick })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
