use iced::widget::{button, column, row, text};
use crate::app::Message;
use crate::ui::theme;

pub fn get_suggestions(query: &str, items: &[String]) -> Vec<String> {
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for item in items {
        let item_trimmed = item.trim();
        let item_lower = item_trimmed.to_lowercase();
        if item_lower.starts_with(&query_lower) && item_lower != query_lower {
            matches.push(item_trimmed.to_string());
        }
    }
    matches.sort();
    matches.dedup();
    matches.truncate(4);
    matches
}

pub fn render_suggestions(
    suggestions: &[String],
    on_select: impl Fn(String) -> Message,
) -> iced::Element<'static, Message> {
    let mut col = column![].spacing(4);
    for chunk in suggestions.chunks(2) {
        let mut row_el = row![].spacing(6);
        for suggestion in chunk {
            row_el = row_el.push(
                button(
                    text(suggestion.clone())
                        .size(10)
                        .color(theme::accent())
                )
                .on_press(on_select(suggestion.clone()))
                .style(theme::secondary_button)
                .padding([2, 6])
            );
        }
        col = col.push(row_el);
    }
    col.into()
}
