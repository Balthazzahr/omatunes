use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Border, Element, Length};

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

    let tooltip_style =
        |_theme: &iced::Theme| -> iced::widget::container::Style {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                border: Border {
                    color: theme::overlay0(),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        };

    let tip = |label: &'static str| {
        container(
            text(label)
                .size(12)
                .font(icons::UI_FONT)
                .color(theme::text()),
        )
        .padding([4, 8])
        .style(tooltip_style)
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
    let repeat_color = if repeat { theme::accent() } else { theme::overlay0() };

    let mut row_children = vec![
        // Previous
        tooltip(
            icon_btn(icons::ICON_PREV, Message::PreviousTrack),
            tip("Previous"),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
        // Play / Pause
        tooltip(
            icon_btn(play_icon, Message::PlayPause),
            tip(if *state == PlaybackState::Playing {
                "Pause"
            } else {
                "Play"
            }),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
        // Next
        tooltip(
            icon_btn(icons::ICON_NEXT, Message::NextTrack),
            tip("Next"),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
        // Shuffle
        tooltip(
            button(
                text(icons::ICON_SHUFFLE)
                    .font(icons::NERD_FONT_MONO)
                    .color(shuffle_color)
                    .size(32),
            )
            .on_press(Message::ToggleShuffle)
            .style(iced::widget::button::text)
            .padding([8, 16]),
            tip("Shuffle"),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
        // Repeat
        tooltip(
            button(
                text(icons::ICON_REPEAT)
                    .font(icons::NERD_FONT_MONO)
                    .color(repeat_color)
                    .size(32),
            )
            .on_press(Message::ToggleRepeat)
            .style(iced::widget::button::text)
            .padding([8, 16]),
            tip("Repeat"),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
    ];

    if let (Some(is_liked), Some(track)) = (liked, current_track) {
        // Add to Playlist
        row_children.push(
            tooltip(
                button(
                    text(icons::ICON_PLAYLIST_PLUS)
                        .font(icons::NERD_FONT_MONO)
                        .color(theme::overlay0())
                        .size(32),
                )
                .on_press(Message::OpenPlaylistDialog(
                    crate::app::PlaylistDialogMode::AddTrack(track.clone()),
                ))
                .style(iced::widget::button::text)
                .padding([8, 16]),
                tip("Add to Playlist"),
                iced::widget::tooltip::Position::Bottom,
            )
            .into(),
        );

        // Like / Unlike
        let like_color = if is_liked { theme::red() } else { theme::overlay0() };
        row_children.push(
            tooltip(
                button(
                    text(icons::ICON_HEART)
                        .font(icons::NERD_FONT_MONO)
                        .color(like_color)
                        .size(32),
                )
                .on_press(Message::ToggleLikeTrack(track.clone()))
                .style(iced::widget::button::text)
                .padding([8, 16]),
                tip(if is_liked { "Unlike" } else { "Like" }),
                iced::widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }

    row(row_children)
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}
