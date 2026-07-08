use iced::widget::{button, column, container, row, text, text_input, Space, checkbox, pick_list};
use iced::{Alignment, Element, Length};

use crate::app::{Message, PlaylistDialogState, PlaylistDialogMode};
use crate::ui::theme;

pub fn view(state: &PlaylistDialogState) -> Element<'static, Message> {
    let custom_playlists = crate::db::get(|db| db.playlists.keys().cloned().collect::<Vec<String>>());

    let title_text = match &state.mode {
        PlaylistDialogMode::Create => "Create New Playlist",
        PlaylistDialogMode::AddTrack(_) => "Add to Playlist",
        PlaylistDialogMode::CreateWithTrack(_) => "Create Playlist with Song",
        PlaylistDialogMode::Rename(_) => "Rename Playlist",
    };

    let mut content = column![
        text(title_text)
            .size(18)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(theme::accent()),
        Space::with_height(12),
    ];

    match &state.mode {
        PlaylistDialogMode::Create => {
            let name_input = text_input("Playlist Name", &state.name_input)
                .on_input(Message::PlaylistInputChanged)
                .padding(8);

            content = content.push(text("Name").size(12).color(theme::subtext()))
                .push(name_input)
                .push(Space::with_height(16));
        }
        PlaylistDialogMode::CreateWithTrack(track) => {
            let name_input = text_input("Playlist Name", &state.name_input)
                .on_input(Message::PlaylistInputChanged)
                .padding(8);

            content = content.push(text("Name").size(12).color(theme::subtext()))
                .push(name_input)
                .push(Space::with_height(8))
                .push(text(format!("Song: {}", track.title)).size(11).color(theme::overlay0()))
                .push(Space::with_height(16));
        }
        PlaylistDialogMode::AddTrack(track) => {
            if custom_playlists.is_empty() {
                content = content.push(text("No custom playlists found. Create one first!").size(14).color(theme::red()))
                    .push(Space::with_height(12));
            } else {
                let current_selection = state.selected_playlist.clone().unwrap_or_else(|| custom_playlists[0].clone());
                let select_dropdown = pick_list(
                    custom_playlists.clone(),
                    Some(current_selection),
                    Message::PlaylistDialogSelect,
                )
                .padding(8);

                content = content.push(text("Select Playlist").size(12).color(theme::subtext()))
                    .push(select_dropdown)
                    .push(Space::with_height(12));
            }

            content = content.push(Space::with_height(16));
        }
        PlaylistDialogMode::Rename(_) => {
            let name_input = text_input("New Playlist Name", &state.name_input)
                .on_input(Message::PlaylistInputChanged)
                .padding(8);

            content = content.push(text("New Name").size(12).color(theme::subtext()))
                .push(name_input)
                .push(Space::with_height(16));
        }
    }

    let submit_enabled = match &state.mode {
        PlaylistDialogMode::Create => !state.name_input.trim().is_empty(),
        PlaylistDialogMode::CreateWithTrack(_) => !state.name_input.trim().is_empty(),
        PlaylistDialogMode::AddTrack(_) => state.selected_playlist.is_some() && !custom_playlists.is_empty(),
        PlaylistDialogMode::Rename(_) => !state.name_input.trim().is_empty(),
    };

    let submit_label = match &state.mode {
        PlaylistDialogMode::CreateWithTrack(_) => "Create Playlist",
        _ => "Submit",
    };

    let submit_btn = if submit_enabled {
        button(text(submit_label).color(theme::base()))
            .on_press(Message::PlaylistDialogSubmit)
            .padding([8, 16])
            .style(theme::primary_button)
    } else {
        button(text(submit_label).color(theme::overlay0()))
            .padding([8, 16])
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                text_color: theme::overlay0(),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    };

    let create_with_song_btn = match &state.mode {
        PlaylistDialogMode::AddTrack(track) => {
            let btn: Element<'static, Message> = button(
                text("Create new playlist with song")
                    .size(12)
                    .color(theme::accent())
            )
            .on_press(Message::PlaylistCreateWithTrack(track.clone()))
            .padding([8, 12])
            .style(theme::secondary_button)
            .into();
            Some(btn)
        }
        _ => None,
    };

    let cancel_btn: Element<'static, Message> = button(text("Cancel").color(theme::text()))
        .on_press(Message::ClosePlaylistDialog)
        .padding([8, 16])
        .style(theme::secondary_button)
        .into();

    let mut buttons = row![cancel_btn].spacing(8).align_y(Alignment::Center);
    if let Some(btn) = create_with_song_btn {
        buttons = buttons.push(Space::with_width(Length::Fill));
        buttons = buttons.push(btn);
    }
    buttons = buttons.push(Space::with_width(12));
    buttons = buttons.push(submit_btn);

    let main_col = content.push(buttons_row)
        .spacing(4)
        .padding(24)
        .width(450);

    container(
        container(main_col)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::mantle())),
                border: iced::Border {
                    color: theme::surface0(),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::with_alpha(theme::base(), 0.8))),
        ..Default::default()
    })
    .into()
}
