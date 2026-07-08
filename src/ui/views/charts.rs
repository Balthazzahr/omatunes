use iced::widget::canvas::{self, Canvas, Frame, Path};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use crate::app::Message;
use crate::ui::theme;

// ── Pie Chart ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PieSlice {
    pub label: String,
    pub count: usize,
    pub percentage: f32,
    pub color: Color,
}

pub struct PieChartProgram {
    pub slices: Vec<PieSlice>,
}

impl<Message> canvas::Program<Message> for PieChartProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) * 0.45).max(10.0);

        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::TRANSPARENT,
        );

        let mut start_angle = 0.0f32;
        for slice in &self.slices {
            if slice.percentage <= 0.0 {
                continue;
            }
            let sweep = slice.percentage * 2.0 * std::f32::consts::PI;
            let end_angle = start_angle + sweep;

            let mut builder = canvas::path::Builder::new();
            builder.move_to(center);
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle: iced::Radians(start_angle),
                end_angle: iced::Radians(end_angle),
            });
            builder.line_to(center);
            
            let path = builder.build();
            frame.fill(&path, slice.color);

            start_angle = end_angle;
        }

        vec![frame.into_geometry()]
    }
}

// ── Chart Helper Views ────────────────────────────────────────────────────────

pub fn view_pie_chart(slices: Vec<PieSlice>) -> Element<'static, Message> {
    Canvas::new(PieChartProgram { slices })
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(120.0))
        .into()
}
