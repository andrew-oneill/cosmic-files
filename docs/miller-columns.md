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

## Testing

- `src/columns.rs` tests cover the root and ancestor rules and listing reuse.
- `src/tab.rs` tests (prefixed `columns_`) drive `Tab::update` over a temporary folder tree.
- Manual checks are in `TESTING.md` under "Columns view".
