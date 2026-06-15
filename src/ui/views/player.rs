use iced::widget::{column, container, image, row, text, Space, button, slider, mouse_area};
use iced::{Alignment, Element, Length};

use crate::app::{AppState, Message};
use crate::audio::PlaybackState;
use crate::ui::components::progress;
use crate::ui::{icons, theme};

pub fn view(state: &AppState) -> Element<'_, Message> {
    // 1. Determine which track to display (active track or selected track as queue fallback)
    let is_playing_or_paused = !matches!(state.playback_state, PlaybackState::Stopped);
    let (display_track, is_queued) = if is_playing_or_paused {
        (state.current_track.as_ref(), false)
    } else {
        (state.selected_track.as_ref(), state.selected_track.is_some())
    };

    let track_info: Element<Message> = if let Some(track) = display_track {
        let title_style = if is_queued {
            theme::subtext()
        } else {
            theme::text()
        };

        let title_text = track.title.clone();

        let song_btn = button(
            text(title_text)
                .color(title_style)
                .size(24)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..crate::ui::icons::UI_FONT
                })
        )
        .on_press(Message::FocusSongName)
        .style(iced::widget::button::text)
        .padding(0);

        let artist_btn = button(
            text(&track.artist)
                .color(theme::subtext())
                .size(16)
        )
        .on_press(Message::FocusArtistName)
        .style(iced::widget::button::text)
        .padding(0);

        let album_label = track.album.clone();
        let album_btn = button(
            text(album_label)
                .color(theme::subtext())
                .size(16)
        )
        .on_press(Message::FocusAlbumName)
        .style(iced::widget::button::text)
        .padding(0);

        column![
            artist_btn,
            song_btn,
            album_btn,
        ]
        .spacing(4)
        .into()
    } else {
        column![
            text(state.strings.no_track).color(theme::overlay0()).size(16),
        ]
        .into()
    };

    // Album cover (Click returns to active source)
    let cover_art: Element<Message> = if let Some(handle) = state.get_display_cover() {
        image(handle)
            .width(216)
            .height(216)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        container(
            text(icons::ICON_MUSIC)
                .font(icons::NERD_FONT_MONO)
                .color(theme::overlay0())
                .size(58),
        )
        .width(216)
        .height(216)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::card)
        .into()
    };

    let cover = button(cover_art)
        .on_press(Message::ReturnToActiveSource)
        .style(iced::widget::button::text)
        .padding(0);

    // Right-aligned volume control
    let vol_slider = slider(0.0..=1.0f32, state.volume, Message::VolumeChanged)
        .step(0.01)
        .width(150);

    let volume_control = row![
        text(icons::ICON_VOL_UP)
            .font(icons::NERD_FONT_MONO)
            .color(theme::subtext())
            .size(24),
        Space::with_width(8),
        vol_slider,
    ]
    .align_y(Alignment::Center)
    .padding([0, 16]);

    let playback_ctrls = crate::ui::components::controls::playback_controls(
        &state.playback_state,
        state.shuffle,
        state.repeat,
        display_track.map(|t| t.liked),
        display_track,
    );

    let bottom_row = row![
        playback_ctrls,
        Space::with_width(Length::Fill),
        volume_control,
    ]
    .align_y(Alignment::Center);

    let player_row = row![
        cover,
        Space::with_width(16),
        column![
            track_info,
            Space::with_height(12),
            progress::progress_bar(state.position, state.duration),
            Space::with_height(8),
            bottom_row,
        ]
        .width(Length::Fill)
        .spacing(0),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding(16);

    let tab_btn = |tab: crate::app::RightPanelTab, label: &'static str| {
        let is_active = state.right_panel_tab == Some(tab);
        let btn_text = text(label)
            .size(11)
            .font(crate::ui::icons::UI_FONT_BOLD);
        
        button(container(btn_text).center_x(Length::Fill).center_y(Length::Fill))
            .on_press(Message::ToggleRightPanelTab(tab))
            .width(Length::Fill)
            .height(28.0)
            .style(move |theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(if is_active {
                        theme::mantle()
                    } else if is_hovered {
                        theme::surface0()
                    } else {
                        iced::Color::TRANSPARENT
                    })),
                    border: iced::Border {
                        color: if is_active { theme::accent() } else { iced::Color::TRANSPARENT },
                        width: if is_active { 1.0 } else { 0.0 },
                        radius: iced::border::Radius {
                            top_left: 4.0,
                            top_right: 4.0,
                            bottom_left: 4.0,
                            bottom_right: 4.0,
                        },
                    },
                    text_color: if is_active { theme::accent() } else { theme::subtext() },
                    ..Default::default()
                }
            })
            .padding(0)
    };

    let tab_strip = container(
        column![
            tab_btn(crate::app::RightPanelTab::Visualizer, "Visualizer"),
            tab_btn(crate::app::RightPanelTab::Lyrics, "Lyrics"),
        ]
        .spacing(8)
        .width(84.0)
    )
    .padding([0, 8])
    .height(Length::Fill)
    .align_y(iced::alignment::Vertical::Center);

    let left_side_width = if state.right_panel_tab.is_some() {
        Length::FillPortion(1)
    } else {
        Length::Fill
    };

    let player_container = container(player_row)
        .style(theme::player_panel)
        .width(left_side_width);

    let vol_step = crate::config::get().volume_step;

    let player_with_scroll = mouse_area(player_container)
        .on_scroll(move |delta| {
            match delta {
                iced::mouse::ScrollDelta::Lines { y, .. } | iced::mouse::ScrollDelta::Pixels { y, .. } => {
                    if y > 0.0 {
                        Message::VolumeStep(vol_step)
                    } else if y < 0.0 {
                        Message::VolumeStep(-vol_step)
                    } else {
                        Message::VolumeStep(0.0)
                    }
                }
            }
        });

    let content_pane = if let Some(tab) = state.right_panel_tab {
        let placeholder_text = match tab {
            crate::app::RightPanelTab::Visualizer => "visualizer to be added here soon",
            crate::app::RightPanelTab::Lyrics => "lyrics to be added here soon",
        };
        
        let content = container(
            text(placeholder_text)
                .color(theme::overlay0())
                .size(16)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        Some(
            container(content)
                .style(theme::player_panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
        )
    } else {
        None
    };

    let mut main_row = row![
        player_with_scroll,
        tab_strip,
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill);

    if let Some(pane) = content_pane {
        main_row = main_row.push(pane);
    }

    main_row.into()
}
