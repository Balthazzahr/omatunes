use iced::widget::{button, column, container, row, text, text_input, Space, slider, pick_list};
use iced::{Alignment, Element, Length};

use crate::app::{Message, SettingsState};
use crate::ui::theme;

pub fn view(state: &SettingsState) -> Element<'static, Message> {
    let languages = vec![
        "auto".to_string(),
        "en".to_string(),
        "pt_BR".to_string(),
        "es".to_string(),
    ];

    let mut content = column![
        text("App Settings")
            .size(20)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(theme::accent()),
        Space::with_height(16),
    ];

    // Music Library Path
    let music_input = text_input("e.g. ~/Music", &state.music_dir)
        .on_input(Message::SettingsMusicDirChanged)
        .padding(8);

    content = content.push(
        column![
            text("Music Library Path").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            music_input,
        ]
        .spacing(2)
    ).push(Space::with_height(12));

    // Language Selection
    let lang_pick = pick_list(
        languages,
        Some(state.language.clone()),
        Message::SettingsLanguageChanged,
    )
    .padding(8)
    .width(Length::Fill);

    content = content.push(
        column![
            text("Interface Language").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            lang_pick,
        ]
        .spacing(2)
    ).push(Space::with_height(12));

    // Seek Step
    let seek_input = text_input("e.g. 5", &state.seek_step)
        .on_input(Message::SettingsSeekStepChanged)
        .padding(8);

    content = content.push(
        column![
            text("Seek Step (seconds)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            seek_input,
        ]
        .spacing(2)
    ).push(Space::with_height(12));

    // Volume Step
    let vol_slider = slider(0.01..=0.20f32, state.volume_step, Message::SettingsVolumeStepChanged)
        .step(0.01);
    let vol_label = text(format!("{:.0}%", state.volume_step * 100.0)).size(12).color(theme::text());

    content = content.push(
        column![
            text("Volume Step (Mouse Wheel)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            row![vol_slider, Space::with_width(12), vol_label].align_y(Alignment::Center),
        ]
        .spacing(2)
    ).push(Space::with_height(12));

    // Font Scale
    let scale_slider = slider(0.5..=2.5f32, state.font_scale, Message::SettingsFontScaleChanged)
        .step(0.05);
    let scale_label = text(format!("{:.2}x", state.font_scale)).size(12).color(theme::text());

    content = content.push(
        column![
            text("Font Scale (Zoom)").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            row![scale_slider, Space::with_width(12), scale_label].align_y(Alignment::Center),
        ]
        .spacing(2)
    ).push(Space::with_height(20));

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

    content = content.push(
        column![
            text("Shortcuts").size(12).font(crate::ui::icons::UI_FONT_BOLD).color(theme::subtext()),
            Space::with_height(4),
            shortcuts_btn,
        ]
        .spacing(2)
    ).push(Space::with_height(16));

    // Buttons
    let save_btn = button(text("Save Settings").color(theme::base()))
        .on_press(Message::SettingsSave)
        .padding([10, 20])
        .style(theme::primary_button);

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

    content = content.push(buttons);

    container(
        container(content)
            .width(420)
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
