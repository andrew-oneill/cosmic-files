# Miller columns view (fork-only)

A third view mode, **Columns** (`View::Columns`, Ctrl+3), showing a folder's ancestors and the
selected folder's contents side by side, as in macOS Finder's column view. This fork does not
upstream it: pop-os/cosmic-files does not accept LLM-generated changes.

## Building this fork

On a Debian-based system (Pop!_OS, Ubuntu):

```sh
sudo apt install git curl build-essential pkgconf libclang-dev libglib2.0-dev libxkbcommon-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust; rust-toolchain.toml pins the version
git clone -b miller-columns https://github.com/andrew-oneill/cosmic-files.git
cd cosmic-files
cargo build --release
./target/release/cosmic-files
```

Running from `target/` leaves any installed cosmic-files untouched. Switch to the columns view
with Ctrl+3 or View → Columns view.

To update:

```sh
git pull
cargo build --release
```

If Homebrew's `pkg-config` comes first on `PATH`, it cannot find the system libraries and the
build fails. Point the build at the system one:

```sh
PKG_CONFIG=/usr/bin/pkg-config cargo build --release
```

This build and the stock cosmic-files share their settings. The stock version does not know the
columns view and falls back to its default view if it reads it.

### Installing system-wide

This replaces the distro's copy in `/usr`, which the next package update restores.

```sh
sudo apt install just
just build-release
sudo just install
```

Installing copies the build into `/usr`, so a new build is not used until it is installed
again. To update an installed copy:

```sh
git pull
just build-release
sudo just install
```

Use `just build-release` rather than `cargo build --release` here: it also builds the
thumbnailer and applet that `just install` copies. If a package update restores the stock
version, `sudo just install` puts this build back without rebuilding.

To go back to the stock version:

```sh
sudo apt install --reinstall cosmic-files
```

## Design

- **The active column is the tab's ordinary listing.** `Tab::location` and `Tab::items_opt` are
  the folder with keyboard focus, so selection, renaming, the context menu, the clipboard,
  type-to-select, the gallery and the details pane work unchanged.
- **Other columns live in `Tab::columns`** (`src/columns.rs`, `ColumnsState`):
  - `ancestors`: folders from `root` down to the parent of the active folder;
  - `preview`: contents of the single selected folder (none for files or multi-select).
- **Root.** A folder chosen in the sidebar (or a drive mounted from it) always becomes the root,
  the leftmost column, as in Finder. Otherwise the root is kept while the active folder stays
  inside it; a jump elsewhere (breadcrumb, sibling) makes the new folder the root, and moving left
  past the root makes its parent the root.
- **Reuse.** Moving left or right hands listings between the active column and the others, so
  the new active folder shows at once while its rescan runs.
- **Right-click in another column.** `Tab::selected_items` is the one source of the items actions
  apply to. While a context menu opened from another column is open, it yields that item alone
  (outlined), so the selection and columns stay put. The menu's `on_close`, a click, or a change of
  folder clears it. "Show details" selects the item first, since the details pane follows the
  selection.
- **Press, then click or drag.** In the columns view, a press on an unselected item (no
  modifiers) or on any item in another column waits for the release (`PendingPress`). A release
  without a drag is the click; a drag (`DndSource::on_start`) instead carries just the pressed
  item, so dragging never selects anything or moves the columns. The list and grid views keep
  selecting on press.
- **Sync.** `Tab::sync_columns` runs at the end of every `Tab::update`. It rebuilds the
  ancestors for the current folder, sets the preview from the selection, and returns
  `Command::ScanColumn` for folders that need scanning and a scroll to the rightmost column.
  The app sends `tab::Message::ColumnsSync` after a rescan or view change made outside `update`.
- **Loading and watching.** `App::rescan_column` scans a column's folder and returns
  `Message::TabColumnRescan`; results for folders no longer shown are dropped. The watcher
  covers every column (`Tab::watched_paths`), and a change in a column's folder rescans it
  (`Tab::columns_affected_by`).
- **Fallback.** Columns apply only to ordinary folders in the app. Search, Trash, Recents,
  Network, desktop mode and the file chooser dialog render the list view instead.

## Keyboard and mouse

| Input | Action |
|---|---|
| Up / Down | Move within the active column |
| Right | Into the selected folder, selecting its first item |
| Left | To the parent column, selecting the folder left |
| Click in another column | Make that column active with the item selected |
| Double-click a file in another column | Open it |
| Drag an unselected item | Drag just that item; the selection and columns stay as they are |
| Right-click in another column | Outline the item and open its context menu, leaving the selection as it is |
| Right-click empty space in another column | Outline the column and open its folder's menu (new folder, paste, …), leaving the selection as it is; Select all there makes that column active with all of it selected |

## Status

Phase 1 (MVP) is in place. Known gaps, roughly in priority order:

- Non-active columns build every row; they should only build visible rows, like the list view.
- No file preview column for a selected file (the details pane still works).
- Column width is fixed (`COLUMN_WIDTH`); long names are ellipsized.
- No file chooser dialog support; no thumbnails in non-active columns.
- Right-to-left layouts are not mirrored.
- The `view-column-symbolic` icon may be missing from some icon themes.
- The active column is not wrapped in an `id_container` like the list view: its position moves,
  and libcosmic panics ("Downcast on stateless state") when grafted state no longer matches.

## Testing

- `src/columns.rs` tests cover the root and ancestor rules and listing reuse.
- `src/tab.rs` tests (prefixed `columns_`) drive `Tab::update` over a temporary folder tree.
- Manual checks are in `TESTING.md` under "Columns view".

## Visual testing without a desktop

The app runs headless under sway, which suits SSH sessions:

1. Install `sway`, `grim` and `wtype`.
2. Start `sway` with `WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1` and
   a private `XDG_RUNTIME_DIR` with a short path (Unix socket paths are limited to 108 bytes).
3. Before starting the app, hold a virtual keyboard open (`wtype -k Shift_L -s 7000000`):
   winit does not rebind the keyboard if the seat loses it, as it does after each short `wtype`.
4. For clicks, keep a virtual pointer open (`zwlr_virtual_pointer_v1`) and drive it; sway's
   `seat cursor` commands alone reach no client while the seat has no pointer.
5. Run the app with separate `XDG_CONFIG_HOME`/`XDG_STATE_HOME` under `dbus-run-session`, and
   take screenshots with `grim`.

Xvfb does not work: the software renderer rejects its window format.
