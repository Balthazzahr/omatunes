use iced::widget::{button, column, container, row, text, text_input, Space, slider, pick_list, scrollable};
use iced::{Alignment, Element, Length};

use crate::app::{Message, SettingsState};
use crate::ui::theme;

pub fn view<'a>(state: &'a SettingsState) -> Element<'a, Message> {
    let languages = vec![
        "auto".to_string(),
        "en".to_string(),
        "pt_BR".to_string(),
        "es".to_string(),
    ];

    // Music Library Path
    let music_input = text_input("e.g. ~/Music", &state.music_dir)
        .on_input(Message::SettingsMusicDirChanged)
        .padding(8);

    // Language Selection
    let lang_pick = pick_list(
        languages,
        Some(state.language.clone()),
        Message::SettingsLanguageChanged,
    )
    .padding(8)
    .width(Length::Fill);

    // Seek Step
    let seek_input = text_input("e.g. 5", &state.seek_step)
        .on_input(Message::SettingsSeekStepChanged)
        .padding(8);

    // Volume Step
    let vol_slider = slider(0.01..=0.20f32, state.volume_step, Message::SettingsVolumeStepChanged)
        .step(0.01);
    let vol_label = text(format!("{:.0}%", state.volume_step * 100.0)).size(12).color(theme::text());

    // Font Scale
    let scale_slider = slider(0.5..=2.5f32, state.font_scale, Message::SettingsFontScaleChanged)
        .step(0.05);
    let scale_label = text(format!("{:.2}x", state.font_scale)).size(12).color(theme::text());

    // Key Bindings
    let shortcuts_btn = button(
        row![
            text("\u{f11c}").font(crate::ui::icons::NERD_FONT_MONO).size(13),
            Space::with_width(8),
            text("Key Bindings").size(12).font(crate::ui::icons::UI_FONT_BOLD)
        ]
        .align_y(Alignment::Center)
    )
    .on_press(Message::OpenShortcuts)
    .padding([8, 12])
    .style(theme::secondary_button);

    // ── THEME SECTION ──────────────────────────────────────────────────────────
    let theme_sources = vec![
        "System".to_string(),
        "Preset".to_string(),
        "Custom".to_string(),
    ];

    let source_pick = pick_list(
        theme_sources,
        Some(state.theme_source.clone()),
        Message::SettingsThemeSourceChanged,
    )
    .padding(8)
    .width(Length::Fill);

    let mut theming_col = column![
        text("Theming").size(14).font(crate::ui::icons::UI_FONT_BOLD).color(theme::accent()),
        Space::with_height(6),
        text("Theme Source").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
        Space::with_height(4),
        source_pick,
    ]
    .spacing(4);

    // Preset Selection
    if state.theme_source == "Preset" {
        let presets = vec![
            "Nord".to_string(),
            "Catppuccin Mocha".to_string(),
            "Catppuccin Latte".to_string(),
            "Dracula".to_string(),
            "Gruvbox (Dark)".to_string(),
            "Everforest (Dark)".to_string(),
            "Monokai".to_string(),
        ];
        let preset_pick = pick_list(
            presets,
            Some(state.theme_preset.clone()),
            Message::SettingsThemePresetChanged,
        )
        .padding(8)
        .width(Length::Fill);

        theming_col = theming_col.push(
            column![
                Space::with_height(8),
                text("Preset Theme").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                preset_pick,
            ]
            .spacing(2)
        );
    }

    // Custom Theme Builder
    if state.theme_source == "Custom" {
        let mut custom_fields = column![
            Space::with_height(8),
            text("Custom Theme Palette").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
        ].spacing(6);

        let render_field = |label: &'static str, token: &'static str, hex_val: &'a str| -> Element<'a, Message> {
            let parsed_color = crate::ui::theme::hex_to_color(hex_val).unwrap_or(iced::Color::TRANSPARENT);
            let has_error = state.custom_validation_errors.contains_key(token);
            
            let swatch = container(Space::new(Length::Fixed(18.0), Length::Fixed(18.0)))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(parsed_color)),
                    border: iced::Border {
                        color: if has_error { theme::red() } else { theme::surface0() },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .width(18.0)
                .height(18.0);

            let input = text_input("#RRGGBB", hex_val)
                .on_input(move |v| Message::SettingsCustomColorChanged(token.to_string(), v))
                .padding(6)
                .size(12)
                .width(Length::Fill);

            let label_col = text(label)
                .size(12)
                .width(100)
                .color(if has_error { theme::red() } else { theme::text() });

            let field_col: Element<'_, Message> = column![
                row![
                    label_col,
                    input,
                    Space::with_width(8),
                    swatch,
                ]
                .align_y(Alignment::Center),
                if has_error {
                    let err_row: Element<'_, Message> = row![
                        Space::with_width(108),
                        text("Invalid hex (#RRGGBB)").size(10).color(theme::red())
                    ].into();
                    err_row
                } else {
                    let empty_space: Element<'_, Message> = Space::with_height(0.0).into();
                    empty_space
                }
            ]
            .spacing(1)
            .into();
            
            field_col
        };

        custom_fields = custom_fields
            .push(render_field("Base (Bg)", "base", &state.custom_theme.base))
            .push(render_field("Mantle", "mantle", &state.custom_theme.mantle))
            .push(render_field("Surface0", "surface0", &state.custom_theme.surface0))
            .push(render_field("Overlay0", "overlay0", &state.custom_theme.overlay0))
            .push(render_field("Text (Fg)", "text", &state.custom_theme.text))
            .push(render_field("Subtext", "subtext", &state.custom_theme.subtext))
            .push(render_field("Accent", "accent", &state.custom_theme.accent))
            .push(render_field("Green", "green", &state.custom_theme.green))
            .push(render_field("Red", "red", &state.custom_theme.red))
            .push(render_field("Yellow", "yellow", &state.custom_theme.yellow))
            .push(render_field("Blue", "blue", &state.custom_theme.blue));

        theming_col = theming_col.push(custom_fields);

        // Contrast Check
        let warnings = crate::ui::theme::check_custom_contrast_warnings(&state.custom_theme);
        if !warnings.is_empty() {
            let mut warning_list = column![
                text("Poor Contrast Detected:").size(11).font(crate::ui::icons::UI_FONT_BOLD).color(theme::yellow()),
                Space::with_height(2),
            ].spacing(2);

            for (name, cr, target) in warnings {
                warning_list = warning_list.push(
                    text(format!("• {} ratio is {:.2} (aim for ≥ {:.1})", name, cr, target))
                        .size(11)
                        .color(theme::subtext())
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

            theming_col = theming_col.push(Space::with_height(10)).push(warning_banner);
        }
    }

    // Wrap scrollable
    let scroll_entries = scrollable(
        column![
            // Music path
            column![
                text("Music Library Path").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                music_input,
            ].spacing(2),
            Space::with_height(12),
            
            // Language
            column![
                text("Interface Language").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                lang_pick,
            ].spacing(2),
            Space::with_height(12),

            // Seek step
            column![
                text("Seek Step (seconds)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                seek_input,
            ].spacing(2),
            Space::with_height(12),

            // Volume step
            column![
                text("Volume Step (Mouse Wheel)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                row![vol_slider, Space::with_width(12), vol_label].align_y(Alignment::Center),
            ].spacing(2),
            Space::with_height(12),

            // Font scale
            column![
                text("Font Scale (Zoom)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                row![scale_slider, Space::with_width(12), scale_label].align_y(Alignment::Center),
            ].spacing(2),
            Space::with_height(12),

            // Shortcuts
            column![
                text("Shortcuts").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
                Space::with_height(4),
                shortcuts_btn,
            ].spacing(2),
            Space::with_height(16),

            // Theming
            theming_col,
        ]
        .spacing(0)
        .padding([0, 16, 0, 0]) // 16px right padding to clear the scrollbar track
    )
    .height(Length::Fixed(400.0));

    // Save button customization based on contrast check state
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

    let dialog_content = column![
        text("App Settings")
            .size(20)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(theme::accent()),
        Space::with_height(16),
        scroll_entries,
        Space::with_height(20),
        buttons,
    ];

    container(
        container(dialog_content)
            .width(550) // Widen from 500 to 550 pixels to resolve scrollbar overlap
            .padding(24)
            .style(theme::card)
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
