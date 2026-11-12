# Miller columns view (fork-only)

A third view mode, **Columns** (`View::Columns`, Ctrl+3), showing a folder's ancestors and the
selected folder's contents side by side, as in macOS Finder's column view. This fork does not
upstream it: pop-os/cosmic-files does not accept LLM-generated changes.

## Design

- **The active column is the tab's ordinary listing.** `Tab::location` and `Tab::items_opt` are
  the folder with keyboard focus, so selection, renaming, the context menu, the clipboard,
  type-to-select, the gallery and the details pane work unchanged.
- **Other columns live in `Tab::columns`** (`src/columns.rs`, `ColumnsState`):
  - `ancestors`: folders from `root` down to the parent of the active folder;
  - `preview`: contents of the single selected folder (none for files or multi-select).
- **Root.** Kept while the active folder stays inside it. A jump elsewhere (sidebar, breadcrumb,
  sibling) makes the new folder the root; moving left past the root makes its parent the root.
- **Reuse.** Moving left or right hands listings between the active column and the others, so
  the new active folder shows at once while its rescan runs.
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
| Right-click in another column | Make it active, then the usual context menu |

## Status

Phase 1 (MVP) is in place. Known gaps, roughly in priority order:

- Non-active columns build every row; they should only build visible rows, like the list view.
- No file preview column for a selected file (the details pane still works).
- Column width is fixed (`COLUMN_WIDTH`); long names are ellipsized.
- No file chooser dialog support; no thumbnails in non-active columns.
- Right-to-left layouts are not mirrored.
- The `view-column-symbolic` icon may be missing from some icon themes.
- Right-clicking in another column opens the menu at the click point, though the columns
  may have scrolled under it.
- A column partly scrolled out on the left draws into the tab's left padding.
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
