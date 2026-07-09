use iced::widget::{column, container, image, row, text, Space, button, slider, mouse_area, stack, scrollable, tooltip};
use iced::{Alignment, Element, Length};

use crate::app::{AppState, Message, MIN_VOLUME_SLIDER_WIDTH, MAX_VOLUME_SLIDER_WIDTH, PLAYER_FIXED_WIDTH};
use crate::audio::PlaybackState;
use crate::ui::components::progress;
use crate::ui::{icons, theme};

/// Half-second offset so lyrics don't appear ahead of the audio
pub const LYRICS_OFFSET: std::time::Duration = std::time::Duration::from_millis(500);

pub fn view(state: &AppState) -> Element<'_, Message> {
    let is_allowed = state.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 450.0);
    let tab_strip_height = state.player_height - 28.0;
    let btn_slot_height = if is_allowed { tab_strip_height / 3.0 } else { 0.0 };
    // button = icon(28) + padding([4,8]) vertical = 28 + 8 = 36px
    let btn_content_height = 28.0 + 8.0; // icon_size + vertical_padding
    let centering_space = if is_allowed { (btn_slot_height - btn_content_height) / 2.0 } else { 0.0 };

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

    // Cover art scales with player height resize split
    let cover_size = (238.0 + (state.player_height - 298.0)).max(238.0);

    // Album cover (Click returns to active source)
    let cover_art: Element<Message> = if let Some(handle) = state.get_display_cover() {
        image(handle)
            .width(cover_size as u16)
            .height(cover_size as u16)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        let note_bytes = include_bytes!("../../../assets/OmaTUNES NOTE.png");
        let handle = iced::widget::image::Handle::from_bytes(note_bytes.to_vec());
        container(
            image(handle)
                .width(cover_size as u16)
                .height(cover_size as u16)
                .content_fit(iced::ContentFit::Cover)
        )
        .width(cover_size)
        .height(cover_size)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::card)
        .into()
    };

    let cover = button(cover_art)
        .on_press(Message::ReturnToActiveSource)
        .style(iced::widget::button::text)
        .padding(0);

    let is_allowed = state.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 450.0);
    let player_width = if state.right_panel_tab.is_some() && is_allowed {
        state.window_width - 62.0 - state.right_panel_width
    } else if is_allowed {
        state.window_width - 56.0
    } else {
        state.window_width
    };
    let vol_slider_width = (player_width - PLAYER_FIXED_WIDTH).clamp(MIN_VOLUME_SLIDER_WIDTH, MAX_VOLUME_SLIDER_WIDTH);

    // Right-aligned volume control
    let vol_slider = slider(0.0..=1.0f32, state.volume, Message::VolumeChanged)
        .step(0.01)
        .width(vol_slider_width);

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

    let left_side_width = if state.right_panel_tab.is_some() {
        Length::Fill
    } else {
        Length::Fill
    };

    let player_container = container(player_row)
        .style(theme::player_panel)
        .width(left_side_width)
        .height(Length::Fixed(state.player_height - 28.0));

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

    player_with_scroll.into()
}

pub fn tab_strip(state: &AppState) -> Element<'_, Message> {
    let tab_btn = |tab: crate::app::RightPanelTab, icon_str: &'static str, tooltip_text: &'static str| {
        let is_active = state.right_panel_tab == Some(tab);
        let btn_icon = text(icon_str)
            .size(28)
            .font(crate::ui::icons::NERD_FONT_MONO);

        let btn = button(container(btn_icon).center_x(Length::Fill).center_y(Length::Fill))
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
                    ..Default::default()
                }
            })
            .padding(0);

        let tooltip_content = container(
            text(tooltip_text)
                .size(13)
                .font(crate::ui::icons::UI_FONT)
                .color(theme::text())
        )
        .padding(8)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            border: iced::Border {
                color: theme::overlay0(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        container(tooltip(btn, tooltip_content, iced::widget::tooltip::Position::Left))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: None,
                ..Default::default()
            })
    };

    let left_sep = container(Space::new(Length::Fixed(1.0), Length::Fill))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        })
        .width(1.0)
        .height(Length::Fill);

    column![
        row![
            left_sep,
            container(
                column![
                    tab_btn(crate::app::RightPanelTab::Visualizer, crate::ui::icons::ICON_VISUALIZER, "Visualizer"),
                    tab_btn(crate::app::RightPanelTab::Lyrics, crate::ui::icons::ICON_LYRICS, "Lyrics"),
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(0)
            )
            .width(55.0)
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: None,
                ..Default::default()
            })
        ]
        .width(56.0)
        .height(Length::Fill),
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                ..Default::default()
            })
            .height(1.0)
    ]
    .width(56.0)
    .height(Length::Fill)
    .into()
}

fn render_stat_row(label: String, value: String) -> Element<'static, Message> {
    row![
        text(label).font(crate::ui::icons::UI_FONT).color(theme::subtext()).width(Length::Fixed(180.0)),
        text(value).font(crate::ui::icons::UI_FONT_BOLD).color(theme::text()).align_x(iced::alignment::Horizontal::Right).width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .padding([4, 0])
    .into()
}

fn render_leaderboard_minutes(title: String, entries: Vec<(String, f64)>) -> Element<'static, Message> {
    let mut col = column![
        text(title).font(crate::ui::icons::UI_FONT_BOLD).color(theme::accent()).size(14),
        Space::with_height(6),
    ].spacing(2);
    
    if entries.is_empty() {
        col = col.push(text("No stats yet").font(crate::ui::icons::UI_FONT).color(theme::overlay0()));
    } else {
        for (idx, (name, mins)) in entries.into_iter().enumerate() {
            let rank = idx + 1;
            let medal_color = match rank {
                1 => theme::yellow(),
                2 => theme::text(),
                3 => theme::red(),
                _ => theme::subtext(),
            };
            
            let row_item = row![
                text(format!("{rank}.")).font(crate::ui::icons::UI_FONT_BOLD).color(medal_color).width(Length::Fixed(24.0)),
                text(name).font(crate::ui::icons::UI_FONT).color(theme::text()).width(Length::Fixed(200.0)),
                text(format!("{:.1}m", mins)).font(crate::ui::icons::UI_FONT_BOLD).color(theme::text()).align_x(iced::alignment::Horizontal::Right).width(Length::Fill),
            ]
            .align_y(Alignment::Center);
            col = col.push(row_item);
        }
    }
    col.into()
}

fn render_leaderboard_counts(title: String, entries: Vec<(String, u32)>) -> Element<'static, Message> {
    let mut col = column![
        text(title).font(crate::ui::icons::UI_FONT_BOLD).color(theme::accent()).size(14),
        Space::with_height(6),
    ].spacing(2);
    
    if entries.is_empty() {
        col = col.push(text("No stats yet").font(crate::ui::icons::UI_FONT).color(theme::overlay0()));
    } else {
        for (idx, (name, count)) in entries.into_iter().enumerate() {
            let rank = idx + 1;
            let medal_color = match rank {
                1 => theme::yellow(),
                2 => theme::text(),
                3 => theme::red(),
                _ => theme::subtext(),
            };
            
            let row_item = row![
                text(format!("{rank}.")).font(crate::ui::icons::UI_FONT_BOLD).color(medal_color).width(Length::Fixed(24.0)),
                text(name).font(crate::ui::icons::UI_FONT).color(theme::text()).width(Length::Fixed(200.0)),
                text(format!("{count} plays")).font(crate::ui::icons::UI_FONT_BOLD).color(theme::text()).align_x(iced::alignment::Horizontal::Right).width(Length::Fill),
            ]
            .align_y(Alignment::Center);
            col = col.push(row_item);
        }
    }
    col.into()
}

pub fn right_panel(state: &AppState) -> Option<Element<'_, Message>> {
    let is_allowed = state.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 450.0);
    if !is_allowed {
        return None;
    }
    let tab = state.right_panel_tab?;
    let pane_content: Element<'_, Message> = match tab {
        crate::app::RightPanelTab::Visualizer => {
            container(
                crate::ui::views::spectrum::view(state.spectrum_bands)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        }
        crate::app::RightPanelTab::Statistics => {
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        crate::app::RightPanelTab::Lyrics => {
            let display_track = if !matches!(state.playback_state, crate::audio::PlaybackState::Stopped) {
                state.current_track.as_ref()
            } else {
                state.selected_track.as_ref()
            };

            if let Some(track) = display_track {
                if track.lyrics.trim().is_empty() {
                    container(
                        text("No lyrics available.")
                            .color(theme::overlay0())
                            .size(14)
                            .align_y(iced::alignment::Vertical::Center)
                            .align_x(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else {
                    let lrc_lines = parse_lrc(&track.lyrics);
                    if !lrc_lines.is_empty() {
                        // Apply half-second delay: use position minus offset
                        let adjusted_pos = state.position.saturating_sub(LYRICS_OFFSET);

                        let active_idx = lrc_lines.iter().position(|l| l.time > adjusted_pos)
                            .map(|idx| if idx > 0 { idx - 1 } else { 0 })
                            .unwrap_or_else(|| lrc_lines.len() - 1);

                         // Show ALL lines in a scrollable container; highlight the active one
                         let mut lines_col = column![].spacing(6).align_x(Alignment::Center).width(Length::Fill);
                         lines_col = lines_col.push(iced::widget::Space::with_height(108.0));

                         let available_width = (state.right_panel_width - 40.0).max(100.0);

                         for i in 0..lrc_lines.len() {
                             let line = &lrc_lines[i];
                             let is_active = i == active_idx;
                             let is_interim = (active_idx > 0 && i == active_idx - 1) || (i == active_idx + 1);
                             let line_time = line.time;

                             let font_size = if is_active { 20 } else { 17 };
                             let char_width = 0.60 * font_size as f32;
                             let max_chars = ((available_width / char_width).floor() as usize).max(10);
                             let sub_lines = wrap_text(&line.text, max_chars);

                             let mut text_col = column![].spacing(2).align_x(Alignment::Center).width(Length::Fill);
                             for sub_line in sub_lines {
                                 let txt = text(sub_line)
                                     .size(font_size)
                                     .font(if is_active { crate::ui::icons::UI_FONT_BOLD } else { crate::ui::icons::UI_FONT })
                                     .width(Length::Fill)
                                     .align_x(iced::alignment::Horizontal::Center);
                                 text_col = text_col.push(txt);
                             }

                             let container_element = container(text_col)
                                 .width(Length::Fill)
                                 .align_x(iced::alignment::Horizontal::Center);

                             // Each line is clickable to seek to that timestamp
                             let line_btn = button(container_element)
                                 .on_press(Message::SeekToLyric(line_time))
                                 .width(Length::Fill)
                                 .padding([4, 8])
                                 .style(move |_theme: &iced::Theme, status: iced::widget::button::Status| {
                                     let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                                     iced::widget::button::Style {
                                         background: if is_hovered {
                                             Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.1)))
                                         } else {
                                             None
                                         },
                                         text_color: if is_active {
                                             theme::accent()
                                         } else if is_hovered {
                                             theme::text()
                                         } else if is_interim {
                                             theme::lerp_color(theme::accent(), theme::overlay0(), 0.5)
                                         } else {
                                             theme::overlay0()
                                         },
                                         border: iced::Border {
                                             radius: 4.0.into(),
                                             ..Default::default()
                                         },
                                         ..Default::default()
                                     }
                                 });

                             lines_col = lines_col.push(line_btn);
                         }
                         lines_col = lines_col.push(iced::widget::Space::with_height(108.0));

                        scrollable(
                            container(lines_col)
                                .width(Length::Fill)
                                .padding([16, 12])
                                .center_x(Length::Fill)
                        )
                        .id(state.lyrics_scroll_id.clone())
                        .height(Length::Fill)
                        .into()
                    } else {
                        // Unsynchronized lyrics: plain scrollable text
                        scrollable(
                            container(
                                text(track.lyrics.clone())
                                    .color(theme::text())
                                    .size(17)
                            )
                            .width(Length::Fill)
                            .padding(12)
                            .center_x(Length::Fill)
                        )
                        .height(Length::Fill)
                        .into()
                    }
                }
            } else {
                container(
                    text("No track selected")
                        .color(theme::overlay0())
                        .size(16)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            }
        }
    };

    let close_btn = button(
        text("\u{f00d}")
            .font(crate::ui::icons::NERD_FONT_MONO)
            .size(14)
    )
    .on_press(Message::ToggleRightPanelTab(tab))
    .padding(6)
    .style(move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
        iced::widget::button::Style {
            background: if is_hovered { Some(iced::Background::Color(theme::surface0())) } else { None },
            text_color: if is_hovered { theme::accent() } else { theme::subtext() },
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let close_container = container(close_btn)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Top)
        .padding([8, 8]);

    let pane_stack = stack![
        pane_content,
        close_container,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let pane = container(pane_stack)
        .style(theme::player_panel)
        .width(Length::Fixed(state.right_panel_width))
        .height(Length::Fixed(state.player_height));

    // Add a draggable resize handle between player and panel
    let panel_drag_handle = mouse_area(
        container(
            container(Space::new(Length::Fixed(1.0), Length::Fill))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        if state.dragging_right_panel || state.is_hovering_right_panel_resizer {
                            theme::accent()
                        } else {
                            iced::Color::TRANSPARENT
                        }
                    )),
                    ..Default::default()
                })
        )
        .width(4.0)
        .height(Length::Fill)
        .center_x(Length::Fixed(4.0))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        })
    )
    .on_press(Message::RightPanelDragStart)
    .on_enter(Message::HoverRightPanelResizer(true))
    .on_exit(Message::HoverRightPanelResizer(false))
    .interaction(iced::mouse::Interaction::ResizingHorizontally);

    Some(
        row![
            panel_drag_handle,
            pane
        ]
        .height(Length::Fixed(state.player_height))
        .into()
    )
}

pub struct LrcLine {
    pub time: std::time::Duration,
    pub text: String,
}

pub fn parse_lrc(lyrics: &str) -> Vec<LrcLine> {
    let mut lines = Vec::new();
    for line in lyrics.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if let Some(end_bracket) = line.find(']') {
                let time_str = &line[1..end_bracket];
                let text_str = &line[end_bracket + 1..];
                if let Some((min_str, sec_str)) = time_str.split_once(':') {
                    if let Ok(min) = min_str.parse::<u64>() {
                        if let Ok(sec) = sec_str.parse::<f32>() {
                            let total_secs = min * 60 + sec.floor() as u64;
                            let ms = ((sec - sec.floor()) * 1000.0) as u32;
                            let time = std::time::Duration::new(total_secs, ms * 1_000_000);
                            lines.push(LrcLine {
                                time,
                                text: text_str.trim().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    lines.sort_by_key(|l| l.time);
    lines
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut sub_lines = Vec::new();
    let text_trimmed = text.trim();
    if text_trimmed.is_empty() {
        return vec![String::new()];
    }

    for paragraph in text_trimmed.lines() {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                sub_lines.push(current_line);
                current_line = String::from(word);
            }
        }
        if !current_line.is_empty() {
            sub_lines.push(current_line);
        }
    }

    if sub_lines.is_empty() {
        sub_lines.push(String::new());
    }
    sub_lines
}

pub fn period_breakdown_view(breakdown: &crate::stats::PeriodBreakdown, active_period: usize) -> Element<'_, Message> {
    let format_hours = |mins: f64| -> String {
        let total_secs = (mins * 60.0) as u64;
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        if h > 0 {
            format!("{h}h {m}m")
        } else {
            format!("{m}m")
        }
    };

    let format_header_time = |mins: f64| -> String {
        let total_secs = (mins * 60.0) as u64;
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        if h > 0 {
            let hour_label = if h == 1 { "hour" } else { "hours" };
            format!("{h} {hour_label} {m} Mins")
        } else {
            format!("{m} Mins")
        }
    };

    let period_tabs_data = [
        (crate::ui::icons::ICON_CALENDAR_DAY, "Day"),
        (crate::ui::icons::ICON_CALENDAR_WEEK, "Week"),
        (crate::ui::icons::ICON_CALENDAR_MONTH_FA, "Month"),
        (crate::ui::icons::ICON_TROPHY_FA, "All-Time"),
    ];

    let mut period_tabs = row![].spacing(8).align_y(Alignment::Center);
    for (i, (icon, label)) in period_tabs_data.iter().enumerate() {
        let is_active = i == active_period;
        let tab_content = row![
            text(*icon)
                .font(crate::ui::icons::NERD_FONT_MONO)
                .size(20)
                .color(if is_active { theme::accent() } else { theme::subtext() }),
            Space::with_width(6),
            text(*label)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(18)
                .color(if is_active { theme::accent() } else { theme::subtext() }),
        ]
        .spacing(0)
        .align_y(Alignment::Center);

        let tab_btn = button(tab_content)
            .on_press(Message::ShowPeriodBreakdown(i))
            .padding([8, 16])
            .style(move |_theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    background: if is_active || is_hovered {
                        Some(iced::Background::Color(theme::surface0()))
                    } else {
                        None
                    },
                    text_color: if is_active {
                        theme::accent()
                    } else if is_hovered {
                        theme::text()
                    } else {
                        theme::subtext()
                    },
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });
        period_tabs = period_tabs.push(tab_btn);
    }

    let close_btn = button(
        text("\u{f00d}")
            .font(crate::ui::icons::NERD_FONT_MONO)
            .size(18)
    )
    .on_press(Message::ClosePeriodBreakdown)
    .padding(6)
    .style(move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
        iced::widget::button::Style {
            background: if is_hovered { Some(iced::Background::Color(theme::surface0())) } else { None },
            text_color: if is_hovered { theme::accent() } else { theme::subtext() },
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let text_size: u16 = 17;
    let small_size: u16 = 16;

    fn build_col<'a>(
        title: &'a str,
        icon_char: char,
        items: &'a [(String, f64, u32)],
        format_hours: &impl Fn(f64) -> String,
        text_size: u16,
        small_size: u16,
        make_on_press: impl Fn(String) -> Message + 'a,
    ) -> Element<'a, Message> {
        use iced::Color;

        let mut col = column![
            text(title)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(text_size)
                .color(theme::subtext()),
            Space::with_height(6),
        ]
        .spacing(6)
        .width(Length::FillPortion(1));

        if items.is_empty() {
            col = col.push(
                text("No data yet")
                    .font(crate::ui::icons::UI_FONT)
                    .size(small_size)
                    .color(theme::overlay0())
            );
        } else {
            for (i, (name, mins, count)) in items.iter().enumerate() {
                let rank = i + 1;
                let rank_color = if rank == 1 {
                    Color::from_rgb(0.98, 0.80, 0.28)
                } else if rank == 2 {
                    Color::from_rgb(0.70, 0.70, 0.70)
                } else if rank == 3 {
                    Color::from_rgb(0.80, 0.52, 0.25)
                } else {
                    theme::subtext()
                };
                let name_color: iced::Color = if rank <= 3 { rank_color } else { theme::text() };

                let name_btn = button(
                    row![
                        text(format!("{:>2}", rank))
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .size(text_size)
                            .color(rank_color),
                        Space::with_width(6),
                        text(icon_char)
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .size(text_size)
                            .color(name_color),
                        Space::with_width(4),
                        text(name.as_str())
                            .font(crate::ui::icons::UI_FONT)
                            .size(text_size)
                            .color(name_color)
                            .width(Length::Fill),
                    ]
                    .spacing(0)
                    .align_y(Alignment::Start)
                    .width(Length::Fill),
                )
                .on_press(make_on_press(name.clone()))
                .padding(0)
                .style(|_theme: &iced::Theme, status: iced::widget::button::Status| {
                    let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                    iced::widget::button::Style {
                        background: None,
                        text_color: if is_hovered { theme::accent() } else { theme::text() },
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                });

                let row_item = row![
                    name_btn.width(Length::Fill),
                    text(format!("({} Songs)", count))
                        .font(crate::ui::icons::UI_FONT)
                        .size(small_size)
                        .color(theme::subtext()),
                    Space::with_width(8),
                    text(format_hours(*mins))
                        .font(crate::ui::icons::UI_FONT_BOLD)
                        .size(text_size)
                        .color(theme::subtext())
                        .align_x(iced::alignment::Horizontal::Right),
                ]
                .spacing(4)
                .align_y(Alignment::Start);
                col = col.push(row_item);
            }
        }
        col.into()
    }

    let sep = || -> Element<'_, Message> {
        container(Space::with_width(0))
            .width(1)
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                ..Default::default()
            })
            .into()
    };

    let tables = row![
        Space::with_width(12),
        build_col("Artist", '\u{f4ff}', &breakdown.artist_minutes, &format_hours, text_size, small_size, |name| Message::SelectArtistFromBreakdown(name)),
        Space::with_width(12),
        sep(),
        Space::with_width(12),
        build_col("Album", '\u{e271}', &breakdown.album_minutes, &format_hours, text_size, small_size, |name| Message::SelectAlbumFromBreakdown(name)),
        Space::with_width(12),
        sep(),
        Space::with_width(12),
        build_col("Genre", '\u{f02b}', &breakdown.genre_minutes, &format_hours, text_size, small_size, |name| Message::SelectGenreFromBreakdown(name)),
        Space::with_width(12),
    ]
    .spacing(0)
    .width(Length::Fill);

    let content = column![
        row![
            Space::with_width(Length::Fill),
            text("LEADERBOARDS")
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(22)
                .color(theme::accent()),
            Space::with_width(Length::Fill),
            close_btn,
        ]
        .align_y(Alignment::Center),
        Space::with_height(12),
        row![
            Space::with_width(Length::Fill),
            period_tabs,
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::with_height(16),
        row![
            Space::with_width(Length::Fill),
            text(&breakdown.period_label)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(15)
                .color(theme::accent()),
            text(" - ")
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(15)
                .color(theme::subtext()),
            text(format!("{} Tracks | {} played", breakdown.total_plays, format_header_time(breakdown.total_minutes)))
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(15)
                .color(theme::subtext()),
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::with_height(16),
        tables,
    ];

    container(
        container(content)
            .padding(28)
            .max_width(1500)
            .max_height(575)
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
