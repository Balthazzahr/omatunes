use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Size};

use crate::app::Message;
use crate::audio::spectrum::NUM_BANDS;
use crate::ui::theme;

pub struct SpectrumView {
    pub bands: [f32; NUM_BANDS],
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

        let width = bounds.width;
        let height = bounds.height;
        let bar_width = (width / NUM_BANDS as f32) - 2.0;
        let gap = 2.0;
        let cy = height / 2.0;

        for (i, &amplitude) in self.bands.iter().enumerate() {
            let x = i as f32 * (bar_width + gap);
            // Centre-anchored: bars extend equally above and below midline
            let half_h = (amplitude * height * 0.45).max(1.0);
            let y = cy - half_h;
            let bar_height = half_h * 2.0;

            let color = theme::spectrum_bar_color(amplitude);

            let path = Path::rectangle(
                Point::new(x, y),
                Size::new(bar_width, bar_height),
            );
            frame.fill(&path, color);
        }

        vec![frame.into_geometry()]
    }
}

pub fn view(bands: [f32; NUM_BANDS]) -> Element<'static, Message> {
    Canvas::new(SpectrumView { bands })
        .width(Length::Fill)
        .height(60)
        .into()
}
