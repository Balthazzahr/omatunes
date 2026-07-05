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

// ── Bar Chart ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct BarItem {
    pub label: String,
    pub value: usize,
    pub color: Color,
}

pub struct BarChartProgram {
    pub bars: Vec<BarItem>,
}

impl<Message> canvas::Program<Message> for BarChartProgram {
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

        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::TRANSPARENT,
        );

        if self.bars.is_empty() {
            return vec![frame.into_geometry()];
        }

        let pad_left = 10.0f32;
        let pad_bottom = 5.0f32;
        let chart_w = bounds.width - pad_left * 2.0;
        let chart_h = bounds.height - pad_bottom * 2.0;

        let max_val = self.bars.iter().map(|b| b.value).max().unwrap_or(1) as f32;
        let num_bars = self.bars.len();

        let bar_w = (chart_w / num_bars as f32) * 0.7;
        let gap_w = (chart_w / num_bars as f32) * 0.3;

        for (i, bar) in self.bars.iter().enumerate() {
            let x = pad_left + i as f32 * (bar_w + gap_w) + gap_w / 2.0;
            let h = if max_val > 0.0 { (bar.value as f32 / max_val) * chart_h } else { 0.0 };
            let y = chart_h - h + pad_bottom;

            let rect_path = Path::rectangle(Point::new(x, y), Size::new(bar_w, h));
            frame.fill(&rect_path, bar.color);
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

pub fn view_bar_chart(bars: Vec<BarItem>) -> Element<'static, Message> {
    Canvas::new(BarChartProgram { bars })
        .width(Length::Fill)
        .height(Length::Fixed(100.0))
        .into()
}
