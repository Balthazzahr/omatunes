use iced::widget::text_editor;

#[derive(Debug, Clone)]
enum Message {
    LyricsAction(text_editor::Action),
}

struct AppState {
    lyrics_content: text_editor::Content,
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::LyricsAction(action) => {
            state.lyrics_content.perform(action);
        }
    }
}

fn view(state: &AppState) -> iced::Element<'_, Message> {
    text_editor(&state.lyrics_content)
        .on_action(Message::LyricsAction)
        .into()
}

fn main() {}
