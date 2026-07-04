use iced::widget::{button, row, text};
use iced::{Alignment, Element};

use crate::app::Message;
use crate::audio::PlaybackState;
use crate::ui::{icons, theme};

pub fn playback_controls<'a>(
    state: &PlaybackState,
    shuffle: bool,
    repeat: bool,
    liked: Option<bool>,
    current_track: Option<&crate::library::models::Track>,
) -> Element<'a, Message> {
    let play_icon = match state {
        PlaybackState::Playing => icons::ICON_PAUSE,
        _ => icons::ICON_PLAY,
    };

    let icon_btn = |icon: &'static str, msg: Message| {
        button(
            text(icon)
                .font(icons::NERD_FONT_MONO)
                .color(theme::text())
                .size(36),
        )
        .on_press(msg)
        .style(iced::widget::button::text)
        .padding([8, 20])
    };

    let shuffle_color = if shuffle { theme::accent() } else { theme::overlay0() };
    let repeat_color  = if repeat { theme::accent() } else { theme::overlay0() };

    let mut row_children = vec![
        button(
            text(icons::ICON_SHUFFLE)
                .font(icons::NERD_FONT_MONO)
                .color(shuffle_color)
                .size(32),
        )
        .on_press(Message::ToggleShuffle)
        .style(iced::widget::button::text)
        .padding([8, 16])
        .into(),

        icon_btn(icons::ICON_PREV, Message::PreviousTrack).into(),
        icon_btn(play_icon, Message::PlayPause).into(),
        icon_btn(icons::ICON_NEXT, Message::NextTrack).into(),

        button(
            text(icons::ICON_REPEAT)
                .font(icons::NERD_FONT_MONO)
                .color(repeat_color)
                .size(32),
        )
        .on_press(Message::ToggleRepeat)
        .style(iced::widget::button::text)
        .padding([8, 16])
        .into(),
    ];

    if let (Some(is_liked), Some(track)) = (liked, current_track) {
        // Add playlist-plus button in Liked button's vacated position
        row_children.push(
            button(
                text(icons::ICON_PLAYLIST_PLUS)
                    .font(icons::NERD_FONT_MONO)
                    .color(theme::overlay0())
                    .size(32),
            )
            .on_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Track(track.clone()))))
            .style(iced::widget::button::text)
            .padding([8, 16])
            .into()
        );

        // Like button shifted to the right
        let like_color = if is_liked { theme::red() } else { theme::overlay0() };
        row_children.push(
            button(
                text(icons::ICON_HEART)
                    .font(icons::NERD_FONT_MONO)
                    .color(like_color)
                    .size(32),
            )
            .on_press(Message::ToggleLikeTrack(track.clone()))
            .style(iced::widget::button::text)
            .padding([8, 16])
            .into()
        );
    }

    row(row_children)
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}
