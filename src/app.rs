use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{button, container, column, row, text, Space, stack, scrollable};
use iced::{Alignment, Element, Length, Subscription, Task, Theme};
use mpris_server::{LoopStatus, PlaybackStatus};

use crate::audio::{AudioCommand, AudioEvent, AudioPlayer, MprisCommand, MprisUpdate, PlaybackState};
use crate::audio::mpris;
use crate::audio::spectrum::SpectrumAnalyzer;
use crate::library::models::Track;
use crate::library::{load_cover, scan_folder};
use crate::ui::{theme, views};

#[derive(Debug, Clone)]
pub enum ContextMenuTarget {
    Artist(String),
    Album(String),
    Track(Track),
    MultipleTracks(Vec<Track>),
    Header(crate::db::TableColumn),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Artists,
    Albums,
    Genres,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab {
    Visualizer,
    Lyrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveFocus {
    SidebarSearch,
    SongSearch,
    SidebarList,
    Tracklist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortColumn {
    TrackNumber,
    Title,
    Artist,
    Album,
    Genre,
    Year,
    DiscNumber,
    Duration,
    Plays,
    DatePlayed,
}

#[derive(Debug, Clone)]
pub enum PlaylistDialogMode {
    Create,
    AddTrack(Track),
    Rename(String),
}

#[derive(Debug, Clone)]
pub struct PlaylistDialogState {
    pub mode: PlaylistDialogMode,
    pub name_input: String,
    pub selected_playlist: Option<String>,
    pub add_album: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectFolder(PathBuf),
    FolderScanned(PathBuf, Vec<Track>),

    PlayTrack(Track),
    PlayTracks(Vec<Track>),
    PlayPause,
    NextTrack,
    PreviousTrack,
    Seek(Duration),
    VolumeChanged(f32),
    HoverAlbumHeader(Option<String>),
    IncreaseScale,
    DecreaseScale,
    ToggleShuffle,
    ToggleRepeat,
    SeekRelative(i64),
    VolumeStep(f32),

    SidebarDragStart,
    SidebarDragMove(f32),
    SidebarDragEnd,

    PlaylistDragStart,
    PlaylistDragMove(f32),
    PlaylistDragEnd,

    RightPanelDragStart,
    RightPanelDragMove(f32),
    RightPanelDragEnd,

    SeekToLyric(Duration),

    PollAudio,
    PollSpectrum,
    CheckTheme,

    // Omatunes feature additions
    SearchChanged(String),
    ToggleFilterTitle,
    ToggleFilterArtist,
    ToggleFilterAlbum,
    ToggleFilterGenre,
    ToggleLikeTrack(Track),
    AddToPlaylist(String, Track),
    CreatePlaylist(String),
    SelectPlaylist(String),
    OpenTagEditor(Vec<Track>),
    CloseTagEditor,
    SearchCoverOnline,
    UpdateTagFieldTitle(String),
    UpdateTagFieldArtist(String),
    UpdateTagFieldAlbum(String),
    UpdateTagFieldGenre(String),
    UpdateTagFieldTrackNumber(String),
    UpdateTagFieldDiscNumber(String),
    UpdateTagFieldCoverPath(String),
    UpdateTagFieldApplyToAlbum(bool),
    UpdateTagFieldYear(String),
    ToggleTagFieldApplyTitle(bool),
    ToggleTagFieldApplyArtist(bool),
    ToggleTagFieldApplyAlbum(bool),
    ToggleTagFieldApplyYear(bool),
    ToggleTagFieldApplyGenre(bool),
    ToggleTagFieldApplyTrackNum(bool),
    ToggleTagFieldApplyDiscNum(bool),
    ToggleTagFieldApplyCover(bool),
    SelectTagEditorTab(TagEditorTab),
    UpdateTagFieldLyrics(iced::widget::text_editor::Action),
    ToggleTagFieldApplyLyrics(bool),
    SearchLyricsOnline,
    SaveTags,
    LibraryScanned(Vec<Track>),
    RescanLibrary,
    KeyboardLike,
    KeyboardEdit,
    KeyboardAdd,
    OpenLocalFolder(std::path::PathBuf),

    // Omatunes enhancements
    SelectViewMode(ViewMode),
    SelectArtist(String),
    SelectAlbum(String),
    SelectAllArtists,
    SelectAllAlbums,
    SelectAllGenres,
    SortBy(SortColumn),
    OpenPlaylistDialog(PlaylistDialogMode),
    ClosePlaylistDialog,
    PlaylistInputChanged(String),
    PlaylistDialogSelect(String),
    PlaylistDialogToggleAddAlbum(bool),
    PlaylistDialogSubmit,
    WindowResized(f32, f32),
    HoverTracklist(bool),
    HoverSidebarList(bool),
    KeyboardArrowUp,
    KeyboardArrowDown,
    DeletePlaylist(String),
    RenamePlaylist(String, String),
    ToggleGroupByAlbum,
    SelectTrack(Track),
    SidebarSearchChanged(String),
    OpenShortcuts,
    CloseShortcuts,
    KeyPressed(iced::keyboard::Key),

    DoubleClickTrack(Track),
    DoubleClickArtist(String),
    DoubleClickAlbum(String),
    DoubleClickPlaylist(String),
    ReturnToActiveSource,
    FocusSongName,
    FocusArtistName,
    FocusAlbumName,

    SelectGenre(String),
    DoubleClickGenre(String),
    HoverPlaylist(Option<String>),
    ToggleContextMenu(Option<ContextMenuTarget>),
    HideAlbumOrArtist(String, bool),            // (Name, IsArtistOrAlbum)
    AddAlbumToPlaylist(String, String),         // (AlbumName, PlaylistName)

    HoverSidebarResizer(bool),
    HoverPlaylistResizer(bool),
    HoverRightPanelResizer(bool),
    RestoreHiddenItems,
    CreatePlaylistFromContext(String, bool),
    ModifiersChanged(iced::keyboard::Modifiers),
    AddTracksToPlaylist(String, Vec<Track>),
    CreatePlaylistWithTracks(String, Vec<Track>),
    ToggleColumnVisibility(crate::db::TableColumn),
    MoveColumnLeft(crate::db::TableColumn),
    MoveColumnRight(crate::db::TableColumn),
    SelectPlaylistTab(PlaylistTab),
    ToggleRightPanelTab(RightPanelTab),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistTab {
    Playlists,
    Autoplaylists,
}

// ── Estado global ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagEditorTab {
    Main,
    Lyrics,
}

#[derive(Debug)]
pub struct TagEditorState {
    pub tracks: Vec<Track>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub track_number: String,
    pub disc_number: String,
    pub cover_path: Option<String>,
    pub apply_to_album: bool,
    pub year: String,
    pub apply_title: bool,
    pub apply_artist: bool,
    pub apply_album: bool,
    pub apply_year: bool,
    pub apply_genre: bool,
    pub apply_track_num: bool,
    pub apply_disc_num: bool,
    pub apply_cover: bool,
    pub apply_lyrics: bool,
    pub lyrics: String,
    pub lyrics_content: iced::widget::text_editor::Content,
    pub active_tab: TagEditorTab,
    pub focused_field: Option<usize>,
}

impl Clone for TagEditorState {
    fn clone(&self) -> Self {
        TagEditorState {
            tracks: self.tracks.clone(),
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            genre: self.genre.clone(),
            track_number: self.track_number.clone(),
            disc_number: self.disc_number.clone(),
            cover_path: self.cover_path.clone(),
            apply_to_album: self.apply_to_album,
            year: self.year.clone(),
            apply_title: self.apply_title,
            apply_artist: self.apply_artist,
            apply_album: self.apply_album,
            apply_year: self.apply_year,
            apply_genre: self.apply_genre,
            apply_track_num: self.apply_track_num,
            apply_disc_num: self.apply_disc_num,
            apply_cover: self.apply_cover,
            apply_lyrics: self.apply_lyrics,
            lyrics: self.lyrics.clone(),
            lyrics_content: iced::widget::text_editor::Content::with_text(&self.lyrics_content.text()),
            active_tab: self.active_tab,
            focused_field: self.focused_field,
        }
    }
}


pub struct CoverCache {
    pub id: Option<i64>,
    pub handle: Option<iced::widget::image::Handle>,
}

pub struct AppState {
    pub playback_state: PlaybackState,
    pub current_track: Option<Track>,
    pub queue: Vec<Track>,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: bool,

    pub folders: Vec<PathBuf>,
    pub selected_folder: Option<PathBuf>,
    pub tracks: Vec<Track>,
    folder_cache: HashMap<PathBuf, Vec<Track>>,

    pub sidebar_width: f32,
    pub dragging_sidebar: bool,

    pub right_panel_width: f32,
    pub dragging_right_panel: bool,
    pub is_hovering_right_panel_resizer: bool,
    pub window_width: f32,

    pub iced_theme: iced::Theme,
    loaded_theme_name: String,

    pub strings: &'static crate::locale::Strings,

    // Omatunes feature additions
    pub all_tracks: Vec<Track>,
    pub search_query: String,
    pub filter_title: bool,
    pub filter_artist: bool,
    pub filter_album: bool,
    pub filter_genre: bool,
    pub selected_playlist: Option<String>,
    pub show_tag_editor: Option<TagEditorState>,

    // Omatunes enhancements
    pub view_mode: ViewMode,
    pub selected_artist: Option<String>,
    pub selected_album: Option<String>,
    pub selected_genre: Option<String>,
    pub playlist_height: f32,
    pub playlist_height_initialized: bool,
    pub dragging_playlist_split: bool,
    pub active_focus: Option<ActiveFocus>,
    pub window_height: f32,
    pub sort_column: Option<SortColumn>,
    pub sort_ascending: bool,
    pub playlist_dialog: Option<PlaylistDialogState>,
    pub current_track_play_counted: bool,

    pub selected_track: Option<Track>,
    pub is_hovering_tracklist: bool,
    pub is_hovering_sidebar_list: bool,
    pub is_hovering_sidebar_resizer: bool,
    pub is_hovering_playlist_resizer: bool,
    pub group_by_album: bool,
    pub sidebar_search: String,
    pub show_shortcuts: bool,

    pub last_click_track: Option<(i64, std::time::Instant)>,
    pub last_click_artist: Option<(String, std::time::Instant)>,
    pub last_click_album: Option<(String, std::time::Instant)>,
    pub last_click_playlist: Option<(String, std::time::Instant)>,
    pub last_click_genre: Option<(String, std::time::Instant)>,

    pub hovered_playlist: Option<String>,
    pub show_context_menu: Option<ContextMenuTarget>,
    pub modifiers: iced::keyboard::Modifiers,
    pub selected_tracks: Vec<Track>,
    pub last_clicked_track: Option<Track>,
    pub hidden_artists_albums: Vec<(String, bool)>,       // (Name, IsArtistOrAlbum)

    pub playlist_tab: PlaylistTab,
    pub right_panel_tab: Option<RightPanelTab>,
    pub right_panel_tab_user_scrolled: bool,
    pub lyrics_scroll_id: scrollable::Id,
    pub last_active_lyric_idx: Option<usize>,
    pub spectrum_bands: [f32; crate::audio::spectrum::NUM_BANDS],
    spectrum_analyzer: SpectrumAnalyzer,
    audio: AudioPlayer,
    mpris_cmd_rx: tokio::sync::mpsc::UnboundedReceiver<MprisCommand>,
    mpris_update_tx: tokio::sync::mpsc::UnboundedSender<MprisUpdate>,
    pub cover_cache: std::sync::Mutex<CoverCache>,
    pub font_scale: f32,
    pub hovered_album_header: Option<String>,
}

impl AppState {
    pub fn get_display_cover(&self) -> Option<iced::widget::image::Handle> {
        let is_playing_or_paused = !matches!(self.playback_state, PlaybackState::Stopped);
        let display_track = if is_playing_or_paused {
            self.current_track.as_ref()
        } else {
            self.selected_track.as_ref()
        };
        let track_id = display_track.map(|t| t.id);
        
        let mut cache = self.cover_cache.lock().unwrap();
        if track_id != cache.id {
            cache.id = track_id;
            cache.handle = display_track
                .and_then(|t| t.cover_data.as_ref())
                .map(|data| iced::widget::image::Handle::from_bytes(data.clone()));
        }
        cache.handle.clone()
    }

    fn new() -> (Self, Task<Message>) {
        let audio = AudioPlayer::spawn();
        let spectrum_analyzer = SpectrumAnalyzer::new(audio.sample_buffer.clone());

        let cfg = crate::config::get();
        let folders = music_subfolders(&cfg.music_path());

        let (mpris_cmd_tx, mpris_cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (mpris_update_tx, mpris_update_rx) = tokio::sync::mpsc::unbounded_channel();
        mpris::launch(mpris_cmd_tx, mpris_update_rx);

        let loaded_theme_name = crate::ui::theme::read_current_theme_name();
        let iced_theme = build_iced_theme();

        let music_dir = cfg.music_path();
        let scan_task = Task::perform(
            async move {
                scan_folder(&music_dir)
            },
            Message::LibraryScanned,
        );

        let state = AppState {
            playback_state: PlaybackState::Stopped,
            current_track: None,
            queue: Vec::new(),
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: cfg.volume.clamp(0.0, 1.0),
            shuffle: cfg.shuffle,
            repeat: cfg.repeat,
            folders,
            selected_folder: None,
            tracks: Vec::new(),
            folder_cache: HashMap::new(),
            sidebar_width: load_sidebar_width(),
            dragging_sidebar: false,
            right_panel_width: load_right_panel_width(),
            dragging_right_panel: false,
            is_hovering_right_panel_resizer: false,
            window_width: 960.0,
            iced_theme,
            loaded_theme_name,
            strings: crate::locale::get(),
            all_tracks: Vec::new(),
            search_query: String::new(),
            filter_title: true,
            filter_artist: true,
            filter_album: true,
            filter_genre: true,
            selected_playlist: None,
            show_tag_editor: None,
            view_mode: ViewMode::Artists,
            selected_artist: None,
            selected_album: None,
            selected_genre: None,
            playlist_height: 141.0,
            playlist_height_initialized: false,
            dragging_playlist_split: false,
            active_focus: None,
            window_height: 640.0,
            sort_column: None,
            sort_ascending: true,
            playlist_dialog: None,
            current_track_play_counted: false,
            selected_track: None,
            is_hovering_tracklist: false,
            is_hovering_sidebar_list: false,
            is_hovering_sidebar_resizer: false,
            is_hovering_playlist_resizer: false,
            group_by_album: crate::db::get(|db| db.group_by_album),
            sidebar_search: String::new(),
            show_shortcuts: false,
            last_click_track: None,
            last_click_artist: None,
            last_click_album: None,
            last_click_playlist: None,
            last_click_genre: None,
            hovered_playlist: None,
            show_context_menu: None,
            modifiers: Default::default(),
            selected_tracks: Vec::new(),
            last_clicked_track: None,
            hidden_artists_albums: crate::db::get(|db| db.hidden_artists_albums.clone()),
            playlist_tab: PlaylistTab::Playlists,
            right_panel_tab: None,
            right_panel_tab_user_scrolled: false,
            lyrics_scroll_id: scrollable::Id::unique(),
            last_active_lyric_idx: None,
            spectrum_bands: [0.0; crate::audio::spectrum::NUM_BANDS],
            spectrum_analyzer,
            audio,
            mpris_cmd_rx,
            mpris_update_tx,
            cover_cache: std::sync::Mutex::new(CoverCache { id: None, handle: None }),
            font_scale: cfg.font_scale(),
            hovered_album_header: None,
        };

        (state, scan_task)
    }


    fn send_mpris(&self, update: MprisUpdate) {
        let _ = self.mpris_update_tx.send(update);
    }

    fn notify_mpris_track(&self, status: PlaybackStatus) {
        if let Some(track) = &self.current_track {
            self.send_mpris(MprisUpdate::Metadata {
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                duration_us: track.duration.as_micros() as i64,
            });
        }
        self.send_mpris(MprisUpdate::Status(status));
    }

    pub fn artists(&self) -> Vec<String> {
        let query = self.sidebar_search.to_lowercase();
        let mut artists: Vec<String> = self.all_tracks.iter()
            .map(|t| if t.artist.trim().is_empty() { "Unknown Artist".to_string() } else { t.artist.clone() })
            .collect();
        artists.sort();
        artists.dedup();
        if !query.is_empty() {
            artists.retain(|a| a.to_lowercase().contains(&query));
        }
        artists.retain(|a| !self.hidden_artists_albums.contains(&(a.clone(), true)));
        artists
    }

    pub fn albums(&self) -> Vec<String> {
        let query = self.sidebar_search.to_lowercase();
        let mut albums: Vec<String> = self.all_tracks.iter()
            .map(|t| if t.album.trim().is_empty() { "Unknown Album".to_string() } else { t.album.clone() })
            .collect();
        albums.sort();
        albums.dedup();
        if !query.is_empty() {
            albums.retain(|a| a.to_lowercase().contains(&query));
        }
        albums.retain(|a| !self.hidden_artists_albums.contains(&(a.clone(), false)));
        albums
    }

    pub fn genres(&self) -> Vec<String> {
        let query = self.sidebar_search.to_lowercase();
        let mut genres: Vec<String> = self.all_tracks.iter()
            .map(|t| if t.genre.trim().is_empty() { "Unknown Genre".to_string() } else { t.genre.clone() })
            .collect();
        genres.sort();
        genres.dedup();
        if !query.is_empty() {
            genres.retain(|g| g.to_lowercase().contains(&query));
        }
        genres
    }

    pub fn update_filtered_tracks(&mut self) {
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.tracks = self.all_tracks.iter().filter(|t| {
                let match_title = self.filter_title && t.title.to_lowercase().contains(&query);
                let match_artist = self.filter_artist && t.artist.to_lowercase().contains(&query);
                let match_album = self.filter_album && t.album.to_lowercase().contains(&query);
                let match_genre = self.filter_genre && t.genre.to_lowercase().contains(&query);
                match_title || match_artist || match_album || match_genre
            }).cloned().collect();
        } else if let Some(playlist_name) = &self.selected_playlist {
            if playlist_name == "Liked Songs" {
                self.tracks = self.all_tracks.iter().filter(|t| t.liked).cloned().collect();
            } else if playlist_name == "Most Played" {
                let mut temp = self.all_tracks.clone();
                temp.sort_by(|a, b| b.play_count.cmp(&a.play_count));
                self.tracks = temp.into_iter().filter(|t| t.play_count > 0).collect();
            } else if playlist_name == "Recently Played" {
                let rp = crate::db::get(|db| db.recently_played.clone());
                let mut temp_tracks = Vec::new();
                for (path, date_str) in rp {
                    if let Some(mut t) = self.all_tracks.iter().find(|t| t.path == path).cloned() {
                        t.date_played = Some(date_str);
                        temp_tracks.push(t);
                    }
                }
                self.tracks = temp_tracks;
            } else if playlist_name == "New Music" {
                use std::time::SystemTime;
                let mut album_times: std::collections::HashMap<String, SystemTime> = std::collections::HashMap::new();
                for t in &self.all_tracks {
                    let mtime = std::fs::metadata(&t.path)
                        .and_then(|meta| meta.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    let entry = album_times.entry(t.album.clone()).or_insert(SystemTime::UNIX_EPOCH);
                    if mtime > *entry {
                        *entry = mtime;
                    }
                }
                
                let mut albums_sorted: Vec<(String, SystemTime)> = album_times.into_iter().collect();
                albums_sorted.sort_by(|a, b| b.1.cmp(&a.1));
                
                let now = SystemTime::now();
                let forty_eight_hours = std::time::Duration::from_secs(48 * 3600);
                
                let mut target_albums = std::collections::HashSet::new();
                for (idx, (album_title, added_time)) in albums_sorted.iter().enumerate() {
                    let is_in_last_48h = now.duration_since(*added_time)
                        .map(|d| d < forty_eight_hours)
                        .unwrap_or(false);
                    if idx < 5 || is_in_last_48h {
                        target_albums.insert(album_title.clone());
                    }
                }
                
                let mut temp_tracks: Vec<Track> = self.all_tracks.iter()
                    .filter(|t| target_albums.contains(&t.album))
                    .cloned()
                    .collect();
                    
                let album_times_ref = &albums_sorted.into_iter().collect::<std::collections::HashMap<_, _>>();
                temp_tracks.sort_by(|a, b| {
                    let time_a = album_times_ref.get(&a.album).unwrap_or(&SystemTime::UNIX_EPOCH);
                    let time_b = album_times_ref.get(&b.album).unwrap_or(&SystemTime::UNIX_EPOCH);
                    let cmp_time = time_b.cmp(time_a);
                    if cmp_time == std::cmp::Ordering::Equal {
                        let cmp_album = a.album.cmp(&b.album);
                        if cmp_album == std::cmp::Ordering::Equal {
                            let cmp_disc = a.disc_number.unwrap_or(0).cmp(&b.disc_number.unwrap_or(0));
                            if cmp_disc == std::cmp::Ordering::Equal {
                                a.track_number.unwrap_or(0).cmp(&b.track_number.unwrap_or(0))
                            } else {
                                cmp_disc
                            }
                        } else {
                            cmp_album
                        }
                    } else {
                        cmp_time
                    }
                });
                
                self.tracks = temp_tracks;
            } else {
                let paths = crate::db::get(|db| db.playlists.get(playlist_name).cloned().unwrap_or_default());
                self.tracks = self.all_tracks.iter().filter(|t| paths.contains(&t.path)).cloned().collect();
            }
        } else {
            match self.view_mode {

                ViewMode::Artists => {
                    if let Some(artist_name) = &self.selected_artist {
                        self.tracks = self.all_tracks.iter().filter(|t| {
                            let a = if t.artist.trim().is_empty() { "Unknown Artist" } else { &t.artist };
                            a == artist_name
                        }).cloned().collect();
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
                ViewMode::Albums => {
                    if let Some(album_name) = &self.selected_album {
                        self.tracks = self.all_tracks.iter().filter(|t| {
                            let al = if t.album.trim().is_empty() { "Unknown Album" } else { &t.album };
                            al == album_name
                        }).cloned().collect();
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
                ViewMode::Genres => {
                    if let Some(genre_name) = &self.selected_genre {
                        self.tracks = self.all_tracks.iter().filter(|t| {
                            let g = if t.genre.trim().is_empty() { "Unknown Genre" } else { &t.genre };
                            g == genre_name
                        }).cloned().collect();
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
            }
        }

        // Apply sorting
        if let Some(ref playlist_name) = self.selected_playlist {
            if playlist_name == "Recently Played" || playlist_name == "Most Played" {
                return;
            }
        }

        if let Some(col) = self.sort_column {
            self.tracks.sort_by(|a, b| {
                let cmp = match col {
                    SortColumn::TrackNumber => {
                        let a_num = a.track_number.unwrap_or(u32::MAX);
                        let b_num = b.track_number.unwrap_or(u32::MAX);
                        a_num.cmp(&b_num)
                    }
                    SortColumn::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                    SortColumn::Artist => a.artist.to_lowercase().cmp(&b.artist.to_lowercase()),
                    SortColumn::Album => a.album.to_lowercase().cmp(&b.album.to_lowercase()),
                    SortColumn::Genre => a.genre.to_lowercase().cmp(&b.genre.to_lowercase()),
                    SortColumn::Year => {
                        let a_yr = a.year.unwrap_or(u32::MAX);
                        let b_yr = b.year.unwrap_or(u32::MAX);
                        a_yr.cmp(&b_yr)
                    }
                    SortColumn::DiscNumber => {
                        let a_dc = a.disc_number.unwrap_or(u32::MAX);
                        let b_dc = b.disc_number.unwrap_or(u32::MAX);
                        a_dc.cmp(&b_dc)
                    }
                    SortColumn::Duration => a.duration.cmp(&b.duration),
                    SortColumn::Plays => a.play_count.cmp(&b.play_count),
                    SortColumn::DatePlayed => {
                        let a_dp = a.date_played.as_deref().unwrap_or("");
                        let b_dp = b.date_played.as_deref().unwrap_or("");
                        a_dp.cmp(b_dp)
                    }
                };
                if self.sort_ascending { cmp } else { cmp.reverse() }
            });
        }
    }


    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectFolder(path) => {
                self.selected_folder = Some(path);
                self.selected_playlist = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::FolderScanned(_, _) => {
                Task::none()
            }

            Message::PlayTrack(track) => {
                self.queue = self.tracks.clone();
                self.play_track_internal(track)
            }

            Message::PlayTracks(tracks) => {
                if let Some(first) = tracks.first().cloned() {
                    self.queue = tracks;
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::HoverAlbumHeader(album) => {
                self.hovered_album_header = album;
                Task::none()
            }

            Message::IncreaseScale => {
                self.font_scale = (self.font_scale + 0.05).min(3.0);
                Task::none()
            }

            Message::DecreaseScale => {
                self.font_scale = (self.font_scale - 0.05).max(0.5);
                Task::none()
            }

            Message::PlayPause => {
                match self.playback_state {
                    PlaybackState::Playing => {
                        if let Some(ref sel) = self.selected_track {
                            if self.current_track.as_ref().map(|ct| ct.id) != Some(sel.id) {
                                self.queue = self.tracks.clone();
                                return self.play_track_internal(sel.clone());
                            }
                        }
                        self.audio.send(AudioCommand::Pause);
                        self.playback_state = PlaybackState::Paused;
                        self.send_mpris(MprisUpdate::Status(PlaybackStatus::Paused));
                        Task::none()
                    }
                    PlaybackState::Paused => {
                        if let Some(ref sel) = self.selected_track {
                            if self.current_track.as_ref().map(|ct| ct.id) != Some(sel.id) {
                                self.queue = self.tracks.clone();
                                return self.play_track_internal(sel.clone());
                            }
                        }
                        self.audio.send(AudioCommand::Resume);
                        self.playback_state = PlaybackState::Playing;
                        self.send_mpris(MprisUpdate::Status(PlaybackStatus::Playing));
                        Task::none()
                    }
                    PlaybackState::Stopped => {
                        if let Some(ref sel) = self.selected_track {
                            self.queue = self.tracks.clone();
                            self.play_track_internal(sel.clone())
                        } else if let Some(first) = self.tracks.first().cloned() {
                            self.queue = self.tracks.clone();
                            self.play_track_internal(first)
                        } else {
                            Task::none()
                        }
                    }
                }
            }


            Message::NextTrack     => { self.advance_track(1) }
            Message::PreviousTrack => { self.advance_track(-1) }

            Message::Seek(dur) => {
                self.audio.send(AudioCommand::Seek(dur));
                self.position = dur;
                Task::none()
            }

            Message::SeekToLyric(dur) => {
                self.audio.send(AudioCommand::Seek(dur));
                self.position = dur;
                self.right_panel_tab_user_scrolled = false;
                Task::none()
            }

            Message::SeekRelative(delta_secs) => {
                let new_pos = if delta_secs < 0 {
                    self.position.saturating_sub(Duration::from_secs(delta_secs.unsigned_abs()))
                } else {
                    (self.position + Duration::from_secs(delta_secs as u64)).min(self.duration)
                };
                self.audio.send(AudioCommand::Seek(new_pos));
                self.position = new_pos;
                Task::none()
            }

            Message::VolumeChanged(v) => {
                self.volume = v;
                self.audio.send(AudioCommand::SetVolume(v));
                self.send_mpris(MprisUpdate::Volume(v as f64));
                Task::none()
            }

            Message::VolumeStep(delta) => {
                let v = (self.volume + delta).clamp(0.0, 1.0);
                self.volume = v;
                self.audio.send(AudioCommand::SetVolume(v));
                self.send_mpris(MprisUpdate::Volume(v as f64));
                Task::none()
            }

            Message::ToggleShuffle => {
                self.shuffle = !self.shuffle;
                self.send_mpris(MprisUpdate::Shuffle(self.shuffle));
                Task::none()
            }

            Message::ToggleRepeat => {
                self.repeat = !self.repeat;
                let loop_status = if self.repeat { LoopStatus::Playlist } else { LoopStatus::None };
                self.send_mpris(MprisUpdate::Loop(loop_status));
                Task::none()
            }

            Message::SidebarDragStart => {
                self.dragging_sidebar = true;
                Task::none()
            }

            Message::SidebarDragMove(x) => {
                self.sidebar_width = x.clamp(120.0, 400.0);
                Task::none()
            }

            Message::SidebarDragEnd => {
                self.dragging_sidebar = false;
                save_sidebar_width(self.sidebar_width);
                Task::none()
            }

            Message::RightPanelDragStart => {
                self.dragging_right_panel = true;
                Task::none()
            }

            Message::RightPanelDragMove(x) => {
                // x is cursor position from left of window.
                // The right panel's left edge is at (window_width - right_panel_width - tab_strip_width - separator).
                // We want: right_panel_width = window_width - x - some offset for the tab strip/separator on the right.
                // Actually the panel is to the right of the drag handle, so:
                // new_width = window_width - x
                // But we need to account for the separator width. The drag handle sits between player and panel.
                let new_width = (self.window_width - x).clamp(150.0, self.window_width * 0.6);
                self.right_panel_width = new_width;
                Task::none()
            }

            Message::RightPanelDragEnd => {
                self.dragging_right_panel = false;
                save_right_panel_width(self.right_panel_width);
                Task::none()
            }

            Message::PollAudio => {
                let mut tasks = Vec::new();
                while let Ok(event) = self.audio.event_rx.try_recv() {
                    match event {
                        AudioEvent::Progress { position, duration } => {
                            self.position = position;
                            self.duration = duration;
                            if !self.current_track_play_counted && duration > Duration::ZERO && position >= duration / 2 {
                                if let Some(ref mut track) = self.current_track {
                                    let count = crate::db::increment_play_count(track.path.clone());
                                    track.play_count = count;
                                    if let Some(t) = self.all_tracks.iter_mut().find(|t| t.path == track.path) {
                                        t.play_count = count;
                                    }
                                    if let Some(t) = self.tracks.iter_mut().find(|t| t.path == track.path) {
                                        t.play_count = count;
                                    }
                                }
                                self.current_track_play_counted = true;
                            }
                        }

                        AudioEvent::Paused => {
                            self.playback_state = PlaybackState::Paused;
                        }
                        AudioEvent::Stopped => {
                            self.playback_state = PlaybackState::Stopped;
                            self.position = Duration::ZERO;
                            self.send_mpris(MprisUpdate::Status(PlaybackStatus::Stopped));
                        }
                        AudioEvent::TrackEnded => {
                            if self.repeat {
                                tasks.push(self.advance_track(1));
                            } else {
                                let current_idx = self.current_track.as_ref()
                                    .and_then(|ct| self.queue.iter().position(|t| t.id == ct.id));
                                let is_last = match current_idx {
                                    Some(idx) => idx + 1 >= self.queue.len(),
                                    None => true,
                                };
                                if is_last && !self.shuffle {
                                    self.audio.send(AudioCommand::Stop);
                                    self.playback_state = PlaybackState::Stopped;
                                    self.position = Duration::ZERO;
                                    self.send_mpris(MprisUpdate::Status(PlaybackStatus::Stopped));
                                } else {
                                    tasks.push(self.advance_track(1));
                                }
                            }
                        }
                        AudioEvent::Error(e) => eprintln!("Erro de áudio: {e}"),
                        AudioEvent::Playing { .. } => {
                            self.playback_state = PlaybackState::Playing;
                        }
                    }
                }

                while let Ok(cmd) = self.mpris_cmd_rx.try_recv() {
                    match cmd {
                        MprisCommand::Play => {
                            if !matches!(self.playback_state, PlaybackState::Playing) {
                                self.audio.send(AudioCommand::Resume);
                                self.playback_state = PlaybackState::Playing;
                                self.send_mpris(MprisUpdate::Status(PlaybackStatus::Playing));
                            }
                        }
                        MprisCommand::Pause => {
                            if matches!(self.playback_state, PlaybackState::Playing) {
                                self.audio.send(AudioCommand::Pause);
                                self.playback_state = PlaybackState::Paused;
                                self.send_mpris(MprisUpdate::Status(PlaybackStatus::Paused));
                            }
                        }
                        MprisCommand::PlayPause => {
                            match self.playback_state {
                                PlaybackState::Playing => {
                                    self.audio.send(AudioCommand::Pause);
                                    self.playback_state = PlaybackState::Paused;
                                    self.send_mpris(MprisUpdate::Status(PlaybackStatus::Paused));
                                }
                                _ => {
                                    self.audio.send(AudioCommand::Resume);
                                    self.playback_state = PlaybackState::Playing;
                                    self.send_mpris(MprisUpdate::Status(PlaybackStatus::Playing));
                                }
                            }
                        }
                        MprisCommand::Next     => { tasks.push(self.advance_track(1)); }
                        MprisCommand::Previous => { tasks.push(self.advance_track(-1)); }
                        MprisCommand::Stop => {
                            self.audio.send(AudioCommand::Stop);
                            self.playback_state = PlaybackState::Stopped;
                            self.position = Duration::ZERO;
                            self.send_mpris(MprisUpdate::Status(PlaybackStatus::Stopped));
                        }
                        MprisCommand::SetVolume(v) => {
                            let clamped = v.clamp(0.0, 1.0) as f32;
                            self.volume = clamped;
                            self.audio.send(AudioCommand::SetVolume(clamped));
                            self.send_mpris(MprisUpdate::Volume(v));
                        }
                    }
                }

                // Auto-scroll lyrics if the active lyric line has changed
                if self.right_panel_tab == Some(RightPanelTab::Lyrics) {
                    if let Some(track) = self.current_track.as_ref() {
                        if !track.lyrics.trim().is_empty() {
                            let lrc_lines = crate::ui::views::player::parse_lrc(&track.lyrics);
                            if !lrc_lines.is_empty() {
                                let adjusted_pos = self.position.saturating_sub(
                                    crate::ui::views::player::LYRICS_OFFSET
                                );
                                let active_idx = lrc_lines.iter().position(|l| l.time > adjusted_pos)
                                    .map(|idx| if idx > 0 { idx - 1 } else { 0 })
                                    .unwrap_or_else(|| lrc_lines.len() - 1);

                                if self.last_active_lyric_idx != Some(active_idx) {
                                    self.last_active_lyric_idx = Some(active_idx);
                                    // Compute relative scroll position to center active line
                                    let total = lrc_lines.len();
                                    let fraction = if total <= 1 {
                                        0.0
                                    } else {
                                        active_idx as f32 / (total - 1) as f32
                                    };
                                    tasks.push(
                                        scrollable::snap_to(
                                            self.lyrics_scroll_id.clone(),
                                            scrollable::RelativeOffset { x: 0.0, y: fraction },
                                        )
                                    );
                                }
                            }
                        }
                    }
                }

                // Update spectrum when visualizer panel is open and playing
                if self.right_panel_tab == Some(RightPanelTab::Visualizer)
                    && matches!(self.playback_state, PlaybackState::Playing)
                {
                    self.spectrum_bands = self.spectrum_analyzer.compute();
                }

                if tasks.is_empty() {

                    Task::none()
                } else {
                    Task::batch(tasks)
                }
            }

            Message::PollSpectrum => {
                if self.right_panel_tab == Some(RightPanelTab::Visualizer) {
                    if matches!(self.playback_state, PlaybackState::Playing) {
                        self.spectrum_bands = self.spectrum_analyzer.compute();
                    } else {
                        self.spectrum_bands = [0.0; crate::audio::spectrum::NUM_BANDS];
                    }
                }
                Task::none()
            }

            Message::CheckTheme => {
                let current = crate::ui::theme::read_current_theme_name();
                if !current.is_empty() && current != self.loaded_theme_name {
                    crate::ui::theme::reload_system_theme();
                    self.iced_theme = build_iced_theme();
                    self.loaded_theme_name = current;
                }
                Task::none()
            }

            Message::SearchChanged(val) => {
                self.search_query = val;
                self.active_focus = Some(ActiveFocus::SongSearch);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleFilterTitle => {
                self.filter_title = !self.filter_title;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleFilterArtist => {
                self.filter_artist = !self.filter_artist;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleFilterAlbum => {
                self.filter_album = !self.filter_album;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleFilterGenre => {
                self.filter_genre = !self.filter_genre;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleLikeTrack(track) => {
                self.show_context_menu = None;
                let liked = crate::db::toggle_favorite(track.path.clone());
                if let Some(t) = self.all_tracks.iter_mut().find(|t| t.path == track.path) {
                    t.liked = liked;
                }
                if let Some(t) = self.tracks.iter_mut().find(|t| t.path == track.path) {
                    t.liked = liked;
                }
                if let Some(ref mut ct) = self.current_track {
                    if ct.path == track.path {
                        ct.liked = liked;
                    }
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::AddToPlaylist(playlist_name, track) => {
                crate::db::add_to_playlist(playlist_name, track.path);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::CreatePlaylist(name) => {
                crate::db::create_playlist(name);
                Task::none()
            }

            Message::SelectPlaylistTab(tab) => {
                self.playlist_tab = tab;
                self.selected_artist = None;
                self.selected_album = None;
                self.selected_genre = None;
                self.selected_folder = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                match tab {
                    PlaylistTab::Playlists => {
                        let custom_playlists = crate::db::get(|db| db.playlists.keys().cloned().collect::<Vec<String>>());
                        if let Some(first) = custom_playlists.first() {
                            self.selected_playlist = Some(first.clone());
                        } else {
                            self.selected_playlist = None;
                        }
                    }
                    PlaylistTab::Autoplaylists => {
                        self.selected_playlist = Some("Liked Songs".to_string());
                    }
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectPlaylist(name) => {
                if name == "Liked Songs" || name == "Recently Played" || name == "Most Played" || name == "New Music" {
                    self.playlist_tab = PlaylistTab::Autoplaylists;
                } else {
                    self.playlist_tab = PlaylistTab::Playlists;
                }
                let now = std::time::Instant::now();
                if let Some((ref prev_name, last_time)) = self.last_click_playlist {
                    if prev_name == &name && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_playlist = None;
                        return Task::done(Message::DoubleClickPlaylist(name));
                    }
                }
                self.last_click_playlist = Some((name.clone(), now));
                self.selected_playlist = Some(name);
                self.selected_folder = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::OpenTagEditor(tracks) => {
                self.show_context_menu = None;
                if tracks.is_empty() {
                    return Task::none();
                }

                let first = &tracks[0];
                let all_same_title = tracks.iter().all(|t| t.title == first.title);
                let all_same_artist = tracks.iter().all(|t| t.artist == first.artist);
                let all_same_album = tracks.iter().all(|t| t.album == first.album);
                let all_same_genre = tracks.iter().all(|t| t.genre == first.genre);
                let all_same_track_num = tracks.iter().all(|t| t.track_number == first.track_number);
                let all_same_disc_num = tracks.iter().all(|t| t.disc_number == first.disc_number);
                let all_same_year = tracks.iter().all(|t| t.year == first.year);
                let all_same_lyrics = tracks.iter().all(|t| t.lyrics == first.lyrics);

                self.show_tag_editor = Some(TagEditorState {
                    tracks: tracks.clone(),
                    title: if all_same_title { first.title.clone() } else { String::new() },
                    artist: if all_same_artist { first.artist.clone() } else { String::new() },
                    album: if all_same_album { first.album.clone() } else { String::new() },
                    genre: if all_same_genre { first.genre.clone() } else { String::new() },
                    track_number: if all_same_track_num { first.track_number.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    disc_number: if all_same_disc_num { first.disc_number.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    cover_path: None,
                    apply_to_album: false,
                    year: if all_same_year { first.year.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    apply_title: false,
                    apply_artist: false,
                    apply_album: false,
                    apply_year: false,
                    apply_genre: false,
                    apply_track_num: false,
                    apply_disc_num: false,
                    apply_cover: false,
                    apply_lyrics: false,
                    lyrics: if all_same_lyrics { first.lyrics.clone() } else { String::new() },
                    lyrics_content: iced::widget::text_editor::Content::with_text(if all_same_lyrics { &first.lyrics } else { "" }),
                    active_tab: TagEditorTab::Main,
                    focused_field: Some(0),
                });
                iced::widget::text_input::focus(iced::widget::text_input::Id::new("id3_title"))
            }

            Message::OpenLocalFolder(path) => {
                self.show_context_menu = None;
                if let Some(parent) = path.parent() {
                    let folder_to_open = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
                    let mut opened = false;
                    for fm in &["nautilus", "thunar", "dolphin", "nemo", "pcmanfm"] {
                        if std::process::Command::new(fm)
                            .arg(&folder_to_open)
                            .spawn()
                            .is_ok()
                        {
                            opened = true;
                            break;
                        }
                    }
                    if !opened {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&folder_to_open)
                            .spawn();
                    }
                }
                Task::none()
            }

            Message::CloseTagEditor => {
                self.show_tag_editor = None;
                Task::none()
            }

            Message::SearchCoverOnline => {
                if let Some(ref state) = self.show_tag_editor {
                    let artist = &state.artist;
                    let album = &state.album;
                    let query = format!("{} {} album art", artist, album);
                    let encoded: String = query
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() {
                                c.to_string()
                            } else if c == ' ' {
                                "+".to_string()
                            } else {
                                format!("%{:02X}", c as u32)
                            }
                        })
                        .collect();
                    let url = format!("https://www.google.com/search?q={}&tbm=isch", encoded);
                    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
                }
                Task::none()
            }

            Message::UpdateTagFieldTitle(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.title = val;
                    state.apply_title = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldArtist(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.artist = val;
                    state.apply_artist = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.album = val;
                    state.apply_album = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldGenre(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.genre = val;
                    state.apply_genre = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldTrackNumber(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.track_number = val;
                    state.apply_track_num = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldDiscNumber(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.disc_number = val;
                    state.apply_disc_num = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldCoverPath(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.cover_path = Some(val);
                    state.apply_cover = true;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldApplyToAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_to_album = val;
                }
                Task::none()
            }
 
            Message::UpdateTagFieldYear(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.year = val;
                    state.apply_year = true;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyTitle(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_title = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyArtist(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_artist = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_album = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyYear(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_year = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyGenre(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_genre = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyTrackNum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_track_num = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyDiscNum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_disc_num = val;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyCover(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_cover = val;
                }
                Task::none()
            }

            Message::SaveTags => {
                if let Some(ref state) = self.show_tag_editor {
                    let track_num = state.track_number.trim().parse::<u32>().ok();
                    let disc_num = state.disc_number.trim().parse::<u32>().ok();
                    let year_num = state.year.trim().parse::<u32>().ok();
                    let lyrics_text = state.lyrics_content.text();

                    let mut tracks_to_update = Vec::new();
                    if state.apply_to_album {
                        let albums: Vec<String> = state.tracks.iter().map(|t| t.album.clone()).collect();
                        for t in &self.all_tracks {
                            if albums.contains(&t.album) {
                                tracks_to_update.push(t.clone());
                            }
                        }
                    } else {
                        tracks_to_update = state.tracks.clone();
                    }

                    println!("SaveTags: apply_to_album={}, updating {} tracks.",
                        state.apply_to_album, tracks_to_update.len());

                    for track in tracks_to_update {
                        let title = if state.apply_title { &state.title } else { &track.title };
                        let artist = if state.apply_artist { &state.artist } else { &track.artist };
                        let album = if state.apply_album { &state.album } else { &track.album };
                        let genre = if state.apply_genre { &state.genre } else { &track.genre };
                        let track_number = if state.apply_track_num { track_num } else { track.track_number };
                        let disc_number = if state.apply_disc_num { disc_num } else { track.disc_number };
                        let year = if state.apply_year { year_num } else { track.year };
                        let cover_path = if state.apply_cover { state.cover_path.as_deref() } else { None };
                        let lyrics_val = if state.apply_lyrics { Some(lyrics_text.as_str()) } else { None };

                        let res = crate::library::write_tags(
                            &track.path,
                            title,
                            artist,
                            album,
                            genre,
                            track_number,
                            disc_number,
                            cover_path,
                            year,
                            lyrics_val,
                        );
                        if let Err(e) = res {
                            eprintln!("Error saving tags for {}: {e}", track.path.display());
                        } else {
                            if let Some(t) = self.all_tracks.iter_mut().find(|t| t.path == track.path) {
                                t.title = title.clone();
                                t.artist = artist.clone();
                                t.album = album.clone();
                                t.genre = genre.clone();
                                t.track_number = track_number;
                                t.disc_number = disc_number;
                                t.year = year;
                                if state.apply_lyrics {
                                    t.lyrics = lyrics_text.clone();
                                }
                                if cover_path.is_some() {
                                    t.cover_data = load_cover(&t.path);
                                }
                            }
                            if let Some(t) = self.tracks.iter_mut().find(|t| t.path == track.path) {
                                t.title = title.clone();
                                t.artist = artist.clone();
                                t.album = album.clone();
                                t.genre = genre.clone();
                                t.track_number = track_number;
                                t.disc_number = disc_number;
                                t.year = year;
                                if state.apply_lyrics {
                                    t.lyrics = lyrics_text.clone();
                                }
                                if cover_path.is_some() {
                                    t.cover_data = load_cover(&t.path);
                                }
                            }
                            if let Some(ref mut ct) = self.current_track {
                                if ct.path == track.path {
                                    ct.title = title.clone();
                                    ct.artist = artist.clone();
                                    ct.album = album.clone();
                                    ct.genre = genre.clone();
                                    ct.track_number = track_number;
                                    ct.disc_number = disc_number;
                                    ct.year = year;
                                    if state.apply_lyrics {
                                        ct.lyrics = lyrics_text.clone();
                                    }
                                    if cover_path.is_some() {
                                        ct.cover_data = load_cover(&ct.path);
                                    }
                                }
                            }
                            if let Some(ref mut st) = self.selected_track {
                                if st.path == track.path {
                                    st.title = title.clone();
                                    st.artist = artist.clone();
                                    st.album = album.clone();
                                    st.genre = genre.clone();
                                    st.track_number = track_number;
                                    st.disc_number = disc_number;
                                    st.year = year;
                                    if state.apply_lyrics {
                                        st.lyrics = lyrics_text.clone();
                                    }
                                    if cover_path.is_some() {
                                        st.cover_data = load_cover(&st.path);
                                    }
                                }
                            }
                            if let Some(t) = self.selected_tracks.iter_mut().find(|t| t.path == track.path) {
                                t.title = title.clone();
                                t.artist = artist.clone();
                                t.album = album.clone();
                                t.genre = genre.clone();
                                t.track_number = track_number;
                                t.disc_number = disc_number;
                                t.year = year;
                                if state.apply_lyrics {
                                    t.lyrics = lyrics_text.clone();
                                }
                                if cover_path.is_some() {
                                    t.cover_data = load_cover(&t.path);
                                }
                            }
                        }
                    }
                }
                self.show_tag_editor = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::LibraryScanned(tracks) => {
                self.all_tracks = tracks;
                let mut cache: HashMap<PathBuf, Vec<Track>> = HashMap::new();
                for track in &self.all_tracks {
                    if let Some(parent) = track.path.parent() {
                        cache.entry(parent.to_path_buf()).or_default().push(track.clone());
                    }
                }
                self.folder_cache = cache;
                let mut keys: Vec<PathBuf> = self.folder_cache.keys().cloned().collect();
                keys.sort();
                self.folders = keys;

                self.update_filtered_tracks();
                Task::none()
            }

            Message::RescanLibrary => {
                let music_dir = crate::config::get().music_path();
                Task::perform(
                    async move {
                        scan_folder(&music_dir)
                    },
                    Message::LibraryScanned,
                )
            }

            Message::KeyboardLike => {
                if let Some(ref track) = self.current_track {
                    let mut t = track.clone();
                    // Strip cover data for messaging to keep it light
                    t.cover_data = None;
                    return Task::done(Message::ToggleLikeTrack(t));
                }
                Task::none()
            }

            Message::KeyboardEdit => {
                let tracks_to_edit = if !self.selected_tracks.is_empty() {
                    self.selected_tracks.clone()
                } else if let Some(ref track) = self.current_track {
                    vec![track.clone()]
                } else {
                    Vec::new()
                };
                if !tracks_to_edit.is_empty() {
                    let mut cleaned = tracks_to_edit;
                    for t in &mut cleaned {
                        t.cover_data = None;
                    }
                    return Task::done(Message::OpenTagEditor(cleaned));
                }
                Task::none()
            }

            Message::KeyboardAdd => {
                if let Some(ref track) = self.current_track {
                    let mut t = track.clone();
                    t.cover_data = None;
                    return Task::done(Message::OpenPlaylistDialog(PlaylistDialogMode::AddTrack(t)));
                }
                Task::none()
            }


            Message::PlaylistDragStart => {
                self.dragging_playlist_split = true;
                Task::none()
            }

            Message::PlaylistDragMove(y) => {
                self.playlist_height = (self.window_height - y - 60.0).clamp(50.0, self.window_height - 200.0);
                Task::none()
            }

            Message::PlaylistDragEnd => {
                self.dragging_playlist_split = false;
                Task::none()
            }

            Message::SelectViewMode(mode) => {
                self.view_mode = mode;
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.selected_genre = None;
                self.selected_tracks.clear();
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectAllArtists => {
                self.selected_artist = None;
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_album = None;
                self.selected_genre = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectAllAlbums => {
                self.selected_album = None;
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_genre = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectAllGenres => {
                self.selected_genre = None;
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectArtist(artist) => {
                let now = std::time::Instant::now();
                if let Some((ref prev_artist, last_time)) = self.last_click_artist {
                    if prev_artist == &artist && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_artist = None;
                        return Task::done(Message::DoubleClickArtist(artist));
                    }
                }
                self.last_click_artist = Some((artist.clone(), now));
                self.selected_artist = Some(artist);
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_album = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectAlbum(album) => {
                let now = std::time::Instant::now();
                if let Some((ref prev_album, last_time)) = self.last_click_album {
                    if prev_album == &album && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_album = None;
                        return Task::done(Message::DoubleClickAlbum(album));
                    }
                }
                self.last_click_album = Some((album.clone(), now));
                self.selected_album = Some(album);
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SortBy(col) => {
                if self.sort_column == Some(col) {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = Some(col);
                    self.sort_ascending = true;
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::OpenPlaylistDialog(mode) => {
                let initial_name = match &mode {
                    PlaylistDialogMode::Create => "My Playlist".to_string(),
                    PlaylistDialogMode::AddTrack(_) => String::new(),
                    PlaylistDialogMode::Rename(old_name) => old_name.clone(),
                };
                let custom_playlists = crate::db::get(|db| db.playlists.keys().cloned().collect::<Vec<String>>());
                let first_playlist = custom_playlists.first().cloned();
                self.playlist_dialog = Some(PlaylistDialogState {
                    mode,
                    name_input: initial_name,
                    selected_playlist: first_playlist,
                    add_album: false,
                });
                Task::none()
            }

            Message::ClosePlaylistDialog => {
                self.playlist_dialog = None;
                Task::none()
            }

            Message::PlaylistInputChanged(val) => {
                if let Some(ref mut dialog) = self.playlist_dialog {
                    dialog.name_input = val;
                }
                Task::none()
            }

            Message::PlaylistDialogSelect(name) => {
                if let Some(ref mut dialog) = self.playlist_dialog {
                    dialog.selected_playlist = Some(name);
                }
                Task::none()
            }

            Message::PlaylistDialogToggleAddAlbum(val) => {
                if let Some(ref mut dialog) = self.playlist_dialog {
                    dialog.add_album = val;
                }
                Task::none()
            }

            Message::PlaylistDialogSubmit => {
                if let Some(dialog) = self.playlist_dialog.clone() {
                    match dialog.mode {
                        PlaylistDialogMode::Create => {
                            let name = dialog.name_input.trim().to_string();
                            if !name.is_empty() {
                                crate::db::create_playlist(name);
                            }
                        }
                        PlaylistDialogMode::AddTrack(track) => {
                            if let Some(playlist_name) = dialog.selected_playlist {
                                if dialog.add_album {
                                    let album_tracks: Vec<Track> = self.all_tracks.iter()
                                        .filter(|t| t.album == track.album)
                                        .cloned()
                                        .collect();
                                    for t in album_tracks {
                                        crate::db::add_to_playlist(playlist_name.clone(), t.path);
                                    }
                                } else {
                                    crate::db::add_to_playlist(playlist_name, track.path);
                                }
                            }
                        }
                        PlaylistDialogMode::Rename(old_name) => {
                            let new_name = dialog.name_input.trim().to_string();
                            if !new_name.is_empty() && new_name != old_name {
                                crate::db::rename_playlist(old_name.clone(), new_name.clone());
                                if self.selected_playlist.as_ref() == Some(&old_name) {
                                    self.selected_playlist = Some(new_name);
                                }
                            }
                        }
                    }
                    self.playlist_dialog = None;
                    self.update_filtered_tracks();
                }
                Task::none()
            }

            Message::WindowResized(w, h) => {
                self.window_height = h;
                self.window_width = w;
                if !self.playlist_height_initialized {
                    self.playlist_height = ((h - 212.0) * 0.33).max(50.0);
                    self.playlist_height_initialized = true;
                }
                Task::none()
            }

            Message::HoverTracklist(val) => {
                self.is_hovering_tracklist = val;
                Task::none()
            }

            Message::HoverSidebarList(val) => {
                self.is_hovering_sidebar_list = val;
                Task::none()
            }

            Message::HoverRightPanelResizer(val) => {
                self.is_hovering_right_panel_resizer = val;
                Task::none()
            }

            Message::HoverSidebarResizer(val) => {
                self.is_hovering_sidebar_resizer = val;
                Task::none()
            }

            Message::HoverPlaylistResizer(val) => {
                self.is_hovering_playlist_resizer = val;
                Task::none()
            }

            Message::KeyboardArrowUp => {
                if (self.is_hovering_tracklist || self.active_focus == Some(ActiveFocus::Tracklist)) && !self.tracks.is_empty() {
                    let current_idx = self.selected_track.as_ref()
                        .and_then(|st| self.tracks.iter().position(|t| t.id == st.id));
                    let next_idx = match current_idx {
                        Some(i) => if i == 0 { self.tracks.len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    if let Some(track) = self.tracks.get(next_idx).cloned() {
                        let cover_data = load_cover(&track.path);
                        let track = Track { cover_data, ..track };
                        self.selected_track = Some(track.clone());
                        self.selected_tracks = vec![track.clone()];
                        self.last_clicked_track = Some(track.clone());
                        if let Some(y) = self.calculate_scroll_offset(track.id) {
                            let target_y = (y - 120.0).max(0.0);
                            return iced::widget::scrollable::scroll_to(
                                iced::widget::scrollable::Id::new("tracklist_scroll"),
                                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target_y }
                            );
                        }
                    }
                } else if self.is_hovering_sidebar_list || self.active_focus == Some(ActiveFocus::SidebarList) {
                    match self.view_mode {
                        ViewMode::Artists => {
                            let artists = self.artists();
                            if !artists.is_empty() {
                                let current_idx = self.selected_artist.as_ref()
                                    .and_then(|sa| artists.iter().position(|a| a == sa));
                                let next_idx = match current_idx {
                                    Some(i) => if i == 0 { artists.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                if let Some(artist) = artists.get(next_idx).cloned() {
                                    self.selected_artist = Some(artist);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                        ViewMode::Albums => {
                            let albums = self.albums();
                            if !albums.is_empty() {
                                let current_idx = self.selected_album.as_ref()
                                    .and_then(|sa| albums.iter().position(|a| a == sa));
                                let next_idx = match current_idx {
                                    Some(i) => if i == 0 { albums.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                if let Some(album) = albums.get(next_idx).cloned() {
                                    self.selected_album = Some(album);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                        ViewMode::Genres => {
                            let genres = self.genres();
                            if !genres.is_empty() {
                                let current_idx = self.selected_genre.as_ref()
                                    .and_then(|sg| genres.iter().position(|g| g == sg));
                                let next_idx = match current_idx {
                                    Some(i) => if i == 0 { genres.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                if let Some(genre) = genres.get(next_idx).cloned() {
                                    self.selected_genre = Some(genre);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::KeyboardArrowDown => {
                if (self.is_hovering_tracklist || self.active_focus == Some(ActiveFocus::Tracklist)) && !self.tracks.is_empty() {
                    let current_idx = self.selected_track.as_ref()
                        .and_then(|st| self.tracks.iter().position(|t| t.id == st.id));
                    let next_idx = match current_idx {
                        Some(i) => (i + 1) % self.tracks.len(),
                        None => 0,
                    };
                    if let Some(track) = self.tracks.get(next_idx).cloned() {
                        let cover_data = load_cover(&track.path);
                        let track = Track { cover_data, ..track };
                        self.selected_track = Some(track.clone());
                        self.selected_tracks = vec![track.clone()];
                        self.last_clicked_track = Some(track.clone());
                        if let Some(y) = self.calculate_scroll_offset(track.id) {
                            let target_y = (y - 120.0).max(0.0);
                            return iced::widget::scrollable::scroll_to(
                                iced::widget::scrollable::Id::new("tracklist_scroll"),
                                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target_y }
                            );
                        }
                    }
                } else if self.is_hovering_sidebar_list || self.active_focus == Some(ActiveFocus::SidebarList) {
                    match self.view_mode {
                        ViewMode::Artists => {
                            let artists = self.artists();
                            if !artists.is_empty() {
                                let current_idx = self.selected_artist.as_ref()
                                    .and_then(|sa| artists.iter().position(|a| a == sa));
                                let next_idx = match current_idx {
                                    Some(i) => (i + 1) % artists.len(),
                                    None => 0,
                                };
                                if let Some(artist) = artists.get(next_idx).cloned() {
                                    self.selected_artist = Some(artist);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                        ViewMode::Albums => {
                            let albums = self.albums();
                            if !albums.is_empty() {
                                let current_idx = self.selected_album.as_ref()
                                    .and_then(|sa| albums.iter().position(|a| a == sa));
                                let next_idx = match current_idx {
                                    Some(i) => (i + 1) % albums.len(),
                                    None => 0,
                                };
                                if let Some(album) = albums.get(next_idx).cloned() {
                                    self.selected_album = Some(album);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                        ViewMode::Genres => {
                            let genres = self.genres();
                            if !genres.is_empty() {
                                let current_idx = self.selected_genre.as_ref()
                                    .and_then(|sg| genres.iter().position(|g| g == sg));
                                let next_idx = match current_idx {
                                    Some(i) => (i + 1) % genres.len(),
                                    None => 0,
                                };
                                if let Some(genre) = genres.get(next_idx).cloned() {
                                    self.selected_genre = Some(genre);
                                    self.update_filtered_tracks();
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::DeletePlaylist(name) => {
                crate::db::delete_playlist(name.clone());
                if self.selected_playlist.as_ref() == Some(&name) {
                    self.selected_playlist = None;
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::RenamePlaylist(old_name, new_name) => {
                crate::db::rename_playlist(old_name.clone(), new_name.clone());
                if self.selected_playlist.as_ref() == Some(&old_name) {
                    self.selected_playlist = Some(new_name);
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleGroupByAlbum => {
                self.group_by_album = !self.group_by_album;
                crate::db::write(|db| db.group_by_album = self.group_by_album);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ModifiersChanged(mods) => {
                self.modifiers = mods;
                Task::none()
            }

            Message::SelectTrack(track) => {
                let now = std::time::Instant::now();
                if let Some((prev_id, last_time)) = self.last_click_track {
                    if prev_id == track.id && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_track = None;
                        return Task::done(Message::DoubleClickTrack(track));
                    }
                }
                self.last_click_track = Some((track.id, now));
                self.active_focus = Some(ActiveFocus::Tracklist);
                let cover_data = load_cover(&track.path);
                let track = Track { cover_data, ..track };

                let shift_held = self.modifiers.shift();
                let ctrl_held = self.modifiers.control() || self.modifiers.command();

                if ctrl_held {
                    if self.selected_tracks.iter().any(|t| t.id == track.id) {
                        self.selected_tracks.retain(|t| t.id != track.id);
                    } else {
                        self.selected_tracks.push(track.clone());
                    }
                    self.last_clicked_track = Some(track.clone());
                } else if shift_held {
                    if let Some(ref start_track) = self.last_clicked_track {
                        let start_idx = self.tracks.iter().position(|t| t.id == start_track.id);
                        let end_idx = self.tracks.iter().position(|t| t.id == track.id);
                        if let (Some(s), Some(e)) = (start_idx, end_idx) {
                            let (min, max) = if s < e { (s, e) } else { (e, s) };
                            self.selected_tracks = self.tracks[min..=max].to_vec();
                        }
                    } else {
                        self.selected_tracks = vec![track.clone()];
                        self.last_clicked_track = Some(track.clone());
                    }
                } else {
                    self.selected_tracks = vec![track.clone()];
                    self.last_clicked_track = Some(track.clone());
                }

                self.selected_track = Some(track);
                Task::none()
            }

            Message::SidebarSearchChanged(query) => {
                self.sidebar_search = query;
                Task::none()
            }

            Message::OpenShortcuts => {
                self.show_shortcuts = true;
                Task::none()
            }

            Message::CloseShortcuts => {
                self.show_shortcuts = false;
                Task::none()
            }

            Message::KeyPressed(key) => {
                use iced::keyboard::Key;
                use iced::keyboard::key::Named;
                let seek = crate::config::get().seek_step as i64;
                let vol  = crate::config::get().volume_step;
                let has_tag_editor = self.show_tag_editor.is_some();
                let has_playlist_dialog = self.playlist_dialog.is_some();
                let has_shortcuts = self.show_shortcuts;
                let has_context_menu = self.show_context_menu.is_some();

                match key {
                    Key::Named(Named::Enter) => {
                        if has_tag_editor {
                            return Task::done(Message::SaveTags);
                        } else if has_playlist_dialog {
                            return Task::done(Message::PlaylistDialogSubmit);
                        } else if !has_shortcuts && !has_context_menu {
                            if self.active_focus == Some(ActiveFocus::Tracklist) {
                                if let Some(ref track) = self.selected_track {
                                    return Task::done(Message::DoubleClickTrack(track.clone()));
                                }
                            }
                        }
                    }
                    Key::Named(Named::Escape) => {
                        if has_shortcuts {
                            return Task::done(Message::CloseShortcuts);
                        } else if has_playlist_dialog {
                            return Task::done(Message::ClosePlaylistDialog);
                        } else if has_tag_editor {
                            return Task::done(Message::CloseTagEditor);
                        } else if has_context_menu {
                            return Task::done(Message::ToggleContextMenu(None));
                        }
                    }
                    Key::Named(Named::Tab) => {
                        if has_tag_editor {
                            if let Some(ref mut state) = self.show_tag_editor {
                                let fields = &[
                                    "id3_title",
                                    "id3_artist",
                                    "id3_album",
                                    "id3_genre",
                                    "id3_track",
                                    "id3_disc",
                                    "id3_year",
                                    "id3_cover",
                                ];
                                let current = state.focused_field.unwrap_or(0);
                                let next = if self.modifiers.shift() {
                                    if current == 0 { fields.len() - 1 } else { current - 1 }
                                } else {
                                    (current + 1) % fields.len()
                                };
                                state.focused_field = Some(next);
                                return iced::widget::text_input::focus(iced::widget::text_input::Id::new(fields[next]));
                            }
                        } else if !has_playlist_dialog && !has_shortcuts && !has_context_menu {
                            if self.active_focus == Some(ActiveFocus::SidebarSearch) {
                                self.active_focus = Some(ActiveFocus::SidebarList);
                                match self.view_mode {
                                    ViewMode::Artists => {
                                        if self.selected_artist.is_none() {
                                            if let Some(artist) = self.artists().first().cloned() {
                                                self.selected_artist = Some(artist);
                                                self.update_filtered_tracks();
                                            }
                                        }
                                    }
                                    ViewMode::Albums => {
                                        if self.selected_album.is_none() {
                                            if let Some(album) = self.albums().first().cloned() {
                                                self.selected_album = Some(album);
                                                self.update_filtered_tracks();
                                            }
                                        }
                                    }
                                    ViewMode::Genres => {
                                        if self.selected_genre.is_none() {
                                            if let Some(genre) = self.genres().first().cloned() {
                                                self.selected_genre = Some(genre);
                                                self.update_filtered_tracks();
                                            }
                                        }
                                    }
                                }
                                return Task::none();
                            } else if self.active_focus == Some(ActiveFocus::SidebarList) {
                                self.active_focus = Some(ActiveFocus::Tracklist);
                                if self.selected_track.is_none() {
                                    if let Some(track) = self.tracks.first().cloned() {
                                        let cover_data = load_cover(&track.path);
                                        let track = Track { cover_data, ..track };
                                        self.selected_track = Some(track.clone());
                                        self.selected_tracks = vec![track.clone()];
                                        self.last_clicked_track = Some(track.clone());
                                    }
                                }
                                return Task::none();
                            } else if self.active_focus == Some(ActiveFocus::SongSearch) {
                                self.active_focus = Some(ActiveFocus::Tracklist);
                                if self.selected_track.is_none() {
                                    if let Some(track) = self.tracks.first().cloned() {
                                        let cover_data = load_cover(&track.path);
                                        let track = Track { cover_data, ..track };
                                        self.selected_track = Some(track.clone());
                                        self.selected_tracks = vec![track.clone()];
                                        self.last_clicked_track = Some(track.clone());
                                    }
                                }
                                return Task::none();
                            } else if self.active_focus == Some(ActiveFocus::Tracklist) {
                                self.active_focus = Some(ActiveFocus::SongSearch);
                                return iced::widget::text_input::focus(iced::widget::text_input::Id::new("song_search_input"));
                            }
                        }
                    }
                    Key::Named(Named::Space) => {
                        if !has_playlist_dialog && !has_tag_editor {
                            return Task::done(Message::PlayPause);
                        }
                    }
                    Key::Named(Named::ArrowRight) => return Task::done(Message::SeekRelative(seek)),
                    Key::Named(Named::ArrowLeft)  => return Task::done(Message::SeekRelative(-seek)),
                    Key::Named(Named::ArrowUp)    => return Task::done(Message::KeyboardArrowUp),
                    Key::Named(Named::ArrowDown)  => return Task::done(Message::KeyboardArrowDown),
                    Key::Named(Named::F5)         => return Task::done(Message::RescanLibrary),
                    Key::Character(ref c) => {
                        if !has_playlist_dialog && !has_tag_editor {
                            match c.as_str() {
                                "n" | "N" => return Task::done(Message::NextTrack),
                                "p" | "P" => return Task::done(Message::PreviousTrack),
                                "s" | "S" => return Task::done(Message::ToggleShuffle),
                                "r" | "R" => return Task::done(Message::ToggleRepeat),
                                "+" | "=" if self.modifiers.control() => return Task::done(Message::IncreaseScale),
                                "-"       if self.modifiers.control() => return Task::done(Message::DecreaseScale),
                                "+" | "=" => return Task::done(Message::VolumeStep(vol)),
                                "-"       => return Task::done(Message::VolumeStep(-vol)),
                                "/" => {
                                    self.active_focus = Some(ActiveFocus::SongSearch);
                                    self.search_query.clear();
                                    self.update_filtered_tracks();
                                    return iced::widget::text_input::focus(iced::widget::text_input::Id::new("song_search_input"));
                                }
                                "l" | "L" | "f" | "F" => return Task::done(Message::KeyboardLike),
                                "e" | "E" => return Task::done(Message::KeyboardEdit),
                                "c" | "C" => return Task::done(Message::OpenPlaylistDialog(PlaylistDialogMode::Create)),
                                "a" | "A" => return Task::done(Message::KeyboardAdd),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                Task::none()
            }

            Message::DoubleClickTrack(track) => {
                self.selected_track = Some(track.clone());
                self.queue = self.tracks.clone();
                self.play_track_internal(track)
            }

            Message::DoubleClickArtist(artist_name) => {
                self.view_mode = ViewMode::Artists;
                self.selected_artist = Some(artist_name.clone());
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_album = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                self.shuffle = true;
                // Shuffle tracks of this artist
                let mut artist_tracks = self.tracks.clone();
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                artist_tracks.shuffle(&mut rng);
                self.queue = artist_tracks.clone();
                if let Some(first) = artist_tracks.first().cloned() {
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::DoubleClickAlbum(album_name) => {
                self.view_mode = ViewMode::Albums;
                self.selected_album = Some(album_name.clone());
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                
                // Sort by track number ascending
                self.tracks.sort_by_key(|t| t.track_number.unwrap_or(u32::MAX));
                self.queue = self.tracks.clone();
                if let Some(first) = self.tracks.first().cloned() {
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::DoubleClickPlaylist(playlist_name) => {
                self.selected_playlist = Some(playlist_name.clone());
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                self.queue = self.tracks.clone();
                if let Some(first) = self.tracks.first().cloned() {
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::ReturnToActiveSource => {
                if let Some(current) = self.current_track.clone() {
                    // Try to restore the album or playlist view mode context
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_artist = None;
                    self.selected_album = Some(current.album.clone());
                    self.view_mode = ViewMode::Albums;
                    self.search_query.clear();
                    self.update_filtered_tracks();
                    self.selected_track = Some(current.clone());
                    if let Some(y) = self.calculate_scroll_offset(current.id) {
                        let target_y = (y - 120.0).max(0.0);
                        iced::widget::scrollable::scroll_to(
                            iced::widget::scrollable::Id::new("tracklist_scroll"),
                            iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target_y }
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }

            Message::FocusSongName => {
                if let Some(current) = self.current_track.clone() {
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_artist = None;
                    self.selected_album = Some(current.album.clone());
                    self.view_mode = ViewMode::Albums;
                    self.search_query.clear();
                    self.update_filtered_tracks();
                    self.selected_track = Some(current.clone());
                    if let Some(y) = self.calculate_scroll_offset(current.id) {
                        let target_y = (y - 120.0).max(0.0);
                        iced::widget::scrollable::scroll_to(
                            iced::widget::scrollable::Id::new("tracklist_scroll"),
                            iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target_y }
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }

            Message::FocusArtistName => {
                if let Some(current) = self.current_track.clone() {
                    self.view_mode = ViewMode::Artists;
                    self.selected_artist = Some(current.artist.clone());
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_album = None;
                    self.search_query.clear();
                    self.update_filtered_tracks();
                }
                Task::none()
            }

            Message::FocusAlbumName => {
                if let Some(current) = self.current_track.clone() {
                    self.view_mode = ViewMode::Albums;
                    self.selected_album = Some(current.album.clone());
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_artist = None;
                    self.search_query.clear();
                    self.update_filtered_tracks();
                }
                Task::none()
            }

            Message::SelectGenre(genre) => {
                let now = std::time::Instant::now();
                if let Some((ref prev_genre, last_time)) = self.last_click_genre {
                    if prev_genre == &genre && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_genre = None;
                        return Task::done(Message::DoubleClickGenre(genre));
                    }
                }
                self.last_click_genre = Some((genre.clone(), now));
                self.selected_genre = Some(genre);
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.active_focus = Some(ActiveFocus::SidebarList);
                self.search_query.clear();
                self.update_filtered_tracks();
                Task::none()
            }

            Message::DoubleClickGenre(genre_name) => {
                self.view_mode = ViewMode::Genres;
                self.selected_genre = Some(genre_name);
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                self.queue = self.tracks.clone();
                if let Some(first) = self.tracks.first().cloned() {
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::HoverPlaylist(name) => {
                self.hovered_playlist = name;
                Task::none()
            }

            Message::ToggleContextMenu(val) => {
                self.show_context_menu = val;
                Task::none()
            }

            Message::ToggleColumnVisibility(col) => {
                crate::db::write(|db| {
                    if db.table_columns.contains(&col) {
                        if db.table_columns.len() > 1 {
                            db.table_columns.retain(|&c| c != col);
                        }
                    } else {
                        db.table_columns.push(col);
                    }
                });
                Task::none()
            }

            Message::MoveColumnLeft(col) => {
                crate::db::write(|db| {
                    if let Some(pos) = db.table_columns.iter().position(|&c| c == col) {
                        if pos > 0 {
                            db.table_columns.swap(pos, pos - 1);
                        }
                    }
                });
                Task::none()
            }

            Message::MoveColumnRight(col) => {
                crate::db::write(|db| {
                    if let Some(pos) = db.table_columns.iter().position(|&c| c == col) {
                        if pos < db.table_columns.len() - 1 {
                            db.table_columns.swap(pos, pos + 1);
                        }
                    }
                });
                Task::none()
            }

            Message::HideAlbumOrArtist(name, is_artist) => {
                self.hidden_artists_albums.push((name.clone(), is_artist));
                crate::db::write(|db| {
                    db.hidden_artists_albums.push((name, is_artist));
                });
                self.show_context_menu = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.selected_genre = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::RestoreHiddenItems => {
                self.hidden_artists_albums.clear();
                crate::db::write(|db| {
                    db.hidden_artists_albums.clear();
                });
                self.update_filtered_tracks();
                Task::none()
            }

            Message::CreatePlaylistFromContext(target_name, is_artist) => {
                crate::db::create_playlist(target_name.clone());
                let matched_tracks: Vec<Track> = self.all_tracks.iter()
                    .filter(|t| {
                        if is_artist {
                            let a = if t.artist.trim().is_empty() { "Unknown Artist" } else { &t.artist };
                            a == target_name
                        } else {
                            let al = if t.album.trim().is_empty() { "Unknown Album" } else { &t.album };
                            al == target_name
                        }
                    })
                    .cloned()
                    .collect();
                for t in matched_tracks {
                    crate::db::add_to_playlist(target_name.clone(), t.path);
                }
                self.show_context_menu = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::AddAlbumToPlaylist(album_name, playlist_name) => {
                let album_tracks: Vec<Track> = self.all_tracks.iter()
                    .filter(|t| t.album == album_name)
                    .cloned()
                    .collect();
                for t in album_tracks {
                    crate::db::add_to_playlist(playlist_name.clone(), t.path);
                }
                self.show_context_menu = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::AddTracksToPlaylist(playlist_name, tracks) => {
                for t in tracks {
                    crate::db::add_to_playlist(playlist_name.clone(), t.path);
                }
                self.show_context_menu = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::CreatePlaylistWithTracks(playlist_name, tracks) => {
                crate::db::create_playlist(playlist_name.clone());
                for t in tracks {
                    crate::db::add_to_playlist(playlist_name.clone(), t.path);
                }
                self.show_context_menu = None;
                self.update_filtered_tracks();
                Task::none()
            }

            Message::ToggleRightPanelTab(tab) => {
                if self.right_panel_tab == Some(tab) {
                    self.right_panel_tab = None;
                } else {
                    self.right_panel_tab = Some(tab);
                }
                Task::none()
            }

            Message::SelectTagEditorTab(tab) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.active_tab = tab;
                }
                Task::none()
            }

            Message::UpdateTagFieldLyrics(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.lyrics_content.perform(val);
                    state.apply_lyrics = true;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyLyrics(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_lyrics = val;
                }
                Task::none()
            }

            Message::SearchLyricsOnline => {
                if let Some(ref state) = self.show_tag_editor {
                    let artist = state.artist.trim();
                    let title = state.title.trim();
                    if !artist.is_empty() && !title.is_empty() {
                        let query = format!("{} {}", artist, title);
                        let mut encoded = String::new();
                        for c in query.chars() {
                            match c {
                                ' ' => encoded.push('+'),
                                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => encoded.push(c),
                                _ => {
                                    encoded.push_str(&format!("%{:02X}", c as u32));
                                }
                            }
                        }
                        let url = format!("https://lrclib.net/api/search?q={}", encoded);
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&url)
                            .spawn();
                    } else {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://lrclib.net/api/search")
                            .spawn();
                    }
                }
                Task::none()
            }
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let main = column![
            views::player::view(self),
            views::library::view(self),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        let app_container = container(main)
            .style(|_: &Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::base())),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let mut view_stack = stack![app_container];

        if let Some(ref editor_state) = self.show_tag_editor {
            let mut unique_artists: Vec<String> = self.all_tracks.iter().map(|t| t.artist.clone()).filter(|s| !s.trim().is_empty()).collect();
            unique_artists.sort();
            unique_artists.dedup();

            let mut unique_albums: Vec<String> = self.all_tracks.iter().map(|t| t.album.clone()).filter(|s| !s.trim().is_empty()).collect();
            unique_albums.sort();
            unique_albums.dedup();

            let mut unique_genres: Vec<String> = self.all_tracks.iter().map(|t| t.genre.clone()).filter(|s| !s.trim().is_empty()).collect();
            unique_genres.sort();
            unique_genres.dedup();

            view_stack = view_stack.push(crate::ui::components::tag_editor::view(
                editor_state,
                &unique_artists,
                &unique_albums,
                &unique_genres,
            ));
        } else if let Some(ref playlist_dialog_state) = self.playlist_dialog {
            view_stack = view_stack.push(crate::ui::components::playlist_dialog::view(playlist_dialog_state));
        } else if self.show_shortcuts {
            view_stack = view_stack.push(self.shortcuts_modal_view());
        }

        if let Some(ref target) = self.show_context_menu {
            let custom_playlists = crate::db::get(|db| db.playlists.keys().cloned().collect::<Vec<String>>());
            
            let item_style = |_theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    background: if is_hovered { Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.2))) } else { None },
                    text_color: if is_hovered { theme::accent() } else { theme::text() },
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            };

            let accent_item_style = |_theme: &iced::Theme, status: iced::widget::button::Status| {
                let is_hovered = status == iced::widget::button::Status::Hovered || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    background: if is_hovered { Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.2))) } else { None },
                    text_color: theme::accent(),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            };

            let mut playlist_select = column![
                text("Add to Playlist:")
                    .size(11)
                    .color(theme::subtext())
                    .font(crate::ui::icons::UI_FONT_BOLD)
            ]
            .spacing(4);

            let (title, hide_btn, create_btn): (String, Option<Element<'_, Message>>, _) = match target {
                ContextMenuTarget::Artist(artist_name) => {
                    let title = format!("Artist Menu: {artist_name}");
                    let hide = button(text("Hide from UI").size(13))
                        .on_press(Message::HideAlbumOrArtist(artist_name.clone(), true))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    
                    for pl in &custom_playlists {
                        let artist_tracks: Vec<Track> = self.all_tracks.iter()
                            .filter(|t| {
                                let a = if t.artist.trim().is_empty() { "Unknown Artist" } else { &t.artist };
                                a == artist_name
                            })
                            .cloned()
                            .collect();
                        playlist_select = playlist_select.push(
                            button(text(format!("  + {}", pl)).size(12))
                                .on_press(Message::AddTracksToPlaylist(pl.clone(), artist_tracks))
                                .style(item_style)
                                .padding([4, 8])
                                .width(Length::Fill)
                        );
                    }

                    let create = button(text("+ Create playlist with this artist").size(12))
                        .on_press(Message::CreatePlaylistFromContext(artist_name.clone(), true))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    (title, Some(hide.into()), create)
                }
                ContextMenuTarget::Album(album_name) => {
                    let title = format!("Album Menu: {album_name}");
                    let hide = button(text("Hide from UI").size(13))
                        .on_press(Message::HideAlbumOrArtist(album_name.clone(), false))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    for pl in &custom_playlists {
                        let album_tracks: Vec<Track> = self.all_tracks.iter()
                            .filter(|t| {
                                let al = if t.album.trim().is_empty() { "Unknown Album" } else { &t.album };
                                al == album_name
                            })
                            .cloned()
                            .collect();
                        playlist_select = playlist_select.push(
                            button(text(format!("  + {}", pl)).size(12))
                                .on_press(Message::AddTracksToPlaylist(pl.clone(), album_tracks))
                                .style(item_style)
                                .padding([4, 8])
                                .width(Length::Fill)
                        );
                    }

                    let create = button(text("+ Create playlist with this album").size(12))
                        .on_press(Message::CreatePlaylistFromContext(album_name.clone(), false))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    (title, Some(hide.into()), create)
                }
                ContextMenuTarget::Track(track) => {
                    let title = format!("Song Menu: {}", track.title);
                    
                    for pl in &custom_playlists {
                        playlist_select = playlist_select.push(
                            button(text(format!("  + {}", pl)).size(12))
                                .on_press(Message::AddTracksToPlaylist(pl.clone(), vec![track.clone()]))
                                .style(item_style)
                                .padding([4, 8])
                                .width(Length::Fill)
                        );
                    }

                    let create = button(text("+ Create playlist with this song").size(12))
                        .on_press(Message::CreatePlaylistWithTracks(track.title.clone(), vec![track.clone()]))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let like_label = if track.liked { "Unlike this song" } else { "Like this song" };
                    let like_btn = button(text(like_label).size(12))
                        .on_press(Message::ToggleLikeTrack(track.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let tag_btn = button(text("Edit ID3 tag").size(12))
                        .on_press(Message::OpenTagEditor(vec![track.clone()]))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let folder_btn = button(text("Open local file folder").size(12))
                        .on_press(Message::OpenLocalFolder(track.path.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let track_actions = column![
                        like_btn,
                        Space::with_height(4),
                        tag_btn,
                        Space::with_height(4),
                        folder_btn,
                    ];

                    (title, Some(track_actions.into()), create)
                }
                ContextMenuTarget::MultipleTracks(tracks) => {
                    let title = format!("Selection Menu: {} Songs", tracks.len());

                    for pl in &custom_playlists {
                        playlist_select = playlist_select.push(
                            button(text(format!("  + {}", pl)).size(12))
                                .on_press(Message::AddTracksToPlaylist(pl.clone(), tracks.clone()))
                                .style(item_style)
                                .padding([4, 8])
                                .width(Length::Fill)
                        );
                    }

                    let tag_btn = button(text("Edit ID3 tags").size(12))
                        .on_press(Message::OpenTagEditor(tracks.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let create = button(text("+ Create playlist with selection").size(12))
                        .on_press(Message::CreatePlaylistWithTracks("Selected Tracks Playlist".to_string(), tracks.clone()))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let selection_actions = column![
                        tag_btn,
                    ];

                    (title, Some(selection_actions.into()), create)
                }
                ContextMenuTarget::Header(clicked_col) => {
                    let title = "Table Columns".to_string();
                    let active_cols = crate::db::get(|db| db.table_columns.clone());
                    
                    let mut cols_col = column![
                        text("Show / Hide:")
                            .size(11)
                            .color(theme::subtext())
                            .font(crate::ui::icons::UI_FONT_BOLD),
                        Space::with_height(4)
                    ].spacing(4);

                    for &col in crate::db::TableColumn::all() {
                        let is_visible = active_cols.contains(&col);
                        let col_label = col.label();
                        
                        let icon_str = if is_visible { " " } else { " " };
                        let btn = button(
                            row![
                                text(icon_str)
                                    .font(crate::ui::icons::NERD_FONT_MONO)
                                    .color(if is_visible { theme::accent() } else { theme::overlay0() })
                                    .size(14),
                                text(col_label).size(13).color(theme::text())
                            ].spacing(8)
                        )
                        .on_press(Message::ToggleColumnVisibility(col))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                        cols_col = cols_col.push(btn);
                    }

                    let move_left_btn = button(text("<- Move Column Left").size(12))
                        .on_press(Message::MoveColumnLeft(*clicked_col))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                        
                    let move_right_btn = button(text("-> Move Column Right").size(12))
                        .on_press(Message::MoveColumnRight(*clicked_col))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let header_actions = column![
                        text(format!("Modify Column: {}", clicked_col.label()))
                            .size(11)
                            .color(theme::subtext())
                            .font(crate::ui::icons::UI_FONT_BOLD),
                        Space::with_height(4),
                        move_left_btn,
                        Space::with_height(4),
                        move_right_btn,
                        Space::with_height(8),
                    ];

                    playlist_select = cols_col;
                    
                    let dummy_create = button(text(""))
                        .style(iced::widget::button::text)
                        .padding(0);

                    (title, Some(header_actions.into()), dummy_create)
                }
            };

            playlist_select = playlist_select.push(Space::with_height(4)).push(create_btn);

            let mut menu_col = column![
                row![
                    text(title)
                        .size(14)
                        .font(crate::ui::icons::UI_FONT_BOLD)
                        .color(theme::accent()),
                    Space::with_width(Length::Fill),
                    button(text("\u{f00d}").font(crate::ui::icons::NERD_FONT_MONO).color(theme::red()).size(14))
                        .on_press(Message::ToggleContextMenu(None))
                        .style(iced::widget::button::text)
                ]
                .align_y(Alignment::Center),
                Space::with_height(8),
            ];

            if let Some(hide) = hide_btn {
                menu_col = menu_col.push(hide).push(Space::with_height(6));
            }

            let menu_content = menu_col.push(playlist_select)
                .spacing(6)
                .padding(16);

            let menu_card = container(menu_content)
                .width(260)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::mantle())),
                    border: iced::Border {
                        color: theme::accent(),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: theme::base(),
                        offset: [0.0, 4.0].into(),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                });

            let full_overlay = container(menu_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                    ..Default::default()
                });

            view_stack = view_stack.push(full_overlay);
        }

        view_stack.into()
    }

    fn shortcuts_modal_view(&self) -> Element<'_, Message> {
        let title = text("Keyboard Shortcuts")
            .size(20)
            .font(crate::ui::icons::UI_FONT_BOLD)
            .color(theme::accent());

        let row_item = |keys: &'static str, desc: &'static str| {
            row![
                text(keys)
                    .width(Length::Fixed(120.0))
                    .font(crate::ui::icons::UI_FONT_BOLD)
                    .color(theme::accent())
                    .size(13),
                text(desc)
                    .color(theme::text())
                    .size(13),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        };

        let content = column![
            row![
                title,
                Space::with_width(Length::Fill),
                button(
                    text("\u{f00d}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(theme::red())
                        .size(16)
                )
                .on_press(Message::CloseShortcuts)
                .style(iced::widget::button::text)
            ]
            .align_y(Alignment::Center),
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
            row_item("Ctrl + + / -", "Increase / Decrease Scaling"),
            row_item("Right/Left", "Seek Forward / Backward"),
            row_item("Tab", "Focus next field / cycle ID3 inputs"),
            row_item("Shift + Tab", "Cycle ID3 input backwards"),
            row_item("/", "Focus song search input"),
        ]
        .spacing(10)
        .padding(24);

        let dialog = container(content)
            .width(420)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::base())),
                border: iced::Border {
                    color: theme::accent(),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: theme::mantle(),
                    offset: [0.0, 4.0].into(),
                    blur_radius: 12.0,
                },
                ..Default::default()
            });

        container(dialog)
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

    fn subscription(&self) -> Subscription<Message> {
        let base = Subscription::batch([
            iced::time::every(Duration::from_millis(100)).map(|_| Message::PollAudio),
            iced::time::every(Duration::from_millis(33)).map(|_| Message::PollSpectrum),
            iced::time::every(Duration::from_secs(3)).map(|_| Message::CheckTheme),
            iced::keyboard::on_key_press(|key, _mods| {
                Some(Message::KeyPressed(key))
            }),
            iced::event::listen_with(|event, _, _| {
                match event {
                    iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(mods)) => {
                        Some(Message::ModifiersChanged(mods))
                    }
                    iced::Event::Window(iced::window::Event::Resized(size)) => {
                        Some(Message::WindowResized(size.width as f32, size.height as f32))
                    }
                    _ => None,
                }
            }),
        ]);

        let mut subs = vec![base];

        if self.dragging_sidebar {
            subs.push(iced::event::listen_with(|event, _, _| {
                use iced::mouse;
                match event {
                    iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::SidebarDragMove(position.x))
                    }
                    iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::SidebarDragEnd)
                    }
                    _ => None,
                }
            }));
        }

        if self.dragging_playlist_split {
            subs.push(iced::event::listen_with(|event, _, _| {
                use iced::mouse;
                match event {
                    iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::PlaylistDragMove(position.y))
                    }
                    iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::PlaylistDragEnd)
                    }
                    _ => None,
                }
            }));
        }

        if self.dragging_right_panel {
            subs.push(iced::event::listen_with(|event, _, _| {
                use iced::mouse;
                match event {
                    iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::RightPanelDragMove(position.x))
                    }
                    iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::RightPanelDragEnd)
                    }
                    _ => None,
                }
            }));
        }

        Subscription::batch(subs)
    }

    fn header_view(&self) -> Element<'_, Message> {
        container(
            row![
                text(crate::ui::icons::ICON_MUSIC)
                    .font(crate::ui::icons::NERD_FONT_MONO)
                    .color(theme::accent())
                    .size(16),
                Space::with_width(6),
                text("omatunes")
                    .color(theme::accent())
                    .size(16)
                    .font(crate::ui::icons::UI_FONT_BOLD),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::header)
        .width(Length::Fill)
        .padding([0, 16])
        .into()
    }

    fn advance_track(&mut self, delta: i32) -> Task<Message> {
        if self.queue.is_empty() {
            return Task::none();
        }

        let next_idx = if self.shuffle {
            use rand::Rng;
            let current_idx = self.current_track.as_ref()
                .and_then(|ct| self.queue.iter().position(|t| t.id == ct.id));
            let len = self.queue.len();
            if len == 1 { 0 } else {
                let mut rng = rand::thread_rng();
                let mut idx = rng.gen_range(0..len);
                if let Some(cur) = current_idx {
                    while idx == cur { idx = rng.gen_range(0..len); }
                }
                idx
            }
        } else {
            let current_idx = self.current_track.as_ref()
                .and_then(|ct| self.queue.iter().position(|t| t.id == ct.id));
            match current_idx {
                Some(i) => {
                    let new = i as i32 + delta;
                    if new < 0 { self.queue.len() - 1 } else { new as usize % self.queue.len() }
                }
                None => 0,
            }
        };

        if let Some(track) = self.queue.get(next_idx).cloned() {
            self.play_track_internal(track)
        } else {
            Task::none()
        }
    }

    pub fn calculate_scroll_offset(&self, track_id: i64) -> Option<f32> {
        let track_height = 34.0;
        let spacing = 1.0;
        if self.group_by_album {
            let mut y = 0.0;
            let mut groups: Vec<(String, Vec<&crate::library::models::Track>)> = Vec::new();
            for track in &self.tracks {
                if let Some(last) = groups.last_mut() {
                    if last.0 == track.album {
                        last.1.push(track);
                        continue;
                    }
                }
                groups.push((track.album.clone(), vec![track]));
            }
            for (_album_name, tracks) in groups {
                let header_height = 28.0;
                if tracks.iter().any(|t| t.id == track_id) {
                    let index_in_album = tracks.iter().position(|t| t.id == track_id).unwrap();
                    y += header_height + spacing;
                    y += index_in_album as f32 * (track_height + spacing);
                    return Some(y);
                } else {
                    y += header_height + spacing;
                    y += tracks.len() as f32 * (track_height + spacing);
                    y += 8.0 + spacing;
                }
            }
        } else {
            if let Some(idx) = self.tracks.iter().position(|t| t.id == track_id) {
                return Some(idx as f32 * (track_height + spacing));
            }
        }
        None
    }

    fn play_track_internal(&mut self, track: Track) -> Task<Message> {
        let cover_data = load_cover(&track.path);
        let track = Track { cover_data, ..track };
        self.audio.send(AudioCommand::Play(track.path.clone()));
        self.audio.send(AudioCommand::SetVolume(self.volume));
        self.current_track = Some(track.clone());
        self.selected_track = Some(track.clone());
        self.playback_state = PlaybackState::Playing;
        self.position = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.current_track_play_counted = false;
        self.notify_mpris_track(PlaybackStatus::Playing);

        crate::db::add_to_recently_played(track.path.clone());
        if self.selected_playlist.as_deref() == Some("Recently Played") {
            self.update_filtered_tracks();
        }

        if let Some(y) = self.calculate_scroll_offset(track.id) {
            let target_y = (y - 120.0).max(0.0);
            iced::widget::scrollable::scroll_to(
                iced::widget::scrollable::Id::new("tracklist_scroll"),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target_y }
            )
        } else {
            Task::none()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn music_subfolders(music_dir: &PathBuf) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = std::fs::read_dir(music_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && !e.file_name().to_string_lossy().starts_with('.')
        })
        .map(|e| e.path())
        .collect();
    folders.sort();
    folders
}

fn sidebar_width_path() -> PathBuf {
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())));
    PathBuf::from(xdg).join("omatunes").join("sidebar_width")
}

fn load_sidebar_width() -> f32 {
    std::fs::read_to_string(sidebar_width_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(200.0)
}

fn save_sidebar_width(width: f32) {
    let path = sidebar_width_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    std::fs::write(path, width.to_string()).ok();
}

fn right_panel_width_path() -> PathBuf {
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())));
    PathBuf::from(xdg).join("omatunes").join("right_panel_width")
}

fn load_right_panel_width() -> f32 {
    std::fs::read_to_string(right_panel_width_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(400.0)
}

fn save_right_panel_width(width: f32) {
    let path = right_panel_width_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    std::fs::write(path, width.to_string()).ok();
}

fn build_iced_theme() -> Theme {
    Theme::custom(
        "Omarchy".into(),
        iced::theme::Palette {
            background: theme::base(),
            text:       theme::text(),
            primary:    theme::accent(),
            success:    theme::green(),
            danger:     theme::red(),
        },
    )
}

// ── Ponto de entrada iced ─────────────────────────────────────────────────────

pub fn run() -> iced::Result {
    iced::application("omatunes", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .default_font(iced::Font {
            family: iced::font::Family::Name("JetBrainsMono Nerd Font Mono"),
            weight: iced::font::Weight::Normal,
            stretch: iced::font::Stretch::Normal,
            style: iced::font::Style::Normal,
        })
        .theme(|state: &AppState| state.iced_theme.clone())
        .scale_factor(|state: &AppState| state.font_scale as f64)
        .window(iced::window::Settings {
            size: iced::Size::new(960.0, 640.0),
            min_size: Some(iced::Size::new(700.0, 480.0)),
            ..Default::default()
        })
        .run_with(AppState::new)
}
