use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use iced::widget::{button, container, column, row, text, Space, stack, scrollable, mouse_area};
use iced::{Alignment, Element, Length, Subscription, Task, Theme};
use mpris_server::{LoopStatus, PlaybackStatus};

use serde::{Serialize, Deserialize};

use crate::audio::{AudioCommand, AudioEvent, AudioPlayer, MprisCommand, MprisUpdate, PlaybackState};
use crate::audio::mpris;
use crate::audio::spectrum::SpectrumAnalyzer;
use crate::library::models::Track;
use crate::library::{load_cover, scan_folder};
use crate::ui::{theme, views};

pub const MIN_SIDEBAR_WIDTH: f32 = 180.0;
pub const MAX_SIDEBAR_WIDTH: f32 = 400.0;

pub const MIN_PLAYLIST_HEIGHT: f32 = 80.0;

pub const MIN_VOLUME_SLIDER_WIDTH: f32 = 80.0;
pub const MAX_VOLUME_SLIDER_WIDTH: f32 = 150.0;

// fixed elements in player: cover (216) + spacing (16) + playback controls (460) + volume icon & spacing & padding (64) = 756.0
pub const PLAYER_FIXED_WIDTH: f32 = 756.0;

// Minimum space allocated to left side player controls when right panel is open:
// PLAYER_FIXED_WIDTH + MIN_VOLUME_SLIDER_WIDTH = 836.0.
// Plus separator (1.0) + tab_strip (56.0) + drag_handle (6.0) = 63.0. Total: 899.0
pub const MIN_NON_DRAWER_WIDTH: f32 = 899.0;

#[derive(Debug, Clone)]
pub enum ContextMenuTarget {
    Artist(String),
    Album(String),
    Track(Track),
    MultipleTracks(Vec<Track>),
    Header(crate::db::TableColumn),
    Playlist(String),
    SmartPlaylist(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Artists,
    Albums,
    Genres,
    NowPlaying,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RightPanelTab {
    Visualizer,
    Statistics,
    Lyrics,
}

#[derive(Debug, Clone)]
pub struct StatsNotification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub created_at: std::time::Instant,
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
    Liked,
}

#[derive(Debug, Clone)]
pub enum PlaylistDialogMode {
    Create,
    AddTrack(Track),
    CreateWithTrack(Track),
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
    PlayAlbum(String),
    ToggleAlbumPlayPause(String),
    PlayPause,
    ToggleLikeCurrent,
    NextTrack,
    PreviousTrack,
    Seek(Duration),
    VolumeChanged(f32),
    HoverAlbumHeader(Option<String>),
    IncreaseScale,
    DecreaseScale,
    TracklistScrolled(iced::widget::scrollable::Viewport),
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
    CancelTagEditor,
    SearchCoverOnline,
    UpdateTagFieldTitle(String),
    UpdateTagFieldArtist(String),
    UpdateTagFieldAlbum(String),
    UpdateTagFieldGenre(usize, String),
    UpdateTagFieldTrackNumber(String),
    UpdateTagFieldDiscNumber(String),
    UpdateTagFieldCoverPath(String),
    UpdateTagFieldApplyToAlbum(bool),
    UpdateTagFieldYear(String),
    ToggleTagFieldApplyTitle(bool),
    ToggleTagFieldApplyArtist(bool),
    ToggleTagFieldApplyAlbum(bool),
    ToggleTagFieldApplyYear(bool),
    ToggleTagFieldApplyGenre(usize, bool),
    ToggleTagFieldApplyTrackNum(bool),
    ToggleTagFieldApplyDiscNum(bool),
    ToggleTagFieldApplyCover(bool),
    SelectTagEditorTab(TagEditorTab),
    UpdateTagFieldLyrics(iced::widget::text_editor::Action),
    ToggleTagFieldApplyLyrics(bool),
    SearchLyricsOnline,
    ChangePendingLyricOffset(f64),
    ApplyPendingLyricOffset,
    ResetPendingLyricOffset,
    SaveTags,
    TagEditorPrevTrack,
    TagEditorNextTrack,
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
    PlaylistCreateWithTrack(Track),
    WindowResized(f32, f32),
    HoverTracklist(bool),
    HoverSidebarList(bool),
    KeyboardArrowUp,
    KeyboardArrowDown,
    DeletePlaylist(String),
    RenamePlaylist(String, String),
    GroupByHoverEnter,
    GroupByHoverExit,
    GroupByCollapseTimeout(u32),
    GroupBySelected(crate::db::GroupBy),
    GroupByCleared,
    GroupByAnimationTick(std::time::Instant),
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
    RemoveTrackFromPlaylist(String, Track),
    TogglePlaylistMenuExpanded,
    CreatePlaylistWithTracks(String, Vec<Track>),
    ToggleColumnVisibility(crate::db::TableColumn),
    MoveColumnLeft(crate::db::TableColumn),
    MoveColumnRight(crate::db::TableColumn),
    SelectPlaylistTab(PlaylistTab),
    ToggleRightPanelTab(RightPanelTab),
    ToggleSongSearch,
    ToggleSidebarSearch,
    GlobalCursorMoved(iced::Point),
    GlobalClick,
    DismissNotification(u64),
    ShowPeriodBreakdown(usize),
    ClosePeriodBreakdown,
    SelectArtistFromBreakdown(String),
    SelectAlbumFromBreakdown(String),
    SelectGenreFromBreakdown(String),
    CloseBreakdownSongView,
    SelectStatsModalTab(StatsModalTab),

    SelectAchievementsSubTab(AchievementsSubTab),
    SelectAchievementsSort(AchievementsSort),
    ShowMoreAchievements,
    ShowPreviousAchievements,
    AchievementsSearchChanged(String),
    Noop,

    OpenSettings,
    CloseSettings,
    SettingsMusicDirChanged(String),
    SettingsLanguageChanged(String),
    SettingsSeekStepChanged(String),
    SettingsVolumeStepChanged(f32),
    SettingsFontScaleChanged(f32),
    SettingsShowAchievementsInUiChanged(bool),
    SettingsSave,
    SettingsThemeSourceChanged(String),
    SettingsThemePresetChanged(String),
    SettingsCustomColorChanged(String, String),
    SettingsColorPickerToggle(String),
    SettingsColorPickerRChanged(f32),
    SettingsColorPickerGChanged(f32),
    SettingsColorPickerBChanged(f32),
    SettingsTabChanged(SettingsTab),
    SettingsInitialVolumeChanged(f32),
    SettingsPlaybackDefaultChanged(String, String, bool),
    SettingsAutoScanModeChanged(String),
    SettingsAutoScanIntervalChanged(String),
    PickMusicFolder,
    MusicFolderPicked(Option<std::path::PathBuf>),

    PlayNext(Vec<Track>),
    AddToQueue(Vec<Track>),
    PlayQueueTrack(usize),
    SelectQueueTrack(usize, Track),
    RemoveQueueTrack(usize),
    MoveQueueTrackUp(usize),
    MoveQueueTrackDown(usize),
    ClearQueue,
    QueueDragStart(usize),
    QueueDragOver(usize),
    QueueDragEnd,
    PlaylistSidebarDragStart(PlaylistTab, usize),
    PlaylistSidebarDragOver(PlaylistTab, usize),
    PlaylistSidebarDragEnd,
    TrackListDragStart(usize),
    TrackListDragOver(usize),
    TrackListDragEnd,
    ResetPlaylistSongOrder,
    ColumnHeaderDragStart(crate::db::TableColumn),
    ColumnHeaderDragOver(crate::db::TableColumn),
    ColumnHeaderDragEnd,

    NewSmartPlaylist,
    EditSmartPlaylist(String),
    DeleteSmartPlaylist(String),
    SmartPlaylistBuilderMsg(SmartPlaylistBuilderEvent),

    PlayerDragStart,
    PlayerDragMove(f32),
    PlayerDragEnd,
    HoverPlayerResizer(bool),
    ToggleQueuePopover,
    CloseQueuePopover,
    FlushBuffers,
    CoverLoaded(i64, Option<Vec<u8>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    Library,
    Playback,
    Display,
    Theme,
    Shortcuts,
}

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub music_dir: String,
    pub language: String,
    pub seek_step: String,
    pub volume_step: f32,
    pub font_scale: f32,
    pub initial_volume: f32,
    pub playback_defaults: crate::config::PlaybackDefaults,
    pub auto_scan: crate::config::AutoScanConfig,
    pub theme_source: String,
    pub theme_preset: String,
    pub custom_theme: crate::config::CustomThemeConfig,
    pub custom_validation_errors: std::collections::HashMap<String, String>,
    pub confirm_save_anyway: bool,
    pub selected_tab: SettingsTab,
    pub color_picker_token: Option<String>,
    pub color_picker_r: f32,
    pub color_picker_g: f32,
    pub color_picker_b: f32,
    pub show_achievements_in_ui: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayingContext {
    Playlist(String),
    SmartPlaylist(String),
    Artist(String),
    Album(String),
    Autoplaylist(String),
    Genre(String),
}

#[derive(Debug, Clone)]
pub struct SavedViewState {
    pub view_mode: ViewMode,
    pub selected_playlist: Option<String>,
    pub selected_artist: Option<String>,
    pub selected_album: Option<String>,
    pub selected_genre: Option<String>,
    pub playlist_tab: PlaylistTab,
}

#[derive(Debug, Clone)]
pub enum SmartPlaylistBuilderEvent {
    NameChanged(String),
    AddRule,
    RemoveRule(usize),
    UpdateRuleField(usize, crate::library::smart_playlist::RuleField),
    UpdateRuleOperator(usize, crate::library::smart_playlist::RuleOperator),
    UpdateRuleValue(usize, String),
    UpdateRuleValue2(usize, String),
    UpdateRuleDateUnit(usize, crate::library::smart_playlist::DateUnit),
    UpdateRuleBoolean(usize, bool),
    ToggleLimit(bool),
    LimitStrChanged(String),
    UpdateOrderBy(crate::library::smart_playlist::SmartPlaylistOrder),
    ToggleLive(bool),
    Save,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistTab {
    Playlists,
    Autoplaylists,
    Smart,
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
    pub original_tracks: std::collections::HashMap<std::path::PathBuf, Track>,
    pub is_saved: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genres: Vec<String>,
    pub genres_original: Vec<String>,
    pub track_number: String,
    pub disc_number: String,
    pub cover_path: Option<String>,
    pub apply_to_album: bool,
    pub year: String,
    pub apply_title: bool,
    pub apply_artist: bool,
    pub apply_album: bool,
    pub apply_year: bool,
    pub apply_genres: Vec<bool>,
    pub apply_track_num: bool,
    pub apply_disc_num: bool,
    pub apply_cover: bool,
    pub apply_lyrics: bool,
    pub lyrics: String,
    pub lyrics_content: iced::widget::text_editor::Content,
    pub active_tab: TagEditorTab,
    pub focused_field: Option<usize>,
    pub pending_offset: f64,
}

impl Clone for TagEditorState {
    fn clone(&self) -> Self {
        TagEditorState {
            tracks: self.tracks.clone(),
            original_tracks: self.original_tracks.clone(),
            is_saved: self.is_saved,
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            genres: self.genres.clone(),
            genres_original: self.genres_original.clone(),
            track_number: self.track_number.clone(),
            disc_number: self.disc_number.clone(),
            cover_path: self.cover_path.clone(),
            apply_to_album: self.apply_to_album,
            year: self.year.clone(),
            apply_title: self.apply_title,
            apply_artist: self.apply_artist,
            apply_album: self.apply_album,
            apply_year: self.apply_year,
            apply_genres: self.apply_genres.clone(),
            apply_track_num: self.apply_track_num,
            apply_disc_num: self.apply_disc_num,
            apply_cover: self.apply_cover,
            apply_lyrics: self.apply_lyrics,
            lyrics: self.lyrics.clone(),
            lyrics_content: iced::widget::text_editor::Content::with_text(&self.lyrics_content.text()),
            active_tab: self.active_tab,
            focused_field: self.focused_field,
            pending_offset: self.pending_offset,
        }
    }
}


pub struct CoverCache {
    pub id: Option<i64>,
    pub version: u64,
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
    pub tracks: Arc<Vec<Track>>,
    folder_cache: HashMap<PathBuf, Vec<Track>>,

    pub sidebar_width: f32,
    pub dragging_sidebar: bool,

    pub player_height: f32,
    pub dragging_player_split: bool,
    pub is_hovering_player_resizer: bool,

    pub right_panel_width: f32,
    pub right_panel_width_initialized: bool,
    pub dragging_right_panel: bool,
    pub is_hovering_right_panel_resizer: bool,
    pub window_width: f32,

    pub iced_theme: iced::Theme,
    loaded_theme_name: String,

    pub strings: &'static crate::locale::Strings,

    // Omatunes feature additions
    pub all_tracks: Arc<Vec<Track>>,
    pub search_query: String,
    pub filter_title: bool,
    pub filter_artist: bool,
    pub filter_album: bool,
    pub filter_genre: bool,
    pub selected_playlist: Option<String>,
    pub show_tag_editor: Option<TagEditorState>,
    pub show_settings: Option<SettingsState>,

    // Omatunes enhancements
    pub dragging_queue_index: Option<usize>,
    pub dragging_playlist_sidebar: Option<(PlaylistTab, usize)>,
    pub dragging_track_index: Option<usize>,
    pub dragging_column_header: Option<crate::db::TableColumn>,
    pub column_drag_moved: bool,
    pub last_browsing_view: ViewMode,
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
    pub group_by: crate::db::GroupBy,
    pub group_by_state: GroupByControlState,
    pub sidebar_search: String,
    pub show_shortcuts: bool,

    pub last_click_track: Option<(i64, std::time::Instant)>,
    pub last_click_artist: Option<(String, std::time::Instant)>,
    pub last_click_album: Option<(String, std::time::Instant)>,
    pub last_click_playlist: Option<(String, std::time::Instant)>,
    pub last_click_genre: Option<(String, std::time::Instant)>,

    pub hovered_playlist: Option<String>,
    pub show_context_menu: Option<ContextMenuTarget>,
    pub playlist_menu_expanded: bool,
    pub modifiers: iced::keyboard::Modifiers,
    pub selected_tracks: Arc<Vec<Track>>,
    pub last_clicked_track: Option<Track>,
    pub hidden_artists_albums: Vec<(String, bool)>,       // (Name, IsArtistOrAlbum)

    pub playlist_tab: PlaylistTab,
    pub right_panel_tab: Option<RightPanelTab>,
    pub right_panel_tab_user_scrolled: bool,
    pub show_song_search: bool,
    pub show_sidebar_search: bool,
    pub cursor_position: iced::Point,
    pub lyrics_scroll_id: scrollable::Id,
    pub last_active_lyric_idx: Option<usize>,
    pub spectrum_bands: [f32; crate::audio::spectrum::NUM_BANDS],
    spectrum_analyzer: SpectrumAnalyzer,
    audio: AudioPlayer,
    mpris_cmd_rx: tokio::sync::mpsc::UnboundedReceiver<MprisCommand>,
    mpris_update_tx: tokio::sync::mpsc::UnboundedSender<MprisUpdate>,
    pub cover_cache: std::sync::Mutex<CoverCache>,
    pub cover_cache_version: u64,
    pub font_scale: f32,
    pub hovered_album_header: Option<String>,
    pub track_list_start: usize,
    pub track_list_end: usize,
    pub smart_playlist_builder: Option<crate::ui::components::smart_playlist_builder::SmartPlaylistBuilderState>,
    pub previous_view_state: Option<SavedViewState>,
    pub playing_context: Option<PlayingContext>,
    pub animation_tick: u32,
    pub show_queue_popover: bool,
    pub queue_scroll_id: scrollable::Id,
    pub last_accumulated_position: Duration,
    pub show_period_breakdown: Option<crate::stats::PeriodBreakdown>,
    pub breakdown_period_idx: usize,
    pub breakdown_song_view: Option<(String, String)>,
    pub active_notifications: Vec<StatsNotification>,
    pub next_notification_id: u64,
    pub last_checked_hour: Option<u32>,
    pub stats_modal_tab: StatsModalTab,

    pub achievements_sub_tab: AchievementsSubTab,
    pub achievements_sort: AchievementsSort,
    pub achievements_offset: usize,
    pub achievements_search_query: String,
    pub achievements_cover_cache: std::sync::Mutex<std::collections::HashMap<String, iced::widget::image::Handle>>,
    pub achievements_items: Vec<AchievementItem>,
}

#[derive(Debug, Clone)]
pub struct AchievementItem {
    pub name: String,
    pub plays: u32,
    pub highest_tier_score: u32,
    pub num_awards: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsModalTab {
    Leaderboard,
    Achievements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementsSubTab {
    Artists,
    Albums,
    Genres,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementsSort {
    Alphabetical,
    AchievementLevel,
}

pub struct GroupByControlState {
    pub active: crate::db::GroupBy,
    pub hover_progress: f32,
    pub is_cluster_hovered: bool,
    pub force_collapsing: bool,
    pub collapse_deadline: Option<std::time::Instant>,
    pub collapse_token: u32,
}

impl GroupByControlState {
    pub fn target(&self) -> f32 {
        let should_be_expanded = self.is_cluster_hovered
            || matches!(self.collapse_deadline, Some(d) if std::time::Instant::now() < d);
        if self.force_collapsing {
            0.0
        } else if should_be_expanded {
            1.0
        } else {
            0.0
        }
    }
}

impl AppState {
    pub fn show_achievements_in_ui(&self) -> bool {
        crate::config::get().show_achievements_in_ui
    }

    pub fn recalculate_achievements_items(&mut self) {
        let achievements = crate::stats::get(|db| db.earned_achievements.clone());
        let mut items = Vec::new();
        let entity_type_str = match self.achievements_sub_tab {
            AchievementsSubTab::Artists => "Artist",
            AchievementsSubTab::Albums => "Album",
            AchievementsSubTab::Genres => "Genre",
        };

        let mut unique_names = std::collections::HashSet::new();
        for track in self.all_tracks.iter() {
            match self.achievements_sub_tab {
                AchievementsSubTab::Artists => {
                    if !track.artist.trim().is_empty() {
                        unique_names.insert(track.artist.clone());
                    }
                }
                AchievementsSubTab::Albums => {
                    if !track.album.trim().is_empty() {
                        unique_names.insert(track.album.clone());
                    }
                }
                AchievementsSubTab::Genres => {
                    if !track.genre.trim().is_empty() {
                        let parts = if track.genre.contains("; ") {
                            track.genre.split("; ").map(|g| g.trim().to_string()).collect::<Vec<_>>()
                        } else {
                            vec![track.genre.trim().to_string()]
                        };
                        for p in parts {
                            if !p.is_empty() && p != "Unknown" {
                                unique_names.insert(p);
                            }
                        }
                    }
                }
            }
        }

        let mut play_map = std::collections::HashMap::new();
        for track in self.all_tracks.iter() {
            match self.achievements_sub_tab {
                AchievementsSubTab::Artists => {
                    *play_map.entry(track.artist.clone()).or_insert(0) += track.play_count;
                }
                AchievementsSubTab::Albums => {
                    *play_map.entry(track.album.clone()).or_insert(0) += track.play_count;
                }
                AchievementsSubTab::Genres => {
                    let parts = if track.genre.contains("; ") {
                        track.genre.split("; ").map(|g| g.trim().to_string()).collect::<Vec<_>>()
                    } else {
                        vec![track.genre.trim().to_string()]
                    };
                    for p in parts {
                        if !p.is_empty() && p != "Unknown" {
                            *play_map.entry(p).or_insert(0) += track.play_count;
                        }
                    }
                }
            }
        }

        for name in unique_names {
            let plays = play_map.get(&name).copied().unwrap_or(0);
            if plays == 0 {
                continue;
            }

            let entity_awards: Vec<_> = achievements.iter()
                .filter(|a| a.entity_type == entity_type_str && a.entity_name == name)
                .collect();
            let num_awards = entity_awards.len();
            let highest_tier_score = entity_awards.iter()
                .map(|a| crate::stats::get_achievement_score(&a.period, &a.tier))
                .max()
                .unwrap_or(0);

            items.push(AchievementItem {
                name,
                plays,
                highest_tier_score,
                num_awards,
            });
        }

        match self.achievements_sort {
            AchievementsSort::Alphabetical => {
                items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            AchievementsSort::AchievementLevel => {
                items.sort_by(|a, b| {
                    b.highest_tier_score.cmp(&a.highest_tier_score)
                        .then(b.num_awards.cmp(&a.num_awards))
                        .then(b.plays.cmp(&a.plays))
                        .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
            }
        }

        self.achievements_items = items;
    }

    pub fn is_draggable_playlist_view(&self) -> bool {
        match &self.selected_playlist {
            Some(name) => {
                name != "Recently Played" && name != "Most Played"
                    && (
                        crate::db::get(|db| db.playlists.contains_key(name.as_str()))
                        || crate::db::get(|db| db.smart_playlists.contains_key(name.as_str()))
                        || name == "Liked Songs"
                        || name == "New Music"
                    )
            }
            None => false,
        }
    }

    pub fn get_display_cover(&self) -> Option<iced::widget::image::Handle> {
        let is_playing_or_paused = !matches!(self.playback_state, PlaybackState::Stopped);
        let display_track = if is_playing_or_paused {
            self.current_track.as_ref()
        } else {
            self.selected_track.as_ref()
        };
        let track_id = display_track.map(|t| t.id);
        let cache_key = (track_id, self.cover_cache_version);
        
        let mut cache = self.cover_cache.lock().unwrap();
        if cache_key != (cache.id, cache.version) {
            cache.id = track_id;
            cache.version = self.cover_cache_version;
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
        let _ = mpris_update_tx.send(crate::audio::mpris::MprisUpdate::Volume(cfg.volume.clamp(0.0, 1.0) as f64));

        let loaded_theme_name = crate::ui::theme::read_current_theme_name();
        let iced_theme = build_iced_theme();

        let (db_sidebar_width, db_playlist_height, db_right_panel_width, db_right_panel_tab, db_player_height) = crate::db::get(|db| (
            db.sidebar_width,
            db.playlist_height,
            db.right_panel_width,
            db.right_panel_tab,
            db.player_height,
        ));
        
        crate::db::write(|db| {
            if db.playlist_order.is_empty() && !db.playlists.is_empty() {
                let mut names: Vec<String> = db.playlists.keys().cloned().collect();
                names.sort();
                db.playlist_order = names;
            }
            if db.smart_playlist_order.is_empty() && !db.smart_playlists.is_empty() {
                let mut names: Vec<String> = db.smart_playlists.keys().cloned().collect();
                names.sort();
                db.smart_playlist_order = names;
            }
        });

        let music_dir = cfg.music_path();
        let scan_task = Task::perform(
            async move {
                scan_folder(&music_dir)
            },
            Message::LibraryScanned,
        );

        let db_group_by = crate::db::get(|db| db.group_by.unwrap_or(crate::db::GroupBy::None));

        let state = AppState {
            playback_state: PlaybackState::Stopped,
            current_track: None,
            queue: Vec::new(),
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: cfg.volume.clamp(0.0, 1.0),
            shuffle: cfg.playback_defaults.album.shuffle,
            repeat: cfg.playback_defaults.album.repeat,
            folders,
            selected_folder: None,
            tracks: Arc::new(Vec::new()),
            folder_cache: HashMap::new(),

            sidebar_width: db_sidebar_width.unwrap_or(200.0).clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH),
            dragging_sidebar: false,
            player_height: db_player_height.unwrap_or(330.0).clamp(330.0, 458.0),
            dragging_player_split: false,
            is_hovering_player_resizer: false,
            right_panel_width: db_right_panel_width.unwrap_or(960.0f32 * 0.33).clamp(450.0, 960.0),
            right_panel_width_initialized: db_right_panel_width.is_some(),
            dragging_right_panel: false,
            is_hovering_right_panel_resizer: false,
            window_width: 960.0,
            iced_theme,
            loaded_theme_name,
            strings: crate::locale::get(),
            all_tracks: Arc::new(Vec::new()),
            search_query: String::new(),
            filter_title: true,
            filter_artist: true,
            filter_album: true,
            filter_genre: true,
            selected_playlist: None,
            show_tag_editor: None,
            show_settings: None,
            dragging_queue_index: None,
            dragging_playlist_sidebar: None,
            dragging_track_index: None,
            dragging_column_header: None,
            column_drag_moved: false,
            last_browsing_view: ViewMode::Artists,
            view_mode: ViewMode::Artists,
            selected_artist: None,
            selected_album: None,
            selected_genre: None,
            playlist_height: db_playlist_height.unwrap_or(114.0),
            playlist_height_initialized: db_playlist_height.is_some(),
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
            group_by: db_group_by,
            group_by_state: GroupByControlState {
                active: db_group_by,
                hover_progress: 0.0,
                is_cluster_hovered: false,
                force_collapsing: false,
                collapse_deadline: None,
                collapse_token: 0,
            },
            sidebar_search: String::new(),
            show_shortcuts: false,
            last_click_track: None,
            last_click_artist: None,
            last_click_album: None,
            last_click_playlist: None,
            last_click_genre: None,
            hovered_playlist: None,
            show_context_menu: None,
            playlist_menu_expanded: false,
            modifiers: Default::default(),
            selected_tracks: Arc::new(Vec::new()),
            last_clicked_track: None,
            hidden_artists_albums: crate::db::get(|db| db.hidden_artists_albums.clone()),
            playlist_tab: PlaylistTab::Playlists,
            right_panel_tab: db_right_panel_tab.and_then(|t| if t == RightPanelTab::Statistics { None } else { Some(t) }),
            right_panel_tab_user_scrolled: false,
            show_song_search: false,
            show_sidebar_search: false,
            cursor_position: iced::Point::ORIGIN,
            lyrics_scroll_id: scrollable::Id::unique(),
            last_active_lyric_idx: None,
            spectrum_bands: [0.0; crate::audio::spectrum::NUM_BANDS],
            spectrum_analyzer,
            audio,
            mpris_cmd_rx,
            mpris_update_tx,
            cover_cache_version: 0,
            cover_cache: std::sync::Mutex::new(CoverCache { id: None, version: 0, handle: None }),
            font_scale: cfg.font_scale(),
            hovered_album_header: None,
            track_list_start: 0,
            track_list_end: 500,
            smart_playlist_builder: None,
            previous_view_state: None,
            playing_context: None,
            animation_tick: 0,
            show_queue_popover: false,
            queue_scroll_id: scrollable::Id::unique(),
            last_accumulated_position: Duration::ZERO,
            show_period_breakdown: None,
            breakdown_period_idx: 0,
            breakdown_song_view: None,
            active_notifications: Vec::new(),
            next_notification_id: 0,
            last_checked_hour: {
                use chrono::Timelike;
                Some(chrono::Local::now().hour())
            },
            stats_modal_tab: StatsModalTab::Leaderboard,

            achievements_sub_tab: AchievementsSubTab::Artists,
            achievements_sort: AchievementsSort::AchievementLevel,
            achievements_offset: 0,
            achievements_search_query: String::new(),
            achievements_cover_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            achievements_items: Vec::new(),
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
                url: track.path.to_string_lossy().to_string(),
            });
        }
        self.send_mpris(MprisUpdate::Status(status));
    }

    pub fn artists(&self) -> Vec<String> {
        let query = self.sidebar_search.to_lowercase();
        let mut artists: Vec<String> = self.all_tracks.iter()
            .map(|t| if t.artist.trim().is_empty() { "Unknown Artist".to_string() } else { t.artist.clone() })
            .collect();
        artists.sort_by(|a, b| {
            let normalize = |s: &str| {
                let lower = s.to_lowercase();
                if lower.starts_with("the ") {
                    lower[4..].to_string()
                } else {
                    lower
                }
            };
            normalize(a).cmp(&normalize(b))
        });
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
            .flat_map(|t| {
                if t.genre.trim().is_empty() {
                    vec!["Unknown Genre".to_string()]
                } else if t.genre.contains("; ") {
                    t.genre.split("; ").map(|g| {
                        let trimmed = g.trim();
                        if trimmed.is_empty() { "Unknown Genre".to_string() } else { trimmed.to_string() }
                    }).collect()
                } else {
                    vec![t.genre.clone()]
                }
            })
            .collect();
        genres.sort();
        genres.dedup();
        if !query.is_empty() {
            genres.retain(|g| g.to_lowercase().contains(&query));
        }
        genres
    }

    pub fn load_track_in_tag_editor(&mut self, track: Track) {
        let active_tab = self.show_tag_editor.as_ref()
            .map(|state| state.active_tab)
            .unwrap_or(TagEditorTab::Main);

        let mut original_tracks = self.show_tag_editor.as_mut()
            .map(|state| std::mem::take(&mut state.original_tracks))
            .unwrap_or_default();

        if !original_tracks.contains_key(&track.path) {
            original_tracks.insert(track.path.clone(), track.clone());
        }

        let tracks = vec![track.clone()];
        let first = &tracks[0];
        let genre_vec: Vec<String> = first.genres().into_iter().map(|s| s.to_string()).collect();
        self.show_tag_editor = Some(TagEditorState {
            tracks: tracks.clone(),
            original_tracks,
            is_saved: false,
            title: first.title.clone(),
            artist: first.artist.clone(),
            album: first.album.clone(),
            genres: genre_vec.clone(),
            genres_original: genre_vec.clone(),
            apply_genres: vec![true; genre_vec.len()],
            track_number: first.track_number.map(|n| n.to_string()).unwrap_or_default(),
            disc_number: first.disc_number.map(|n| n.to_string()).unwrap_or_default(),
            cover_path: None,
            apply_to_album: false,
            year: first.year.map(|n| n.to_string()).unwrap_or_default(),
            apply_title: false,
            apply_artist: false,
            apply_album: false,
            apply_year: false,
            apply_track_num: false,
            apply_disc_num: false,
            apply_cover: false,
            apply_lyrics: false,
            lyrics: first.lyrics.clone(),
            lyrics_content: iced::widget::text_editor::Content::with_text(&first.lyrics),
            active_tab,
            focused_field: Some(0),
            pending_offset: 0.0,
        });
    }

    pub fn update_filtered_tracks(&mut self) {
        self.track_list_start = 0;
        self.track_list_end = 500;
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.tracks = Arc::new(self.all_tracks.iter().filter(|t| {
                let match_title = self.filter_title && t.title.to_lowercase().contains(&query);
                let match_artist = self.filter_artist && t.artist.to_lowercase().contains(&query);
                let match_album = self.filter_album && t.album.to_lowercase().contains(&query);
                let match_genre = self.filter_genre && t.genres().iter().any(|g| g.to_lowercase().contains(&query));
                match_title || match_artist || match_album || match_genre
            }).cloned().collect::<Vec<_>>());
        } else if let Some(playlist_name) = &self.selected_playlist {
            if playlist_name == "Liked Songs" {
                self.tracks = Arc::new(self.all_tracks.iter().filter(|t| t.liked).cloned().collect::<Vec<_>>());
            } else if playlist_name == "Most Played" {
                let mut temp = (*self.all_tracks).clone();
                temp.sort_by(|a, b| b.play_count.cmp(&a.play_count));
                self.tracks = Arc::new(temp.into_iter().filter(|t| t.play_count > 0).collect::<Vec<_>>());
            } else if playlist_name == "Recently Played" {
                let rp = crate::db::get(|db| db.recently_played.clone());
                let mut temp_tracks = Vec::new();
                for (path, date_str) in rp {
                    if let Some(mut t) = self.all_tracks.iter().find(|t| t.path == path).cloned() {
                        t.date_played = Some(date_str);
                        temp_tracks.push(t);
                    }
                }
                self.tracks = Arc::new(temp_tracks);
            } else if playlist_name == "New Music" {
                use std::time::SystemTime;
                let mut album_times: std::collections::HashMap<String, SystemTime> = std::collections::HashMap::new();
                for t in self.all_tracks.iter() {
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
                
                self.tracks = Arc::new(temp_tracks);
        } else if let Some(sp) = crate::db::get(|db| db.smart_playlists.get(playlist_name).cloned()) {
                self.tracks = Arc::new(self.evaluate_smart_playlist(&sp));
            } else {
                let paths = crate::db::get(|db| db.playlists.get(playlist_name).cloned().unwrap_or_default());
                let track_map: std::collections::HashMap<std::path::PathBuf, Track> =
                    self.all_tracks.iter().map(|t| (t.path.clone(), t.clone())).collect();
                self.tracks = Arc::new(paths.iter().filter_map(|p| track_map.get(p).cloned()).collect::<Vec<_>>());
            }
            
            if playlist_name == "Liked Songs" || playlist_name == "New Music" {
                let manual = crate::db::get(|db| db.auto_playlist_song_order.get(playlist_name).cloned());
                if let Some(manual_order) = manual {
                    let live_paths: Vec<PathBuf> = self.tracks.iter().map(|t| t.path.clone()).collect();
                    let merged_paths = merge_song_order(&manual_order, &live_paths);
                    let track_map: std::collections::HashMap<PathBuf, Track> =
                        self.tracks.iter().map(|t| (t.path.clone(), t.clone())).collect();
                    self.tracks = Arc::new(merged_paths.iter()
                        .filter_map(|p| track_map.get(p).cloned())
                        .collect::<Vec<_>>());
                    crate::db::write(|db| {
                        db.auto_playlist_song_order.insert(playlist_name.clone(), merged_paths);
                    });
                }
            } else if crate::db::get(|db| db.smart_playlists.contains_key(playlist_name)) {
                let manual = crate::db::get(|db| db.smart_playlist_song_order.get(playlist_name).cloned());
                if let Some(manual_order) = manual {
                    let live_paths: Vec<PathBuf> = self.tracks.iter().map(|t| t.path.clone()).collect();
                    let merged_paths = merge_song_order(&manual_order, &live_paths);
                    let track_map: std::collections::HashMap<PathBuf, Track> =
                        self.tracks.iter().map(|t| (t.path.clone(), t.clone())).collect();
                    self.tracks = Arc::new(merged_paths.iter()
                        .filter_map(|p| track_map.get(p).cloned())
                        .collect::<Vec<_>>());
                    crate::db::write(|db| {
                        db.smart_playlist_song_order.insert(playlist_name.clone(), merged_paths);
                    });
                }
            }
        } else {
            match self.view_mode {

                ViewMode::Artists => {
                    if let Some(artist_name) = &self.selected_artist {
                        self.tracks = Arc::new(self.all_tracks.iter().filter(|t| {
                            let a = if t.artist.trim().is_empty() { "Unknown Artist" } else { &t.artist };
                            a == artist_name
                        }).cloned().collect::<Vec<_>>());
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
                ViewMode::Albums => {
                    if let Some(album_name) = &self.selected_album {
                        self.tracks = Arc::new(self.all_tracks.iter().filter(|t| {
                            let al = if t.album.trim().is_empty() { "Unknown Album" } else { &t.album };
                            al == album_name
                        }).cloned().collect::<Vec<_>>());
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
                ViewMode::Genres => {
                    if let Some(genre_name) = &self.selected_genre {
                        self.tracks = Arc::new(self.all_tracks.iter().filter(|t| {
                            t.genres().iter().any(|g| {
                                let clean = if g.trim().is_empty() { "Unknown Genre" } else { g.trim() };
                                clean == genre_name
                            })
                        }).cloned().collect::<Vec<_>>());
                    } else {
                        self.tracks = self.all_tracks.clone();
                    }
                }
                ViewMode::NowPlaying => {
                    self.tracks = Arc::new(self.queue.clone());
                }
            }
        }

        // Apply sorting
        if let Some(ref playlist_name) = self.selected_playlist {
            if playlist_name == "Recently Played" || playlist_name == "Most Played" {
                return;
            }
        }

        let group_by = self.group_by;
        let sort_column = self.sort_column;
        let sort_ascending = self.sort_ascending;

        if group_by != crate::db::GroupBy::None || sort_column.is_some() {
            Arc::make_mut(&mut self.tracks).sort_by(|a, b| {
                // 1. Sort by group key first if grouping is active
                if group_by != crate::db::GroupBy::None {
                    let key_a = match group_by {
                        crate::db::GroupBy::Album => a.album.to_lowercase(),
                        crate::db::GroupBy::Artist => a.artist.to_lowercase(),
                        crate::db::GroupBy::Genre => a.primary_genre().to_lowercase(),
                        crate::db::GroupBy::Year => a.year.map(|y| y.to_string()).unwrap_or_default(),
                        crate::db::GroupBy::None => unreachable!(),
                    };
                    let key_b = match group_by {
                        crate::db::GroupBy::Album => b.album.to_lowercase(),
                        crate::db::GroupBy::Artist => b.artist.to_lowercase(),
                        crate::db::GroupBy::Genre => b.primary_genre().to_lowercase(),
                        crate::db::GroupBy::Year => b.year.map(|y| y.to_string()).unwrap_or_default(),
                        crate::db::GroupBy::None => unreachable!(),
                    };
                    let cmp_group = key_a.cmp(&key_b);
                    if cmp_group != std::cmp::Ordering::Equal {
                        return cmp_group;
                    }
                }

                // 2. Sort by column within group
                if let Some(col) = sort_column {
                    let cmp = match col {
                        SortColumn::TrackNumber => {
                            let a_dc = a.disc_number.unwrap_or(0);
                            let b_dc = b.disc_number.unwrap_or(0);
                            let cmp_dc = a_dc.cmp(&b_dc);
                            if cmp_dc == std::cmp::Ordering::Equal {
                                let a_num = a.track_number.unwrap_or(u32::MAX);
                                let b_num = b.track_number.unwrap_or(u32::MAX);
                                a_num.cmp(&b_num)
                            } else {
                                cmp_dc
                            }
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
                            let cmp_dc = a_dc.cmp(&b_dc);
                            if cmp_dc == std::cmp::Ordering::Equal {
                                let a_num = a.track_number.unwrap_or(u32::MAX);
                                let b_num = b.track_number.unwrap_or(u32::MAX);
                                a_num.cmp(&b_num)
                            } else {
                                cmp_dc
                            }
                        }
                        SortColumn::Duration => a.duration.cmp(&b.duration),
                        SortColumn::Plays => a.play_count.cmp(&b.play_count),
                        SortColumn::DatePlayed => {
                            let a_dp = a.date_played.as_deref().unwrap_or("");
                            let b_dp = b.date_played.as_deref().unwrap_or("");
                            a_dp.cmp(b_dp)
                        }
                        SortColumn::Liked => a.liked.cmp(&b.liked),
                    };
                    if sort_ascending { cmp } else { cmp.reverse() }
                } else {
                    // Default layout sorting (Disc, then Track number)
                    let a_dc = a.disc_number.unwrap_or(0);
                    let b_dc = b.disc_number.unwrap_or(0);
                    let cmp_dc = a_dc.cmp(&b_dc);
                    if cmp_dc == std::cmp::Ordering::Equal {
                        let a_num = a.track_number.unwrap_or(u32::MAX);
                        let b_num = b.track_number.unwrap_or(u32::MAX);
                        a_num.cmp(&b_num)
                    } else {
                        cmp_dc
                    }
                }
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
                self.queue = self.tracks.to_vec();
                self.set_playing_context_from_current_view();
                self.play_track_internal(track)
            }

            Message::PlayTracks(tracks) => {
                if let Some(first) = tracks.first().cloned() {
                    self.queue = tracks;
                    self.set_playing_context_from_current_view();
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::PlayAlbum(album_name) => {
                self.selected_album = Some(album_name.clone());
                self.selected_playlist = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                self.playing_context = Some(PlayingContext::Album(album_name));
                let tracks_to_play = self.tracks.to_vec();
                if let Some(first) = tracks_to_play.first().cloned() {
                    self.queue = tracks_to_play;
                    self.play_track_internal(first)
                } else {
                    Task::none()
                }
            }

            Message::ToggleAlbumPlayPause(album_name) => {
                let is_current_album_playing = self.current_track.as_ref().map(|t| &t.album) == Some(&album_name);
                if is_current_album_playing {
                    match self.playback_state {
                        PlaybackState::Playing => {
                            self.audio.send(AudioCommand::Pause);
                            self.playback_state = PlaybackState::Paused;
                            self.send_mpris(MprisUpdate::Status(PlaybackStatus::Paused));
                        }
                        PlaybackState::Paused => {
                            self.audio.send(AudioCommand::Resume);
                            self.playback_state = PlaybackState::Playing;
                            self.send_mpris(MprisUpdate::Status(PlaybackStatus::Playing));
                        }
                        PlaybackState::Stopped => {
                            self.view_mode = ViewMode::Albums;
                            self.selected_album = Some(album_name);
                            self.selected_playlist = None;
                            self.selected_folder = None;
                            self.selected_artist = None;
                            self.search_query.clear();
                            self.update_filtered_tracks();
                            let tracks_to_play = self.tracks.to_vec();
                            if let Some(first) = tracks_to_play.first().cloned() {
                                self.queue = tracks_to_play;
                                return self.play_track_internal(first);
                            }
                        }
                    }
                } else {
                    self.view_mode = ViewMode::Albums;
                    self.selected_album = Some(album_name);
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_artist = None;
                    self.search_query.clear();
                    self.update_filtered_tracks();
                    let tracks_to_play = self.tracks.to_vec();
                    if let Some(first) = tracks_to_play.first().cloned() {
                        self.queue = tracks_to_play;
                        return self.play_track_internal(first);
                    }
                }
                Task::none()
            }

            Message::HoverAlbumHeader(album) => {
                self.hovered_album_header = album;
                Task::none()
            }

            Message::IncreaseScale => {
                self.font_scale = (self.font_scale + 0.05).min(3.0);
                crate::config::update_font_scale(self.font_scale);
                Task::none()
            }

            Message::DecreaseScale => {
                self.font_scale = (self.font_scale - 0.05).max(0.5);
                crate::config::update_font_scale(self.font_scale);
                Task::none()
            }

            Message::TracklistScrolled(viewport) => {
                let rel_y = viewport.relative_offset().y;
                let absolute_y = viewport.absolute_offset().y;
                let content_height = viewport.content_bounds().height;
                let bounds_height = viewport.bounds().height;
                let total_tracks = self.tracks.len();
                
                let near_bottom = rel_y > 0.75 || (absolute_y + bounds_height >= content_height - 1000.0);
                
                if near_bottom && self.track_list_end < total_tracks {
                    self.track_list_end = (self.track_list_end + 200).min(total_tracks);
                }
                Task::none()
            }

            Message::PlayPause => {
                match self.playback_state {
                    PlaybackState::Playing => {
                        if let Some(ref sel) = self.selected_track {
                            if self.current_track.as_ref().map(|ct| ct.id) != Some(sel.id) {
                                self.queue = (*self.tracks).clone();
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
                                self.queue = (*self.tracks).clone();
                                return self.play_track_internal(sel.clone());
                            }
                        }
                        self.audio.send(AudioCommand::Resume);
                        self.playback_state = PlaybackState::Playing;
                        self.send_mpris(MprisUpdate::Status(PlaybackStatus::Playing));
                        Task::none()
                    }
                    PlaybackState::Stopped => {
                        if let Some(sel) = self.selected_track.clone() {
                            self.queue = (*self.tracks).clone();
                            self.set_playing_context_from_current_view();
                            self.play_track_internal(sel)
                        } else if let Some(first) = self.tracks.first().cloned() {
                            self.queue = (*self.tracks).clone();
                            self.set_playing_context_from_current_view();
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
                self.last_accumulated_position = dur;
                Task::none()
            }

            Message::SeekToLyric(dur) => {
                self.audio.send(AudioCommand::Seek(dur));
                self.position = dur;
                self.last_accumulated_position = dur;
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
                self.last_accumulated_position = new_pos;
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
                if self.shuffle && !self.queue.is_empty() {
                    // Shuffling queue in-place, keeping the current track at index 0 or its current position
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    let current_track_id = self.current_track.as_ref().map(|t| t.id);
                    if let Some(ct_id) = current_track_id {
                        if let Some(pos) = self.queue.iter().position(|t| t.id == ct_id) {
                            let current_item = self.queue.remove(pos);
                            self.queue.shuffle(&mut rng);
                            self.queue.insert(0, current_item);
                        } else {
                            self.queue.shuffle(&mut rng);
                        }
                    } else {
                        self.queue.shuffle(&mut rng);
                    }
                    let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                    crate::db::write(|db| {
                        db.last_queue_paths = queue_paths;
                    });
                }
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
                self.sidebar_width = x.clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH);
                Task::none()
            }

            Message::SidebarDragEnd => {
                self.dragging_sidebar = false;
                crate::db::write(|db| db.sidebar_width = Some(self.sidebar_width));
                Task::none()
            }



            Message::RightPanelDragStart => {
                self.dragging_right_panel = true;
                Task::none()
            }

            Message::RightPanelDragMove(x) => {
                let max_drawer_width = (self.window_width - MIN_NON_DRAWER_WIDTH).max(450.0);
                let new_width = (self.window_width - x).clamp(450.0, max_drawer_width);
                self.right_panel_width = new_width;
                Task::none()
            }

            Message::RightPanelDragEnd => {
                self.dragging_right_panel = false;
                crate::db::write(|db| db.right_panel_width = Some(self.right_panel_width));
                Task::none()
            }

            Message::PlayerDragStart => {
                self.dragging_player_split = true;
                Task::none()
            }

            Message::PlayerDragMove(y) => {
                self.player_height = y.clamp(330.0, 458.0);
                Task::none()
            }

            Message::PlayerDragEnd => {
                self.dragging_player_split = false;
                crate::db::write(|db| db.player_height = Some(self.player_height));
                Task::none()
            }

            Message::HoverPlayerResizer(val) => {
                self.is_hovering_player_resizer = val;
                Task::none()
            }

            Message::PollAudio => {
                // Hourly check
                let current_hour = {
                    use chrono::Timelike;
                    chrono::Local::now().hour()
                };
                if self.last_checked_hour.is_none() {
                    self.last_checked_hour = Some(current_hour);
                } else if self.last_checked_hour != Some(current_hour) {
                    self.last_checked_hour = Some(current_hour);
                    if self.playback_state == PlaybackState::Playing {
                        let nid = self.next_notification_id;
                        self.next_notification_id += 1;
                        self.active_notifications.push(StatsNotification {
                            id: nid,
                            title: "Time Flies".to_string(),
                            message: "You've been listening for another hour!".to_string(),
                            created_at: std::time::Instant::now(),
                        });
                    }
                }

                let mut tasks = Vec::new();
                while let Ok(event) = self.audio.event_rx.try_recv() {
                    match event {
                        AudioEvent::Progress { position, duration } => {
                            self.position = position;
                            self.duration = duration;
                            self.send_mpris(MprisUpdate::Position(position));

                            let current_secs = position.as_secs();
                            let old_secs = crate::db::get(|db| db.last_position_secs);
                            if current_secs != old_secs {
                                crate::db::write(|db| db.last_position_secs = current_secs);
                            }

                            // Accumulate playback time
                            if self.playback_state == PlaybackState::Playing {
                                let diff = position.saturating_sub(self.last_accumulated_position);
                                if diff > Duration::ZERO && diff <= Duration::from_secs(1) {
                                    if let Some(ref track) = self.current_track {
                                        let new_awards = crate::stats::add_playback_time(&track.artist, &track.album, &track.genre, diff.as_secs_f64());
                                        for award in new_awards {
                                            let nid = self.next_notification_id;
                                            self.next_notification_id += 1;
                                            let award_name = match award.period.as_str() {
                                                "Daily" => "Ribbon",
                                                "Weekly" => "Medal",
                                                "Monthly" => "Crown",
                                                "Yearly" => "Trophy",
                                                "All-Time" => "Diamond",
                                                _ => "Award",
                                            };
                                            self.active_notifications.push(StatsNotification {
                                                id: nid,
                                                title: format!("{} Earned!", award_name),
                                                message: format!(
                                                    "You've unlocked a {} {} ({}) for {}!",
                                                    award.tier, award_name, award.period, award.entity_name
                                                ),
                                                created_at: std::time::Instant::now(),
                                            });
                                        }
                                    }
                                }
                            }
                            self.last_accumulated_position = position;

                            if !self.current_track_play_counted && duration > Duration::ZERO {
                                let is_estimated = duration == position;
                                let threshold = if is_estimated {
                                    Duration::from_secs(60)
                                } else {
                                    duration / 2
                                };
                                if position >= threshold {
                                if let Some(ref mut track) = self.current_track {
                                    let count = crate::db::increment_play_count(track.path.clone());
                                    track.play_count = count;

                                    let toasts = crate::stats::on_track_play(
                                        &track.artist,
                                        &track.genre,
                                        &track.album,
                                        track.path.clone(),
                                        &self.all_tracks,
                                    );
                                    for (title, msg) in toasts {
                                        let nid = self.next_notification_id;
                                        self.next_notification_id += 1;
                                        self.active_notifications.push(StatsNotification {
                                            id: nid,
                                            title,
                                            message: msg,
                                            created_at: std::time::Instant::now(),
                                        });
                                    }

                                    if let Some(t) = Arc::make_mut(&mut self.all_tracks).iter_mut().find(|t| t.path == track.path) {
                                        t.play_count = count;
                                    }
                                    if let Some(t) = Arc::make_mut(&mut self.tracks).iter_mut().find(|t| t.path == track.path) {
                                        t.play_count = count;
                                    }
                                }
                                self.current_track_play_counted = true;
                                }
                            }
                        }

                        AudioEvent::Paused => {
                            self.playback_state = PlaybackState::Paused;
                        }
                        AudioEvent::Stopped => {
                            self.playback_state = PlaybackState::Stopped;
                            self.position = Duration::ZERO;
                            self.last_accumulated_position = Duration::ZERO;
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
                        AudioEvent::Error(e) => eprintln!("Audio error: {e}"),
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
                                        (active_idx as f32 + 0.5) / total as f32
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
                if matches!(self.playback_state, PlaybackState::Playing) {
                    self.animation_tick = self.animation_tick.wrapping_add(1);
                }
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
                if crate::config::get().theme_source == "System" {
                    let current = crate::ui::theme::read_current_theme_name();
                    if !current.is_empty() && current != self.loaded_theme_name {
                        crate::ui::theme::reload_system_theme();
                        self.iced_theme = build_iced_theme();
                        self.loaded_theme_name = current;
                    }
                }
                Task::none()
            }

            Message::FlushBuffers => {
                crate::db::flush();
                crate::stats::flush();
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

            Message::ToggleLikeCurrent => {
                if let Some(track) = self.current_track.clone() {
                    return Task::done(Message::ToggleLikeTrack(track));
                }
                Task::none()
            }

            Message::ToggleLikeTrack(track) => {
                self.show_context_menu = None;
                let liked = !track.liked;
                if let Err(e) = crate::library::scanner::write_like_status(&track.path, liked) {
                    eprintln!("Failed to write like status to file: {e}");
                }
                if let Some(t) = Arc::make_mut(&mut self.all_tracks).iter_mut().find(|t| t.path == track.path) {
                    t.liked = liked;
                }
                if let Some(t) = Arc::make_mut(&mut self.tracks).iter_mut().find(|t| t.path == track.path) {
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
                    PlaylistTab::Smart => {
                        self.selected_playlist = None;
                    }
                }
                self.update_filtered_tracks();
                Task::none()
            }

            Message::SelectPlaylist(name) => {
                if name == "Liked Songs" || name == "Recently Played" || name == "Most Played" || name == "New Music" {
                    self.playlist_tab = PlaylistTab::Autoplaylists;
                } else if crate::db::get(|db| db.smart_playlists.contains_key(&name)) {
                    self.playlist_tab = PlaylistTab::Smart;
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
                let _all_same_genre = tracks.iter().all(|t| t.genre == first.genre);
                let all_same_track_num = tracks.iter().all(|t| t.track_number == first.track_number);
                let all_same_disc_num = tracks.iter().all(|t| t.disc_number == first.disc_number);
                let all_same_year = tracks.iter().all(|t| t.year == first.year);
                let all_same_lyrics = tracks.iter().all(|t| t.lyrics == first.lyrics);

                let mut original_tracks = std::collections::HashMap::new();
                for t in &tracks {
                    original_tracks.insert(t.path.clone(), t.clone());
                }

                // Find genres that appear in EVERY selected track (value-based, not position-based)
                let all_genre_values: Vec<Vec<&str>> = tracks.iter().map(|t| t.genres()).collect();
                let mut unique_common: Vec<&str> = Vec::new();
                for val in all_genre_values.iter().flat_map(|v| v.iter()).copied() {
                    if !val.is_empty()
                        && !unique_common.contains(&val)
                        && all_genre_values.iter().all(|track_vals| track_vals.contains(&val))
                    {
                        unique_common.push(val);
                    }
                }
                let genres: Vec<String> = unique_common.iter().map(|s| s.to_string()).collect();
                let genres_original: Vec<String> = genres.clone();
                let apply_genres: Vec<bool> = vec![true; genres.len()];

                self.show_tag_editor = Some(TagEditorState {
                    tracks: tracks.clone(),
                    original_tracks,
                    is_saved: false,
                    title: if all_same_title { first.title.clone() } else { String::new() },
                    artist: if all_same_artist { first.artist.clone() } else { String::new() },
                    album: if all_same_album { first.album.clone() } else { String::new() },
                    genres,
                    genres_original,
                    apply_genres,
                    track_number: if all_same_track_num { first.track_number.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    disc_number: if all_same_disc_num { first.disc_number.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    cover_path: None,
                    apply_to_album: false,
                    year: if all_same_year { first.year.map(|n| n.to_string()).unwrap_or_default() } else { String::new() },
                    apply_title: false,
                    apply_artist: false,
                    apply_album: false,
                    apply_year: false,
                    apply_track_num: false,
                    apply_disc_num: false,
                    apply_cover: false,
                    apply_lyrics: false,
                    lyrics: if all_same_lyrics { first.lyrics.clone() } else { String::new() },
                    lyrics_content: iced::widget::text_editor::Content::with_text(if all_same_lyrics { &first.lyrics } else { "" }),
                    active_tab: TagEditorTab::Main,
                    focused_field: Some(0),
                    pending_offset: 0.0,
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

            Message::CancelTagEditor => {
                if let Some(state) = self.show_tag_editor.take() {
                    for (_, original_track) in state.original_tracks {
                        let res = crate::library::write_tags(
                            &original_track.path,
                            &original_track.title,
                            &original_track.artist,
                            &original_track.album,
                            &original_track.genre,
                            original_track.track_number,
                            original_track.disc_number,
                            None,
                            original_track.year,
                            Some(&original_track.lyrics),
                        );
                        if let Err(e) = res {
                            eprintln!("Error restoring tags for {}: {e}", original_track.path.display());
                        } else {
                            if let Some(t) = Arc::make_mut(&mut self.all_tracks).iter_mut().find(|t| t.path == original_track.path) {
                                *t = original_track.clone();
                            }
                            if let Some(t) = Arc::make_mut(&mut self.tracks).iter_mut().find(|t| t.path == original_track.path) {
                                *t = original_track.clone();
                            }
                            if let Some(ref mut ct) = self.current_track {
                                if ct.path == original_track.path {
                                    *ct = original_track.clone();
                                }
                            }
                            if let Some(ref mut st) = self.selected_track {
                                if st.path == original_track.path {
                                    *st = original_track.clone();
                                }
                            }
                            if let Some(t) = Arc::make_mut(&mut self.selected_tracks).iter_mut().find(|t| t.path == original_track.path) {
                                *t = original_track.clone();
                            }
                        }
                    }
                }
                self.update_filtered_tracks();
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
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldArtist(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.artist = val;
                    state.apply_artist = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.album = val;
                    state.apply_album = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldGenre(slot, val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    if slot < state.genres.len() {
                        state.genres[slot] = val;
                        state.apply_genres[slot] = true;
                        state.is_saved = false;
                    }
                }
                Task::none()
            }

            Message::UpdateTagFieldTrackNumber(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.track_number = val;
                    state.apply_track_num = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldDiscNumber(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.disc_number = val;
                    state.apply_disc_num = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldCoverPath(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.cover_path = Some(val);
                    state.apply_cover = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldApplyToAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_to_album = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::UpdateTagFieldYear(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.year = val;
                    state.apply_year = true;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyTitle(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_title = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyArtist(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_artist = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyAlbum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_album = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyYear(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_year = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyGenre(slot, val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    if slot < state.apply_genres.len() {
                        state.apply_genres[slot] = val;
                        state.is_saved = false;
                    }
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyTrackNum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_track_num = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyDiscNum(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_disc_num = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyCover(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_cover = val;
                    state.is_saved = false;
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
                        for t in self.all_tracks.iter() {
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
                        let genre: String = if state.apply_genres.iter().any(|a| *a) {
                            let mut original_parts: Vec<String> = track.genres().into_iter().map(|s| s.to_string()).collect();
                            for i in 0..state.genres.len() {
                                if i < state.apply_genres.len() && state.apply_genres[i] {
                                    if i < state.genres_original.len() && !state.genres_original[i].is_empty() {
                                        // Replace first occurrence of original value
                                        if let Some(pos) = original_parts.iter().position(|p| *p == state.genres_original[i]) {
                                            if state.genres[i].is_empty() {
                                                original_parts.remove(pos);
                                            } else {
                                                original_parts[pos] = state.genres[i].clone();
                                            }
                                        }
                                    } else if !state.genres[i].is_empty() {
                                        // New genre: append
                                        original_parts.push(state.genres[i].clone());
                                    }
                                }
                            }
                            original_parts.retain(|p| !p.is_empty());
                            original_parts.join("; ")
                        } else {
                            track.genre.clone()
                        };
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
                             &genre,
                            track_number,
                            disc_number,
                            cover_path,
                            year,
                            lyrics_val,
                        );
                        if let Err(e) = res {
                            eprintln!("Error saving tags for {}: {e}", track.path.display());
                        } else {
                            if let Some(t) = Arc::make_mut(&mut self.all_tracks).iter_mut().find(|t| t.path == track.path) {
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
                            if let Some(t) = Arc::make_mut(&mut self.tracks).iter_mut().find(|t| t.path == track.path) {
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
                            if let Some(t) = Arc::make_mut(&mut self.selected_tracks).iter_mut().find(|t| t.path == track.path) {
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
                if let Some(ref mut state) = self.show_tag_editor {
                    for track in &mut state.tracks {
                        if let Some(updated_track) = self.all_tracks.iter().find(|t| t.path == track.path) {
                            *track = updated_track.clone();
                        }
                    }
                    state.apply_title = false;
                    state.apply_artist = false;
                    state.apply_album = false;
                    state.apply_year = false;
                    for a in &mut state.apply_genres { *a = false; }
                    state.apply_track_num = false;
                    state.apply_disc_num = false;
                    state.apply_cover = false;
                    state.apply_lyrics = false;
                    state.is_saved = true;
                }
                self.cover_cache_version = self.cover_cache_version.wrapping_add(1);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::NewSmartPlaylist => {
                let saved_view = SavedViewState {
                    view_mode: self.view_mode,
                    selected_playlist: self.selected_playlist.clone(),
                    selected_artist: self.selected_artist.clone(),
                    selected_album: self.selected_album.clone(),
                    selected_genre: self.selected_genre.clone(),
                    playlist_tab: self.playlist_tab,
                };
                self.previous_view_state = Some(saved_view);
                self.smart_playlist_builder = Some(crate::ui::components::smart_playlist_builder::SmartPlaylistBuilderState {
                    name: String::new(),
                    rules: vec![crate::ui::components::smart_playlist_builder::RuleRowState::new(crate::library::smart_playlist::RuleField::Title)],
                    limit_enabled: false,
                    limit_str: "25".to_string(),
                    order_by: crate::library::smart_playlist::SmartPlaylistOrder::Random,
                    live_updating: true,
                    editing_name: None,
                });
                self.selected_playlist = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.selected_genre = None;
                self.active_focus = None;
                Task::none()
            }

            Message::EditSmartPlaylist(name) => {
                if let Some(sp) = crate::db::get(|db| db.smart_playlists.get(&name).cloned()) {
                    let saved_view = SavedViewState {
                        view_mode: self.view_mode,
                        selected_playlist: self.selected_playlist.clone(),
                        selected_artist: self.selected_artist.clone(),
                        selected_album: self.selected_album.clone(),
                        selected_genre: self.selected_genre.clone(),
                        playlist_tab: self.playlist_tab,
                    };
                    self.previous_view_state = Some(saved_view);
                    
                    let rules = sp.rules.iter().map(|r| crate::ui::components::smart_playlist_builder::RuleRowState::from_rule(r)).collect();
                    self.smart_playlist_builder = Some(crate::ui::components::smart_playlist_builder::SmartPlaylistBuilderState {
                        name: sp.name.clone(),
                        rules,
                        limit_enabled: sp.limit.is_some(),
                        limit_str: sp.limit.map(|l| l.to_string()).unwrap_or_else(|| "25".to_string()),
                        order_by: sp.order_by,
                        live_updating: sp.live_updating,
                        editing_name: Some(sp.name.clone()),
                    });
                    self.selected_playlist = None;
                    self.selected_artist = None;
                    self.selected_album = None;
                    self.selected_genre = None;
                    self.active_focus = None;
                }
                Task::none()
            }

            Message::DeleteSmartPlaylist(name) => {
                crate::db::delete_smart_playlist(name.clone());
                if self.selected_playlist.as_ref() == Some(&name) {
                    self.selected_playlist = None;
                    self.update_filtered_tracks();
                }
                Task::none()
            }

            Message::SmartPlaylistBuilderMsg(event) => {
                if let Some(ref mut builder) = self.smart_playlist_builder {
                    match event {
                        SmartPlaylistBuilderEvent::NameChanged(s) => {
                            builder.name = s;
                        }
                        SmartPlaylistBuilderEvent::AddRule => {
                            builder.rules.push(crate::ui::components::smart_playlist_builder::RuleRowState::new(crate::library::smart_playlist::RuleField::Title));
                        }
                        SmartPlaylistBuilderEvent::RemoveRule(idx) => {
                            if idx < builder.rules.len() {
                                builder.rules.remove(idx);
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleField(idx, f) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx] = crate::ui::components::smart_playlist_builder::RuleRowState::new(f);
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleOperator(idx, o) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx].operator = o;
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleValue(idx, v) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx].value = v;
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleValue2(idx, v) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx].value2 = v;
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleDateUnit(idx, u) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx].date_unit = u;
                            }
                        }
                        SmartPlaylistBuilderEvent::UpdateRuleBoolean(idx, b) => {
                            if idx < builder.rules.len() {
                                builder.rules[idx].boolean_value = b;
                            }
                        }
                        SmartPlaylistBuilderEvent::ToggleLimit(b) => {
                            builder.limit_enabled = b;
                        }
                        SmartPlaylistBuilderEvent::LimitStrChanged(s) => {
                            builder.limit_str = s;
                        }
                        SmartPlaylistBuilderEvent::UpdateOrderBy(o) => {
                            builder.order_by = o;
                        }
                        SmartPlaylistBuilderEvent::ToggleLive(b) => {
                            builder.live_updating = b;
                        }
                        SmartPlaylistBuilderEvent::Cancel => {
                            self.smart_playlist_builder = None;
                            if let Some(prev) = self.previous_view_state.take() {
                                self.view_mode = prev.view_mode;
                                self.selected_playlist = prev.selected_playlist;
                                self.selected_artist = prev.selected_artist;
                                self.selected_album = prev.selected_album;
                                self.selected_genre = prev.selected_genre;
                                self.playlist_tab = prev.playlist_tab;
                                self.update_filtered_tracks();
                            }
                        }
                        SmartPlaylistBuilderEvent::Save => {
                            let name = builder.name.clone();
                            let rules: Vec<crate::library::smart_playlist::SmartPlaylistRule> = builder.rules.iter().map(|r| r.to_rule()).collect();
                            let limit = if builder.limit_enabled {
                                builder.limit_str.trim().parse::<usize>().ok()
                            } else {
                                None
                            };
                            let order_by = builder.order_by;
                            let live_updating = builder.live_updating;
                            let editing_name = builder.editing_name.clone();

                            if !name.trim().is_empty() {
                                let mut sp = crate::library::smart_playlist::SmartPlaylist {
                                    name: name.clone(),
                                    rules,
                                    limit,
                                    order_by,
                                    live_updating,
                                    tracks: Vec::new(),
                                };
                                
                                // Evaluate immediately
                                let evaluated_tracks = self.evaluate_smart_playlist(&sp);
                                sp.tracks = evaluated_tracks.iter().map(|t| t.path.clone()).collect();
                                
                                // Delete old name if renamed
                                if let Some(ref old_name) = editing_name {
                                    if old_name != &sp.name {
                                        crate::db::delete_smart_playlist(old_name.clone());
                                    }
                                }
                                
                                crate::db::save_smart_playlist(sp.name.clone(), sp);
                                
                                self.smart_playlist_builder = None;
                                self.previous_view_state = None;
                                
                                // Select it
                                self.selected_playlist = Some(name);
                                self.playlist_tab = PlaylistTab::Smart;
                                self.update_filtered_tracks();
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::TagEditorPrevTrack => {
                if let Some(ref state) = self.show_tag_editor {
                    if let Some(first_track) = state.tracks.first() {
                        if let Some(pos) = self.tracks.iter().position(|t| t.path == first_track.path) {
                            let prev_idx = if pos == 0 { self.tracks.len() - 1 } else { pos - 1 };
                            if let Some(track) = self.tracks.get(prev_idx).cloned() {
                                self.load_track_in_tag_editor(track);
                                return iced::widget::text_input::focus(iced::widget::text_input::Id::new("id3_title"));
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::TagEditorNextTrack => {
                if let Some(ref state) = self.show_tag_editor {
                    if let Some(first_track) = state.tracks.first() {
                        if let Some(pos) = self.tracks.iter().position(|t| t.path == first_track.path) {
                            let next_idx = (pos + 1) % self.tracks.len();
                            if let Some(track) = self.tracks.get(next_idx).cloned() {
                                self.load_track_in_tag_editor(track);
                                return iced::widget::text_input::focus(iced::widget::text_input::Id::new("id3_title"));
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::LibraryScanned(tracks) => {
                self.all_tracks = Arc::new(tracks);
                crate::stats::backfill_album_data(&self.all_tracks);
                crate::stats::backfill_achievements(&self.all_tracks);
                self.update_live_smart_playlists();
                let mut cache: HashMap<PathBuf, Vec<Track>> = HashMap::new();
                for track in self.all_tracks.iter() {
                    if let Some(parent) = track.path.parent() {
                        cache.entry(parent.to_path_buf()).or_default().push(track.clone());
                    }
                }
                self.folder_cache = cache;
                let mut keys: Vec<PathBuf> = self.folder_cache.keys().cloned().collect();
                keys.sort();
                self.folders = keys;

                let saved = crate::db::get(|db| (
                    db.last_view_mode,
                    db.last_selected_playlist.clone(),
                    db.last_selected_folder.clone(),
                    db.last_selected_artist.clone(),
                    db.last_selected_album.clone(),
                    db.last_selected_genre.clone(),
                    db.last_track_path.clone(),
                    db.last_queue_paths.clone(),
                    db.last_position_secs,
                ));

                if let (Some(vm), sel_playlist, sel_folder, sel_artist, sel_album, sel_genre, last_track, last_queue, last_pos) = saved {
                    let restore_vm = if vm == ViewMode::NowPlaying { ViewMode::Artists } else { vm };
                    self.view_mode = restore_vm;
                    self.last_browsing_view = restore_vm;
                    self.selected_playlist = sel_playlist;
                    self.selected_folder = sel_folder;
                    self.selected_artist = sel_artist;
                    self.selected_album = sel_album;
                    self.selected_genre = sel_genre;

                    if self.selected_artist.is_none() {
                        let artists_list = self.artists();
                        if !artists_list.is_empty() {
                            self.selected_artist = Some(artists_list[0].clone());
                        }
                    }

                    self.update_filtered_tracks();

                    let mut restored_queue = Vec::new();
                    for path in last_queue {
                        if let Some(t) = self.all_tracks.iter().find(|track| track.path == path) {
                            restored_queue.push(t.clone());
                        }
                    }
                    if !restored_queue.is_empty() {
                        self.queue = restored_queue;
                    } else {
                        self.queue = (*self.tracks).clone();
                    }

                    if let Some(track_path) = last_track {
                        if let Some(track) = self.all_tracks.iter().find(|t| t.path == track_path) {
                            let cover_data = load_cover(&track.path);
                            let t = Track { cover_data, ..track.clone() };
                            self.current_track = Some(t.clone());
                            self.selected_track = Some(t.clone());
                            self.playback_state = PlaybackState::Paused;
                            self.position = Duration::from_secs(last_pos);
                            self.duration = t.duration;
                            self.current_track_play_counted = false;
                            self.notify_mpris_track(PlaybackStatus::Paused);

                            self.audio.send(AudioCommand::Play(t.path.clone()));
                            self.audio.send(AudioCommand::Seek(Duration::from_secs(last_pos)));
                            self.audio.send(AudioCommand::Pause);
                        }
                    }
                } else {
                    self.update_filtered_tracks();
                }

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
                    (*self.selected_tracks).clone()
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
                self.playlist_height = (self.window_height - y - 60.0).clamp(MIN_PLAYLIST_HEIGHT, (self.window_height - 300.0).max(MIN_PLAYLIST_HEIGHT));
                Task::none()
            }

            Message::PlaylistDragEnd => {
                self.dragging_playlist_split = false;
                crate::db::write(|db| db.playlist_height = Some(self.playlist_height));
                Task::none()
            }

            Message::SelectViewMode(mode) => {
                if mode != ViewMode::NowPlaying {
                    self.last_browsing_view = mode;
                    self.view_mode = mode;
                    self.show_queue_popover = false;
                    self.selected_playlist = None;
                    self.selected_folder = None;
                    self.selected_artist = None;
                    self.selected_album = None;
                    self.selected_genre = None;
                    Arc::make_mut(&mut self.selected_tracks).clear();
                    self.search_query.clear();
                    self.update_filtered_tracks();
                }
                Task::none()
            }

            Message::ToggleQueuePopover => {
                self.show_queue_popover = !self.show_queue_popover;
                if self.show_queue_popover {
                    let current_track_id = self.current_track.as_ref().map(|t| t.id);
                    if let Some(ct_id) = current_track_id {
                        if let Some(idx) = self.queue.iter().position(|t| t.id == ct_id) {
                            // Each item is approx 42px tall, scroll to center it (subtracting half height of viewport, ~200px)
                            let offset_y = (idx as f32 * 42.0 - 150.0).max(0.0);
                            return scrollable::scroll_to(
                                self.queue_scroll_id.clone(),
                                scrollable::AbsoluteOffset { x: 0.0, y: offset_y },
                            );
                        }
                    }
                }
                Task::none()
            }

            Message::CloseQueuePopover => {
                self.show_queue_popover = false;
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
                self.view_mode = ViewMode::Artists;
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
                self.show_context_menu = None;
                let initial_name = match &mode {
                    PlaylistDialogMode::Create => "My Playlist".to_string(),
                    PlaylistDialogMode::AddTrack(_) => String::new(),
                    PlaylistDialogMode::CreateWithTrack(track) => format!("{} Playlist", track.title),
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

            Message::PlaylistCreateWithTrack(track) => {
                if let Some(ref mut dialog) = self.playlist_dialog {
                    dialog.mode = PlaylistDialogMode::CreateWithTrack(track.clone());
                    dialog.name_input = format!("{} Playlist", track.title);
                }
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
                        PlaylistDialogMode::CreateWithTrack(track) => {
                            let name = dialog.name_input.trim().to_string();
                            if !name.is_empty() {
                                crate::db::create_playlist(name.clone());
                                crate::db::add_to_playlist(name, track.path);
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
                    self.playlist_height = ((h - 212.0) * 0.27).max(MIN_PLAYLIST_HEIGHT);
                    self.playlist_height_initialized = true;
                }
                let max_drawer_width = (w - MIN_NON_DRAWER_WIDTH).max(450.0);
                if !self.right_panel_width_initialized {
                    self.right_panel_width = (w * 0.33).clamp(450.0, max_drawer_width);
                    self.right_panel_width_initialized = true;
                } else {
                    self.right_panel_width = self.right_panel_width.clamp(450.0, max_drawer_width);
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
                        self.selected_tracks = Arc::new(vec![track.clone()]);
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
                        ViewMode::NowPlaying => {}
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
                        self.selected_tracks = Arc::new(vec![track.clone()]);
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
                        ViewMode::NowPlaying => {}
                    }
                }
                Task::none()
            }

            Message::DeletePlaylist(name) => {
                self.show_context_menu = None;
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

            Message::GroupByHoverEnter => {
                self.group_by_state.is_cluster_hovered = true;
                self.group_by_state.collapse_deadline = None;
                self.group_by_state.collapse_token = self.group_by_state.collapse_token.wrapping_add(1);
                Task::none()
            }

            Message::GroupByHoverExit => {
                self.group_by_state.is_cluster_hovered = false;
                if self.group_by_state.hover_progress > 0.0 {
                    self.group_by_state.collapse_deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis(1500));
                    self.group_by_state.collapse_token = self.group_by_state.collapse_token.wrapping_add(1);
                    let token = self.group_by_state.collapse_token;
                    Task::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                        },
                        move |_| Message::GroupByCollapseTimeout(token)
                    )
                } else {
                    Task::none()
                }
            }

            Message::GroupByCollapseTimeout(token) => {
                if token == self.group_by_state.collapse_token && !self.group_by_state.is_cluster_hovered {
                    self.group_by_state.collapse_deadline = None;
                }
                Task::none()
            }

            Message::GroupBySelected(grouping) => {
                self.group_by = grouping;
                self.group_by_state.active = grouping;
                crate::db::write(|db| {
                    db.group_by = Some(grouping);
                    db.group_by_album = grouping == crate::db::GroupBy::Album;
                });
                self.group_by_state.force_collapsing = true;
                self.group_by_state.is_cluster_hovered = false;
                self.group_by_state.collapse_deadline = None;
                self.group_by_state.collapse_token = self.group_by_state.collapse_token.wrapping_add(1);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::GroupByCleared => {
                let grouping = crate::db::GroupBy::None;
                self.group_by = grouping;
                self.group_by_state.active = grouping;
                crate::db::write(|db| {
                    db.group_by = Some(grouping);
                    db.group_by_album = false;
                });
                self.group_by_state.force_collapsing = true;
                self.group_by_state.is_cluster_hovered = false;
                self.group_by_state.collapse_deadline = None;
                self.group_by_state.collapse_token = self.group_by_state.collapse_token.wrapping_add(1);
                self.update_filtered_tracks();
                Task::none()
            }

            Message::GroupByAnimationTick(_instant) => {
                let target = self.group_by_state.target();
                let diff = target - self.group_by_state.hover_progress;
                if diff.abs() < 0.01 {
                    self.group_by_state.hover_progress = target;
                    if target == 0.0 {
                        self.group_by_state.force_collapsing = false;
                        self.group_by_state.collapse_deadline = None;
                    }
                } else {
                    self.group_by_state.hover_progress += diff * 0.15;
                }
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

                let shift_held = self.modifiers.shift();
                let ctrl_held = self.modifiers.control() || self.modifiers.command();

                if ctrl_held {
                    if self.selected_tracks.iter().any(|t| t.id == track.id) {
                        Arc::make_mut(&mut self.selected_tracks).retain(|t| t.id != track.id);
                    } else {
                        Arc::make_mut(&mut self.selected_tracks).push(track.clone());
                    }
                    self.last_clicked_track = Some(track.clone());
                } else if shift_held {
                    if let Some(ref start_track) = self.last_clicked_track {
                        let start_idx = self.tracks.iter().position(|t| t.id == start_track.id);
                        let end_idx = self.tracks.iter().position(|t| t.id == track.id);
                        if let (Some(s), Some(e)) = (start_idx, end_idx) {
                            let (min, max) = if s < e { (s, e) } else { (e, s) };
                            self.selected_tracks = Arc::new(self.tracks[min..=max].to_vec());
                        }
                    } else {
                        self.selected_tracks = Arc::new(vec![track.clone()]);
                        self.last_clicked_track = Some(track.clone());
                    }
                } else {
                    self.selected_tracks = Arc::new(vec![track.clone()]);
                    self.last_clicked_track = Some(track.clone());
                }

                self.selected_track = Some(track.clone());

                let path = track.path.clone();
                let track_id = track.id;
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            crate::library::scanner::load_cover(&path)
                        })
                        .await
                        .unwrap_or(None)
                    },
                    move |cover| Message::CoverLoaded(track_id, cover),
                )
            }

            Message::CoverLoaded(track_id, cover_data) => {
                if let Some(ref mut track) = self.selected_track {
                    if track.id == track_id {
                        track.cover_data = cover_data.clone();
                    }
                }
                if let Some(ref mut track) = self.current_track {
                    if track.id == track_id {
                        track.cover_data = cover_data;
                    }
                }
                Task::none()
            }

            Message::SidebarSearchChanged(query) => {
                self.sidebar_search = query;
                Task::none()
            }

            Message::OpenShortcuts => {
                let cfg = crate::config::get();
                self.show_settings = Some(SettingsState {
                    music_dir: cfg.music_dir.clone(),
                    language: cfg.language.clone(),
                    seek_step: cfg.seek_step.to_string(),
                    volume_step: cfg.volume_step,
                    font_scale: self.font_scale,
                    initial_volume: cfg.volume,
                    playback_defaults: cfg.playback_defaults.clone(),
                    auto_scan: cfg.auto_scan.clone(),
                    theme_source: cfg.theme_source,
                    theme_preset: cfg.theme_preset,
                    custom_theme: cfg.custom_theme.unwrap_or_default(),
                    custom_validation_errors: std::collections::HashMap::new(),
                    confirm_save_anyway: false,
                    selected_tab: SettingsTab::Shortcuts,
                    color_picker_token: None,
                    color_picker_r: 0.0,
                    color_picker_g: 0.0,
                    color_picker_b: 0.0,
                    show_achievements_in_ui: cfg.show_achievements_in_ui,
                });
                self.show_shortcuts = false;
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
                        if self.show_period_breakdown.is_some() {
                            return Task::done(Message::ClosePeriodBreakdown);
                        } else if has_shortcuts {
                            return Task::done(Message::CloseShortcuts);
                        } else if has_playlist_dialog {
                            return Task::done(Message::ClosePlaylistDialog);
                        } else if has_tag_editor {
                            return Task::done(Message::CloseTagEditor);
                        } else if has_context_menu {
                            return Task::done(Message::ToggleContextMenu(None));
                        } else if self.show_settings.is_some() {
                            return Task::done(Message::CloseSettings);
                        } else if self.show_song_search {
                            return Task::done(Message::ToggleSongSearch);
                        } else if self.show_sidebar_search {
                            return Task::done(Message::ToggleSidebarSearch);
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
                                    ViewMode::NowPlaying => {}
                                }
                                return Task::none();
                            } else if self.active_focus == Some(ActiveFocus::SidebarList) {
                                self.active_focus = Some(ActiveFocus::Tracklist);
                                if self.selected_track.is_none() {
                                    if let Some(track) = self.tracks.first().cloned() {
                                        let cover_data = load_cover(&track.path);
                                        let track = Track { cover_data, ..track };
                                        self.selected_track = Some(track.clone());
                                        self.selected_tracks = Arc::new(vec![track.clone()]);
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
                                        self.selected_tracks = Arc::new(vec![track.clone()]);
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
                                "]" => return Task::done(Message::IncreaseScale),
                                "[" => return Task::done(Message::DecreaseScale),
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
                self.queue = (*self.tracks).clone();
                self.set_playing_context_from_current_view();
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
                self.playing_context = Some(PlayingContext::Artist(artist_name.clone()));
                self.shuffle = crate::config::get().playback_defaults.artist.shuffle;
                // Shuffle tracks of this artist
                let mut artist_tracks = (*self.tracks).clone();
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
                self.playing_context = Some(PlayingContext::Album(album_name.clone()));
                let album_defaults = &crate::config::get().playback_defaults.album;
                self.shuffle = album_defaults.shuffle;
                self.repeat = album_defaults.repeat;
                
                // Sort by track number ascending
                Arc::make_mut(&mut self.tracks).sort_by_key(|t| t.track_number.unwrap_or(u32::MAX));
                self.queue = (*self.tracks).clone();
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
                let pd = &crate::config::get().playback_defaults;
                if playlist_name == "Liked Songs" || playlist_name == "Recently Played" || playlist_name == "Most Played" || playlist_name == "New Music" {
                    self.playing_context = Some(PlayingContext::Autoplaylist(playlist_name.clone()));
                    self.shuffle = pd.artist.shuffle;
                    self.repeat = pd.artist.repeat;
                } else if crate::db::get(|db| db.smart_playlists.contains_key(&playlist_name)) {
                    self.playing_context = Some(PlayingContext::SmartPlaylist(playlist_name.clone()));
                    self.shuffle = pd.smart_playlist.shuffle;
                    self.repeat = pd.smart_playlist.repeat;
                } else {
                    self.playing_context = Some(PlayingContext::Playlist(playlist_name.clone()));
                    self.shuffle = pd.user_playlist.shuffle;
                    self.repeat = pd.user_playlist.repeat;
                }
                self.queue = (*self.tracks).clone();
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
                self.view_mode = ViewMode::Genres;
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
                self.selected_genre = Some(genre_name.clone());
                self.selected_playlist = None;
                self.selected_folder = None;
                self.selected_artist = None;
                self.selected_album = None;
                self.search_query.clear();
                self.update_filtered_tracks();
                self.playing_context = Some(PlayingContext::Genre(genre_name));
                self.queue = (*self.tracks).clone();
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
                self.playlist_menu_expanded = false;
                Task::none()
            }

            Message::TogglePlaylistMenuExpanded => {
                self.playlist_menu_expanded = !self.playlist_menu_expanded;
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

            Message::RemoveTrackFromPlaylist(playlist_name, track) => {
                crate::db::remove_from_playlist(playlist_name, track.path);
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
                crate::db::write(|db| db.right_panel_tab = self.right_panel_tab);
                Task::none()
            }

            Message::ToggleSongSearch => {
                self.show_song_search = !self.show_song_search;
                if !self.show_song_search {
                    self.search_query.clear();
                    self.update_filtered_tracks();
                    Task::none()
                } else {
                    iced::widget::text_input::focus(iced::widget::text_input::Id::new("song_search_input"))
                }
            }

            Message::ToggleSidebarSearch => {
                self.show_sidebar_search = !self.show_sidebar_search;
                if !self.show_sidebar_search {
                    self.sidebar_search.clear();
                    self.update_filtered_tracks();
                    Task::none()
                } else {
                    iced::widget::text_input::focus(iced::widget::text_input::Id::new("sidebar_search_input"))
                }
            }

            Message::GlobalCursorMoved(pos) => {
                self.cursor_position = pos;
                Task::none()
            }

            Message::GlobalClick => {
                let tab_strip_visible = self.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 450.0);
                let tab_strip_offset = if tab_strip_visible { 56.0 } else { 0.0 };

                if self.show_song_search && self.search_query.is_empty() {
                    let search_right = self.window_width - tab_strip_offset - 12.0;
                    let search_left = search_right - 220.0 - 40.0;
                    let search_top = self.player_height - 28.0;
                    let search_bottom = self.player_height;

                    let px = self.cursor_position.x;
                    let py = self.cursor_position.y;
                    let clicked_inside_search = px >= search_left && px <= search_right && py >= search_top && py <= search_bottom;
                    if !clicked_inside_search {
                        self.show_song_search = false;
                        self.search_query.clear();
                        self.update_filtered_tracks();
                    }
                }

                if self.show_sidebar_search && self.sidebar_search.is_empty() {
                    let sidebar_right = self.sidebar_width;
                    let sidebar_left = 0.0;
                    let sidebar_top = self.player_height + 28.0;
                    let sidebar_bottom = sidebar_top + 28.0;

                    let px = self.cursor_position.x;
                    let py = self.cursor_position.y;
                    let clicked_inside_sidebar_search = px >= sidebar_left && px <= sidebar_right && py >= sidebar_top && py <= sidebar_bottom;
                    if !clicked_inside_sidebar_search {
                        self.show_sidebar_search = false;
                        self.sidebar_search.clear();
                        self.update_filtered_tracks();
                    }
                }

                Task::none()
            }

            Message::DismissNotification(id) => {
                self.active_notifications.retain(|n| n.id != id);
                Task::none()
            }

            Message::ShowPeriodBreakdown(period_idx) => {
                let breakdown = crate::stats::get_period_breakdown(period_idx, &self.all_tracks);
                self.show_period_breakdown = Some(breakdown);
                self.breakdown_period_idx = period_idx;
                self.recalculate_achievements_items();
                Task::none()
            }

            Message::ClosePeriodBreakdown => {
                self.show_period_breakdown = None;
                self.breakdown_song_view = None;
                self.achievements_cover_cache.lock().unwrap().clear();
                Task::none()
            }

            Message::CloseBreakdownSongView => {
                self.breakdown_song_view = None;
                Task::none()
            }

            Message::SelectStatsModalTab(tab) => {
                self.stats_modal_tab = tab;
                self.achievements_offset = 0;
                self.achievements_search_query.clear();
                if tab == StatsModalTab::Achievements {
                    self.recalculate_achievements_items();
                }
                Task::none()
            }

            Message::SelectAchievementsSubTab(sub_tab) => {
                self.achievements_sub_tab = sub_tab;
                self.achievements_offset = 0;
                self.achievements_search_query.clear();
                self.recalculate_achievements_items();
                Task::none()
            }

            Message::SelectAchievementsSort(sort) => {
                self.achievements_sort = sort;
                self.achievements_offset = 0;
                self.recalculate_achievements_items();
                Task::none()
            }

            Message::ShowMoreAchievements => {
                self.achievements_offset += 3;
                Task::none()
            }

            Message::ShowPreviousAchievements => {
                self.achievements_offset = self.achievements_offset.saturating_sub(3);
                Task::none()
            }

            Message::AchievementsSearchChanged(query) => {
                self.achievements_search_query = query;
                self.achievements_offset = 0;
                self.recalculate_achievements_items();
                Task::none()
            }

            Message::Noop => Task::none(),

            Message::SelectArtistFromBreakdown(artist) => {
                self.breakdown_song_view = Some(("Artist".to_string(), artist));
                Task::none()
            }

            Message::SelectAlbumFromBreakdown(album) => {
                self.breakdown_song_view = Some(("Album".to_string(), album));
                Task::none()
            }

            Message::SelectGenreFromBreakdown(genre) => {
                self.breakdown_song_view = Some(("Genre".to_string(), genre));
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
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ToggleTagFieldApplyLyrics(val) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.apply_lyrics = val;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ChangePendingLyricOffset(offset) => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.pending_offset += offset;
                    state.is_saved = false;
                }
                Task::none()
            }

            Message::ApplyPendingLyricOffset => {
                if let Some(ref mut state) = self.show_tag_editor {
                    let current_text = state.lyrics_content.text();
                    let new_text = crate::ui::lyrics_shift::shift_lrc_timestamps(&current_text, state.pending_offset);
                    state.lyrics_content = iced::widget::text_editor::Content::with_text(&new_text);
                    state.apply_lyrics = true;
                    state.pending_offset = 0.0;
                }
                Task::none()
            }

            Message::ResetPendingLyricOffset => {
                if let Some(ref mut state) = self.show_tag_editor {
                    state.pending_offset = 0.0;
                }
                Task::none()
            }

            Message::SearchLyricsOnline => {
                if let Some(ref state) = self.show_tag_editor {
                    let artist = state.artist.trim();
                    let album = state.album.trim();
                    let title = state.title.trim();
                    
                    let mut query_parts = Vec::new();
                    if !artist.is_empty() { query_parts.push(artist); }
                    if !album.is_empty() { query_parts.push(album); }
                    if !title.is_empty() { query_parts.push(title); }
                    
                    if !query_parts.is_empty() {
                        let query = query_parts.join(" ");
                        let mut encoded = String::new();
                        for c in query.chars() {
                            match c {
                                ' ' => encoded.push_str("%20"),
                                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => encoded.push(c),
                                _ => {
                                    encoded.push_str(&format!("%{:02X}", c as u32));
                                }
                            }
                        }
                        let url = format!("https://lrclib.net/search/{}", encoded);
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&url)
                            .spawn();
                    } else {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://lrclib.net")
                            .spawn();
                    }
                }
                Task::none()
            }

            Message::OpenSettings => {
                let cfg = crate::config::get();
                self.show_settings = Some(SettingsState {
                    music_dir: cfg.music_dir.clone(),
                    language: cfg.language.clone(),
                    seek_step: cfg.seek_step.to_string(),
                    volume_step: cfg.volume_step,
                    font_scale: self.font_scale,
                    initial_volume: cfg.volume,
                    playback_defaults: cfg.playback_defaults.clone(),
                    auto_scan: cfg.auto_scan.clone(),
                    theme_source: cfg.theme_source,
                    theme_preset: cfg.theme_preset,
                    custom_theme: cfg.custom_theme.unwrap_or_default(),
                    custom_validation_errors: std::collections::HashMap::new(),
                    confirm_save_anyway: false,
                    selected_tab: SettingsTab::Library,
                    color_picker_token: None,
                    color_picker_r: 0.0,
                    color_picker_g: 0.0,
                    color_picker_b: 0.0,
                    show_achievements_in_ui: cfg.show_achievements_in_ui,
                });
                Task::none()
            }

            Message::CloseSettings => {
                self.show_settings = None;
                let original_palette = crate::ui::theme::load_palette_from_config();
                crate::ui::theme::apply_palette(original_palette);
                self.iced_theme = build_iced_theme();
                Task::none()
            }

            Message::SettingsMusicDirChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.music_dir = val;
                }
                Task::none()
            }

            Message::SettingsLanguageChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.language = val;
                }
                Task::none()
            }

            Message::SettingsSeekStepChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.seek_step = val;
                }
                Task::none()
            }

            Message::SettingsVolumeStepChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.volume_step = val;
                }
                Task::none()
            }

            Message::SettingsFontScaleChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.font_scale = val;
                    self.font_scale = val;
                }
                Task::none()
            }

            Message::SettingsShowAchievementsInUiChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.show_achievements_in_ui = val;
                }
                Task::none()
            }

            Message::SettingsSave => {
                if let Some(ref mut state) = self.show_settings {
                    // Don't save if there are validation errors in hex codes
                    if state.theme_source == "Custom" && !state.custom_validation_errors.is_empty() {
                        return Task::none();
                    }

                    // Contrast warnings
                    if state.theme_source == "Custom" {
                        let warnings = crate::ui::theme::check_custom_contrast_warnings(&state.custom_theme);
                        if !warnings.is_empty() && !state.confirm_save_anyway {
                            state.confirm_save_anyway = true;
                            return Task::none();
                        }
                    }

                    let mut cfg = crate::config::get();
                    let old_music_path = cfg.music_path();
                    
                    cfg.music_dir = state.music_dir.clone();
                    cfg.language = state.language.clone();
                    if let Ok(seek) = state.seek_step.trim().parse::<u64>() {
                        cfg.seek_step = seek;
                    }
                    cfg.volume_step = state.volume_step;
                    cfg.font_scale = Some(state.font_scale);
                    cfg.volume = state.initial_volume;
                    cfg.playback_defaults = state.playback_defaults.clone();
                    cfg.auto_scan = state.auto_scan.clone();
                    cfg.show_achievements_in_ui = state.show_achievements_in_ui;
                    
                    cfg.theme_source = state.theme_source.clone();
                    cfg.theme_preset = state.theme_preset.clone();
                    cfg.custom_theme = Some(state.custom_theme.clone());
                    
                    crate::config::save(cfg.clone());
                    
                    // Reload active theme
                    let active_palette = crate::ui::theme::load_palette_from_config();
                    crate::ui::theme::apply_palette(active_palette);
                    self.iced_theme = build_iced_theme();
                    self.loaded_theme_name = if cfg.theme_source == "System" {
                        crate::ui::theme::read_current_theme_name()
                    } else {
                        String::new()
                    };

                    // Reload strings/locale
                    self.strings = crate::locale::get();
                    
                    self.show_settings = None;
                    
                    if cfg.music_path() != old_music_path {
                        let new_music_dir = cfg.music_path();
                        return Task::perform(
                            async move {
                                scan_folder(&new_music_dir)
                            },
                            Message::LibraryScanned,
                        );
                    }
                }
                Task::none()
            }

            Message::SettingsThemeSourceChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.theme_source = val.clone();
                    state.confirm_save_anyway = false;
                    
                    let preview_palette = match val.as_str() {
                        "Preset" => {
                            crate::ui::theme::get_preset_palette(&state.theme_preset)
                                .unwrap_or_else(|| crate::ui::theme::Palette::default_lavender())
                        }
                        "Custom" => {
                            let current_palette = crate::ui::theme::load_palette_from_config();
                            crate::ui::theme::Palette {
                                base: crate::ui::theme::hex_to_color(&state.custom_theme.base).unwrap_or(current_palette.base),
                                mantle: crate::ui::theme::hex_to_color(&state.custom_theme.mantle).unwrap_or(current_palette.mantle),
                                surface0: crate::ui::theme::hex_to_color(&state.custom_theme.surface0).unwrap_or(current_palette.surface0),
                                overlay0: crate::ui::theme::hex_to_color(&state.custom_theme.overlay0).unwrap_or(current_palette.overlay0),
                                text: crate::ui::theme::hex_to_color(&state.custom_theme.text).unwrap_or(current_palette.text),
                                subtext: crate::ui::theme::hex_to_color(&state.custom_theme.subtext).unwrap_or(current_palette.subtext),
                                accent: crate::ui::theme::hex_to_color(&state.custom_theme.accent).unwrap_or(current_palette.accent),
                                green: crate::ui::theme::hex_to_color(&state.custom_theme.green).unwrap_or(current_palette.green),
                                red: crate::ui::theme::hex_to_color(&state.custom_theme.red).unwrap_or(current_palette.red),
                                yellow: crate::ui::theme::hex_to_color(&state.custom_theme.yellow).unwrap_or(current_palette.yellow),
                                blue: crate::ui::theme::hex_to_color(&state.custom_theme.blue).unwrap_or(current_palette.blue),
                            }
                        }
                        _ => {
                            crate::ui::theme::load_palette_from_config()
                        }
                    };
                    crate::ui::theme::apply_palette(preview_palette);
                    self.iced_theme = build_iced_theme();
                }
                Task::none()
            }

            Message::SettingsThemePresetChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.theme_preset = val.clone();
                    state.confirm_save_anyway = false;
                    
                    if let Some(preset) = crate::ui::theme::get_preset_palette(&val) {
                        crate::ui::theme::apply_palette(preset);
                        self.iced_theme = build_iced_theme();
                    }
                }
                Task::none()
            }

            Message::SettingsCustomColorChanged(token, val) => {
                if let Some(ref mut state) = self.show_settings {
                    match token.as_str() {
                        "base" => state.custom_theme.base = val.clone(),
                        "text" => state.custom_theme.text = val.clone(),
                        "accent" => state.custom_theme.accent = val.clone(),
                        "green" => state.custom_theme.green = val.clone(),
                        "red" => state.custom_theme.red = val.clone(),
                        "yellow" => state.custom_theme.yellow = val.clone(),
                        "blue" => state.custom_theme.blue = val.clone(),
                        _ => {}
                    }
                    state.confirm_save_anyway = false;
                    
                    let is_valid = crate::ui::theme::hex_to_color(&val).is_some();
                    if is_valid {
                        state.custom_validation_errors.remove(&token);
                    } else {
                        state.custom_validation_errors.insert(token.clone(), "Invalid hex (format: #RRGGBB)".to_string());
                    }
                    
                    if is_valid && (token == "base" || token == "text") {
                        if let (Some(base_col), Some(text_col)) = (
                            crate::ui::theme::hex_to_color(&state.custom_theme.base),
                            crate::ui::theme::hex_to_color(&state.custom_theme.text),
                        ) {
                            let is_dark = crate::ui::theme::luminance(base_col) < 0.5;
                            let mantle_col = crate::ui::theme::derive_mantle(base_col, is_dark);
                            let surface0_col = crate::ui::theme::derive_surface0(base_col, is_dark);
                            let overlay0_col = crate::ui::theme::derive_overlay0(base_col, is_dark);
                            let subtext_col = crate::ui::theme::derive_subtext(text_col, is_dark);

                            state.custom_theme.mantle = format!("#{:02x}{:02x}{:02x}", (mantle_col.r * 255.0) as u8, (mantle_col.g * 255.0) as u8, (mantle_col.b * 255.0) as u8);
                            state.custom_theme.surface0 = format!("#{:02x}{:02x}{:02x}", (surface0_col.r * 255.0) as u8, (surface0_col.g * 255.0) as u8, (surface0_col.b * 255.0) as u8);
                            state.custom_theme.overlay0 = format!("#{:02x}{:02x}{:02x}", (overlay0_col.r * 255.0) as u8, (overlay0_col.g * 255.0) as u8, (overlay0_col.b * 255.0) as u8);
                            state.custom_theme.subtext = format!("#{:02x}{:02x}{:02x}", (subtext_col.r * 255.0) as u8, (subtext_col.g * 255.0) as u8, (subtext_col.b * 255.0) as u8);
                        }
                    }
                    
                    if state.custom_validation_errors.is_empty() {
                        let current_palette = crate::ui::theme::load_palette_from_config();
                        let preview_palette = crate::ui::theme::Palette {
                            base: crate::ui::theme::hex_to_color(&state.custom_theme.base).unwrap_or(current_palette.base),
                            mantle: crate::ui::theme::hex_to_color(&state.custom_theme.mantle).unwrap_or(current_palette.mantle),
                            surface0: crate::ui::theme::hex_to_color(&state.custom_theme.surface0).unwrap_or(current_palette.surface0),
                            overlay0: crate::ui::theme::hex_to_color(&state.custom_theme.overlay0).unwrap_or(current_palette.overlay0),
                            text: crate::ui::theme::hex_to_color(&state.custom_theme.text).unwrap_or(current_palette.text),
                            subtext: crate::ui::theme::hex_to_color(&state.custom_theme.subtext).unwrap_or(current_palette.subtext),
                            accent: crate::ui::theme::hex_to_color(&state.custom_theme.accent).unwrap_or(current_palette.accent),
                            green: crate::ui::theme::hex_to_color(&state.custom_theme.green).unwrap_or(current_palette.green),
                            red: crate::ui::theme::hex_to_color(&state.custom_theme.red).unwrap_or(current_palette.red),
                            yellow: crate::ui::theme::hex_to_color(&state.custom_theme.yellow).unwrap_or(current_palette.yellow),
                            blue: crate::ui::theme::hex_to_color(&state.custom_theme.blue).unwrap_or(current_palette.blue),
                        };
                        crate::ui::theme::apply_palette(preview_palette);
                        self.iced_theme = build_iced_theme();
                    }
                }
                Task::none()
            }

            Message::SettingsColorPickerToggle(token) => {
                if let Some(ref mut state) = self.show_settings {
                    if state.color_picker_token.as_deref() == Some(&token) {
                        state.color_picker_token = None;
                    } else {
                        let hex = match token.as_str() {
                            "base" => &state.custom_theme.base,
                            "text" => &state.custom_theme.text,
                            "accent" => &state.custom_theme.accent,
                            "green" => &state.custom_theme.green,
                            "red" => &state.custom_theme.red,
                            "yellow" => &state.custom_theme.yellow,
                            "blue" => &state.custom_theme.blue,
                            _ => "#000000",
                        };
                        let clean = hex.trim_start_matches('#');
                        if clean.len() >= 6 {
                            state.color_picker_r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f32;
                            state.color_picker_g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f32;
                            state.color_picker_b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f32;
                        }
                        state.color_picker_token = Some(token);
                    }
                }
                Task::none()
            }

            Message::SettingsColorPickerRChanged(val) => {
                let result = self.show_settings.as_mut().map(|state| {
                    state.color_picker_r = val;
                    state.color_picker_token.clone().map(|t| {
                        let hex = format!("#{:02x}{:02x}{:02x}",
                            state.color_picker_r.round() as u8,
                            state.color_picker_g.round() as u8,
                            state.color_picker_b.round() as u8);
                        (t, hex)
                    })
                }).flatten();
                if let Some((token, hex)) = result {
                    Task::perform(async move { (token, hex) }, |(t, h)| Message::SettingsCustomColorChanged(t, h))
                } else {
                    Task::none()
                }
            }

            Message::SettingsColorPickerGChanged(val) => {
                let result = self.show_settings.as_mut().map(|state| {
                    state.color_picker_g = val;
                    state.color_picker_token.clone().map(|t| {
                        let hex = format!("#{:02x}{:02x}{:02x}",
                            state.color_picker_r.round() as u8,
                            state.color_picker_g.round() as u8,
                            state.color_picker_b.round() as u8);
                        (t, hex)
                    })
                }).flatten();
                if let Some((token, hex)) = result {
                    Task::perform(async move { (token, hex) }, |(t, h)| Message::SettingsCustomColorChanged(t, h))
                } else {
                    Task::none()
                }
            }

            Message::SettingsColorPickerBChanged(val) => {
                let result = self.show_settings.as_mut().map(|state| {
                    state.color_picker_b = val;
                    state.color_picker_token.clone().map(|t| {
                        let hex = format!("#{:02x}{:02x}{:02x}",
                            state.color_picker_r.round() as u8,
                            state.color_picker_g.round() as u8,
                            state.color_picker_b.round() as u8);
                        (t, hex)
                    })
                }).flatten();
                if let Some((token, hex)) = result {
                    Task::perform(async move { (token, hex) }, |(t, h)| Message::SettingsCustomColorChanged(t, h))
                } else {
                    Task::none()
                }
            }

            Message::SettingsTabChanged(tab) => {
                if let Some(ref mut state) = self.show_settings {
                    state.selected_tab = tab;
                }
                Task::none()
            }

            Message::SettingsInitialVolumeChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.initial_volume = val.clamp(0.0, 1.0);
                }
                Task::none()
            }

            Message::SettingsPlaybackDefaultChanged(context, field, value) => {
                if let Some(ref mut state) = self.show_settings {
                    let entry = match context.as_str() {
                        "album" => &mut state.playback_defaults.album,
                        "artist" => &mut state.playback_defaults.artist,
                        "genre" => &mut state.playback_defaults.genre,
                        "user_playlist" => &mut state.playback_defaults.user_playlist,
                        "smart_playlist" => &mut state.playback_defaults.smart_playlist,
                        _ => return Task::none(),
                    };
                    match field.as_str() {
                        "shuffle" => entry.shuffle = value,
                        "repeat" => entry.repeat = value,
                        _ => {}
                    }
                }
                Task::none()
            }

            Message::SettingsAutoScanModeChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    state.auto_scan.mode = val;
                }
                Task::none()
            }

            Message::SettingsAutoScanIntervalChanged(val) => {
                if let Some(ref mut state) = self.show_settings {
                    if let Ok(interval) = val.trim().parse::<u64>() {
                        state.auto_scan.interval_minutes = interval.max(1);
                    }
                }
                Task::none()
            }

            Message::PickMusicFolder => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Choose Music Library Folder")
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::MusicFolderPicked,
                );
            }

            Message::MusicFolderPicked(opt) => {
                if let Some(path) = opt {
                    if let Some(ref mut state) = self.show_settings {
                        state.music_dir = path.to_string_lossy().to_string();
                    }
                }
                Task::none()
            }

            Message::PlayNext(tracks) => {
                if self.queue.is_empty() {
                    self.queue = tracks;
                    if let Some(first) = self.queue.first().cloned() {
                        return self.play_track_internal(first);
                    }
                } else {
                    let current_idx = self.current_track.as_ref()
                        .and_then(|ct| self.queue.iter().position(|t| t.id == ct.id));
                    if let Some(idx) = current_idx {
                        for (offset, track) in tracks.into_iter().enumerate() {
                            self.queue.insert(idx + 1 + offset, track);
                        }
                    } else {
                        self.queue.extend(tracks);
                    }
                    let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                    crate::db::write(|db| {
                        db.last_queue_paths = queue_paths;
                    });
                }
                Task::none()
            }

            Message::AddToQueue(tracks) => {
                let play_first = self.queue.is_empty();
                self.queue.extend(tracks);
                let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                crate::db::write(|db| {
                    db.last_queue_paths = queue_paths;
                });
                if play_first {
                    if let Some(first) = self.queue.first().cloned() {
                        return self.play_track_internal(first);
                    }
                }
                Task::none()
            }

            Message::PlayQueueTrack(index) => {
                if let Some(track) = self.queue.get(index).cloned() {
                    return self.play_track_internal(track);
                }
                Task::none()
            }

            Message::SelectQueueTrack(index, track) => {
                let now = std::time::Instant::now();
                if let Some((prev_id, last_time)) = self.last_click_track {
                    if prev_id == track.id && now.duration_since(last_time) < std::time::Duration::from_millis(350) {
                        self.last_click_track = None;
                        return Task::done(Message::PlayQueueTrack(index));
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
                        Arc::make_mut(&mut self.selected_tracks).retain(|t| t.id != track.id);
                    } else {
                        Arc::make_mut(&mut self.selected_tracks).push(track.clone());
                    }
                    self.last_clicked_track = Some(track.clone());
                } else if shift_held {
                    if let Some(ref start_track) = self.last_clicked_track {
                        let start_idx = self.tracks.iter().position(|t| t.id == start_track.id);
                        let end_idx = self.tracks.iter().position(|t| t.id == track.id);
                        if let (Some(s), Some(e)) = (start_idx, end_idx) {
                            let (min, max) = if s < e { (s, e) } else { (e, s) };
                            self.selected_tracks = Arc::new(self.tracks[min..=max].to_vec());
                        }
                    } else {
                        self.selected_tracks = Arc::new(vec![track.clone()]);
                        self.last_clicked_track = Some(track.clone());
                    }
                } else {
                    self.selected_tracks = Arc::new(vec![track.clone()]);
                    self.last_clicked_track = Some(track.clone());
                }

                self.selected_track = Some(track);
                Task::none()
            }

            Message::RemoveQueueTrack(index) => {
                if index < self.queue.len() {
                    self.queue.remove(index);
                    let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                    crate::db::write(|db| {
                        db.last_queue_paths = queue_paths;
                    });
                }
                Task::none()
            }

            Message::MoveQueueTrackUp(index) => {
                if index > 0 && index < self.queue.len() {
                    self.queue.swap(index, index - 1);
                    let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                    crate::db::write(|db| {
                        db.last_queue_paths = queue_paths;
                    });
                }
                Task::none()
            }

            Message::MoveQueueTrackDown(index) => {
                if index < self.queue.len() - 1 {
                    self.queue.swap(index, index + 1);
                    let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                    crate::db::write(|db| {
                        db.last_queue_paths = queue_paths;
                    });
                }
                Task::none()
            }

            Message::ClearQueue => {
                self.queue.clear();
                let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                crate::db::write(|db| {
                    db.last_queue_paths = queue_paths;
                });
                Task::none()
            }

            Message::QueueDragStart(index) => {
                self.dragging_queue_index = Some(index);
                Task::none()
            }

            Message::QueueDragOver(target_idx) => {
                if let Some(source_idx) = self.dragging_queue_index {
                    if source_idx != target_idx && source_idx < self.queue.len() && target_idx < self.queue.len() {
                        let item = self.queue.remove(source_idx);
                        self.queue.insert(target_idx, item);
                        self.dragging_queue_index = Some(target_idx);
                        let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
                        crate::db::write(|db| {
                            db.last_queue_paths = queue_paths;
                        });
                    }
                }
                Task::none()
            }

            Message::QueueDragEnd => {
                self.dragging_queue_index = None;
                Task::none()
            }

            Message::PlaylistSidebarDragStart(tab, idx) => {
                self.dragging_playlist_sidebar = Some((tab, idx));
                Task::none()
            }

            Message::PlaylistSidebarDragOver(tab, target_idx) => {
                if let Some((source_tab, source_idx)) = self.dragging_playlist_sidebar {
                    if source_tab == tab && source_idx != target_idx {
                        crate::db::write(|db| {
                            let order = match tab {
                                PlaylistTab::Playlists => &mut db.playlist_order,
                                PlaylistTab::Smart => &mut db.smart_playlist_order,
                                _ => return,
                            };
                            if source_idx < order.len() && target_idx < order.len() {
                                let item = order.remove(source_idx);
                                order.insert(target_idx, item);
                            }
                        });
                        self.dragging_playlist_sidebar = Some((tab, target_idx));
                    }
                }
                Task::none()
            }

            Message::PlaylistSidebarDragEnd => {
                self.dragging_playlist_sidebar = None;
                Task::none()
            }

            Message::TrackListDragStart(idx) => {
                self.dragging_track_index = Some(idx);
                Task::none()
            }

            Message::TrackListDragOver(target_idx) => {
                if let Some(source_idx) = self.dragging_track_index {
                    if source_idx != target_idx && source_idx < self.tracks.len() && target_idx < self.tracks.len() {
                        let tracks = Arc::make_mut(&mut self.tracks);
                        let track = tracks.remove(source_idx);
                        tracks.insert(target_idx, track);
                        self.dragging_track_index = Some(target_idx);

                        let new_paths: Vec<PathBuf> = self.tracks.iter().map(|t| t.path.clone()).collect();
                        if let Some(name) = &self.selected_playlist.clone() {
                            let name = name.clone();
                            if crate::db::get(|db| db.playlists.contains_key(&name)) {
                                crate::db::write(|db| {
                                    db.playlists.insert(name, new_paths);
                                });
                            } else if crate::db::get(|db| db.smart_playlists.contains_key(&name)) {
                                crate::db::write(|db| {
                                    db.smart_playlist_song_order.insert(name, new_paths);
                                });
                            } else if name == "Liked Songs" || name == "New Music" {
                                crate::db::write(|db| {
                                    db.auto_playlist_song_order.insert(name, new_paths);
                                });
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::TrackListDragEnd => {
                self.dragging_track_index = None;
                Task::none()
            }

            Message::ResetPlaylistSongOrder => {
                if let Some(name) = &self.selected_playlist.clone() {
                    let name = name.clone();
                    crate::db::write(|db| {
                        db.smart_playlist_song_order.remove(&name);
                        db.auto_playlist_song_order.remove(&name);
                    });
                    self.update_filtered_tracks();
                }
                self.show_context_menu = None;
                Task::none()
            }

            Message::ColumnHeaderDragStart(col) => {
                self.dragging_column_header = Some(col);
                self.column_drag_moved = false;
                Task::none()
            }

            Message::ColumnHeaderDragOver(target_col) => {
                if let Some(source_col) = self.dragging_column_header {
                    if source_col != target_col {
                        crate::db::write(|db| {
                            let cols = &mut db.table_columns;
                            if let (Some(src_pos), Some(tgt_pos)) = (
                                cols.iter().position(|&c| c == source_col),
                                cols.iter().position(|&c| c == target_col),
                            ) {
                                let item = cols.remove(src_pos);
                                cols.insert(tgt_pos, item);
                            }
                        });
                        self.dragging_column_header = Some(target_col);
                        self.column_drag_moved = true;
                    }
                }
                Task::none()
            }

            Message::ColumnHeaderDragEnd => {
                if let Some(col) = self.dragging_column_header {
                    if !self.column_drag_moved {
                        let sort_col = crate::ui::views::library::table_col_to_sort_col(col);
                        if self.sort_column == Some(sort_col) {
                            self.sort_ascending = !self.sort_ascending;
                        } else {
                            self.sort_column = Some(sort_col);
                            self.sort_ascending = true;
                        }
                        self.update_filtered_tracks();
                    }
                }
                self.dragging_column_header = None;
                self.column_drag_moved = false;
                Task::none()
            }
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let player_controls = views::player::view(self);
        let library_tabs = views::library::library_top_bar(self);

        let tab_strip_visible = self.window_width >= (crate::app::MIN_NON_DRAWER_WIDTH + 450.0);

        let main_left_content = stack![
            container(player_controls)
                .width(Length::Fill)
                .height(iced::Length::Fixed(self.player_height - 28.0)),
            container(library_tabs)
                .padding(iced::Padding { top: self.player_height - 29.0, right: 0.0, bottom: 0.0, left: 0.0 })
                .width(Length::Fill)
                .height(iced::Length::Fixed(self.player_height)),
        ]
        .width(Length::Fill)
        .height(iced::Length::Fixed(self.player_height));

        let left_top: Element<'_, Message> = if tab_strip_visible {
            let tab_strip = views::player::tab_strip(self);
            row![
                main_left_content,
                tab_strip
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(iced::Length::Fixed(self.player_height))
            .into()
        } else {
            main_left_content.into()
        };

        let mut top_row = row![left_top]
            .width(Length::Fill)
            .height(iced::Length::Fixed(self.player_height));

        if let Some(pane) = views::player::right_panel(self) {
            top_row = top_row.push(pane);
        }

        let player_drag_handle = mouse_area(
            container(
                container(Space::new(Length::Fill, Length::Fixed(1.0)))
                    .style(move |_| iced::widget::container::Style {
                        background: Some(iced::Background::Color(
                            if self.dragging_player_split || self.is_hovering_player_resizer {
                                theme::accent()
                            } else {
                                theme::surface0()
                            }
                        )),
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .height(6.0)
            .center_y(Length::Fixed(6.0))
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::base())),
                ..Default::default()
            })
        )
        .on_press(Message::PlayerDragStart)
        .on_enter(Message::HoverPlayerResizer(true))
        .on_exit(Message::HoverPlayerResizer(false))
        .interaction(iced::mouse::Interaction::ResizingVertically);

        let main = column![
            top_row,
            player_drag_handle,
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

            let mut unique_genres: Vec<String> = self.all_tracks.iter()
                .flat_map(|t| {
                    if t.genre.contains("; ") {
                        t.genre.split("; ").map(|g| g.trim().to_string()).collect::<Vec<_>>()
                    } else {
                        vec![t.genre.clone()]
                    }
                })
                .filter(|s| !s.trim().is_empty())
                .collect();
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
        } else if let Some(ref settings_state) = self.show_settings {
            view_stack = view_stack.push(crate::ui::components::settings_dialog::view(settings_state));
        } else if self.show_shortcuts {
            view_stack = view_stack.push(self.shortcuts_modal_view());
        }

        // Period breakdown popup overlay
        if let Some(ref breakdown) = self.show_period_breakdown {
            if let Some(ref song_view) = self.breakdown_song_view {
                view_stack = view_stack.push(
                    crate::ui::views::player::song_breakdown_view(
                        &song_view.0, &song_view.1, self.breakdown_period_idx, &self.all_tracks,
                    )
                );
            } else {
                view_stack = view_stack.push(crate::ui::views::player::period_breakdown_view(self));
            }
        }


        // Queue popover overlay
        if self.show_queue_popover
            && self.show_tag_editor.is_none()
            && self.playlist_dialog.is_none()
            && self.show_settings.is_none()
            && !self.show_shortcuts
        {
            view_stack = view_stack.push(self.queue_popover_view());
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

            let is_playlist_target = matches!(
                target,
                ContextMenuTarget::Artist(_)
                    | ContextMenuTarget::Album(_)
                    | ContextMenuTarget::Track(_)
                    | ContextMenuTarget::MultipleTracks(_)
            );

            let mut playlist_select = column![].spacing(4);

            if is_playlist_target {
                let arrow_icon = if self.playlist_menu_expanded { "▼ " } else { "▶ " };
                let header_btn = button(
                    row![
                        text(arrow_icon).font(crate::ui::icons::NERD_FONT_MONO).size(11).color(theme::subtext()),
                        text("Add to Playlist")
                            .size(14)
                            .color(theme::subtext())
                            .font(crate::ui::icons::UI_FONT_BOLD)
                    ].spacing(4)
                )
                .on_press(Message::TogglePlaylistMenuExpanded)
                .style(iced::widget::button::text)
                .padding([2, 4])
                .width(Length::Fill);

                playlist_select = playlist_select.push(header_btn);
            }

            let (title, hide_btn, create_btn): (String, Option<Element<'_, Message>>, _) = match target {
                ContextMenuTarget::Artist(artist_name) => {
                    let title = format!("Artist Menu: {artist_name}");
                    let hide = button(text("Hide from UI").size(15))
                        .on_press(Message::HideAlbumOrArtist(artist_name.clone(), true))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    
                    if self.playlist_menu_expanded {
                        let mut pl_col = column![].spacing(4);
                        for pl in &custom_playlists {
                            let artist_tracks: Vec<Track> = self.all_tracks.iter()
                                .filter(|t| {
                                    let a = if t.artist.trim().is_empty() { "Unknown Artist" } else { &t.artist };
                                    a == artist_name
                                })
                                .cloned()
                                .collect();
                            pl_col = pl_col.push(
                                button(text(format!("  + {}", pl)).size(15))
                                    .on_press(Message::AddTracksToPlaylist(pl.clone(), artist_tracks))
                                    .style(item_style)
                                    .padding([4, 8])
                                    .width(Length::Fill)
                            );
                        }
                        playlist_select = playlist_select.push(pl_col);
                    }

                    let create = button(text("+ Create playlist with this artist").size(15))
                        .on_press(Message::CreatePlaylistFromContext(artist_name.clone(), true))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    (title, Some(hide.into()), create)
                }
                ContextMenuTarget::Album(album_name) => {
                    let title = format!("Album Menu: {album_name}");
                    let hide = button(text("Hide from UI").size(15))
                        .on_press(Message::HideAlbumOrArtist(album_name.clone(), false))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    if self.playlist_menu_expanded {
                        let mut pl_col = column![].spacing(4);
                        for pl in &custom_playlists {
                            let album_tracks: Vec<Track> = self.all_tracks.iter()
                                .filter(|t| {
                                    let al = if t.album.trim().is_empty() { "Unknown Album" } else { &t.album };
                                    al == album_name
                                })
                                .cloned()
                                .collect();
                            pl_col = pl_col.push(
                                button(text(format!("  + {}", pl)).size(15))
                                    .on_press(Message::AddTracksToPlaylist(pl.clone(), album_tracks))
                                    .style(item_style)
                                    .padding([4, 8])
                                    .width(Length::Fill)
                            );
                        }
                        playlist_select = playlist_select.push(pl_col);
                    }

                    let create = button(text("+ Create playlist with this album").size(15))
                        .on_press(Message::CreatePlaylistFromContext(album_name.clone(), false))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    (title, Some(hide.into()), create)
                }
                ContextMenuTarget::Track(track) => {
                    let title = format!("Song Menu: {}", track.title);
                    
                    if self.playlist_menu_expanded {
                        let mut pl_col = column![].spacing(4);
                        for pl in &custom_playlists {
                            pl_col = pl_col.push(
                                button(text(format!("  + {}", pl)).size(15))
                                    .on_press(Message::AddTracksToPlaylist(pl.clone(), vec![track.clone()]))
                                    .style(item_style)
                                    .padding([4, 8])
                                    .width(Length::Fill)
                            );
                        }
                        playlist_select = playlist_select.push(pl_col);
                    }

                    let create = button(text("+ Create playlist with this song").size(15))
                        .on_press(Message::CreatePlaylistWithTracks(track.title.clone(), vec![track.clone()]))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let play_next_btn = button(text("Play Next").size(15))
                        .on_press(Message::PlayNext(vec![track.clone()]))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let add_queue_btn = button(text("Add to Queue").size(15))
                        .on_press(Message::AddToQueue(vec![track.clone()]))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let like_label = if track.liked { "Unlike this song" } else { "Like this song" };
                    let like_btn = button(text(like_label).size(15))
                        .on_press(Message::ToggleLikeTrack(track.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let tag_btn = button(text("Edit ID3 tag").size(15))
                        .on_press(Message::OpenTagEditor(vec![track.clone()]))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let folder_btn = button(text("Open local file folder").size(15))
                        .on_press(Message::OpenLocalFolder(track.path.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let mut track_actions = column![
                        play_next_btn,
                        Space::with_height(4),
                        add_queue_btn,
                        Space::with_height(4),
                        like_btn,
                        Space::with_height(4),
                        tag_btn,
                        Space::with_height(4),
                        folder_btn,
                    ];

                    if self.playlist_tab == PlaylistTab::Playlists {
                        if let Some(playlist_name) = &self.selected_playlist {
                            let is_member = crate::db::get(|db| {
                                db.playlists.get(playlist_name)
                                    .map(|paths| paths.contains(&track.path))
                                    .unwrap_or(false)
                            });

                            if is_member {
                                let remove_btn = button(text("Remove from current playlist").size(15))
                                    .on_press(Message::RemoveTrackFromPlaylist(playlist_name.clone(), track.clone()))
                                    .style(item_style)
                                    .padding([4, 8])
                                    .width(Length::Fill);
                                track_actions = track_actions.push(Space::with_height(4)).push(remove_btn);
                            }
                        }
                    }

                    let mut show_reset_order = false;
                    if let Some(name) = &self.selected_playlist {
                        if name != "Recently Played" && name != "Most Played" {
                            let has_smart = crate::db::get(|db| db.smart_playlist_song_order.contains_key(name));
                            let has_auto = crate::db::get(|db| db.auto_playlist_song_order.contains_key(name));
                            show_reset_order = has_smart || has_auto;
                        }
                    }

                    if show_reset_order {
                        let reset_order_btn = button(text("Reset to natural order").size(15))
                            .on_press(Message::ResetPlaylistSongOrder)
                            .style(item_style)
                            .padding([4, 8])
                            .width(Length::Fill);

                        track_actions = track_actions.push(Space::with_height(4)).push(reset_order_btn);
                    }

                    (title, Some(track_actions.into()), create)
                }
                ContextMenuTarget::MultipleTracks(tracks) => {
                    let title = format!("Selection Menu: {} Songs", tracks.len());

                    if self.playlist_menu_expanded {
                        let mut pl_col = column![].spacing(4);
                        for pl in &custom_playlists {
                            pl_col = pl_col.push(
                                button(text(format!("  + {}", pl)).size(15))
                                    .on_press(Message::AddTracksToPlaylist(pl.clone(), tracks.clone()))
                                    .style(item_style)
                                    .padding([4, 8])
                                    .width(Length::Fill)
                            );
                        }
                        playlist_select = playlist_select.push(pl_col);
                    }

                    let play_next_btn = button(text("Play Next").size(15))
                        .on_press(Message::PlayNext(tracks.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let add_queue_btn = button(text("Add to Queue").size(15))
                        .on_press(Message::AddToQueue(tracks.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let tag_btn = button(text("Edit ID3 tags").size(15))
                        .on_press(Message::OpenTagEditor(tracks.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let create = button(text("+ Create playlist with selection").size(15))
                        .on_press(Message::CreatePlaylistWithTracks("Selected Tracks Playlist".to_string(), tracks.clone()))
                        .style(accent_item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                    let selection_actions = column![
                        play_next_btn,
                        Space::with_height(4),
                        add_queue_btn,
                        Space::with_height(4),
                        tag_btn,
                    ];

                    (title, Some(selection_actions.into()), create)
                }
                ContextMenuTarget::Playlist(name) => {
                    let title = format!("Playlist: {name}");
                    let rename_btn = button(text("Rename Playlist").size(15))
                        .on_press(Message::OpenPlaylistDialog(PlaylistDialogMode::Rename(name.clone())))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    let delete_btn = button(text("Delete Playlist").size(15))
                        .on_press(Message::DeletePlaylist(name.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    let playlist_actions = column![
                        rename_btn,
                        Space::with_height(4),
                        delete_btn,
                    ];
                    let dummy_create = button(text(""))
                        .style(iced::widget::button::text)
                        .padding(0);
                    (title, Some(playlist_actions.into()), dummy_create)
                }
                ContextMenuTarget::SmartPlaylist(name) => {
                    let title = format!("Smart Playlist: {name}");
                    let edit_btn = button(text("Edit Smart Playlist").size(15))
                        .on_press(Message::EditSmartPlaylist(name.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    let delete_btn = button(text("Delete Smart Playlist").size(15))
                        .on_press(Message::DeleteSmartPlaylist(name.clone()))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);
                    let playlist_actions = column![
                        edit_btn,
                        Space::with_height(4),
                        delete_btn,
                    ];
                    let dummy_create = button(text(""))
                        .style(iced::widget::button::text)
                        .padding(0);
                    (title, Some(playlist_actions.into()), dummy_create)
                }
                ContextMenuTarget::Header(clicked_col) => {
                    let title = "Table Columns".to_string();
                    let active_cols = crate::db::get(|db| db.table_columns.clone());
                    
                    let mut cols_col = column![
                        text("Show / Hide:")
                            .size(13)
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
                                text(col_label).size(14).color(theme::text())
                            ].spacing(8)
                        )
                        .on_press(Message::ToggleColumnVisibility(col))
                        .style(item_style)
                        .padding([4, 8])
                        .width(Length::Fill);

                        cols_col = cols_col.push(btn);
                    }

                    playlist_select = cols_col;
                    
                    let dummy_create = button(text(""))
                        .style(iced::widget::button::text)
                        .padding(0);

                    (title, None, dummy_create)
                }
            };

            if !is_playlist_target || self.playlist_menu_expanded {
                playlist_select = playlist_select.push(Space::with_height(4)).push(create_btn);
            }

            let mut menu_col = column![
                row![
                    text(title)
                        .size(15)
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

        if !self.active_notifications.is_empty() {
            let mut toasts_col = column![].spacing(8);
            for n in &self.active_notifications {
                let close_btn = button(
                    text("\u{f00d}")
                        .size(12)
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .color(theme::subtext())
                )
                .on_press(Message::DismissNotification(n.id))
                .padding(4)
                .style(|_, status| {
                    let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
                    iced::widget::button::Style {
                        background: None,
                        text_color: if is_hovered { theme::accent() } else { theme::subtext() },
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                });

                let msg_lines: Vec<&str> = n.message.split('\n').collect();
                let mut msg_col = column![].spacing(2);
                for line in msg_lines {
                    msg_col = msg_col.push(
                        text(line)
                            .size(13)
                            .font(crate::ui::icons::UI_FONT)
                            .color(theme::text())
                    );
                }

                let toast_card = container(
                    column![
                        row![
                            text(crate::ui::icons::ICON_MUSIC)
                                .font(crate::ui::icons::NERD_FONT_MONO)
                                .size(16)
                                .color(theme::accent()),
                            Space::with_width(8),
                            text(&n.title)
                                .size(14)
                                .font(crate::ui::icons::UI_FONT_BOLD)
                                .color(theme::accent())
                                .width(Length::Fill),
                            close_btn,
                        ]
                        .align_y(Alignment::Center),
                        Space::with_height(6),
                        msg_col,
                    ]
                    .spacing(0)
                )
                .width(Length::Fixed(360.0))
                .padding(12)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::surface0())),
                    border: iced::Border {
                        color: theme::accent(),
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                });
                toasts_col = toasts_col.push(toast_card);
            }
            
            let toasts_overlay = container(toasts_col)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .padding(iced::Padding { top: 0.0, right: 24.0, bottom: 24.0, left: 0.0 });
                
            view_stack = view_stack.push(toasts_overlay);
        }

        view_stack.into()
    }

    fn queue_popover_view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space, stack};
        use iced::{Alignment, Length};
        use crate::ui::theme;

        // Dismiss layer: transparent full-window click target behind the panel
        let dismiss_layer = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(
                        iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0)
                    )),
                    ..Default::default()
                })
        )
        .on_press(Message::CloseQueuePopover);

        // Header row: "Queue" label + count + Clear button
        let queue_count = self.queue.len();
        let header = row![
            text(format!("Queue ({queue_count})"))
                .size(12)
                .font(crate::ui::icons::UI_FONT_BOLD)
                .color(theme::subtext())
                .width(Length::Fill),
            button(
                text("Clear")
                    .size(11)
                    .color(theme::red())
            )
            .on_press(Message::ClearQueue)
            .style(move |_: &iced::Theme, status: iced::widget::button::Status| {
                let hovered = status == iced::widget::button::Status::Hovered
                    || status == iced::widget::button::Status::Pressed;
                iced::widget::button::Style {
                    text_color: theme::red(),
                    background: if hovered {
                        Some(iced::Background::Color(theme::surface0()))
                    } else {
                        None
                    },
                    border: iced::Border {
                        color: if hovered { theme::red() } else { iced::Color::TRANSPARENT },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .padding([2, 6]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([8, 12]);

        // Queue rows
        let current_track_id = self.current_track.as_ref().map(|t| t.id);
        let mut rows: Vec<Element<'_, Message>> = Vec::new();

        if self.queue.is_empty() {
            rows.push(
                container(
                    text("The play queue is empty.")
                        .size(13)
                        .color(theme::overlay0())
                )
                .padding([12, 12])
                .width(Length::Fill)
                .into()
            );
        } else {
            for (idx, track) in self.queue.iter().enumerate() {
                let is_current = current_track_id == Some(track.id);
                let title_color = if is_current { theme::accent() } else { theme::text() };

                // Drag handle
                let drag_handle = mouse_area(
                    container(
                        text("\u{f0c9}")
                            .font(crate::ui::icons::NERD_FONT_MONO)
                            .color(if self.dragging_queue_index == Some(idx) {
                                theme::accent()
                            } else {
                                theme::overlay0()
                            })
                            .size(11)
                    )
                    .padding([4, 6])
                )
                .on_press(Message::QueueDragStart(idx))
                .on_release(Message::QueueDragEnd)
                .interaction(iced::mouse::Interaction::Grab);

                // Remove (✕) button
                let remove_btn = button(
                    text("\u{f00d}")
                        .font(crate::ui::icons::NERD_FONT_MONO)
                        .size(12)
                        .color(theme::red())
                )
                .on_press(Message::RemoveQueueTrack(idx))
                .style(iced::widget::button::text)
                .padding([4, 4]);

                // Track info
                let title_txt = text(track.title.clone())
                    .size(13)
                    .color(title_color)
                    .width(Length::Fill);
                let artist_txt = text(track.artist.clone())
                    .size(11)
                    .color(theme::subtext())
                    .width(Length::Fill);

                let info_col = column![title_txt, artist_txt]
                    .spacing(2)
                    .width(Length::Fill);

                // Position number
                let pos_txt = text(format!("{}", idx + 1))
                    .size(11)
                    .color(theme::overlay0())
                    .width(Length::Fixed(20.0));

                let track_row_inner = row![
                    drag_handle,
                    pos_txt,
                    info_col,
                    remove_btn,
                ]
                .spacing(4)
                .align_y(Alignment::Center)
                .padding([4, 8]);

                // Detect if background is light or dark to compute the custom saturated panel style
                let base_color = theme::base();
                let is_dark = (base_color.r + base_color.g + base_color.b) / 3.0 < 0.5;

                // Alternate background logic:
                // We construct two contrasting colors using base and mantle, slightly shifting saturation/lightness.
                let row_bg_even = if is_current {
                    Some(iced::Background::Color(theme::with_alpha(theme::accent(), 0.12)))
                } else if idx % 2 == 1 {
                    if is_dark {
                        // Less saturated/slightly lighter for alternate rows on dark theme
                        Some(iced::Background::Color(theme::mantle()))
                    } else {
                        // More saturated/slightly darker for alternate rows on light theme
                        Some(iced::Background::Color(theme::mantle()))
                    }
                } else {
                    if is_dark {
                        Some(iced::Background::Color(theme::base()))
                    } else {
                        None
                    }
                };

                let mut row_element: Element<'_, Message> = mouse_area(
                    container(track_row_inner)
                        .width(Length::Fill)
                        .style(move |_| iced::widget::container::Style {
                            background: row_bg_even,
                            ..Default::default()
                        })
                )
                .on_press(Message::PlayQueueTrack(idx))
                .into();

                // Drag-over highlight: when dragging, wrap with mouse_area to detect hover
                if self.dragging_queue_index.is_some() {
                    row_element = mouse_area(row_element)
                        .on_enter(Message::QueueDragOver(idx))
                        .into();
                }

                rows.push(row_element);
            }
        }

        let scroll_content = scrollable(
            column(rows).spacing(0).width(Length::Fill)
        )
        .id(self.queue_scroll_id.clone())
        .height(Length::Shrink);

        // The panel itself (30% wider: 360 * 1.3 = 468)
        let panel_content = column![
            header,
            container(Space::new(Length::Fill, Length::Fixed(1.0)))
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::surface0())),
                    ..Default::default()
                })
                .width(Length::Fill),
            scroll_content,
        ]
        .spacing(0)
        .width(Length::Fixed(468.0));

        // Background color styling based on theme light/dark saturation shift
        let base_col = theme::base();
        let is_dark = (base_col.r + base_col.g + base_col.b) / 3.0 < 0.5;
        let popover_bg = if is_dark {
            // For dark backgrounds: blend with mantle to make it slightly less saturated / deeper
            theme::lerp_color(base_col, theme::mantle(), 0.5)
        } else {
            // For light backgrounds: blend with mantle or surface0 to make it more saturated / distinct
            theme::lerp_color(base_col, theme::surface0(), 0.15)
        };

        let panel = container(panel_content)
            .width(Length::Fixed(468.0))
            .max_height(588.0) // 40% taller: 420 * 1.4 = 588
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(popover_bg)),
                border: iced::Border {
                    color: theme::surface0(),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: [0.0, 4.0].into(),
                    blur_radius: 12.0,
                },
                ..Default::default()
            });

        // Position: Anchored directly below the "Now Playing" tab
        let panel_left_offset = (self.sidebar_width.round() + 6.0).max(0.0);
        let panel_top_offset = self.player_height;

        let positioned_panel = container(panel)
            .padding(iced::Padding {
                top: panel_top_offset,
                left: panel_left_offset,
                right: 0.0,
                bottom: 0.0,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        // Stack: dismiss layer behind, panel in front
        stack![
            dismiss_layer,
            positioned_panel,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
            row_item("] / [", "Increase / Decrease Scaling"),
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
        let mut base_subs = vec![
            iced::time::every(Duration::from_millis(100)).map(|_| Message::PollAudio),
            iced::time::every(Duration::from_secs(3)).map(|_| Message::CheckTheme),
            iced::time::every(Duration::from_secs(5)).map(|_| Message::FlushBuffers),
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
                    iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                        Some(Message::GlobalCursorMoved(position))
                    }
                    iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                        Some(Message::GlobalClick)
                    }
                    _ => None,
                }
            }),
        ];

        if matches!(self.playback_state, PlaybackState::Playing) {
            base_subs.push(iced::time::every(Duration::from_millis(33)).map(|_| Message::PollSpectrum));
        }

        let mut subs = vec![Subscription::batch(base_subs)];

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

        if self.dragging_player_split {
            subs.push(iced::event::listen_with(|event, _, _| {
                use iced::mouse;
                match event {
                    iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::PlayerDragMove(position.y))
                    }
                    iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::PlayerDragEnd)
                    }
                    _ => None,
                }
            }));
        }
        if self.dragging_queue_index.is_some() {
            subs.push(iced::event::listen_with(|event, _, _| {
                use iced::mouse;
                match event {
                    iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::QueueDragEnd)
                    }
                    _ => None,
                }
            }));
        }

        struct UdpSubscriptionId;
        subs.push(iced::Subscription::run_with_id(
            std::any::TypeId::of::<UdpSubscriptionId>(),
            iced::futures::stream::unfold(None, |state| async {
                let socket = match state {
                    Some(s) => Some(s),
                    None => match tokio::net::UdpSocket::bind("127.0.0.1:18888").await {
                        Ok(s) => Some(s),
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            None
                        }
                    }
                };
                
                if let Some(s) = socket {
                    let mut buf = [0u8; 1024];
                    loop {
                        if let Ok((len, _)) = s.recv_from(&mut buf).await {
                            let msg = String::from_utf8_lossy(&buf[..len]);
                            match msg.trim() {
                                "like" => return Some((Message::ToggleLikeCurrent, Some(s))),
                                "play-pause" => return Some((Message::PlayPause, Some(s))),
                                "next" => return Some((Message::NextTrack, Some(s))),
                                "prev" => return Some((Message::PreviousTrack, Some(s))),
                                "shuffle" => return Some((Message::ToggleShuffle, Some(s))),
                                "repeat" => return Some((Message::ToggleRepeat, Some(s))),
                                _ => {}
                            }
                        }
                    }
                } else {
                    Some((Message::PollAudio, None))
                }
            })
        ));

        let target = self.group_by_state.target();
        let is_animating = (self.group_by_state.hover_progress - target).abs() > 0.001;
        if is_animating {
            subs.push(iced::time::every(Duration::from_millis(16)).map(|_| Message::GroupByAnimationTick(std::time::Instant::now())));
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

        let current_idx = self.current_track.as_ref()
            .and_then(|ct| self.queue.iter().position(|t| t.id == ct.id));
        let next_idx = match current_idx {
            Some(i) => {
                let new = i as i32 + delta;
                if new < 0 { self.queue.len() - 1 } else { new as usize % self.queue.len() }
            }
            None => 0,
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
        if self.group_by != crate::db::GroupBy::None {
            let mut y = 0.0;
            let mut groups: Vec<(String, Vec<&crate::library::models::Track>)> = Vec::new();
            for track in self.tracks.iter() {
                let group_key = match self.group_by {
                    crate::db::GroupBy::Album => track.album.clone(),
                    crate::db::GroupBy::Artist => track.artist.clone(),
                    crate::db::GroupBy::Genre => track.primary_genre().to_string(),
                    crate::db::GroupBy::Year => track.year.map(|y| y.to_string()).unwrap_or_default(),
                    crate::db::GroupBy::None => unreachable!(),
                };
                if let Some(last) = groups.last_mut() {
                    if last.0 == group_key {
                        last.1.push(track);
                        continue;
                    }
                }
                groups.push((group_key, vec![track]));
            }
            for (_group_name, tracks) in groups {
                let header_height = 28.0;
                if let Some(index_in_album) = tracks.iter().position(|t| t.id == track_id) {
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

    pub fn evaluate_smart_playlist(&self, sp: &crate::library::smart_playlist::SmartPlaylist) -> Vec<Track> {
        let rp = crate::db::get(|db| db.recently_played.clone());
        let mut matched_tracks: Vec<Track> = self.all_tracks.iter()
            .filter(|t| crate::library::smart_playlist::evaluate_rules(t, &sp.rules, &rp))
            .cloned()
            .collect();

        // Hydrate date_played if available in recently_played
        for t in &mut matched_tracks {
            if let Some((_, date_str)) = rp.iter().find(|(p, _)| p == &t.path) {
                t.date_played = Some(date_str.clone());
            }
        }

        crate::library::smart_playlist::sort_and_limit_tracks(&mut matched_tracks, sp.order_by, sp.limit, &rp);
        matched_tracks
    }

    pub fn update_live_smart_playlists(&mut self) {
        let smart_playlists = crate::db::get(|db| db.smart_playlists.clone());
        for (name, mut sp) in smart_playlists {
            if sp.live_updating {
                let evaluated = self.evaluate_smart_playlist(&sp);
                sp.tracks = evaluated.iter().map(|t| t.path.clone()).collect();
                crate::db::save_smart_playlist(name, sp);
            }
        }
    }

    pub fn set_playing_context_from_current_view(&mut self) {
        if let Some(ref name) = self.selected_playlist {
            if name == "Liked Songs" || name == "Recently Played" || name == "Most Played" || name == "New Music" {
                self.playing_context = Some(PlayingContext::Autoplaylist(name.clone()));
            } else if crate::db::get(|db| db.smart_playlists.contains_key(name)) {
                self.playing_context = Some(PlayingContext::SmartPlaylist(name.clone()));
            } else {
                self.playing_context = Some(PlayingContext::Playlist(name.clone()));
            }
        } else if let Some(ref album) = self.selected_album {
            self.playing_context = Some(PlayingContext::Album(album.clone()));
        } else if let Some(ref artist) = self.selected_artist {
            self.playing_context = Some(PlayingContext::Artist(artist.clone()));
        } else if let Some(ref genre) = self.selected_genre {
            self.playing_context = Some(PlayingContext::Genre(genre.clone()));
        } else {
            self.playing_context = None;
        }
    }

    fn play_track_internal(&mut self, track: Track) -> Task<Message> {
        let cover_data = load_cover(&track.path);
        let track = Track { cover_data, ..track };
        self.audio.send(AudioCommand::Play(track.path.clone()));
        self.audio.send(AudioCommand::SetVolume(self.volume));
        self.current_track = Some(track.clone());
        self.selected_track = Some(track.clone());
        self.update_live_smart_playlists();
        self.playback_state = PlaybackState::Playing;
        self.position = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.last_accumulated_position = Duration::ZERO;
        self.current_track_play_counted = false;
        self.notify_mpris_track(PlaybackStatus::Playing);

        let queue_paths: Vec<PathBuf> = self.queue.iter().map(|t| t.path.clone()).collect();
        crate::db::write(|db| {
            db.last_track_path = Some(track.path.clone());
            db.last_queue_paths = queue_paths;
            db.last_position_secs = 0;
            db.last_view_mode = Some(self.view_mode);
            db.last_selected_playlist = self.selected_playlist.clone();
            db.last_selected_folder = self.selected_folder.clone();
            db.last_selected_artist = self.selected_artist.clone();
            db.last_selected_album = self.selected_album.clone();
            db.last_selected_genre = self.selected_genre.clone();
        });

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
        .unwrap_or(200.0f32)
        .clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH)
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

fn load_right_panel_width() -> Option<f32> {
    std::fs::read_to_string(right_panel_width_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
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

fn merge_song_order(manual_order: &[PathBuf], live_set: &[PathBuf]) -> Vec<PathBuf> {
    let live_set_hs: std::collections::HashSet<&PathBuf> = live_set.iter().collect();
    let mut result: Vec<PathBuf> = manual_order.iter()
        .filter(|p| live_set_hs.contains(p))
        .cloned()
        .collect();
    for path in live_set {
        if !result.contains(path) {
            result.push(path.clone());
        }
    }
    result
}

// ── Ponto de entrada iced ─────────────────────────────────────────────────────

pub fn run() -> iced::Result {
    iced::application("omatunes", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .font(include_bytes!("../assets/JetBrainsMonoNerdFontMono-Regular.ttf"))
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
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "omatunes".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(AppState::new)
}
