# Testing

This document provides a regression testing checklist for COSMIC Files. The checklist provides a starting point for Quality Assurance reviews.

## Checklist

### Basic navigation

- [ ] Middle-click opens directory in a new tab (not focused).
- [ ] Open two scrollable tabs. Scroll one tab, then switch to the other tab; it should not have scrolled.
- [ ] Hover over the top item in the folder, then scroll down until it's out of view (while still hovered).
      On scrolling back up (with the mouse in a different position), the item should not have the hover highlight.
- [ ] Right-click an item in the sidebar. No visual change should occur with the rest of the items.
- [ ] Remove an item from the sidebar, then re-pin it.

### File operations

- [ ] Right-click -> Create a new folder, then enter it.
- [ ] Right-click in the empty folder -> Create a new file.
- [ ] Navigate to the parent folder, create another new file, then drag it into the created folder.
- [ ] Files can be renamed.
- [ ] Files can be opened with non-default apps & browsing store for new apps works.
- [ ] Normal right-click shows `Move to trash` option.
- [ ] Shift right-click, and right-click followed by Shift, both show `Permanently delete` option.

### Advanced navigation & view settings

- [ ] Image and video thumbnails generate & display in local folders.
- [ ] Gallery preview shows with Spacebar.
- [ ] Details pane shows with Ctrl+Spacebar.
- [ ] Zoom in/out and reset to default zoom work.
- [ ] Ctrl+1 and Ctrl+2 switch between list and icon view.
- [ ] Ctrl+H shows/hides hidden files.
- [ ] Directories can be sorted at top or inline.
- [ ] Settings -> Theme works.
- [ ] Settings -> Type to Search affects behavior as designed.
- [ ] Single-click to open setting takes effect.
- [ ] Sorting options work.
- [ ] Cutting, copying, and pasting files works.
- [ ] F5 reloads current directory.
- [ ] Left sidebar can be collapsed and expanded.

### Columns view

- [ ] Ctrl+3 and View -> Columns view switch to columns; Ctrl+1/Ctrl+2 switch back.
- [ ] Selecting a folder shows its contents in a new column to the right; selecting a file or several items shows none.
- [ ] Right arrow enters the selected folder and selects its first item; Left arrow returns and selects the folder left.
- [ ] Left arrow in the leftmost column adds the parent folder as a new leftmost column.
- [ ] Clicking an item in an earlier column makes that column active and removes the columns after it.
- [ ] Double-clicking a file in another column opens it.
- [ ] Right-clicking an item in another column selects it and shows the context menu for it.
- [ ] Dragging files onto a folder in any column, or onto a column's background, moves or copies them there.
- [ ] Creating, renaming or deleting a file in a folder shown as a non-active column updates that column.
- [ ] Hidden files and the sort order apply to every column.
- [ ] Many columns scroll horizontally, keeping the rightmost column in view.
- [ ] Search, Trash and Recents fall back to the list view while Columns is selected.

### External filesystems

- [ ] Add a network drive (e.g. SFTP) and navigate into it.
- [ ] Plug in a USB drive; able to mount, browse, and eject.

### Integrations

- [ ] Desktop icons display as expected
- [ ] Drag-and-drop into Firefox works
