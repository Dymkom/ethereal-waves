// SPDX-License-Identifier: GPL-3.0

use crate::app::{AppModel, MenuAction, Message};
use crate::constants::MENU_CONDENSED_THRESHOLD;
use crate::fl;
use crate::playback_state::RepeatMode;
use cosmic::{
    Apply, Element,
    iced::Length,
    widget::{self, menu},
};

pub fn menu_bar<'a>(app: &AppModel) -> Element<'a, Message> {
    let is_condensed = app.state.window_width < MENU_CONDENSED_THRESHOLD;

    let has_playlist = app.view_playlist.is_some();

    let repeat_one = if app.state.repeat_mode == RepeatMode::One {
        true
    } else {
        false
    };

    let repeat_all = if app.state.repeat_mode == RepeatMode::All {
        true
    } else {
        false
    };

    let file_items = vec![menu::Item::Button(fl!("quit"), None, MenuAction::Quit)];

    let selected_playlist = match app.view_playlist {
        Some(id) => match app.playlist_service.get(id) {
            Ok(playlist) => playlist,
            Err(_) => {
                // If we can't get the playlist, return a minimal menu
                return if is_condensed {
                    build_menu(
                        vec![menu::Tree::with_children(
                            menu::menu_button(vec![
                                widget::icon::from_name("open-menu-symbolic").apply(Element::from),
                            ])
                            .apply(Element::from),
                            menu::items(
                                &app.key_binds,
                                vec![menu::Item::Folder(fl!("file"), file_items)],
                            ),
                        )],
                        Length::Shrink,
                    )
                } else {
                    build_menu(
                        vec![menu::Tree::with_children(
                            menu::root(fl!("file")).apply(Element::from),
                            menu::items(&app.key_binds, file_items),
                        )],
                        Length::Fill,
                    )
                };
            }
        },
        None => {
            // If we can't get the playlist, return a minimal menu
            return if is_condensed {
                build_menu(
                    vec![menu::Tree::with_children(
                        menu::menu_button(vec![
                            widget::icon::from_name("open-menu-symbolic").apply(Element::from),
                        ])
                        .apply(Element::from),
                        menu::items(
                            &app.key_binds,
                            vec![menu::Item::Folder(fl!("file"), file_items)],
                        ),
                    )],
                    Length::Shrink,
                )
            } else {
                build_menu(
                    vec![menu::Tree::with_children(
                        menu::root(fl!("file")).apply(Element::from),
                        menu::items(&app.key_binds, file_items),
                    )],
                    Length::Fill,
                )
            };
        }
    };

    let mut selected_playlist_list = Vec::new();
    let mut now_playing_playlist_list = Vec::new();

    let selected_count: usize = if app.view_playlist.is_some() {
        app.playlist_service
            .get(app.view_playlist.unwrap())
            .map(|p| p.selected_iter().count())
            .unwrap_or(0)
    } else {
        0
    };

    // Add ordered playlists
    app.state.playlist_nav_order.iter().for_each(|p| {
        if let Ok(playlist) = app.playlist_service.get(*p) {
            selected_playlist_list.push(menu::Item::Button(
                playlist.name().to_string(),
                None,
                MenuAction::AddSelectedToPlaylist(playlist.id()),
            ));
            if app.playback_service.now_playing().is_some() {
                now_playing_playlist_list.push(menu::Item::Button(
                    playlist.name().to_string(),
                    None,
                    MenuAction::AddNowPlayingToPlaylist(playlist.id()),
                ));
            }
        }
    });
    // Add unordered playlists
    app.playlist_service
        .user_playlists()
        .filter(|p| !app.state.playlist_nav_order.contains(&p.id()))
        .for_each(|p| {
            selected_playlist_list.push(menu::Item::Button(
                p.name().to_string(),
                None,
                MenuAction::AddSelectedToPlaylist(p.id()),
            ));
            if app.playback_service.now_playing().is_some() {
                now_playing_playlist_list.push(menu::Item::Button(
                    p.name().to_string(),
                    None,
                    MenuAction::AddNowPlayingToPlaylist(p.id()),
                ));
            }
        });

    let file_items = vec![
        menu_button_optional(
            fl!("track-info"),
            MenuAction::TrackInfoPanel,
            selected_count > 0,
        ),
        menu::Item::Divider,
        menu_button_optional(
            fl!("update-library"),
            MenuAction::UpdateLibrary,
            !app.is_updating,
        ),
        menu::Item::Divider,
        menu::Item::Button(fl!("quit"), None, MenuAction::Quit),
    ];

    let playlist_items = vec![
        menu::Item::Button(fl!("new-playlist-menu"), None, MenuAction::NewPlaylist),
        menu_button_optional(
            fl!("rename-playlist-menu"),
            MenuAction::RenamePlaylist,
            !selected_playlist.is_library(),
        ),
        menu_button_optional(
            fl!("delete-playlist-menu"),
            MenuAction::DeletePlaylist,
            !selected_playlist.is_library(),
        ),
        menu::Item::Divider,
        menu::Item::Folder(fl!("add-selected-to"), selected_playlist_list),
        menu_button_optional(
            fl!("remove-selected"),
            MenuAction::RemoveSelectedFromPlaylist,
            has_playlist && !selected_playlist.is_library(),
        ),
        menu::Item::Divider,
        menu::Item::Folder(fl!("add-now-playing-to"), now_playing_playlist_list),
        menu::Item::Divider,
        menu::Item::Button(fl!("select-all"), None, MenuAction::SelectAll),
        menu::Item::Divider,
        menu_button_optional(fl!("move-up"), MenuAction::MoveNavUp, has_playlist),
        menu_button_optional(fl!("move-down"), MenuAction::MoveNavDown, has_playlist),
    ];

    let playback_items = vec![
        menu::Item::CheckBox(
            fl!("shuffle"),
            None,
            app.state.shuffle,
            MenuAction::ToggleShuffle,
        ),
        menu::Item::CheckBox(
            fl!("repeat"),
            None,
            app.state.repeat,
            MenuAction::ToggleRepeat,
        ),
        menu::Item::Divider,
        menu::Item::CheckBox(
            fl!("repeat-one"),
            None,
            repeat_one,
            MenuAction::ToggleRepeatMode,
        ),
        menu::Item::CheckBox(
            fl!("repeat-all"),
            None,
            repeat_all,
            MenuAction::ToggleRepeatMode,
        ),
    ];

    let view_items = vec![
        menu::Item::Button(fl!("zoom-in"), None, MenuAction::ZoomIn),
        menu::Item::Button(fl!("zoom-out"), None, MenuAction::ZoomOut),
        menu::Item::Divider,
        menu::Item::Button(fl!("settings-menu"), None, MenuAction::Settings),
        menu::Item::Divider,
        menu::Item::Button(fl!("about-ethereal-waves"), None, MenuAction::About),
    ];

    if is_condensed {
        return build_menu(
            vec![menu::Tree::with_children(
                menu::menu_button(vec![
                    widget::icon::from_name("open-menu-symbolic").apply(Element::from),
                ])
                .apply(Element::from),
                menu::items(
                    &app.key_binds,
                    vec![
                        menu::Item::Folder(fl!("file"), file_items),
                        menu::Item::Folder(fl!("playlist"), playlist_items),
                        menu::Item::Folder(fl!("playback"), playback_items),
                        menu::Item::Folder(fl!("view"), view_items),
                    ],
                ),
            )],
            Length::Shrink,
        );
    }

    build_menu(
        vec![
            menu::Tree::with_children(
                menu::root(fl!("file")).apply(Element::from),
                menu::items(&app.key_binds, file_items),
            ),
            menu::Tree::with_children(
                menu::root(fl!("playlist")).apply(Element::from),
                menu::items(&app.key_binds, playlist_items),
            ),
            menu::Tree::with_children(
                menu::root(fl!("playback")).apply(Element::from),
                menu::items(&app.key_binds, playback_items),
            ),
            menu::Tree::with_children(
                menu::root(fl!("view")).apply(Element::from),
                menu::items(&app.key_binds, view_items),
            ),
        ],
        Length::Fill,
    )
}

fn build_menu<'a>(roots: Vec<menu::Tree<Message>>, width: Length) -> Element<'a, Message> {
    menu::bar(roots)
        .item_width(menu::ItemWidth::Uniform(250))
        .item_height(menu::ItemHeight::Dynamic(40))
        .spacing(1.0)
        .width(width)
        .into()
}

const fn menu_button_optional(
    label: String,
    action: MenuAction,
    enabled: bool,
) -> menu::Item<MenuAction, String> {
    if enabled {
        menu::Item::Button(label, None, action)
    } else {
        menu::Item::ButtonDisabled(label, None, action)
    }
}
