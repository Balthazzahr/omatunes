use iced::widget::{button, column, container, row, text, text_input, Space, checkbox};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TagEditorState};
use crate::ui::theme;

fn get_suggestions(query: &str, items: &[String]) -> Vec<String> {
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for item in items {
        let item_trimmed = item.trim();
        let item_lower = item_trimmed.to_lowercase();
        if item_lower.starts_with(&query_lower) && item_lower != query_lower {
            matches.push(item_trimmed.to_string());
        }
    }
    matches.sort();
    matches.dedup();
    matches.truncate(4);
    matches
}

fn render_suggestions(
    suggestions: &[String],
    on_select: impl Fn(String) -> Message,
) -> Element<'static, Message> {
    let mut col = column![].spacing(4);
    for chunk in suggestions.chunks(2) {
        let mut row_el = row![].spacing(6);
        for suggestion in chunk {
            row_el = row_el.push(
                button(
                    text(suggestion.clone())
                        .size(10)
                        .color(theme::accent())
                )
                .on_press(on_select(suggestion.clone()))
                .style(theme::secondary_button)
                .padding([2, 6])
            );
        }
        col = col.push(row_el);
    }
    col.into()
}

pub fn view(
    state: &TagEditorState,
    unique_artists: &[String],
    unique_albums: &[String],
    unique_genres: &[String],
) -> Element<'_, Message> {
    let title_input = text_input("Title", &state.title)
        .id(iced::widget::text_input::Id::new("id3_title"))
        .on_input(Message::UpdateTagFieldTitle)
        .padding(8);

    let artist_input = text_input("Artist", &state.artist)
        .id(iced::widget::text_input::Id::new("id3_artist"))
        .on_input(Message::UpdateTagFieldArtist)
        .padding(8);

    let album_input = text_input("Album", &state.album)
        .id(iced::widget::text_input::Id::new("id3_album"))
        .on_input(Message::UpdateTagFieldAlbum)
        .padding(8);

    let genre_input = text_input("Genre", &state.genre)
        .id(iced::widget::text_input::Id::new("id3_genre"))
        .on_input(Message::UpdateTagFieldGenre)
        .padding(8);

    let track_num_input = text_input("Track Number", &state.track_number)
        .id(iced::widget::text_input::Id::new("id3_track"))
        .on_input(Message::UpdateTagFieldTrackNumber)
        .padding(8);

    let disc_num_input = text_input("Disc Number", &state.disc_number)
        .id(iced::widget::text_input::Id::new("id3_disc"))
        .on_input(Message::UpdateTagFieldDiscNumber)
        .padding(8);

    let year_input = text_input("Year", &state.year)
        .id(iced::widget::text_input::Id::new("id3_year"))
        .on_input(Message::UpdateTagFieldYear)
        .padding(8);

    let cover_path_val = state.cover_path.clone().unwrap_or_default();
    let cover_input = text_input("Cover Image Path (jpg/png)", &cover_path_val)
        .id(iced::widget::text_input::Id::new("id3_cover"))
        .on_input(Message::UpdateTagFieldCoverPath)
        .padding(8);

    let apply_to_album_checkbox = checkbox(
        "Apply changes (ticked fields) to entire album",
        state.apply_to_album,
    )
    .on_toggle(Message::UpdateTagFieldApplyToAlbum)
    .size(16);

    let artist_suggestions = get_suggestions(&state.artist, unique_artists);
    let album_suggestions = get_suggestions(&state.album, unique_albums);
    let genre_suggestions = get_suggestions(&state.genre, unique_genres);

    let tab_btn = |tab: crate::app::TagEditorTab, label: &'static str| {
        let is_active = state.active_tab == tab;
        
        button(container(text(label).font(crate::ui::icons::UI_FONT_BOLD).size(12)).center_x(Length::Fill).center_y(Length::Fill))
            .on_press(Message::SelectTagEditorTab(tab))
            .width(Length::FillPortion(1))
            .height(36.0)
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
                    border: iced::Border {
                        color: if is_active { theme::accent() } else { theme::surface0() },
                        width: if is_active { 1.0 } else { 0.5 },
                        radius: 0.0.into(),
                    },
                    text_color: if is_active { theme::accent() } else { theme::subtext() },
                    ..Default::default()
                }
            })
            .padding(0)
    };

    let tabs_header = row![
        tab_btn(crate::app::TagEditorTab::Main, "Main"),
        tab_btn(crate::app::TagEditorTab::Lyrics, "Lyrics"),
    ]
    .width(Length::Fill);

    let mut body = column![].spacing(6);

    if state.active_tab == crate::app::TagEditorTab::Main {
        body = body
            .push(
                row![
                    checkbox("", state.apply_title)
                        .on_toggle(Message::ToggleTagFieldApplyTitle)
                        .size(16),
                    column![
                        text("Title").size(12).color(theme::subtext()),
                        title_input
                    ].width(Length::Fill)
                ].align_y(Alignment::Center).spacing(8)
            )
            .push(Space::with_height(2))
            .push(
                row![
                    checkbox("", state.apply_artist)
                        .on_toggle(Message::ToggleTagFieldApplyArtist)
                        .size(16),
                    column![
                        text("Artist").size(12).color(theme::subtext()),
                        artist_input,
                        if !artist_suggestions.is_empty() {
                            iced::Element::from(column![
                                Space::with_height(4),
                                render_suggestions(&artist_suggestions, Message::UpdateTagFieldArtist)
                            ])
                        } else {
                            iced::Element::from(Space::with_height(0))
                        }
                    ].width(Length::Fill)
                ].align_y(Alignment::Center).spacing(8)
            )
            .push(Space::with_height(2))
            .push(
                row![
                    checkbox("", state.apply_album)
                        .on_toggle(Message::ToggleTagFieldApplyAlbum)
                        .size(16),
                    column![
                        text("Album").size(12).color(theme::subtext()),
                        album_input,
                        if !album_suggestions.is_empty() {
                            iced::Element::from(column![
                                Space::with_height(4),
                                render_suggestions(&album_suggestions, Message::UpdateTagFieldAlbum)
                            ])
                        } else {
                            iced::Element::from(Space::with_height(0))
                        }
                    ].width(Length::Fill)
                ].align_y(Alignment::Center).spacing(8)
            )
            .push(Space::with_height(2))
            .push(
                row![
                    checkbox("", state.apply_genre)
                        .on_toggle(Message::ToggleTagFieldApplyGenre)
                        .size(16),
                    column![
                        text("Genre").size(12).color(theme::subtext()),
                        genre_input,
                        if !genre_suggestions.is_empty() {
                            iced::Element::from(column![
                                Space::with_height(4),
                                render_suggestions(&genre_suggestions, Message::UpdateTagFieldGenre)
                            ])
                        } else {
                            iced::Element::from(Space::with_height(0))
                        }
                    ].width(Length::Fill)
                ].align_y(Alignment::Center).spacing(8)
            )
            .push(Space::with_height(2))
            .push(
                row![
                    row![
                        checkbox("", state.apply_track_num)
                            .on_toggle(Message::ToggleTagFieldApplyTrackNum)
                            .size(16),
                        column![
                            text("Track #").size(12).color(theme::subtext()),
                            track_num_input
                        ].width(Length::Fill)
                    ].align_y(Alignment::Center).spacing(8).width(Length::FillPortion(1)),
                    Space::with_width(12),
                    row![
                        checkbox("", state.apply_disc_num)
                            .on_toggle(Message::ToggleTagFieldApplyDiscNum)
                            .size(16),
                        column![
                            text("Disc #").size(12).color(theme::subtext()),
                            disc_num_input
                        ].width(Length::Fill)
                    ].align_y(Alignment::Center).spacing(8).width(Length::FillPortion(1)),
                    Space::with_width(12),
                    row![
                        checkbox("", state.apply_year)
                            .on_toggle(Message::ToggleTagFieldApplyYear)
                            .size(16),
                        column![
                            text("Year").size(12).color(theme::subtext()),
                            year_input
                        ].width(Length::Fill)
                    ].align_y(Alignment::Center).spacing(8).width(Length::FillPortion(1)),
                ]
            )
            .push(Space::with_height(2))
            .push(
                row![
                    checkbox("", state.apply_cover)
                        .on_toggle(Message::ToggleTagFieldApplyCover)
                        .size(16),
                    column![
                        row![
                            text("Cover Path").size(12).color(theme::subtext()),
                            Space::with_width(Length::Fill),
                            button(text("Search Online").size(10))
                                .on_press(Message::SearchCoverOnline)
                                .style(theme::secondary_button)
                                .padding([2, 6])
                        ].align_y(Alignment::Center),
                        cover_input
                    ].width(Length::Fill)
                ].align_y(Alignment::Center).spacing(8)
            );
    } else {
        body = body.push(
            row![
                checkbox("", state.apply_lyrics)
                    .on_toggle(Message::ToggleTagFieldApplyLyrics)
                    .size(16),
                column![
                    row![
                        text("Lyrics").size(12).color(theme::subtext()),
                        Space::with_width(Length::Fill),
                        button(text("Search Online").size(10))
                            .on_press(Message::SearchLyricsOnline)
                            .style(theme::secondary_button)
                            .padding([2, 6])
                    ].align_y(Alignment::Center),
                    Space::with_height(4),
                    container(
                        iced::widget::text_editor(&state.lyrics_content)
                            .on_action(Message::UpdateTagFieldLyrics)
                            .height(Length::Fixed(240.0))
                    )
                    .style(|_| iced::widget::container::Style {
                        border: iced::Border {
                            color: theme::surface0(),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                ].width(Length::Fill)
            ].spacing(8)
        );
    }

    let mut content = column![
        text("Edit ID3 Tags")
            .size(18)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(theme::accent()),
        Space::with_height(8),
        tabs_header,
        Space::with_height(12),
        body,
        Space::with_height(12)
    ]
    .spacing(4)
    .padding(24)
    .width(500);

    content = content.push(apply_to_album_checkbox);
    content = content.push(Space::with_height(16));

    content = content.push(
        row![
            button(text("Cancel").color(theme::text()))
                .on_press(Message::CloseTagEditor)
                .padding([8, 16])
                .style(theme::secondary_button),
            Space::with_width(12),
            button(text("Save").color(theme::base()))
                .on_press(Message::SaveTags)
                .padding([8, 16])
                .style(theme::primary_button)
        ]
        .align_y(Alignment::Center)
    );

    // Modal background overlay
    container(
        container(content)
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
