use iced::widget::{button, checkbox, column, container, pick_list, radio, row, scrollable, slider, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::app::{Message, SettingsState, SettingsTab};
use crate::ui::theme;
use crate::ui::icons::{ICON_LIBRARY, ICON_SLIDERS, ICON_MONITOR, ICON_PALETTE, ICON_KEYBOARD, ICON_FOLDER, ICON_AUTO_SCAN, ICON_VOLUME_HIGH, ICON_CHECK, ICON_TIMES, NERD_FONT_MONO, UI_FONT_BOLD, UI_FONT};

fn tab_button<'a>(
    icon: &'static str,
    label: &'static str,
    tab: SettingsTab,
    is_selected: bool,
) -> Element<'a, Message> {
    button(
        row![
            text(icon).font(NERD_FONT_MONO).size(16).color(
                if is_selected { theme::accent() } else { theme::overlay0() }
            ),
            Space::with_width(8),
            text(label).font(UI_FONT_BOLD).size(14).color(
                if is_selected { theme::text() } else { theme::subtext() }
            ),
        ]
        .align_y(Alignment::Center)
    )
    .on_press(Message::SettingsTabChanged(tab))
    .padding([10, 12])
    .width(Length::Fill)
    .style(move |_t: &iced::Theme, status: iced::widget::button::Status| {
        let hovered = status == iced::widget::button::Status::Hovered
            || status == iced::widget::button::Status::Pressed;
        iced::widget::button::Style {
            background: if is_selected || hovered {
                Some(iced::Background::Color(theme::surface0()))
            } else {
                None
            },
            text_color: if is_selected { theme::text() } else { theme::subtext() },
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

fn field_label(label: &'static str) -> Element<'static, Message> {
    text(label).size(12).font(UI_FONT_BOLD).color(theme::subtext()).into()
}

fn section_header(label: &'static str) -> Element<'static, Message> {
    text(label).size(15).font(UI_FONT_BOLD).color(theme::accent()).into()
}

pub fn view<'a>(state: &'a SettingsState) -> Element<'a, Message> {
    // ── Shared data ───────────────────────────────────────────────────────────
    let theme_sources = vec![
        "System".to_string(),
        "Preset".to_string(),
        "Custom".to_string(),
    ];

    let presets = vec![
        "Nord".to_string(),
        "Catppuccin Mocha".to_string(),
        "Catppuccin Latte".to_string(),
        "Dracula".to_string(),
        "Gruvbox (Dark)".to_string(),
        "Everforest (Dark)".to_string(),
        "Monokai".to_string(),
    ];

    let playback_contexts: Vec<(&str, &str, &str)> = vec![
        ("album", "Album", "\u{f001}"),
        ("artist", "Artist", "\u{f4ff}"),
        ("genre", "Genre", "\u{f02b}"),
        ("user_playlist", "User Playlist", "\u{f0cb8}"),
        ("smart_playlist", "Smart Playlist", "\u{f0d25}"),
    ];

    // ── Left nav ──────────────────────────────────────────────────────────────
    let nav = column![
        tab_button(ICON_LIBRARY, "Library", SettingsTab::Library, state.selected_tab == SettingsTab::Library),
        tab_button(ICON_SLIDERS, "Playback", SettingsTab::Playback, state.selected_tab == SettingsTab::Playback),
        tab_button(ICON_MONITOR, "Display", SettingsTab::Display, state.selected_tab == SettingsTab::Display),
        tab_button(ICON_PALETTE, "Theme", SettingsTab::Theme, state.selected_tab == SettingsTab::Theme),
        tab_button(crate::ui::icons::ICON_VISUALIZER, "Visualizer", SettingsTab::Visualizer, state.selected_tab == SettingsTab::Visualizer),
        tab_button(ICON_KEYBOARD, "Shortcuts", SettingsTab::Shortcuts, state.selected_tab == SettingsTab::Shortcuts),
    ]
    .spacing(4)
    .width(160)
    .padding(iced::Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 0.0 });

    // ── Right panel content ───────────────────────────────────────────────────
    let content: Element<'a, Message> = match state.selected_tab {
        SettingsTab::Library => {
            let music_input = text_input("e.g. ~/Music", &state.music_dir)
                .on_input(Message::SettingsMusicDirChanged)
                .padding(8);

            let browse_btn = button(text("Browse").color(theme::text()))
                .on_press(Message::PickMusicFolder)
                .padding([8, 14])
                .style(theme::secondary_button);

            let interval_input = text_input("e.g. 15", &state.auto_scan.interval_minutes.to_string())
                .on_input(Message::SettingsAutoScanIntervalChanged)
                .padding(8);

            let mut panel = column![
                section_header("Library"),
                Space::with_height(16),
                row![
                    text(ICON_FOLDER).font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                    Space::with_width(6),
                    field_label("Music Library Path"),
                ].align_y(Alignment::Center),
                Space::with_height(6),
                row![
                    music_input.width(Length::Fill),
                    Space::with_width(8),
                    browse_btn,
                ].align_y(Alignment::Center),
                Space::with_height(16),
                row![
                    text(ICON_AUTO_SCAN).font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                    Space::with_width(6),
                    field_label("Library Scan Mode"),
                ].align_y(Alignment::Center),
                Space::with_height(8),
                column![
                    radio("Manual (F5 to scan)", "manual", Some(state.auto_scan.mode.as_str()), |_| Message::SettingsAutoScanModeChanged("manual".to_string()))
                        .spacing(8),
                    radio("On Startup", "startup", Some(state.auto_scan.mode.as_str()), |_| Message::SettingsAutoScanModeChanged("startup".to_string()))
                        .spacing(8),
                    radio("Periodic", "periodic", Some(state.auto_scan.mode.as_str()), |_| Message::SettingsAutoScanModeChanged("periodic".to_string()))
                        .spacing(8),
                ].spacing(4),
            ].spacing(0);

            if state.auto_scan.mode == "periodic" {
                panel = panel.push(Space::with_height(12));
                panel = panel.push(field_label("Scan Interval (minutes)"));
                panel = panel.push(Space::with_height(6));
                panel = panel.push(interval_input);
            }

            scrollable(column![panel].spacing(0).padding(iced::Padding {
                top: 0.0,
                right: 8.0,
                bottom: 0.0,
                left: 0.0,
            }))
            .height(Length::Fill)
            .into()
        }

        SettingsTab::Playback => {
            let vol_slider = slider(0.0..=1.0f32, state.initial_volume, Message::SettingsInitialVolumeChanged)
                .step(0.01);
            let vol_label = text(format!("{:.0}%", state.initial_volume * 100.0))
                .size(12)
                .color(theme::text());

            let mut defaults_table = column![
                Space::with_height(8),
                row![
                    text("Context").font(UI_FONT_BOLD).size(12).color(theme::subtext()).width(Length::FillPortion(3)),
                    text("Shuffle").font(UI_FONT_BOLD).size(12).color(theme::subtext()).width(Length::FillPortion(1)),
                    text("Repeat").font(UI_FONT_BOLD).size(12).color(theme::subtext()).width(Length::FillPortion(1)),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
            ]
            .spacing(0);

            let toggle_btn = |enabled: bool, on_press: Message| -> Element<'a, Message> {
                button(
                    text(if enabled { ICON_CHECK } else { ICON_TIMES })
                        .font(NERD_FONT_MONO)
                        .size(14)
                        .color(if enabled { theme::accent() } else { theme::overlay0() })
                )
                .on_press(on_press)
                .padding([4, 8])
                .style(move |_t: &iced::Theme, status: iced::widget::button::Status| {
                    let hovered = status == iced::widget::button::Status::Hovered
                        || status == iced::widget::button::Status::Pressed;
                    iced::widget::button::Style {
                        background: if hovered {
                            Some(iced::Background::Color(theme::surface0()))
                        } else {
                            None
                        },
                        text_color: if enabled { theme::accent() } else { theme::overlay0() },
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .into()
            };

            for (key, label, icon) in &playback_contexts {
                let entry = match *key {
                    "album" => &state.playback_defaults.album,
                    "artist" => &state.playback_defaults.artist,
                    "genre" => &state.playback_defaults.genre,
                    "user_playlist" => &state.playback_defaults.user_playlist,
                    "smart_playlist" => &state.playback_defaults.smart_playlist,
                    _ => unreachable!(),
                };
                defaults_table = defaults_table.push(
                    row![
                        row![
                            text(*icon).font(NERD_FONT_MONO).size(12).color(theme::overlay0()),
                            Space::with_width(6),
                            text(*label).font(UI_FONT).size(13).color(theme::text()),
                        ]
                        .align_y(Alignment::Center)
                        .width(Length::FillPortion(3)),
                        container(toggle_btn(
                            entry.shuffle,
                            Message::SettingsPlaybackDefaultChanged(key.to_string(), "shuffle".to_string(), !entry.shuffle),
                        ))
                        .width(Length::FillPortion(1))
                        .center_x(Length::Fill),
                        container(toggle_btn(
                            entry.repeat,
                            Message::SettingsPlaybackDefaultChanged(key.to_string(), "repeat".to_string(), !entry.repeat),
                        ))
                        .width(Length::FillPortion(1))
                        .center_x(Length::Fill),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8),
                );
            }

            scrollable(
                column![
                    section_header("Playback"),
                    Space::with_height(16),
                    row![
                        text(ICON_VOLUME_HIGH).font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                        Space::with_width(6),
                        field_label("Initial Volume"),
                    ].align_y(Alignment::Center),
                    Space::with_height(6),
                    row![vol_slider, Space::with_width(12), vol_label].align_y(Alignment::Center),
                    Space::with_height(16),
                    section_header("Per-Context Defaults"),
                    Space::with_height(6),
                    text("When you start playback from each context, these default shuffle/repeat states apply.")
                        .size(11)
                        .color(theme::overlay0()),
                    defaults_table,
                ]
                .spacing(0)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
            )
            .height(Length::Fill)
            .into()
        }

        SettingsTab::Display => {
            let scale_slider = slider(0.5..=2.5f32, state.font_scale, Message::SettingsFontScaleChanged)
                .step(0.05);
            let scale_label = text(format!("{:.2}x", state.font_scale)).size(12).color(theme::text());

            let show_achievements_cb = checkbox(
                "Show achievements icons in the main UI",
                state.show_achievements_in_ui,
            )
            .on_toggle(Message::SettingsShowAchievementsInUiChanged)
            .size(16);

            let show_toasts_cb = checkbox(
                "Show toast notifications",
                state.show_toasts,
            )
            .on_toggle(Message::SettingsShowToastsChanged)
            .size(16);

            scrollable(
                column![
                    section_header("Display"),
                    Space::with_height(16),
                    row![
                        text("Aa").font(UI_FONT_BOLD).size(14).color(theme::overlay0()),
                        Space::with_width(6),
                        field_label("Font Scale (Zoom)"),
                    ].align_y(Alignment::Center),
                    Space::with_height(6),
                    row![scale_slider, Space::with_width(12), scale_label].align_y(Alignment::Center),
                    Space::with_height(20),
                    row![
                        text("\u{f053f}").font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                        Space::with_width(6),
                        field_label("Achievements UI Visibility"),
                    ].align_y(Alignment::Center),
                    Space::with_height(8),
                    show_achievements_cb,
                    Space::with_height(16),
                    row![
                        text("\u{f0f3}").font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                        Space::with_width(6),
                        field_label("Toast Notifications"),
                    ].align_y(Alignment::Center),
                    Space::with_height(8),
                    show_toasts_cb,
                ]
                .spacing(0)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
            )
            .height(Length::Fill)
            .into()
        }

        SettingsTab::Theme => {
            let source_pick = pick_list(
                theme_sources,
                Some(state.theme_source.clone()),
                Message::SettingsThemeSourceChanged,
            )
            .padding(8)
            .width(Length::Fill);

            let mut theming_col = column![
                section_header("Theme"),
                Space::with_height(16),
                row![
                    text(ICON_PALETTE).font(NERD_FONT_MONO).size(14).color(theme::overlay0()),
                    Space::with_width(6),
                    field_label("Theme Source"),
                ].align_y(Alignment::Center),
                Space::with_height(6),
                source_pick,
            ]
            .spacing(0);

            if state.theme_source == "Preset" {
                let preset_pick = pick_list(
                    presets,
                    Some(state.theme_preset.clone()),
                    Message::SettingsThemePresetChanged,
                )
                .padding(8)
                .width(Length::Fill);

                theming_col = theming_col.push(Space::with_height(12));
                theming_col = theming_col.push(field_label("Preset Theme"));
                theming_col = theming_col.push(Space::with_height(6));
                theming_col = theming_col.push(preset_pick);
            }

            if state.theme_source == "Custom" {
                let render_field = |label: &'static str, token: &'static str, hex_val: &'a str| -> Element<'a, Message> {
                    let parsed_color = crate::ui::theme::hex_to_color(hex_val).unwrap_or(iced::Color::TRANSPARENT);
                    let has_error = state.custom_validation_errors.contains_key(token);
                    let is_expanded = state.color_picker_token.as_deref() == Some(token);
                    let picker_r = state.color_picker_r;
                    let picker_g = state.color_picker_g;
                    let picker_b = state.color_picker_b;

                    let swatch = container(Space::new(Length::Fixed(18.0), Length::Fixed(18.0)))
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(parsed_color)),
                            border: iced::Border {
                                color: if has_error { theme::red() } else if is_expanded { theme::accent() } else { theme::surface0() },
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        })
                        .width(18.0)
                        .height(18.0);

                    let swatch_btn = button(swatch)
                        .on_press(Message::SettingsColorPickerToggle(token.to_string()))
                        .padding(0)
                        .style(|_, _| iced::widget::button::Style {
                            background: None,
                            border: iced::Border { width: 0.0, ..Default::default() },
                            shadow: iced::Shadow::default(),
                            text_color: iced::Color::TRANSPARENT,
                        });

                    let input = text_input("#RRGGBB", hex_val)
                        .on_input(move |v| Message::SettingsCustomColorChanged(token.to_string(), v))
                        .padding(6)
                        .size(12)
                        .width(Length::Fill);

                    let label_col = text(label)
                        .size(12)
                        .width(140)
                        .color(if has_error { theme::red() } else { theme::text() });

                    let picker: Element<'a, Message> = if is_expanded {
                        column![
                            Space::with_height(4),
                            row![
                                text("R").size(11).width(14).color(theme::subtext()),
                                slider(0.0..=255.0, picker_r, move |v| Message::SettingsColorPickerRChanged(v))
                                    .width(Length::Fill),
                                text(format!("{}", picker_r.round() as u8)).size(11).width(24).color(theme::text()),
                            ].spacing(6).align_y(Alignment::Center),
                            row![
                                text("G").size(11).width(14).color(theme::subtext()),
                                slider(0.0..=255.0, picker_g, move |v| Message::SettingsColorPickerGChanged(v))
                                    .width(Length::Fill),
                                text(format!("{}", picker_g.round() as u8)).size(11).width(24).color(theme::text()),
                            ].spacing(6).align_y(Alignment::Center),
                            row![
                                text("B").size(11).width(14).color(theme::subtext()),
                                slider(0.0..=255.0, picker_b, move |v| Message::SettingsColorPickerBChanged(v))
                                    .width(Length::Fill),
                                text(format!("{}", picker_b.round() as u8)).size(11).width(24).color(theme::text()),
                            ].spacing(6).align_y(Alignment::Center),
                        ].spacing(2).padding(iced::Padding { top: 2.0, right: 0.0, bottom: 0.0, left: 0.0 }).into()
                    } else {
                        Space::with_height(0).into()
                    };

                    column![
                        row![
                            label_col,
                            input,
                            Space::with_width(8),
                            swatch_btn,
                        ]
                        .align_y(Alignment::Center),
                        if has_error {
                            let err_row: Element<'a, Message> = row![
                                Space::with_width(148),
                                text("Invalid hex (#RRGGBB)").size(10).color(theme::red()),
                            ].into();
                            err_row
                        } else {
                            Space::with_height(0).into()
                        },
                        picker,
                    ]
                    .spacing(1)
                    .into()
                };

                let render_derived_swatch = |label: &'static str, hex_val: &'a str| -> Element<'a, Message> {
                    let parsed_color = crate::ui::theme::hex_to_color(hex_val).unwrap_or(iced::Color::TRANSPARENT);
                    let swatch = container(Space::new(Length::Fixed(18.0), Length::Fixed(18.0)))
                        .style(move |_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(parsed_color)),
                            border: iced::Border {
                                color: theme::surface0(),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        })
                        .width(18.0)
                        .height(18.0);

                    let value_display = text(format!("{} (derived)", hex_val))
                        .size(11)
                        .font(UI_FONT_BOLD)
                        .color(theme::overlay0());

                    let label_col = text(label)
                        .size(12)
                        .width(140)
                        .color(theme::subtext());

                    row![
                        label_col,
                        value_display,
                        Space::with_width(Length::Fill),
                        swatch,
                    ]
                    .align_y(Alignment::Center)
                    .into()
                };

                theming_col = theming_col.push(Space::with_height(12));
                theming_col = theming_col.push(field_label("Custom Theme Palette"));
                theming_col = theming_col.push(Space::with_height(8));
                theming_col = theming_col.push(render_field("Background", "base", &state.custom_theme.base));
                theming_col = theming_col.push(render_derived_swatch("Background (Deep)", &state.custom_theme.mantle));
                theming_col = theming_col.push(render_derived_swatch("Panel Background", &state.custom_theme.surface0));
                theming_col = theming_col.push(render_field("Primary Text", "text", &state.custom_theme.text));
                theming_col = theming_col.push(render_derived_swatch("Secondary Text", &state.custom_theme.subtext));
                theming_col = theming_col.push(render_derived_swatch("Muted / Icons", &state.custom_theme.overlay0));
                theming_col = theming_col.push(render_field("Accent", "accent", &state.custom_theme.accent));
                theming_col = theming_col.push(render_field("Green", "green", &state.custom_theme.green));
                theming_col = theming_col.push(render_field("Red", "red", &state.custom_theme.red));
                theming_col = theming_col.push(render_field("Yellow", "yellow", &state.custom_theme.yellow));
                theming_col = theming_col.push(render_field("Blue", "blue", &state.custom_theme.blue));

                let warnings = crate::ui::theme::check_custom_contrast_warnings(&state.custom_theme);
                if !warnings.is_empty() {
                    let mut warning_list = column![
                        text("Poor Contrast Detected:")
                            .size(11)
                            .font(UI_FONT_BOLD)
                            .color(theme::yellow()),
                        Space::with_height(2),
                    ]
                    .spacing(2);

                    for (name, cr, target) in warnings {
                        warning_list = warning_list.push(
                            text(format!("• {} ratio is {:.2} (aim for ≥ {:.1})", name, cr, target))
                                .size(11)
                                .color(theme::subtext()),
                        );
                    }

                    let warning_banner = container(warning_list)
                        .width(Length::Fill)
                        .padding(8)
                        .style(|_| iced::widget::container::Style {
                            background: Some(iced::Background::Color(theme::with_alpha(theme::yellow(), 0.08))),
                            border: iced::Border {
                                color: theme::yellow(),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        });

                    theming_col = theming_col.push(Space::with_height(10));
                    theming_col = theming_col.push(warning_banner);
                }
            }

            scrollable(
                column![theming_col]
                    .spacing(0)
                    .padding(iced::Padding {
                        top: 0.0,
                        right: 8.0,
                        bottom: 0.0,
                        left: 0.0,
                    }),
            )
            .height(Length::Fill)
            .into()
        }

        SettingsTab::Visualizer => {
            let sensitivity_val = state.visualizer_sensitivity;
            let sensitivity_label = format!("{:.1}x ({})", sensitivity_val, match sensitivity_val {
                v if v < 0.6 => "Subtle",
                v if v < 1.4 => "Dynamic",
                _ => "High Energy",
            });
            let sensitivity_slider = slider(0.2_f32..=2.5_f32, sensitivity_val, Message::SettingsVisualizerSensitivityChanged).step(0.1_f32);

            let shift_val = state.color_shift_speed;
            let shift_label = format!("{:.1}x ({})", shift_val, match shift_val {
                v if v < 0.1 => "Static Theme Colors",
                v if v < 1.2 => "Smooth Spectrum Shift",
                _ => "Fast Spectrum Shift",
            });
            let shift_slider = slider(0.0_f32..=2.0_f32, shift_val, Message::SettingsColorShiftSpeedChanged).step(0.1_f32);

            let mode_names = vec![
                "0: Mirrored Spectrograph",
                "1: Radial Pulse",
                "2: Liquid Ribbon",
                "3: Particle Constellation",
                "4: Depth Tunnel",
                "5: 3D Wireframe Grid",
                "6: Kaleidoscope Mirror",
                "7: Cosmic Aurora",
                "8: Retro Synthwave Horizon",
            ];

            let selected_mode_name = mode_names.get(state.selected_visualizer_settings_mode.min(8)).cloned();

            let mode_picker = pick_list(
                mode_names,
                selected_mode_name,
                |chosen| {
                    let idx = match chosen {
                        "0: Mirrored Spectrograph" => 0,
                        "1: Radial Pulse" => 1,
                        "2: Liquid Ribbon" => 2,
                        "3: Particle Constellation" => 3,
                        "4: Depth Tunnel" => 4,
                        "5: 3D Wireframe Grid" => 5,
                        "6: Kaleidoscope Mirror" => 6,
                        "7: Cosmic Aurora" => 7,
                        "8: Retro Synthwave Horizon" => 8,
                        _ => 0,
                    };
                    Message::SettingsVisualizerSettingsModeSelected(idx)
                }
            );

            // Contextual mode-specific settings
            let mode_settings_panel: Element<'static, Message> = match state.selected_visualizer_settings_mode {
                0 => {
                    let bar_cnt = state.spectrograph_bar_count;
                    let bar_slider = slider(10.0_f32..=144.0_f32, bar_cnt as f32, |val| Message::SettingsSpectrographBarCountChanged(val as usize)).step(2.0_f32);
                    column![
                        field_label("Spectrograph Bar Count (10 - 144)"),
                        row![text(format!("{} spectrum bars", bar_cnt)).size(13).color(theme::text())],
                        Space::with_height(4),
                        bar_slider,
                    ].spacing(4).into()
                }
                1 => {
                    column![
                        field_label("Radial Pulse Behavior"),
                        text("Clean energy shockwave burst launching into infinity. (Center core removed)").size(13).color(theme::subtext()),
                    ].spacing(4).into()
                }
                2 => {
                    let trail_val = state.ghost_trail_length;
                    let trail_slider = slider(2.0_f32..=16.0_f32, trail_val as f32, |val| Message::SettingsGhostTrailLengthChanged(val as usize)).step(1.0_f32);
                    column![
                        field_label("Liquid Ribbon Trail Length"),
                        row![text(format!("{} ghost frames", trail_val)).size(13).color(theme::text())],
                        Space::with_height(4),
                        trail_slider,
                    ].spacing(4).into()
                }
                3 => {
                    column![
                        field_label("Particle Constellation Web"),
                        text("Multi-loop Lissajous path with extended node connection reach.").size(13).color(theme::subtext()),
                    ].spacing(4).into()
                }
                4 => {
                    column![
                        field_label("Hyperdrive Depth Tunnel Flight"),
                        text("Continuous forward warp tunnel flight with audio-reactive rings.").size(13).color(theme::subtext()),
                    ].spacing(4).into()
                }
                8 => {
                    column![
                        field_label("Retro Synthwave Horizon Aesthetics"),
                        text("Neon fluorescence palette, silhouette mountains, horizon equalizer, & forward grid.").size(13).color(theme::subtext()),
                    ].spacing(4).into()
                }
                _ => {
                    column![
                        field_label("General Mode Settings"),
                        text("Uses global reaction sensitivity & spectrum color shift.").size(13).color(theme::subtext()),
                    ].spacing(4).into()
                }
            };

            let panel = column![
                section_header("Visualizer Global & Mode Settings"),
                Space::with_height(16),

                field_label("Reaction Intensity / Sensitivity"),
                row![text(sensitivity_label).size(13).color(theme::text())],
                Space::with_height(4),
                sensitivity_slider,

                Space::with_height(16),
                field_label("Spectrum Color Shift Speed"),
                row![text(shift_label).size(13).color(theme::text())],
                Space::with_height(4),
                shift_slider,

                Space::with_height(20),
                section_header("Customize Mode Parameters"),
                Space::with_height(8),
                field_label("Select Visualizer Mode to Customize"),
                mode_picker,
                Space::with_height(12),
                mode_settings_panel,
            ]
            .spacing(4)
            .width(Length::Fill);

            scrollable(panel).height(Length::Fill).into()
        }

        SettingsTab::Shortcuts => {
            let row_item = |keys: &'static str, desc: &'static str| {
                row![
                    text(keys)
                        .width(Length::Fixed(130.0))
                        .font(UI_FONT_BOLD)
                        .color(theme::accent())
                        .size(13),
                    text(desc).color(theme::text()).size(13),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            };

            scrollable(
                column![
                    section_header("Keyboard Shortcuts"),
                    Space::with_height(16),
                    row_item("Space", "Play / Pause / Play Selected Track"),
                    row_item("N", "Next Track"),
                    row_item("P", "Previous Track"),
                    row_item("L / F", "Like / Unlike Song"),
                    row_item("E", "Edit Metadata Tags"),
                    row_item("C", "Create Custom Playlist"),
                    row_item("A", "Add Current Song to Playlist"),
                    row_item("Arrow Up/Down", "Navigate Lists (Sidebar/Tracks)"),
                    row_item("F5", "Rescan Music Library Folder"),
                    row_item("+ / -", "Increase / Decrease Volume"),
                    row_item("] / [", "Increase / Decrease Scaling"),
                    row_item("Right/Left", "Seek Forward / Backward"),
                    row_item("Tab", "Focus next field / cycle ID3 inputs"),
                    row_item("Shift + Tab", "Cycle ID3 input backwards"),
                    row_item("/", "Focus song search input"),
                ]
                .spacing(8)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
            )
            .height(Length::Fill)
            .into()
        }
    };

    // ── Divider between nav and content ──────────────────────────────────────
    let divider = container(Space::with_width(0))
        .width(1)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        });

    // ── Save / Cancel bar ─────────────────────────────────────────────────────
    let mut has_contrast_warning = false;
    if state.theme_source == "Custom" {
        has_contrast_warning = !crate::ui::theme::check_custom_contrast_warnings(&state.custom_theme).is_empty();
    }

    let save_label = if has_contrast_warning {
        if state.confirm_save_anyway {
            "Confirm Save Anyway"
        } else {
            "Save Settings (Low Contrast)"
        }
    } else {
        "Save Settings"
    };

    let save_btn = button(text(save_label).color(theme::base()))
        .on_press(Message::SettingsSave)
        .padding([10, 20])
        .style(move |t, s| {
            if has_contrast_warning {
                let mut style = theme::primary_button(t, s);
                style.background = Some(iced::Background::Color(theme::yellow()));
                style.text_color = theme::base();
                style
            } else {
                theme::primary_button(t, s)
            }
        });

    let cancel_btn = button(text("Cancel").color(theme::text()))
        .on_press(Message::CloseSettings)
        .padding([10, 20])
        .style(theme::secondary_button);

    let buttons = row![
        cancel_btn,
        Space::with_width(Length::Fill),
        save_btn,
    ]
    .align_y(Alignment::Center);

    // ── Assemble dialog ───────────────────────────────────────────────────────
    let body = container(
        row![
            nav,
            divider,
            Space::with_width(12),
            content,
        ]
        .spacing(0)
        .height(Length::Fill),
    )
    .height(Length::Fill);

    let close_btn = button(
        text(ICON_TIMES)
            .font(NERD_FONT_MONO)
            .size(16)
    )
    .on_press(Message::CloseSettings)
    .padding(6)
    .style(|_theme: &iced::Theme, status: iced::widget::button::Status| {
        let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
        iced::widget::button::Style {
            background: if is_hovered { Some(iced::Background::Color(theme::surface0())) } else { None },
            text_color: if is_hovered { theme::accent() } else { theme::text() },
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let dialog_content = column![
        row![
            text("App Settings")
                .size(20)
                .font(UI_FONT_BOLD)
                .color(theme::accent()),
            Space::with_width(Length::Fill),
            close_btn,
        ]
        .align_y(Alignment::Center),
        Space::with_height(16),
        body,
        Space::with_height(20),
        buttons,
    ]
    .height(Length::Fill);

    container(
        container(dialog_content)
            .width(800)
            .height(Length::Fill)
            .max_height(450)
            .padding(24)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::mantle())),
                border: iced::Border {
                    color: theme::surface0(),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
        ..Default::default()
    })
    .into()
}

