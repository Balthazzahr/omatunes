# OmaTUNES User Manual

Welcome to the comprehensive user manual for **OmaTUNES**, a native Wayland music player written in Rust. This document serves as a detailed reference for all player features, custom configurations, keyboard shortcuts, database mappings, and system integrations.

---

## 1. Player & Controls / Album Art

The player interface is located in the upper panel of the application. It handles playback state, track metadata tracking, and active audio controls.

<p align="center">
  <img src="assets/Main Player Controls.png" alt="Main Player Controls" width="600">
</p>

### Playback Controls
- **Play/Pause**: Toggles audio playback. (Shortcut: `Space`)
- **Previous / Next**: Skips to the previous or next track in the current queue. (Shortcuts: `p` / `n`)
- **Seeking**: Move forward or backward in the active track by clicking anywhere along the progress bar timeline. (Shortcuts: `&leftarrow;` to seek back, `&rightarrow;` to seek forward; seek step is configurable via `seek_step`, defaults to 5 seconds).
- **Volume**: Adjust volume using the slider on the right side of the control row, or scroll anywhere in the player panel. (Shortcuts: `+` or `=` to increase volume, `-` to decrease volume by the configured `volume_step`, defaults to 5%).
- **Shuffle**: Shuffles the current queue. Enabled state is indicated by the shuffle icon turning into your theme's active accent color. (Shortcut: `s`)
- **Repeat**: Toggles repeat modes (Off, Repeat All, Repeat One). Enabled state is indicated by the repeat icon turning into your theme's active accent color. (Shortcut: `r`)

### Album Art Behavior
- OmaTUNES dynamically extracts album art from the playing track (embedded ID3 tags) or reads a fallback image (e.g., `cover.jpg`, `Cover.jpg`, `folder.png`) inside the track's local folder.
- If no art is found, it renders a custom default note artwork.
- The album art is displayed in the upper-left of the player row, enlarged to `238x238` pixels for high visibility on high-resolution screens.
- The player controls container is set to `270px` height (with the total top section height being `298px`) to ensure the artwork renders cleanly without top/bottom cropping.
- **Interactivity**: Clicking the album art focuses the player back on the active track view (returns to the currently playing album/track).

### Disabled States
- When no track is loaded, the playhead timeline remains empty, volume controls remain active, and clicking play/pause or double-clicking in the tracklist will automatically load and play the first track in the current view.
- The Like (heart) button is hidden from the player row when no track is currently playing.

### Unified Top Layout & Settings Button
- **Combined Header Row**: The player controls (height `270px`) and library tabs/search bar (height `28px`) are stacked vertically as a single visual panel (total height `298px`). The horizontal separator between them is removed using a `1px` stack overlap.
- **Top Drawer Alignment**: The lyrics/visualizer right-hand drawer occupies exactly `298px` next to this left column, creating a aligned top section that does not cover the main song list area.
- **Tab Row Integration**: The `Now Playing` tab resides in the same row as `Artists`, `Albums`, and `Genres`, shifted slightly to the right to visually demarcate it. Search options, grouping controls, and queue controls reside on the right-aligned side of this tab row.
- **Settings Button Placement**: The settings button (`\u{f013}`) is positioned at the far right of the library tab row. It is styled to be exactly `56px` wide (matching the width of the tab strip above it), separated by a vertical divider, and contains a top horizontal divider line. Its borders align perfectly on integer pixels due to rounded sidebar width layouts.

---

## 2. Visualizer / Spectrogram

OmaTUNES includes a high-performance audio spectrum visualizer that computes real-time frequency analysis.

<p align="center">
  <img src="assets/Visualiser.png" alt="OmaTUNES Visualizer" width="400">
</p>

- **How it Works**: The backend computes a real-time 2048-point Hann-windowed FFT on the decoded PCM audio buffer via Symphonia, mapping frequencies logarithmicly to 64 distinct bands.
- **Visuals**: The visualizer is colored with an amplitude gradient that dynamically shifts colors (e.g., green &rightarrow; lavender &rightarrow; pink) as amplitude spikes.
- **Trigger**: Click the visualizer tab (waveform icon `\u{f147d}`) on the vertical sidebar strip on the right side of the window to slide out the visualizer. Click the tab again, or press `Escape`, to close it. You can also drag the resize handle between the panels to adjust its width.

---

## 3. Live Lyrics

OmaTUNES features an interactive live lyrics panel supporting both synchronized and unsynchronized lyrics.

<p align="center">
  <img src="assets/Live Lyrics.png" alt="Live Lyrics View" width="400">
</p>

### Synced (LRC) vs. Unsynchronized Lyrics
- **Synced Lyrics**: If a track has `.lrc` metadata (containing timestamps like `[02:14.20]`), the view displays them line-by-line.
- **Unsynchronized Lyrics**: If the track has plain-text lyrics, they are displayed as a plain scrollable text block.

### Visual Styling & Color Tiers
For synchronized lyrics, active lines are styled dynamically in three color tiers:
1. **Active Line**: Highlighted in the theme's primary accent color, enlarged (size 20), and styled in bold.
2. **Adjacent Lines (Preceding/Following)**: Highlighted in an interim blend color (blended 50% between accent and overlay colors) at size 17 to guide the eye.
3. **Other Lines**: Dimmed in the inactive theme overlay color (size 17).

### Scrolling and Seeking Interaction
- **Auto-Scrolling**: The lyrics view automatically scrolls to align the active line precisely at the vertical center of the viewport (computed using a midpoint alignment formula with 108px spacers at the top and bottom).
- **Manual Scroll**: The user can scroll freely to read ahead or look back. The scroll panel remains unfrozen until the player progresses to the next timestamp, at which point it snaps back to center the active line.
- **Seek-on-Click**: Every line in the synchronized lyrics view is interactive. Clicking a lyric line instantly seeks the audio playback to that line's precise timestamp.

---

## 4. Left Library Sidebar & Search

The left sidebar provides full-width tabs and filters to navigate your music collection.

### Tab Filtering
- **Artists**: Filters the library view to show all tracks by the selected artist.
- **Albums**: Filters the library view to show all tracks in the selected album.
- **Genres**: Filters the library view to show all tracks matching the selected genre.
- **Folders**: Allows browsing tracks by their folder structure in your file system.

### Interactive Search & Focus
- The search bar at the top of the sidebar allows you to filter the lists by matching keywords instantly.
- **Focus Retention**: Keyboard inputs and filters are structured so that typing inside the search box does not lose focus, preventing interruptions while typing queries.

### Sidebar Right-Click Context Menus
Right-clicking any row under the Artists or Albums tabs opens a context menu with the following choices:
- **Hide from UI**: Hides the artist or album from the browsing views (stored in `db.json`).
- **Add to Playlist**: Lists your custom playlists and appends all tracks belonging to this artist/album.
- **+ Create playlist with this artist/album**: Instantly creates a new custom playlist populated with all tracks from that artist or album.

---

## 5. Main Library View (Track Table)

The main library view displays your tracks in a highly customizable table layout.

<p align="center">
  <img src="assets/Group By Album.png" alt="Group By Album" width="600">
</p>

### Grouping and Customization
- **Group by Album**: Toggle album grouping to display tracks clustered by their respective albums with visual album header dividers. (Saved in `db.json` under `group_by_album`).
- **Column Customization**: Right-click the track table header to trigger the `Table Columns` context menu:
  - **Show/Hide**: Check/uncheck columns (including Track #, Title, Artist, Album, Genre, Year, Plays, Duration, Date Played, and the new **Liked** column) to toggle their visibility.
  - **Reorder**: Select `Move Left` or `Move Right` to rearrange the columns. Preferences are saved automatically to `db.json`.

### Liking a Song (The Liked Column)
- The heart toggle is integrated directly as a standard, first-class table column (**Liked**). Clicking the heart icon on any track row immediately toggles its liked state.
- Redundant per-row action buttons (e.g. metadata editor and playlist addition buttons) have been removed from the end of the rows to keep the interface clean. All metadata and playlist actions are fully accessible via keyboard shortcuts or the right-click menu.

### Responsive Column Collapsing
- To prevent columns from wrapping and bunching up on smaller monitors, the track table dynamically collapses columns as the window width narrows.
- Hiding priority is designed to discard lower-priority columns first:
  1. `Disc #` (Hides first)
  2. `Plays`
  3. `Date Played`
  4. `Genre`
  5. `Liked` (heart column)
  6. `Year`
  7. `Album`
  8. `Artist`
- Core columns `#` (Track Number), `Title`, and `Duration` remain visible. If space becomes extremely tight, only `Title` and `Duration` are shown.
- Hiding columns in this responsive mode is temporary and **does not** overwrite your saved column visibility preferences in `db.json`. Expanding the window restores them in reverse order.

### Track Right-Click Context Menu
Right-clicking an individual track opens the `Song Menu`:
- **Like / Unlike this song**: Toggles favorite status.
- **Edit ID3 tag**: Opens the metadata editor for this track.
- **Open local file folder**: Spawns your file manager (via `xdg-open`) targeting the folder containing the audio file.
- **Add to Playlist**: Appends the track to an existing custom playlist.
- **+ Create playlist with this song**: Creates a new playlist containing only this track.

### Multi-Select Actions
- **Range Selection**: Select a track, then hold `Shift` and click another track to select all tracks in between.
- **Bulk Tag Editing**: Press `E` (or right-click selection and choose `Edit ID3 tags`) to open the tag editor for all selected tracks at once.
- **Bulk Playlist Creation**: Create a new playlist containing all selected tracks.

---

## 6. Now Playing / Up Next Floating Queue Popover
The Now Playing view has been redesigned into a compact, highly interactive **floating popover panel** that overlays the library without swapping screens.
* **Activating the View**: Click the **Now Playing** tab in the library top tab bar. The tab background highlights in your theme's active accent color.
* **Toggle & Dismissal**: Clicking the active tab again toggles it off. Alternatively, clicking anywhere outside the popover container dismisses it immediately, bringing focus back to the library view in the background.
* **Anchor & Placement**: The popover anchors directly underneath the Now Playing tab button on the left of the tab bar (offset by the sidebar width plus margin spacers) for a clean visual alignment.
* **Dimensions**: The panel is designed to be 30% wider (`468px`) and 40% taller (max height `588px`) to fit long track details comfortably and show more songs.
* **Visual Contrast**: The popover background color automatically shifts saturation based on your theme's base color: it gets slightly less saturated on dark themes and more saturated/distinct on light themes. Adjacent row backgrounds alternate between base and mantle colors to provide clear delineation between successive tracks.
* **Active Track Highlight**: The currently playing song is highlighted using your theme's accent color (and with a lower-opacity accent tint in the row background).
* **Automatic Scroll-to-Center**: Opening the popover automatically triggers a smooth scrolling action to center the currently playing track within the scrollable queue list.
* **Queue Interaction**:
  * **Play on Click**: Clicking any row title/artist instantly plays that track (shifting playback context to the queue) without closing the popover.
  * **Remove Song**: Hovering/viewing rows reveals a larger, red remove icon (✕). Clicking it instantly deletes the track from the queue.
  * **Drag Reordering**: Drag and drop track handles (`\u{f0c9}`) to dynamically rearrange the playback sequence.
  * **Clear Queue**: Click the **Clear** button in the popover header to empty the active queue.
* **Deterministic Shuffle Integration**: When shuffle is toggled on, the tracks in the queue are physically shuffled in-place (keeping the current track at index 0). The popover displays the exact, shuffled playing order deterministically, letting you see exactly what will play next. Toggling shuffle off retains the current layout.
 
---

## 7. Edit ID3 Tag Editor

The metadata editor allows editing files individually or in bulk.

### Targeted Gating Checkboxes
- Next to each metadata field (Title, Artist, Album, Genre, Track #, Disc #, Year, Cover Path, and Lyrics) is a **checkbox**.
- When you begin typing in a text field, that field's checkbox is **automatically checked**. Only checked fields will be overwritten and saved when you submit. This prevents accidentally overwriting distinct fields (like titles) when bulk editing tracks.

### Autocomplete Pills
- Typing in the Artist, Album, or Genre fields triggers autocomplete suggestion pills based on existing values in your music database. Clicking a pill fills the field and automatically checks its active box.

### Visual Options
- **Apply to Entire Album**: Gathers all tracks matching the current album and applies the checked fields to them.
- **Cover Path**: Edit the cover art path for the file.

---

## 8. Live Lyrics Tab in Tag Editor

The Tag Editor contains a dedicated **Lyrics** tab to view, edit, and adjust timings.

### Lyrics Text Editor
- Provides a full-featured text editor area to edit raw lyrics text, including LRC format timestamp lines.

### Timeline Offset Adjustment Controls
If lyrics are synchronized, you can adjust timings using the offset control panel:
- **Shift Buttons**: Click `+0.5s`, `+1.0s`, `-0.5s`, or `-1.0s` to add a positive or negative pending offset.
- **Apply**: Shifts every timestamp (e.g. `[01:10.50]`) in the lyrics text by the pending offset in seconds, updates the text area, checks the lyrics checkbox, and resets the pending offset to `0.0`.
- **Reset**: Resets the pending offset to `0.0` without altering the lyrics text.

---

## 9. Online Integration Buttons

Inside the Tag Editor, there are two helper buttons for fetching assets:
- **Search Lyrics Online**: Clicking this button reads the active track metadata and opens your web browser to `https://lrclib.net/search/{query}` (pre-filled with the song details) via `xdg-open`. You can copy the synced lyrics from the browser and paste them into the editor.
- **Search Cover Online**: Clicking this button opens your browser to Google Images pre-filled with the query `{artist} {album} album art` so you can retrieve and save the cover file.

---

## 10. Playlists & Smart Playlists

OmaTUNES manages playlists locally inside `~/.config/omatunes/db.json`. The playlist section at the bottom of the sidebar is organized into three compact icon-based tabs, each showing tooltips on hover:
1. **User Playlists** (List icon `\u{f03a}`): Standard manually-curated user playlists.
2. **Auto Playlists** (Magic Wand/Sparkles icon `\u{ebcf}` or auto-list equivalent): Automatic lists based on history/activity.
3. **Smart Playlists** (Wand icon `\u{f0d0}`): Rule-based playlists that compile automatically.

### User Playlists
- **Creation**: Click the `New Playlist` button at the bottom of the sidebar list, or right-click any track/selection/artist/album and select the create option.
- **Management**: Hovering over a playlist in the sidebar reveals a Pencil icon (Rename) and a Trash icon (Delete).
- **Adding Tracks**: Right-click any track or selection, go to `Add to Playlist`, and click the `+ {Playlist Name}` entry.
- **Single-line Truncation**: Playlist names in the sidebar are automatically truncated with an ellipsis (`...`) if they exceed the available column width, preventing visual overflows.

### Autoplaylists
Autoplaylists require no manual curation and populate dynamically:
- **Liked Songs**: Every track that has been favorited (liked) in the main interface.
- **Recently Played**: A list of your most recently played tracks sorted chronologically.
- **Most Played**: Your tracks sorted in descending order of play count.

### Smart Playlists (New in v0.7.0)
Smart Playlists work similarly to iTunes smart playlists, compiling tracks dynamically from your music library according to a set of user-defined rules.
- **Creation**: Navigate to the Smart Playlists tab in the sidebar and click **New Smart Playlist**. This opens the interactive Rule Builder panel in the main content area.
- **Rule Matrix**: Each rule row consists of:
  - **Field selector**: Choose criteria like Title, Artist, Album, Genre, Year, Play Count, Duration, Disc Number, Liked, Has Lyrics (checks if tracks have lyrics stored in their ID3 tags), or Last Played.
  - **Operator selector**: Choose matching operations like Contains, Is, Greater Than, Less Than, Within Last, or Between.
  - **Value inputs**: Enter matching text, numbers, checkbox states, or time duration strings. Text fields like Artist, Album, and Genre feature autocomplete chips for quick selection.
- **AND Logic**: Multiple rule rows are combined using strict logical `AND` matching — a track must satisfy all defined rules to be included.
- **Sorting & Truncation**: Customize how matching tracks are sorted and limited:
  - **Order By**: Sort tracks by Title, Album, Artist, Play Count, Year, or Random.
  - **Limit**: Check the limit box and define a maximum number of tracks (e.g. limit to 25 songs).
- **Live Updating**: Check the live updating box to automatically re-evaluate rules in the background. Re-evaluation is debounced and runs on app launch/scan and on every track change (when the playing track transitions).
- **Edit / Delete**: Right-click any Smart Playlist in the sidebar to open the context menu. Select **Edit Smart Playlist** to reload its configuration inside the Rule Builder, or **Delete Smart Playlist** to remove it.
- **View Restoration**: Clicking Cancel or Save in the Rule Builder automatically restores the exact view context (Artist, Album, Genre, or Playlist) that was open before you launched the builder.

---

## 11. Waybar Integration

OmaTUNES exposes player states over a UDP socket listener on port `18888` and writes statuses to `/tmp/omatunes_waybar_state.json`, facilitating rich Waybar configurations.

### Waybar CSS Styling
To style the grouped Waybar modules into a unified pill design that collapses cleanly when OmaTunes is closed, use the following rules in your `~/.config/waybar/style.css`:

```css
#omatunes-group {
  background-color: transparent;
  border: none;
  padding: 0;
  margin: 0;
}

#custom-omatunes-play {
  background-color: @theme_bg;
  border: 2px solid @active_border;
  border-right: none;
  border-radius: 50px 0 0 50px;
  padding-left: 15px;
  padding-right: 5px;
  margin-top: 3px;
  margin-bottom: 3px;
  transition: all 0.2s ease;
}

#custom-omatunes-play:hover {
  background-color: #414559;
}

#custom-omatunes-next {
  background-color: @theme_bg;
  border-top: 2px solid @active_border;
  border-bottom: 2px solid @active_border;
  border-left: none;
  border-right: none;
  padding-left: 5px;
  padding-right: 5px;
  margin-top: 3px;
  margin-bottom: 3px;
  transition: all 0.2s ease;
}

#custom-omatunes-next:hover {
  background-color: #414559;
}

#custom-omatunes-text {
  background-color: @theme_bg;
  border-top: 2px solid @active_border;
  border-bottom: 2px solid @active_border;
  border-left: none;
  border-right: none;
  padding-left: 10px;
  padding-right: 10px;
  margin-top: 3px;
  margin-bottom: 3px;
}

#custom-omatunes-like {
  background-color: @theme_bg;
  border: 2px solid @active_border;
  border-left: none;
  border-radius: 0 50px 50px 0;
  padding-left: 5px;
  padding-right: 15px;
  margin-top: 3px;
  margin-bottom: 3px;
  margin-right: 10px;
  transition: all 0.2s ease;
}

#custom-omatunes-like:hover {
  background-color: #414559;
}
```

### Click & Scroll Bindings
- **Play/Pause**: Handled by `--click play` (sends UDP `play-pause` command to port 18888).
- **Next**: Handled by `--click next` (sends UDP `next` command).
- **Like**: Handled by `--click like` (sends UDP `like` command).
- **Focus Player**: Clicking the track text module runs `hyprctl dispatch focuswindow class:^omatunes$ || hyprctl dispatch focuswindow title:^omatunes$`.
- **Volume**: Scrolling up or down over the text module runs `omatunes_volume.sh up` or `omatunes_volume.sh down`.

### Milestone Notifications & Stats
- **Track Milestones**: Triggers a desktop notification via `notify-send` when you listen to your 10th, 50th, and every 100th track of the day.
- **Hourly Milestones**: Sends a notification warning you that "Time Flies!" for every active hour of listening completed today.
- **Leaderboards**: The hover tooltip displays your daily/weekly/monthly stats alongside a **Monthly Top 5 Artists** leaderboard and an **All-Time Top 10 Legends** board.
- **Live Theme Sync**: The script reads the active Alacritty or Omarchy theme to apply matching colors inside the pango markup tooltips.

---

## 12. Full Keybinding Reference

The following table documents all keyboard controls available when the OmaTUNES window is focused:

| Key | Context | Action |
|---|---|---|
| `Space` | Main Player | Play / Pause |
| `&rightarrow;` | Main Player | Seek forward (configurable step, default 5s) |
| `&leftarrow;` | Main Player | Seek backward (configurable step, default 5s) |
| `ArrowUp` | Track List | Move selected track focus up |
| `ArrowDown` | Track List | Move selected track focus down |
| `n` / `N` | Main Player | Next Track |
| `p` / `P` | Main Player | Previous Track |
| `s` / `S` | Main Player | Toggle Shuffle |
| `r` / `R` | Main Player | Toggle Repeat |
| `+` or `=` | Main Player | Increase volume by step (default 5%) |
| `-` | Main Player | Decrease volume by step (default 5%) |
| `l` / `L` / `f` / `F` | Track List | Toggle Liked state for selected track |
| `e` / `E` | Track List | Open ID3 metadata tag editor for selection |
| `c` / `C` | Sidebar | Open New Playlist dialog |
| `a` / `A` | Track List | Open playlist addition dialog for selected track |
| `/` | Main Player | Focus track list search input and clear query |
| `F5` | Main Player | Trigger full scan of the music library folder |
| `Tab` | Main Player | Cycle focus: Sidebar Search &rarr; Sidebar List &rarr; Tracklist &rarr; Song Search &rarr; Sidebar Search |
| `Enter` | Dialog / Editor | Submit / Save tags (or double-click selected track) |
| `Escape` | Dialog / Editor | Close active dialog, tag editor, or context menu |
| `]` | Main Player | Increase UI Font Scaling (scales font size up) |
| `[` | Main Player | Decrease UI Font Scaling (scales font size down) |

---

## 12. Theming System & Custom Theme Editor

OmaTUNES includes an extensive custom theming pipeline that handles colors dynamically. Open the settings dialog (Gear icon at the far right of the library tab strip) to configure it.

### Theme Sources
- **System Theme**: Detects and applies your active Omarchy system theme live in real-time.
- **Preset Themes**: Swap in any of the built-in preset palettes:
  - *Nord*
  - *Catppuccin Mocha*
  - *Catppuccin Latte*
  - *Dracula*
  - *Gruvbox (Dark)*
  - *Everforest (Dark)*
  - *Monokai*
- **Custom Theme**: Allows full customization. To prevent poor layout color combinations, only **7 base colors** are directly editable:
  - **Background** (`base`): The main application backdrop.
  - **Primary Text** (`text`): Used for primary headers and text lines.
  - **Accent**: Used for active tracks, button overlays, and highlighting.
  - **Green / Red / Yellow / Blue**: Swatches for indicators, warnings, and states.

### Contrast Protection & Derived Colors
The remaining **4 structural tokens** are automatically calculated from your Base and Text choices using target WCAG relative luminance contrast ratios:
* **Background (Deep)** (`mantle`): Derived from Background at a `1.20` contrast target to serve as a darker/deeper backdrop (darker in dark mode, lighter in light mode).
* **Panel Background** (`surface0`): Derived from Background at a `1.40` contrast ratio to act as a structural highlight (lighter in dark mode, darker in light mode).
* **Muted Text / Icons** (`overlay0`): Derived from Background at a `2.80` contrast target to ensure inactive icons, unliked hearts, and **non-highlighted lyric lines** remain readable.
* **Secondary Text** (`subtext`): Derived from Primary Text at a `1.25` contrast target to ensure secondary labels remain legible but de-emphasized.
These derived swatches display live in the Custom builder as read-only preview swatches. Saving the settings writes the generated hex values directly to `config.toml`.

---

## 13. Now Playing & Up Next Queue

The **Now Playing** tab (located in the main library view tab row) switches to the active Up Next queue manager.

### Queue Actions & Controls
- **Up Next Header Row**: Displays dynamic headers matching your active column visibility setup.
- **Drag-to-Reorder Handles**: Hover over a track in the queue to reveal the drag handle icon (`\u{f0c9}`). Click and drag the handle up or down to re-order the playing queue in real-time.
- **Move Up / Down Arrows**: Quick action buttons next to each queue track to shift its position by one slot.
- **Remove Button**: Click the red cross icon (`\u{f00d}`) on any row to remove that specific track from the queue.
- **Queue Search**: Use the right-hand search filter inside the Now Playing tab to narrow down queue items by keyword without losing playback state.
- **Clear Queue**: Click the garbage icon in the queue controls to purge all upcoming tracks.

