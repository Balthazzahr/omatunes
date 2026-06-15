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

    let tab_btn = |tab: crate::app::RightPanelTab, icon_str: &'static str| {
        let is_active = state.right_panel_tab == Some(tab);
        let btn_icon = text(icon_str)
            .size(28)
            .font(crate::ui::icons::NERD_FONT_MONO);
        
        button(container(btn_icon).center_x(Length::Fill).center_y(Length::Fill))
            .on_press(Message::ToggleRightPanelTab(tab))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(if is_active {
                        theme::surface0()
                    } else if is_hovered {
                        theme::surface0()
                    } else {
                        iced::Color::TRANSPARENT
                    })),
                    text_color: if is_active {
                        theme::accent()
                    } else if is_hovered {
                        theme::text()
                    } else {
                        theme::subtext()
                    },
                    border: iced::Border {
                        radius: 0.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding(0)
    };

    let horizontal_sep = container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(1.0);

    let tab_strip = container(
        column![
            tab_btn(crate::app::RightPanelTab::Visualizer, crate::ui::icons::ICON_VISUALIZER),
            horizontal_sep,
            tab_btn(crate::app::RightPanelTab::Lyrics, crate::ui::icons::ICON_LYRICS),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(56.0)
    .height(Length::Fixed(248.0))
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::mantle())),
        ..Default::default()
    });

    let left_side_width = if state.right_panel_tab.is_some() {
        Length::FillPortion(1)
    } else {
        Length::Fill
    };

    let player_container = container(player_row)
        .style(theme::player_panel)
        .width(left_side_width)
        .height(Length::Fixed(248.0));

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
                .height(Length::Fixed(248.0))
        )
    } else {
        None
    };

    let separator = container(Space::new(Length::Fixed(1.0), Length::Fill))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        })
        .width(1.0)
        .height(Length::Fixed(248.0));

    let mut main_row = row![
        player_with_scroll,
        separator,
        tab_strip,
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fixed(248.0));

    if let Some(pane) = content_pane {
        // Add another vertical separator before the sliding pane
        let pane_separator = container(Space::new(Length::Fixed(1.0), Length::Fill))
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                ..Default::default()
            })
            .width(1.0)
            .height(Length::Fixed(248.0));
        main_row = main_row.push(pane_separator).push(pane);
    }

    main_row.into()
}
