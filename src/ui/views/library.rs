use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space, checkbox, text_input, stack, tooltip};
use iced::{Alignment, Element, Length};

use crate::app::{AppState, Message, ViewMode, SortColumn, PlaylistDialogMode};
use crate::ui::theme;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let sidebar = folder_sidebar(state);
    let main_content: Element<'_, Message> = if let Some(ref builder_state) = state.smart_playlist_builder {
        let mut unique_artists: Vec<String> = state.all_tracks.iter().map(|t| t.artist.clone()).collect();
        unique_artists.sort();
        unique_artists.dedup();

        let mut unique_albums: Vec<String> = state.all_tracks.iter().map(|t| t.album.clone()).collect();
        unique_albums.sort();
        unique_albums.dedup();

        let mut unique_genres: Vec<String> = state.all_tracks.iter().map(|t| t.genre.clone()).collect();
        unique_genres.sort();
        unique_genres.dedup();

        crate::ui::components::smart_playlist_builder::view(
            builder_state,
            &unique_artists,
            &unique_albums,
            &unique_genres,
        )
    } else {
        track_list_view(state)
    };

    let drag_handle = mouse_area(
        container(
            container(Space::new(Length::Fixed(2.0), Length::Fill))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(if state.dragging_sidebar || state.is_hovering_sidebar_resizer { theme::accent() } else { theme::surface0() })),
                    ..Default::default()
                })
        )
        .width(6.0)
        .height(Length::Fill)
        .center_x(Length::Fixed(6.0))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::base())),
            ..Default::default()
        })
    )
    .on_press(Message::SidebarDragStart)
    .on_enter(Message::HoverSidebarResizer(true))
    .on_exit(Message::HoverSidebarResizer(false))
    .interaction(iced::mouse::Interaction::ResizingHorizontally);

    row![sidebar, drag_handle, main_content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn folder_sidebar(state: &AppState) -> Element<'_, Message> {
    let sidebar_clear_btn: Element<'_, Message> = if !state.sidebar_search.is_empty() {
        button(
            text("\u{f00d}")
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(theme::red())
                .size(12)
        )
        .on_press(Message::SidebarSearchChanged(String::new()))
        .style(iced::widget::button::text)
        .padding(4)
        .into()
    } else {
        Space::with_width(0.0).into()
    };

    let sidebar_search_input = row![
        text_input("Search...", &state.sidebar_search)
            .id(iced::widget::text_input::Id::new("sidebar_search_input"))
            .on_input(Message::SidebarSearchChanged)
            .padding(6)
            .size(12)
            .width(Length::Fill),
        sidebar_clear_btn,
    ]
    .align_y(Alignment::Center)
    .spacing(4);

    let sidebar_items: Element<Message> = match state.view_mode {
        ViewMode::Artists | ViewMode::NowPlaying => {
            column(
                state.artists().into_iter().map(|artist| {
                    let is_selected = state.selected_artist.as_ref() == Some(&artist) && state.selected_playlist.is_none();

                    let label = text(artist.clone())
                        .color(if is_selected { theme::accent() } else { theme::text() })
                        .size(13);

                    let context_btn = button(
                        text("\u{f142}") // vertical ellipsis Nerd Font
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(theme::overlay0())
                            .size(13)
                    )
                    .on_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Artist(artist.clone()))))
                    .style(iced::widget::button::text);

                    let btn_row = row![
                        button(label)
                            .on_press(Message::SelectArtist(artist.clone()))
                            .style(iced::widget::button::text)
                            .width(Length::Fill)
                            .padding([6, 12]),
                        context_btn,
                        Space::with_width(4),
                    ]
                    .align_y(Alignment::Center);

                    let row_widget = mouse_area(btn_row)
                        .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Artist(artist.clone()))));

                    if is_selected {
                        container(row_widget).style(theme::selected_row).width(Length::Fill).into()
                    } else {
                        container(row_widget).width(Length::Fill).into()
                    }
                })
                .collect::<Vec<_>>(),
            )
            .spacing(2)
            .into()
        }
        ViewMode::Albums => {
            column(
                state.albums().into_iter().map(|album| {
                    let is_selected = state.selected_album.as_ref() == Some(&album) && state.selected_playlist.is_none();

                    let label = text(album.clone())
                        .color(if is_selected { theme::accent() } else { theme::text() })
                        .size(13);

                    let context_btn = button(
                        text("\u{f142}")
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(theme::overlay0())
                            .size(13)
                    )
                    .on_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Album(album.clone()))))
                    .style(iced::widget::button::text);

                    let btn_row = row![
                        button(label)
                            .on_press(Message::SelectAlbum(album.clone()))
                            .style(iced::widget::button::text)
                            .width(Length::Fill)
                            .padding([6, 12]),
                        context_btn,
                        Space::with_width(4),
                    ]
                    .align_y(Alignment::Center);

                    let row_widget = mouse_area(btn_row)
                        .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Album(album.clone()))));

                    if is_selected {
                        container(row_widget).style(theme::selected_row).width(Length::Fill).into()
                    } else {
                        container(row_widget).width(Length::Fill).into()
                    }
                })
                .collect::<Vec<_>>(),
            )
            .spacing(2)
            .into()
        }
        ViewMode::Genres => {
            column(
                state.genres().into_iter().map(|genre| {
                    let is_selected = state.selected_genre.as_ref() == Some(&genre) && state.selected_playlist.is_none();

                    let label = text(genre.clone())
                        .color(if is_selected { theme::accent() } else { theme::text() })
                        .size(13);

                    let btn = button(label)
                        .on_press(Message::SelectGenre(genre.clone()))
                        .style(iced::widget::button::text)
                        .width(Length::Fill)
                        .padding([6, 12]);

                    if is_selected {
                        container(btn).style(theme::selected_row).width(Length::Fill).into()
                    } else {
                        container(btn).width(Length::Fill).into()
                    }
                })
                .collect::<Vec<_>>(),
            )
            .spacing(2)
            .into()
        }
    };




    let render_playlist_item = |name: String, is_auto: bool| -> Element<'_, Message> {
        let is_selected = state.selected_playlist.as_ref() == Some(&name);

        let icon_str = if name == "Liked Songs" {
            crate::ui::icons::ICON_HEART
        } else if name == "Most Played" {
            crate::ui::icons::ICON_PODIUM
        } else if name == "Recently Played" {
            "\u{f017}"
        } else {
            crate::ui::icons::ICON_MUSIC
        };

        let is_custom = !is_auto;

        let label_text = text(name.clone())
            .color(if is_selected { theme::accent() } else if is_auto { theme::subtext() } else { theme::text() })
            .font(if is_auto { crate::ui::icons::UI_FONT_BOLD } else { crate::ui::icons::UI_FONT })
            .size(14);

        let label_container = container(label_text)
            .width(Length::Fill)
            .clip(true);

        let label_row = row![
            text(icon_str)
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(if is_selected { theme::accent() } else { theme::overlay0() })
                .size(14),
            Space::with_width(8),
            label_container,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let is_hovered = state.hovered_playlist.as_ref() == Some(&name);

        let btn = if is_custom {
            let rename_btn = button(
                text("\u{f044}")
                    .font(crate::ui::icons::NERD_FONT_MONO)
                    .color(theme::overlay0())
                    .size(12)
            )
            .on_press(Message::OpenPlaylistDialog(PlaylistDialogMode::Rename(name.clone())))
            .style(iced::widget::button::text);

            let delete_btn = button(
                text("\u{f1f8}")
                    .font(crate::ui::icons::NERD_FONT_MONO)
                    .color(theme::red())
                    .size(12)
            )
            .on_press(Message::DeletePlaylist(name.clone()))
            .style(iced::widget::button::text);

            let mut action_row = row![
                button(label_row)
                    .on_press(Message::SelectPlaylist(name.clone()))
                    .style(iced::widget::button::text)
                    .width(Length::Fill)
                    .padding([6, 12])
            ];

            if is_hovered {
                action_row = action_row.push(rename_btn).push(Space::with_width(4)).push(delete_btn).push(Space::with_width(6));
            }

            action_row.align_y(Alignment::Center).width(Length::Fill)
        } else {
            row![
                button(label_row)
                    .on_press(Message::SelectPlaylist(name.clone()))
                    .style(iced::widget::button::text)
                    .width(Length::Fill)
                    .padding([6, 12])
            ]
            .width(Length::Fill)
        };

        let row_container = if is_selected {
            container(btn).style(theme::selected_row).width(Length::Fill)
        } else {
            container(btn).width(Length::Fill)
        };

        mouse_area(row_container)
            .on_enter(Message::HoverPlaylist(Some(name.clone())))
            .on_exit(Message::HoverPlaylist(None))
            .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Playlist(name.clone()))))
            .into()
    };

    let render_smart_playlist_item = |name: String| -> Element<'_, Message> {
        let is_selected = state.selected_playlist.as_ref() == Some(&name);
        let icon_str = "\u{ebcf}";

        let label_text = text(name.clone())
            .color(if is_selected { theme::accent() } else { theme::text() })
            .font(crate::ui::icons::UI_FONT)
            .size(14);

        let label_container = container(label_text)
            .width(Length::Fill)
            .clip(true);

        let label_row = row![
            text(icon_str)
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(if is_selected { theme::accent() } else { theme::overlay0() })
                .size(14),
            Space::with_width(8),
            label_container,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let is_hovered = state.hovered_playlist.as_ref() == Some(&name);

        let edit_btn = button(
            text("\u{f044}")
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(theme::overlay0())
                .size(12)
        )
        .on_press(Message::EditSmartPlaylist(name.clone()))
        .style(iced::widget::button::text);

        let delete_btn = button(
            text("\u{f1f8}")
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(theme::red())
                .size(12)
        )
        .on_press(Message::DeleteSmartPlaylist(name.clone()))
        .style(iced::widget::button::text);

        let mut action_row = row![
            button(label_row)
                .on_press(Message::SelectPlaylist(name.clone()))
                .style(iced::widget::button::text)
                .width(Length::Fill)
                .padding([6, 12])
        ];

        if is_hovered {
            action_row = action_row.push(edit_btn).push(Space::with_width(4)).push(delete_btn).push(Space::with_width(6));
        }

        let btn = action_row.align_y(Alignment::Center).width(Length::Fill);

        let row_container = if is_selected {
            container(btn).style(theme::selected_row).width(Length::Fill)
        } else {
            container(btn).width(Length::Fill)
        };

        mouse_area(row_container)
            .on_enter(Message::HoverPlaylist(Some(name.clone())))
            .on_exit(Message::HoverPlaylist(None))
            .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::SmartPlaylist(name.clone()))))
            .into()
    };

    let playlist_total_width = state.sidebar_width.round() - 16.0;
    let playlist_tab_width_1 = (playlist_total_width / 3.0).floor();
    let playlist_tab_width_2 = (playlist_total_width / 3.0).floor();
    let playlist_tab_width_3 = playlist_total_width - playlist_tab_width_1 - playlist_tab_width_2;

    let playlist_tab_btn = |tab: crate::app::PlaylistTab, icon: &'static str, width: f32, tooltip_text: &'static str| {
        let is_active = state.playlist_tab == tab && (state.selected_playlist.is_some() || tab == crate::app::PlaylistTab::Smart);
        let btn_icon = text(icon)
            .size(18)
            .font(crate::ui::icons::NERD_FONT_MONO);
        
        let btn = button(container(btn_icon).center_x(Length::Fill).center_y(Length::Fill))
            .on_press(Message::SelectPlaylistTab(tab))
            .width(width)
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
                        color: if is_active { theme::accent() } else { theme::surface0() },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: if is_active { theme::accent() } else { theme::subtext() },
                    ..Default::default()
                }
            })
            .padding(0);

        let tooltip_content = container(
            text(tooltip_text)
                .size(11)
                .font(crate::ui::icons::UI_FONT)
                .color(theme::text())
        )
        .padding([4, 8])
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            border: iced::Border {
                color: theme::overlay0(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        tooltip(btn, tooltip_content, tooltip::Position::Top)
    };

    let playlist_tabs = row![
        playlist_tab_btn(crate::app::PlaylistTab::Playlists, crate::ui::icons::ICON_LIST, playlist_tab_width_1, "User Playlists"),
        playlist_tab_btn(crate::app::PlaylistTab::Autoplaylists, crate::ui::icons::ICON_BOLT, playlist_tab_width_2, "Auto Playlists"),
        playlist_tab_btn(crate::app::PlaylistTab::Smart, crate::ui::icons::ICON_WAND, playlist_tab_width_3, "Smart Playlists"),
    ]
    .spacing(0)
    .align_y(Alignment::Center);

    let mut playlists_area_col = column![].spacing(6).height(Length::Fill).width(Length::Fill);
    
    if state.playlist_tab == crate::app::PlaylistTab::Playlists {
        let mut user_playlists_col = column![].spacing(2).width(Length::Fill);
        let playlist_order = crate::db::get(|db| db.playlist_order.clone());
        let is_sidebar_dragging = matches!(state.dragging_playlist_sidebar, Some((crate::app::PlaylistTab::Playlists, _)));
        
        for (idx, name) in playlist_order.iter().enumerate() {
            let handle: Element<'_, Message> = mouse_area(
                container(
                    text("\u{f0c9}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(if state.dragging_playlist_sidebar == Some((crate::app::PlaylistTab::Playlists, idx)) { theme::accent() } else { theme::overlay0() })
                        .size(12)
                ).padding([4, 8])
            )
            .on_press(Message::PlaylistSidebarDragStart(crate::app::PlaylistTab::Playlists, idx))
            .on_release(Message::PlaylistSidebarDragEnd)
            .interaction(iced::mouse::Interaction::Grab)
            .into();

            let row_content = row![handle, render_playlist_item(name.clone(), false)]
                .align_y(Alignment::Center);

            let row_el: Element<'_, Message> = if is_sidebar_dragging {
                mouse_area(row_content)
                    .on_enter(Message::PlaylistSidebarDragOver(crate::app::PlaylistTab::Playlists, idx))
                    .into()
            } else {
                row_content.into()
            };

            user_playlists_col = user_playlists_col.push(row_el);
        }
        
        playlists_area_col = playlists_area_col.push(
            container(scrollable(user_playlists_col).width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
        );

        let add_playlist_btn = button(
            container(
                row![
                    text("\u{f07b}\u{f067}").font(crate::ui::icons::NERD_FONT_MONO).size(11),
                    Space::with_width(6),
                    text("New Playlist").size(11).font(crate::ui::icons::UI_FONT_BOLD)
                ].align_y(Alignment::Center)
            ).center_x(Length::Fill)
        )
        .on_press(Message::OpenPlaylistDialog(PlaylistDialogMode::Create))
        .style(theme::secondary_button)
        .padding([4, 12])
        .width(Length::Fill);

        playlists_area_col = playlists_area_col.push(add_playlist_btn);
    } else if state.playlist_tab == crate::app::PlaylistTab::Autoplaylists {
        let mut auto_playlists_col = column![].spacing(2).width(Length::Fill);
        auto_playlists_col = auto_playlists_col.push(render_playlist_item("Liked Songs".to_string(), true));
        auto_playlists_col = auto_playlists_col.push(render_playlist_item("Recently Played".to_string(), true));
        auto_playlists_col = auto_playlists_col.push(render_playlist_item("Most Played".to_string(), true));
        auto_playlists_col = auto_playlists_col.push(render_playlist_item("New Music".to_string(), true));

        playlists_area_col = playlists_area_col.push(
            container(scrollable(auto_playlists_col).width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
        );
    } else {
        let mut smart_playlists_col = column![].spacing(2).width(Length::Fill);
        let smart_playlist_order = crate::db::get(|db| db.smart_playlist_order.clone());
        let is_sidebar_dragging = matches!(state.dragging_playlist_sidebar, Some((crate::app::PlaylistTab::Smart, _)));

        for (idx, name) in smart_playlist_order.iter().enumerate() {
            let handle: Element<'_, Message> = mouse_area(
                container(
                    text("\u{f0c9}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(if state.dragging_playlist_sidebar == Some((crate::app::PlaylistTab::Smart, idx)) { theme::accent() } else { theme::overlay0() })
                        .size(12)
                ).padding([4, 8])
            )
            .on_press(Message::PlaylistSidebarDragStart(crate::app::PlaylistTab::Smart, idx))
            .on_release(Message::PlaylistSidebarDragEnd)
            .interaction(iced::mouse::Interaction::Grab)
            .into();

            let row_content = row![handle, render_smart_playlist_item(name.clone())]
                .align_y(Alignment::Center);

            let row_el: Element<'_, Message> = if is_sidebar_dragging {
                mouse_area(row_content)
                    .on_enter(Message::PlaylistSidebarDragOver(crate::app::PlaylistTab::Smart, idx))
                    .into()
            } else {
                row_content.into()
            };

            smart_playlists_col = smart_playlists_col.push(row_el);
        }

        playlists_area_col = playlists_area_col.push(
            container(scrollable(smart_playlists_col).width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
        );

        let add_smart_playlist_btn = button(
            container(
                row![
                    text("\u{ebcf}").font(crate::ui::icons::NERD_FONT_MONO).size(11),
                    Space::with_width(6),
                    text("New Smart Playlist").size(11).font(crate::ui::icons::UI_FONT_BOLD)
                ].align_y(Alignment::Center)
            ).center_x(Length::Fill)
        )
        .on_press(Message::NewSmartPlaylist)
        .style(theme::secondary_button)
        .padding([4, 12])
        .width(Length::Fill);

        playlists_area_col = playlists_area_col.push(add_smart_playlist_btn);
    }

    let playlist_drag_handle = mouse_area(
        container(
            container(Space::new(Length::Fill, Length::Fixed(2.0)))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(if state.dragging_playlist_split || state.is_hovering_playlist_resizer { theme::accent() } else { theme::surface0() })),
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(6.0)
        .center_y(Length::Fixed(6.0))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::mantle())),
            ..Default::default()
        })
    )
    .on_press(Message::PlaylistDragStart)
    .on_enter(Message::HoverPlaylistResizer(true))
    .on_exit(Message::HoverPlaylistResizer(false))
    .interaction(iced::mouse::Interaction::ResizingVertically);

    let sidebar_items_hover = mouse_area(scrollable(sidebar_items))
        .on_enter(Message::HoverSidebarList(true))
        .on_exit(Message::HoverSidebarList(false));

    let mut sidebar_items_col = column![sidebar_items_hover];
    if !state.hidden_artists_albums.is_empty() {
        let restore_btn = button(
            text("Restore Hidden Items")
                .size(11)
                .color(theme::accent())
        )
        .on_press(Message::RestoreHiddenItems)
        .style(iced::widget::button::text)
        .padding(4);
        sidebar_items_col = sidebar_items_col.push(Space::with_height(4)).push(restore_btn);
    }

    let all_category_row: Element<'_, Message> = match state.view_mode {
        ViewMode::Artists | ViewMode::NowPlaying => {
            let is_selected = state.selected_artist.is_none() && state.selected_playlist.is_none();
            let label = text("All Artists")
                .color(if is_selected { theme::accent() } else { theme::text() })
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(13);
            let btn = button(label)
                .on_press(Message::SelectAllArtists)
                .style(iced::widget::button::text)
                .width(Length::Fill)
                .padding([6, 12]);
            let row_container = if is_selected {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.15))),
                        border: iced::Border {
                            color: theme::with_alpha(theme::accent(), 0.4),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            } else {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::surface0())),
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            };
            row_container.into()
        }
        ViewMode::Albums => {
            let is_selected = state.selected_album.is_none() && state.selected_playlist.is_none();
            let label = text("All Albums")
                .color(if is_selected { theme::accent() } else { theme::text() })
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(13);
            let btn = button(label)
                .on_press(Message::SelectAllAlbums)
                .style(iced::widget::button::text)
                .width(Length::Fill)
                .padding([6, 12]);
            let row_container = if is_selected {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.15))),
                        border: iced::Border {
                            color: theme::with_alpha(theme::accent(), 0.4),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            } else {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::surface0())),
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            };
            row_container.into()
        }
        ViewMode::Genres => {
            let is_selected = state.selected_genre.is_none() && state.selected_playlist.is_none();
            let label = text("All Genres")
                .color(if is_selected { theme::accent() } else { theme::text() })
                .font(crate::ui::icons::UI_FONT_BOLD)
                .size(13);
            let btn = button(label)
                .on_press(Message::SelectAllGenres)
                .style(iced::widget::button::text)
                .width(Length::Fill)
                .padding([6, 12]);
            let row_container = if is_selected {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.15))),
                        border: iced::Border {
                            color: theme::with_alpha(theme::accent(), 0.4),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            } else {
                container(btn)
                    .style(|_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::surface0())),
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
            };
            row_container.into()
        }
    };

    container(
        column![
            sidebar_search_input,
            Space::with_height(8),
            all_category_row,
            Space::with_height(4),
            container(sidebar_items_col)
                .height(Length::Fill),
            playlist_drag_handle,
            Space::with_height(8),
            container(
                column![
                    playlist_tabs,
                    Space::with_height(6),
                    playlists_area_col,
                ]
                .height(Length::Fill)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fixed(state.playlist_height)),
        ]
        .padding(8),
    )
    .style(theme::sidebar)
    .width(state.sidebar_width.round())
    .height(Length::Fill)
    .into()
}

struct TrackListDependency {
    tracks: Vec<crate::library::models::Track>,
    current_track_id: Option<i64>,
    current_track_album: Option<String>,
    pulse_tick: u32,
    is_playing: bool,
    is_paused: bool,
    selected_tracks: Vec<crate::library::models::Track>,
    group_by_album: bool,
    sort_column: Option<SortColumn>,
    sort_ascending: bool,
    strings: &'static crate::locale::Strings,
    hovered_album_header: Option<String>,
    visible_start: usize,
    visible_end: usize,
    responsive_columns: Vec<crate::db::TableColumn>,
}

impl std::hash::Hash for TrackListDependency {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.group_by_album.hash(state);
        self.sort_column.hash(state);
        self.sort_ascending.hash(state);
        self.current_track_id.hash(state);
        self.current_track_album.hash(state);
        self.pulse_tick.hash(state);
        self.is_playing.hash(state);
        self.is_paused.hash(state);
        self.selected_tracks.len().hash(state);
        self.tracks.len().hash(state);
        self.hovered_album_header.hash(state);
        self.visible_start.hash(state);
        self.visible_end.hash(state);
        self.responsive_columns.hash(state);
        for t in &self.selected_tracks {
            t.id.hash(state);
            t.title.hash(state);
            t.artist.hash(state);
            t.album.hash(state);
            t.genre.hash(state);
            t.year.hash(state);
            t.track_number.hash(state);
            t.disc_number.hash(state);
        }
        for t in &self.tracks {
            t.id.hash(state);
            t.liked.hash(state);
            t.play_count.hash(state);
            t.title.hash(state);
            t.artist.hash(state);
            t.album.hash(state);
            t.genre.hash(state);
            t.year.hash(state);
            t.track_number.hash(state);
            t.disc_number.hash(state);
            t.lyrics.hash(state);
        }
    }
}

pub fn get_available_track_list_width(state: &AppState) -> f32 {
    let sidebar_visible = state.selected_playlist.is_none() && (state.view_mode != ViewMode::NowPlaying);
    let sidebar_w = if sidebar_visible { state.sidebar_width.round() + 6.0 } else { 0.0 };
    
    let is_right_open = state.right_panel_tab.is_some() && state.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 600.0);
    let right_w = if is_right_open { 6.0 + state.right_panel_width } else { 0.0 };
    
    state.window_width - sidebar_w - right_w
}

pub fn get_responsive_columns(state: &AppState) -> Vec<crate::db::TableColumn> {
    let saved_cols = crate::db::get(|db| db.table_columns.clone());
    let available_width = get_available_track_list_width(state) - 24.0;
    
    let hide_priority = &[
        crate::db::TableColumn::DiscNumber,
        crate::db::TableColumn::Plays,
        crate::db::TableColumn::DatePlayed,
        crate::db::TableColumn::Genre,
        crate::db::TableColumn::Liked,
        crate::db::TableColumn::Year,
        crate::db::TableColumn::Album,
        crate::db::TableColumn::Artist,
        crate::db::TableColumn::TrackNumber,
    ];
    
    let mut visible_cols = saved_cols.clone();
    
    let calc_width = |cols: &[crate::db::TableColumn]| -> f32 {
        let mut total_fixed = 0.0;
        let mut fill_count = 0;
        for &col in cols {
            match col {
                crate::db::TableColumn::TrackNumber => total_fixed += 30.0,
                crate::db::TableColumn::Liked => total_fixed += 40.0,
                crate::db::TableColumn::Plays => total_fixed += 40.0,
                crate::db::TableColumn::Year => total_fixed += 50.0,
                crate::db::TableColumn::DiscNumber => total_fixed += 50.0,
                crate::db::TableColumn::Duration => total_fixed += 80.0,
                _ => fill_count += 1,
            }
        }
        let spacing = if cols.is_empty() { 0.0 } else { (cols.len() - 1) as f32 * 12.0 };
        total_fixed + (fill_count as f32 * 80.0) + spacing
    };
    
    for &col_to_hide in hide_priority {
        if calc_width(&visible_cols) <= available_width {
            break;
        }
        if visible_cols.contains(&col_to_hide) {
            visible_cols.retain(|&c| c != col_to_hide);
        }
    }
    
    if calc_width(&visible_cols) > available_width {
        let mut core_set = Vec::new();
        if saved_cols.contains(&crate::db::TableColumn::Title) {
            core_set.push(crate::db::TableColumn::Title);
        }
        if saved_cols.contains(&crate::db::TableColumn::Duration) {
            core_set.push(crate::db::TableColumn::Duration);
        }
        visible_cols = core_set;
    }
    
    visible_cols
}

fn track_list_view(state: &AppState) -> Element<'_, Message> {
    let is_recently_played = state.selected_playlist.as_deref() == Some("Recently Played");
    let group_by_album = state.group_by_album && !is_recently_played;

    let table_columns = get_responsive_columns(state);
    let mut header_widgets: Vec<Element<'_, Message>> = Vec::new();
    
    for col in table_columns {
        let label = col.label();
        let width = col_width(col);
        let sort_col = col_to_sort_col(col);
        
        let is_sorted = state.sort_column == Some(sort_col);
        let arrow = if is_sorted {
            if state.sort_ascending { " ▲" } else { " ▼" }
        } else {
            ""
        };
        let txt = text(format!("{label}{arrow}"))
            .size(11)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(if is_sorted { theme::accent() } else { theme::subtext() });
            
        let btn = button(txt)
            .on_press(Message::SortBy(sort_col))
            .style(iced::widget::button::text)
            .padding(0)
            .width(width);

        let header_area = mouse_area(btn)
            .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Header(col))));

        header_widgets.push(header_area.into());
    }

    let table_headers = container(
        row(header_widgets)
            .spacing(12)
            .align_y(Alignment::Center)
            .padding([8, 12])
    )
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::mantle())),
        border: iced::Border {
            color: theme::surface0(),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .width(Length::Fill);

    let pulse_tick = if matches!(state.playback_state, crate::audio::PlaybackState::Playing) {
        (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() / 250) as u32
    } else {
        0
    };

    let is_playing = matches!(state.playback_state, crate::audio::PlaybackState::Playing);
    let is_paused = matches!(state.playback_state, crate::audio::PlaybackState::Paused);

    let track_list_dep = TrackListDependency {
        tracks: state.tracks.clone(),
        current_track_id: state.current_track.as_ref().map(|t| t.id),
        current_track_album: state.current_track.as_ref().map(|t| t.album.clone()),
        pulse_tick,
        is_playing,
        is_paused,
        selected_tracks: state.selected_tracks.clone(),
        group_by_album,
        sort_column: state.sort_column,
        sort_ascending: state.sort_ascending,
        strings: state.strings,
        hovered_album_header: state.hovered_album_header.clone(),
        visible_start: state.track_list_start,
        visible_end: state.track_list_end,
        responsive_columns: get_responsive_columns(state),
    };

    let tracklist_scroll = iced::widget::lazy(track_list_dep, move |dep| -> Element<'static, Message> {
        let current_id = dep.current_track_id;
        let mut rows: Vec<Element<Message>> = Vec::new();

        if dep.group_by_album {
            // Group tracks in the visible window by album keeping insertion order
            let start = dep.visible_start;
            let end = dep.visible_end.min(dep.tracks.len());
            let tracks_in_window = &dep.tracks[start..end];

            let mut groups: Vec<(String, Vec<&crate::library::models::Track>)> = Vec::new();
            for track in tracks_in_window {
                if let Some(last) = groups.last_mut() {
                    if last.0 == track.album {
                        last.1.push(track);
                        continue;
                    }
                }
                groups.push((track.album.clone(), vec![track]));
            }

            for (album_name, tracks) in groups.into_iter() {
                let n = tracks.len();
                let is_hovered = dep.hovered_album_header.as_ref() == Some(&album_name);
                let is_current_album_playing = dep.current_track_album.as_deref() == Some(&album_name);

                let album_display_name = if album_name.trim().is_empty() {
                    "Unknown Album".to_string()
                } else {
                    album_name.clone()
                };

                let is_active_playing = is_current_album_playing && dep.is_playing;

                let album_name_btn = button(
                    text(album_display_name)
                        .color(if is_active_playing {
                            theme::accent()
                        } else {
                            theme::text()
                        })
                        .size(13)
                        .font(crate::ui::icons::UI_FONT_BOLD)
                )
                .on_press(Message::ToggleAlbumPlayPause(album_name.clone()))
                .style(iced::widget::button::text)
                .padding(0);

                let play_btn: Element<'static, Message> = if is_current_album_playing || is_hovered {
                    let btn_color = if is_active_playing {
                        theme::accent()
                    } else {
                        theme::subtext()
                    };

                    let (btn_icon, btn_label) = if is_current_album_playing {
                        if dep.is_playing {
                            if is_hovered {
                                (crate::ui::icons::ICON_PAUSE, "  PAUSE ")
                            } else {
                                (crate::ui::icons::ICON_PAUSE, "  PLAYING ")
                            }
                        } else {
                            if is_hovered {
                                (crate::ui::icons::ICON_PLAY, "  PLAY ALBUM ")
                            } else {
                                (crate::ui::icons::ICON_PLAY, "  PAUSED ")
                            }
                        }
                    } else {
                        (crate::ui::icons::ICON_PLAY, "  PLAY ALBUM ")
                    };

                    button(
                        row![
                            text(btn_icon)
                                .font(crate::ui::icons::NERD_FONT_MONO)
                                .size(13),
                            text(btn_label)
                                .size(13)
                                .font(crate::ui::icons::UI_FONT_BOLD),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    )
                    .on_press(Message::ToggleAlbumPlayPause(album_name.clone()))
                    .style(move |_, _| iced::widget::button::Style {
                        text_color: btn_color,
                        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                        ..Default::default()
                    })
                    .padding(0)
                    .into()
                } else {
                    Space::with_width(0).into()
                };

                let header = mouse_area(
                    container(
                        row![
                            album_name_btn,
                            Space::with_width(8),
                            play_btn,
                            Space::with_width(Length::Fill),
                            text(dep.strings.track_count(n))
                                .color(theme::overlay0())
                                .size(11),
                        ]
                        .align_y(Alignment::Center)
                        .padding([6, 12]),
                    )
                    .style(if is_active_playing {
                        theme::album_header_active
                    } else {
                        theme::album_header
                    })
                    .width(Length::Fill)
                )
                .on_enter(Message::HoverAlbumHeader(Some(album_name.clone())))
                .on_exit(Message::HoverAlbumHeader(None));

                rows.push(header.into());

                for track in tracks.into_iter() {
                    rows.push(render_track_row(dep, track, true, current_id));
                }
                rows.push(Space::with_height(8).into());
            }
        } else {
            let start = dep.visible_start;
            let end = dep.visible_end.min(dep.tracks.len());
            let tracks_to_render = &dep.tracks[start..end];
            for track in tracks_to_render {
                rows.push(render_track_row(dep, track, false, current_id));
            }
        }

        mouse_area(
            scrollable(column(rows).spacing(0))
                .id(scrollable::Id::new("tracklist_scroll"))
                .on_scroll(Message::TracklistScrolled)
        )
        .on_enter(Message::HoverTracklist(true))
        .on_exit(Message::HoverTracklist(false))
        .into()
    });





    let filter_options: Element<'_, Message> = if !state.search_query.is_empty() {
        container(
            row![
                checkbox("Title", state.filter_title).on_toggle(|_| Message::ToggleFilterTitle).size(14),
                checkbox("Artist", state.filter_artist).on_toggle(|_| Message::ToggleFilterArtist).size(14),
                checkbox("Album", state.filter_album).on_toggle(|_| Message::ToggleFilterAlbum).size(14),
                checkbox("Genre", state.filter_genre).on_toggle(|_| Message::ToggleFilterGenre).size(14),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        )
        .padding([4, 12])
        .into()
    } else {
        Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    };

    let headers: Element<'_, Message> = if state.view_mode == ViewMode::NowPlaying {
        let table_columns = get_responsive_columns(state);
        let mut header_widgets: Vec<Element<'_, Message>> = Vec::new();
        header_widgets.push(Space::with_width(Length::Fixed(28.0)).into());
        
        for col in table_columns {
            let label = col.label();
            let width = col_width(col);
            let sort_col = col_to_sort_col(col);
            let is_sorted = state.sort_column == Some(sort_col);
            let arrow = if is_sorted {
                if state.sort_ascending { " ▲" } else { " ▼" }
            } else {
                ""
            };
            let txt = text(format!("{label}{arrow}"))
                .size(11)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .color(if is_sorted { theme::accent() } else { theme::subtext() });
            let btn = button(txt)
                .on_press(Message::SortBy(sort_col))
                .style(iced::widget::button::text)
                .padding(0)
                .width(width);
            let header_area = mouse_area(btn)
                .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Header(col))));
            header_widgets.push(header_area.into());
        }
        header_widgets.push(Space::with_width(Length::Fixed(120.0)).into());

        container(
            row(header_widgets)
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([8, 12])
        )
        .style(theme::header)
        .width(Length::Fill)
        .into()
    } else if state.tracks.is_empty() {
        Space::with_height(0.0).into()
    } else {
        table_headers.into()
    };

    let content_area: Element<'_, Message> = if state.view_mode == ViewMode::NowPlaying {
        if state.queue.is_empty() {
            container(
                text("The play queue is empty.")
                    .color(theme::overlay0())
                    .size(15),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if state.tracks.is_empty() {
            container(
                text("No matching queue items found.")
                    .color(theme::overlay0())
                    .size(15),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            let current_track_id = state.current_track.as_ref().map(|t| t.id);
            let mut rows: Vec<Element<'_, Message>> = Vec::new();
            
            for (idx, track) in state.tracks.iter().enumerate() {
                let original_idx = state.queue.iter().position(|t| t.id == track.id).unwrap_or(idx);
                let is_current = current_track_id == Some(track.id);
                let is_selected_track = state.selected_tracks.iter().any(|t| t.id == track.id);
                let row_color = if is_current { theme::accent() } else { theme::text() };
                
                let drag_handle = container(
                    text("\u{f0c9}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(if state.dragging_queue_index == Some(original_idx) { theme::accent() } else { theme::overlay0() })
                        .size(12)
                )
                .padding([4, 8]);
                
                let drag_handle_widget: Element<'_, Message> = mouse_area(drag_handle)
                    .on_press(Message::QueueDragStart(original_idx))
                    .on_release(Message::QueueDragEnd)
                    .interaction(iced::mouse::Interaction::Grab)
                    .into();

                let up_btn: Element<'_, Message> = if original_idx > 0 {
                    button(
                        text("\u{f062}")
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(theme::overlay0())
                            .size(12)
                    )
                    .on_press(Message::MoveQueueTrackUp(original_idx))
                    .style(iced::widget::button::text)
                    .padding(2)
                    .into()
                } else {
                    Space::with_width(16.0).into()
                };

                let down_btn: Element<'_, Message> = if original_idx < state.queue.len() - 1 {
                    button(
                        text("\u{f063}")
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(theme::overlay0())
                            .size(12)
                    )
                    .on_press(Message::MoveQueueTrackDown(original_idx))
                    .style(iced::widget::button::text)
                    .padding(2)
                    .into()
                } else {
                    Space::with_width(16.0).into()
                };

                let remove_btn = button(
                    text("\u{f00d}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(theme::red())
                        .size(12)
                )
                .on_press(Message::RemoveQueueTrack(original_idx))
                .style(iced::widget::button::text)
                .padding(2);

                let controls = row![
                    up_btn,
                    down_btn,
                    remove_btn,
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fixed(120.0));

                let track_no = (original_idx + 1).to_string();
                let table_columns = get_responsive_columns(state);
                let mut row_widgets: Vec<Element<'_, Message>> = Vec::new();
                row_widgets.push(drag_handle_widget);
                
                for col in table_columns {
                    let width = col_width(col);
                    let el: Element<'_, Message> = match col {
                        crate::db::TableColumn::TrackNumber => {
                            text(track_no.clone()).color(theme::overlay0()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Title => {
                            text(track.title.clone()).color(row_color).size(14).width(width).into()
                        }
                        crate::db::TableColumn::Artist => {
                            text(track.artist.clone()).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Album => {
                            text(track.album.clone()).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Genre => {
                            text(track.genre.clone()).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Year => {
                            let yr_str = track.year.map(|y| y.to_string()).unwrap_or_else(|| "·".to_string());
                            text(yr_str).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::DiscNumber => {
                            let dc_str = track.disc_number.map(|d| d.to_string()).unwrap_or_else(|| "·".to_string());
                            text(dc_str).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Duration => {
                            text(track.duration_str()).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Plays => {
                            text(track.play_count.to_string()).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::DatePlayed => {
                            let dp_str = track.date_played.clone().unwrap_or_else(|| "·".to_string());
                            text(dp_str).color(theme::subtext()).size(13).width(width).into()
                        }
                        crate::db::TableColumn::Liked => {
                            let like_color = if track.liked { theme::red() } else { theme::overlay0() };
                            container(
                                button(
                                    text(crate::ui::icons::ICON_HEART)
                                        .font(crate::ui::icons::NERD_FONT_MONO)
                                        .color(like_color)
                                        .size(13)
                                )
                                .on_press(Message::ToggleLikeTrack(track.clone()))
                                .style(iced::widget::button::text)
                            )
                            .width(width)
                            .center_x(width)
                            .into()
                        }
                    };
                    row_widgets.push(el);
                }
                row_widgets.push(controls.into());

                let row_content = mouse_area(
                    container(
                        row(row_widgets)
                            .spacing(12)
                            .align_y(Alignment::Center)
                            .padding([6, 12])
                    )
                    .style(move |_| iced::widget::container::Style {
                        background: if is_selected_track {
                            Some(iced::Background::Color(theme::surface0()))
                        } else if is_current {
                            Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.15)))
                        } else if idx % 2 == 1 {
                            Some(iced::Background::Color(theme::mantle()))
                        } else {
                            None
                        },
                        border: if is_selected_track {
                            iced::Border {
                                color: theme::accent(),
                                width: 1.0,
                                radius: 4.0.into(),
                            }
                        } else {
                            iced::Border::default()
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
                )
                .on_press(Message::SelectQueueTrack(original_idx, track.clone()))
                .on_right_press(Message::ToggleContextMenu(Some(crate::app::ContextMenuTarget::Track(track.clone()))));

                let row_with_drag_over = if state.dragging_queue_index.is_some() {
                    mouse_area(row_content)
                        .on_enter(Message::QueueDragOver(original_idx))
                } else {
                    row_content
                };

                rows.push(row_with_drag_over.into());
            }

            container(scrollable(column(rows).spacing(0)))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::surface0())),
                    ..Default::default()
                })
                .into()
        }
    } else if state.tracks.is_empty() {
        container(
            text(if state.selected_folder.is_some() || state.selected_playlist.is_some() || !state.search_query.is_empty() {
                state.strings.no_tracks_found
            } else {
                state.strings.select_folder
            })
            .color(theme::overlay0())
            .size(15),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(tracklist_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    column![
        filter_options,
        headers,
        content_area,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn render_track_row(
    dep: &TrackListDependency,
    track: &crate::library::models::Track,
    grouped: bool,
    current_id: Option<i64>,
) -> Element<'static, Message> {
    let is_current = current_id == Some(track.id);
    let is_selected_track = dep.selected_tracks.iter().any(|t| t.id == track.id);
    let row_color = if is_current { theme::accent() } else { theme::text() };


    let mut track_no_cover = track.clone();
    track_no_cover.cover_data = None;

    let table_columns = &dep.responsive_columns;
    let mut track_row_widgets: Vec<Element<'static, Message>> = Vec::new();

    for &col in table_columns {
        let width = col_width(col);
        let el: Element<'static, Message> = match col {
            crate::db::TableColumn::TrackNumber => {
                let num_str = track.track_number.map(|n| n.to_string()).unwrap_or_else(|| "·".to_string());
                text(num_str).color(theme::overlay0()).size(13).width(width).into()
            }
            crate::db::TableColumn::Title => {
                text(track.title.clone()).color(row_color).size(14).width(width).into()
            }
            crate::db::TableColumn::Artist => {
                text(track.artist.clone()).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Album => {
                text(track.album.clone()).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Genre => {
                text(track.genre.clone()).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Year => {
                let yr_str = track.year.map(|y| y.to_string()).unwrap_or_else(|| "·".to_string());
                text(yr_str).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::DiscNumber => {
                let dc_str = track.disc_number.map(|d| d.to_string()).unwrap_or_else(|| "·".to_string());
                text(dc_str).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Duration => {
                text(track.duration_str()).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Plays => {
                text(track.play_count.to_string()).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::DatePlayed => {
                let dp_str = track.date_played.clone().unwrap_or_else(|| "·".to_string());
                text(dp_str).color(theme::subtext()).size(13).width(width).into()
            }
            crate::db::TableColumn::Liked => {
                let like_color = if track.liked { theme::red() } else { theme::overlay0() };
                container(
                    button(
                        text(crate::ui::icons::ICON_HEART)
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(like_color)
                            .size(13)
                    )
                    .on_press(Message::ToggleLikeTrack(track.clone()))
                    .style(iced::widget::button::text)
                )
                .width(width)
                .center_x(width)
                .into()
            }
        };
        track_row_widgets.push(el);
    }

    let track_row = row(track_row_widgets)
        .spacing(12)
        .align_y(Alignment::Center)
        .padding([5, 12]);

    let current_idx = dep.tracks.iter().position(|t| t.id == track.id);
    let prev_selected = current_idx
        .and_then(|idx| if idx > 0 { dep.tracks.get(idx - 1) } else { None })
        .map(|prev_t| {
            let same_album = !grouped || prev_t.album == track.album;
            same_album && dep.selected_tracks.iter().any(|t| t.id == prev_t.id)
        })
        .unwrap_or(false);
    let next_selected = current_idx
        .and_then(|idx| dep.tracks.get(idx + 1))
        .map(|next_t| {
            let same_album = !grouped || next_t.album == track.album;
            same_album && dep.selected_tracks.iter().any(|t| t.id == next_t.id)
        })
        .unwrap_or(false);

    let styled = if is_selected_track {
        let radius = iced::border::Radius {
            top_left: if prev_selected { 0.0 } else { 4.0 },
            top_right: if prev_selected { 0.0 } else { 4.0 },
            bottom_left: if next_selected { 0.0 } else { 4.0 },
            bottom_right: if next_selected { 0.0 } else { 4.0 },
        };

        // Base container with background
        let content_container = container(track_row)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::surface0())),
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius,
                },
                ..Default::default()
            });

        // Left border overlay
        let left_border_overlay = container(
            container(Space::new(Length::Fixed(1.0), Length::Fill))
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::accent())),
                    ..Default::default()
                })
                .width(1.0)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 1.0 })
        .align_x(iced::alignment::Horizontal::Left);

        // Right border overlay
        let right_border_overlay = container(
            container(Space::new(Length::Fixed(1.0), Length::Fill))
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::accent())),
                    ..Default::default()
                })
                .width(1.0)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding { top: 0.0, right: 1.0, bottom: 0.0, left: 0.0 })
        .align_x(iced::alignment::Horizontal::Right);

        let mut s = stack![
            content_container,
            left_border_overlay,
            right_border_overlay,
        ]
        .width(Length::Fill)
        .height(Length::Shrink);

        if !prev_selected {
            let top_border = container(
                container(Space::new(Length::Fill, Length::Fixed(1.0)))
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::accent())),
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .height(1.0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding { top: 1.0, right: 0.0, bottom: 0.0, left: 0.0 })
            .align_y(iced::alignment::Vertical::Top);
            
            s = s.push(top_border);
        }

        if !next_selected {
            let bottom_border = container(
                container(Space::new(Length::Fill, Length::Fixed(1.0)))
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(theme::accent())),
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .height(1.0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 2.0, left: 0.0 })
            .align_y(iced::alignment::Vertical::Bottom);
            
            s = s.push(bottom_border);
        }

        container(s).width(Length::Fill)
    } else if is_current {
        container(track_row).style(theme::selected_row).width(Length::Fill)
    } else {
        container(track_row).width(Length::Fill)
    };

    let select_btn = button(styled)
        .on_press(Message::SelectTrack(track.clone()))
        .style(iced::widget::button::text)
        .width(Length::Fill)
        .padding(0);

    let row_target = if dep.selected_tracks.len() > 1 && dep.selected_tracks.iter().any(|t| t.id == track.id) {
        crate::app::ContextMenuTarget::MultipleTracks(dep.selected_tracks.clone())
    } else {
        crate::app::ContextMenuTarget::Track(track_no_cover)
    };

    mouse_area(select_btn)
        .on_right_press(Message::ToggleContextMenu(Some(row_target)))
        .into()
}

fn col_width(col: crate::db::TableColumn) -> Length {
    match col {
        crate::db::TableColumn::TrackNumber => Length::Fixed(30.0),
        crate::db::TableColumn::Title => Length::FillPortion(3),
        crate::db::TableColumn::Artist => Length::FillPortion(2),
        crate::db::TableColumn::Album => Length::FillPortion(2),
        crate::db::TableColumn::Genre => Length::FillPortion(2),
        crate::db::TableColumn::Year => Length::Fixed(50.0),
        crate::db::TableColumn::DiscNumber => Length::Fixed(50.0),
        crate::db::TableColumn::Duration => Length::Fixed(80.0),
        crate::db::TableColumn::Plays => Length::Fixed(40.0),
        crate::db::TableColumn::DatePlayed => Length::FillPortion(2),
        crate::db::TableColumn::Liked => Length::Fixed(40.0),
    }
}

fn col_to_sort_col(col: crate::db::TableColumn) -> SortColumn {
    match col {
        crate::db::TableColumn::TrackNumber => SortColumn::TrackNumber,
        crate::db::TableColumn::Title => SortColumn::Title,
        crate::db::TableColumn::Artist => SortColumn::Artist,
        crate::db::TableColumn::Album => SortColumn::Album,
        crate::db::TableColumn::Genre => SortColumn::Genre,
        crate::db::TableColumn::Year => SortColumn::Year,
        crate::db::TableColumn::DiscNumber => SortColumn::DiscNumber,
        crate::db::TableColumn::Duration => SortColumn::Duration,
        crate::db::TableColumn::Plays => SortColumn::Plays,
        crate::db::TableColumn::DatePlayed => SortColumn::DatePlayed,
        crate::db::TableColumn::Liked => SortColumn::Liked,
    }
}

pub fn library_top_bar(state: &AppState) -> Element<'_, Message> {
    let total_width = state.sidebar_width.round() - 16.0;
    let tab_width_1 = (total_width / 3.0).floor();
    let tab_width_2 = (total_width / 3.0).floor();
    let tab_width_3 = total_width - tab_width_1 - tab_width_2;

    let tab_btn = |mode: ViewMode, icon: &'static str, label: &'static str, width: f32| {
        let is_active = state.view_mode == mode && state.selected_playlist.is_none();
        let btn_icon = text(icon)
            .size(18)
            .font(crate::ui::icons::NERD_FONT_MONO);
        
        let btn = button(container(btn_icon).center_x(Length::Fill).center_y(Length::Fill))
            .on_press(Message::SelectViewMode(mode))
            .width(width)
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
                        color: if is_active { theme::accent() } else { theme::surface0() },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: if is_active { theme::accent() } else { theme::subtext() },
                    ..Default::default()
                }
            })
            .padding(0);

        let tooltip_content = container(
            text(label)
                .size(11)
                .font(crate::ui::icons::UI_FONT)
                .color(theme::text())
        )
        .padding([4, 8])
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            border: iced::Border {
                color: theme::overlay0(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        tooltip(btn, tooltip_content, tooltip::Position::Top)
    };

    let left_tabs = row![
        tab_btn(ViewMode::Artists, crate::ui::icons::ICON_PERSON, "Artists", tab_width_1),
        tab_btn(ViewMode::Albums, crate::ui::icons::ICON_CD, "Albums", tab_width_2),
        tab_btn(ViewMode::Genres, crate::ui::icons::ICON_TAG, "Genres", tab_width_3),
    ]
    .spacing(0)
    .align_y(Alignment::Center);

    let left_tabs_container = container(left_tabs)
        .width(state.sidebar_width.round())
        .padding([0, 8])
        .height(28.0);

    let is_now_playing_active = state.view_mode == ViewMode::NowPlaying;

    // Calculate contrast-compliant text colors
    let light_text = theme::text();
    let dark_text = theme::base();
    let active_text_color = if theme::contrast_ratio(theme::accent(), light_text) > theme::contrast_ratio(theme::accent(), dark_text) {
        light_text
    } else {
        dark_text
    };

    let text_color_main = if is_now_playing_active { active_text_color } else { theme::text() };
    let text_color_sub = if is_now_playing_active { active_text_color } else { theme::subtext() };

    let mut now_playing_row = row![].spacing(6).align_y(Alignment::Center);

    // Hide equalizer when stopped/idle (or no track loaded)
    let show_eq = state.current_track.is_some() && (matches!(state.playback_state, crate::audio::PlaybackState::Playing) || matches!(state.playback_state, crate::audio::PlaybackState::Paused));
    let is_playing = matches!(state.playback_state, crate::audio::PlaybackState::Playing);
    
    if show_eq {
        let (h1, h2, h3) = if is_playing {
            let tick = state.animation_tick;
            (
                ((tick as f32 * 0.15).sin() * 0.5 + 0.5) * 8.0 + 2.0,
                ((tick as f32 * 0.25).sin() * 0.5 + 0.5) * 8.0 + 2.0,
                ((tick as f32 * 0.1).sin() * 0.5 + 0.5) * 8.0 + 2.0,
            )
        } else {
            (2.0, 2.0, 2.0)
        };

        let bar = |h: f32| {
            container(Space::new(Length::Fixed(2.0), Length::Fixed(h)))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(text_color_main)),
                    ..Default::default()
                })
        };

        let eq = row![bar(h1), bar(h2), bar(h3)]
            .spacing(2)
            .align_y(Alignment::End)
            .height(10.0);

        now_playing_row = now_playing_row.push(eq);
    }

    let is_playing_or_paused = state.current_track.is_some() && !matches!(state.playback_state, crate::audio::PlaybackState::Stopped);
    let now_playing_font = if is_playing_or_paused {
        crate::ui::icons::UI_FONT_BOLD
    } else {
        crate::ui::icons::UI_FONT
    };

    now_playing_row = now_playing_row.push(
        text("Now Playing")
            .size(13)
            .font(now_playing_font)
            .color(text_color_main)
    );

    if let Some(ref ctx) = state.playing_context {
        let context_name = match ctx {
            crate::app::PlayingContext::Playlist(name) => name.clone(),
            crate::app::PlayingContext::SmartPlaylist(name) => name.clone(),
            crate::app::PlayingContext::Artist(name) => name.clone(),
            crate::app::PlayingContext::Album(name) => name.clone(),
            crate::app::PlayingContext::Autoplaylist(name) => name.clone(),
            crate::app::PlayingContext::Genre(name) => name.clone(),
        };

        let max_context_width = if state.window_width < crate::app::MIN_NON_DRAWER_WIDTH {
            0.0
        } else if state.window_width < crate::app::MIN_NON_DRAWER_WIDTH + 150.0 {
            120.0
        } else {
            200.0
        };

        let max_len = (max_context_width / 8.0) as usize;
        let display_name = if context_name.chars().count() > max_len && max_len > 3 {
            let truncated: String = context_name.chars().take(max_len - 3).collect();
            format!("{}...", truncated)
        } else {
            context_name.clone()
        };

        if max_context_width > 0.0 {
            now_playing_row = now_playing_row
                .push(
                    text(" · ")
                        .size(13)
                        .color(text_color_sub)
                )
                .push(
                    text(display_name)
                        .size(13)
                        .color(text_color_sub)
                );
        }
    }

    let now_playing_tab = button(container(now_playing_row).center_y(Length::Fill).padding([0, 12]))
        .on_press(Message::SelectViewMode(ViewMode::NowPlaying))
        .height(28)
        .style(move |theme: &iced::Theme, status: iced::widget::button::Status| {
            let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
            iced::widget::button::Style {
                background: Some(iced::Background::Color(if is_now_playing_active {
                    theme::accent()
                } else if is_hovered {
                    theme::surface0()
                } else {
                    iced::Color::TRANSPARENT
                })),
                border: iced::Border {
                    color: if is_now_playing_active { theme::accent() } else { theme::surface0() },
                    width: 1.0,
                    radius: iced::border::Radius {
                        top_left: 4.0,
                        top_right: 4.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                },
                text_color: if is_now_playing_active { theme::base() } else { theme::subtext() },
                ..Default::default()
            }
        })
        .padding(0);

    let song_clear_btn: Element<'_, Message> = if !state.search_query.is_empty() {
        button(
            text("\u{f00d}")
                .font(crate::ui::icons::NERD_FONT_MONO)
                .color(theme::red())
                .size(12)
        )
        .on_press(Message::SearchChanged(String::new()))
        .style(iced::widget::button::text)
        .padding(4)
        .into()
    } else {
        Space::with_width(0.0).into()
    };

    let search_placeholder = if state.view_mode == ViewMode::NowPlaying {
        "Search queue..."
    } else {
        "Search songs..."
    };

    let song_search_input = row![
        text_input(search_placeholder, &state.search_query)
            .id(iced::widget::text_input::Id::new("song_search_input"))
            .on_input(Message::SearchChanged)
            .padding(4)
            .size(11)
            .width(Length::Fill),
        song_clear_btn
    ]
    .align_y(Alignment::Center)
    .spacing(4)
    .width(Length::Fixed(200.0));

    let is_settings_active = state.show_settings.is_some();
    let settings_icon = text("\u{f013}")
        .size(16)
        .font(crate::ui::icons::NERD_FONT_MONO);

    let settings_top_sep = container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(1.0);

    let settings_btn_content = column![
        settings_top_sep,
        container(settings_icon)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .height(Length::Fixed(27.0))
    ]
    .spacing(0);

    let settings_btn = button(settings_btn_content)
        .on_press(Message::OpenSettings)
        .width(56.0)
        .height(28.0)
        .style(move |theme: &iced::Theme, status: iced::widget::button::Status| {
            let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
            iced::widget::button::Style {
                background: Some(iced::Background::Color(if is_settings_active {
                    theme::surface0()
                } else if is_hovered {
                    theme::surface0()
                } else {
                    iced::Color::TRANSPARENT
                })),
                text_color: if is_settings_active {
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
        .padding(0);

    let clear_queue_btn: Element<'_, Message> = if state.view_mode == ViewMode::NowPlaying {
        button(text("Clear Queue").size(11))
            .on_press(Message::ClearQueue)
            .style(move |theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    text_color: theme::red(),
                    background: Some(iced::Background::Color(if is_hovered { theme::surface0() } else { iced::Color::TRANSPARENT })),
                    border: iced::Border {
                        color: theme::red(),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 8])
            .into()
    } else {
        Space::with_width(0.0).into()
    };

    let group_by_album_checkbox = if state.view_mode != ViewMode::NowPlaying {
        Element::from(
            row![
                checkbox("Group by Album", state.group_by_album)
                    .on_toggle(|_| Message::ToggleGroupByAlbum)
                    .size(14),
                Space::with_width(12),
            ]
            .align_y(Alignment::Center)
        )
    } else {
        Space::with_width(0.0).into()
    };

    let clear_queue_spacer: Element<'_, Message> = if state.view_mode == ViewMode::NowPlaying {
        row![Space::with_width(12), clear_queue_btn].into()
    } else {
        Space::with_width(0.0).into()
    };

    let right_controls = row![
        group_by_album_checkbox,
        song_search_input,
        clear_queue_spacer
    ]
    .align_y(Alignment::Center)
    .padding(iced::Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 0.0 });

    let settings_separator = container(Space::new(Length::Fixed(1.0), Length::Fill))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::surface0())),
            ..Default::default()
        })
        .width(1.0)
        .height(Length::Fixed(28.0));

    let right_bar = row![
        now_playing_tab,
        Space::with_width(Length::Fill),
        right_controls,
        settings_separator,
        settings_btn
    ]
    .spacing(0)
    .align_y(Alignment::End)
    .height(28.0)
    .width(Length::Fill);

    container(
        row![
            left_tabs_container,
            Space::with_width(6.0),
            right_bar
        ]
        .spacing(0)
        .width(Length::Fill)
        .align_y(Alignment::End)
    )
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::mantle())),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(28.0)
    .into()
}
