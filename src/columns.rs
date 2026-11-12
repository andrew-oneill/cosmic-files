// SPDX-License-Identifier: GPL-3.0-only

//! State for the Miller columns view.
//!
//! The tab's own listing (`Tab::items_opt`) is the active column, so selection, renaming,
//! the context menu and the clipboard work exactly as in the list view. This module holds the
//! other columns: the ancestors of the tab's folder, from a root folder down to its parent, and
//! a preview of the single selected folder.

use std::mem;
use std::path::{Path, PathBuf};

use cosmic::widget;

use crate::item::Item;

/// Width of each column, in logical pixels
pub const COLUMN_WIDTH: f32 = 260.0;

/// Width of the file preview column: a full thumbnail and its padding
pub const PREVIEW_COLUMN_WIDTH: f32 = 340.0;

/// One folder's listing shown as a column other than the active one
pub struct ColumnListing {
    pub path: PathBuf,
    /// `None` while the folder is being scanned
    pub items: Option<Vec<Item>>,
    pub scrollable_id: widget::Id,
}

impl ColumnListing {
    pub fn new(path: PathBuf, items: Option<Vec<Item>>) -> Self {
        Self {
            path,
            items,
            scrollable_id: widget::Id::unique(),
        }
    }
}

/// Which non-active column a message refers to
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnTarget {
    /// Index into [`ColumnsState::ancestors`], root first
    Ancestor(usize),
    Preview,
}

#[derive(Default)]
pub struct ColumnsState {
    /// The leftmost column's folder. `None` when the columns view is not in use.
    pub root: Option<PathBuf>,
    /// The active folder these columns were built for
    pub location: Option<PathBuf>,
    /// Columns from `root` down to the parent of `location`
    pub ancestors: Vec<ColumnListing>,
    /// Contents of the single selected folder in the active column
    pub preview: Option<ColumnListing>,
    /// The single selected file in the active column, shown as a preview after it
    pub preview_file: Option<PathBuf>,
    /// Listings dropped by the last rebuild, kept so a following preview or active column can
    /// reuse them instead of waiting for a rescan
    cache: Vec<ColumnListing>,
    /// Folders that need scanning, drained by [`ColumnsState::take_pending_scans`]
    pending_scans: Vec<PathBuf>,
    /// Set when the visible columns change, so the view can scroll to the rightmost one
    pub scroll_to_end: bool,
}

/// The leftmost folder to show for `location`: the old root when `location` is inside it,
/// otherwise `location` itself (a jump elsewhere, or moving left past the root).
pub fn root_for(old_root: Option<&Path>, location: &Path) -> PathBuf {
    match old_root {
        Some(root) if location.starts_with(root) => root.to_path_buf(),
        _ => location.to_path_buf(),
    }
}

/// Folders from `root` down to the parent of `location`, root first. Empty when
/// `location` is the root or not inside it.
pub fn ancestor_chain(root: &Path, location: &Path) -> Vec<PathBuf> {
    let Ok(rel) = location.strip_prefix(root) else {
        return Vec::new();
    };
    let mut chain = Vec::new();
    let mut path = root.to_path_buf();
    let mut components = rel.components().peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        chain.push(path.clone());
        path.push(component);
    }
    if location != root {
        chain.push(path);
    }
    chain
}

impl ColumnsState {
    /// Forget every column, for when the view is not columns or the location is not a folder.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Make the next location the leftmost column, as when chosen in the sidebar
    pub fn reset_root(&mut self) {
        self.root = None;
        self.location = None;
    }

    pub fn is_active(&self) -> bool {
        self.location.is_some()
    }

    /// Keep a listing (usually the previous active column) for reuse by the next rebuild.
    pub fn stash(&mut self, listing: ColumnListing) {
        self.cache.retain(|l| l.path != listing.path);
        self.cache.push(listing);
    }

    /// Remove and return a cached listing of `path`, if one is held.
    pub fn take_cached(&mut self, path: &Path) -> Option<ColumnListing> {
        let i = self.cache.iter().position(|l| l.path == path)?;
        Some(self.cache.swap_remove(i))
    }

    /// Rebuild the ancestor columns for a new active folder, reusing listings already held.
    /// Folders with no listing are queued for scanning.
    pub fn set_location(&mut self, location: &Path) {
        if self.location.as_deref() == Some(location) {
            return;
        }
        let root = root_for(self.root.as_deref(), location);
        let chain = ancestor_chain(&root, location);

        let old_ancestors = mem::take(&mut self.ancestors);
        for listing in old_ancestors.into_iter().chain(self.preview.take()) {
            self.stash(listing);
        }
        for path in chain {
            let listing = self.take_loaded_or_scan(&path);
            self.ancestors.push(listing);
        }

        self.root = Some(root);
        self.location = Some(location.to_path_buf());
        self.scroll_to_end = true;
    }

    /// Show `path` (the single selected folder) as the preview column, or no preview for `None`.
    pub fn set_preview(&mut self, path: Option<&Path>) {
        if self.preview.as_ref().map(|l| l.path.as_path()) == path {
            return;
        }
        if let Some(old) = self.preview.take() {
            self.stash(old);
        }
        if let Some(path) = path {
            self.preview = Some(self.take_loaded_or_scan(path));
        }
        self.scroll_to_end = true;
    }

    /// A cached listing of `path` if it finished loading, otherwise an empty one queued for
    /// scanning.
    fn take_loaded_or_scan(&mut self, path: &Path) -> ColumnListing {
        match self.take_cached(path) {
            Some(listing) if listing.items.is_some() => listing,
            _ => {
                self.pending_scans.push(path.to_path_buf());
                ColumnListing::new(path.to_path_buf(), None)
            }
        }
    }

    /// Drop listings no longer shown. Called once the active column and preview are settled.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn take_pending_scans(&mut self) -> Vec<PathBuf> {
        let mut scans = mem::take(&mut self.pending_scans);
        // A folder may have been queued and then dropped before the scan was sent
        scans.retain(|path| self.listing(path).is_some());
        scans.dedup();
        scans
    }

    /// Every shown column other than the active one, left to right
    pub fn listings(&self) -> impl Iterator<Item = &ColumnListing> {
        self.ancestors.iter().chain(self.preview.as_ref())
    }

    /// Items of every non-active column
    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut Item> {
        self.ancestors
            .iter_mut()
            .chain(self.preview.as_mut())
            .filter_map(|listing| listing.items.as_mut())
            .flatten()
    }

    fn listing(&self, path: &Path) -> Option<&ColumnListing> {
        self.listings().find(|l| l.path == path)
    }

    pub fn target(&self, target: ColumnTarget) -> Option<&ColumnListing> {
        match target {
            ColumnTarget::Ancestor(i) => self.ancestors.get(i),
            ColumnTarget::Preview => self.preview.as_ref(),
        }
    }

    /// Store scanned items for every column showing `path`. Returns whether any did.
    pub fn set_items(&mut self, path: &Path, items: Vec<Item>) -> bool {
        let listing = self
            .ancestors
            .iter_mut()
            .chain(self.preview.as_mut())
            .find(|l| l.path == path);
        match listing {
            Some(listing) => {
                listing.items = Some(items);
                true
            }
            None => false,
        }
    }

    /// The folder selected in column `index` of the ancestors: the next column's folder,
    /// or the active folder for the last ancestor.
    pub fn selected_child(&self, index: usize) -> Option<&Path> {
        match self.ancestors.get(index + 1) {
            Some(next) => Some(&next.path),
            None => self.location.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn root_kept_for_descendants_and_reset_otherwise() {
        assert_eq!(root_for(Some(&p("/a")), &p("/a/b/c")), p("/a"));
        assert_eq!(root_for(Some(&p("/a")), &p("/a")), p("/a"));
        // Moving left past the root makes the parent the new root
        assert_eq!(root_for(Some(&p("/a/b")), &p("/a")), p("/a"));
        // Sibling jump
        assert_eq!(root_for(Some(&p("/a/b")), &p("/a/c")), p("/a/c"));
        // Prefix that is not a path ancestor
        assert_eq!(root_for(Some(&p("/a/b")), &p("/a/bc")), p("/a/bc"));
        assert_eq!(root_for(None, &p("/x")), p("/x"));
    }

    #[test]
    fn ancestor_chain_runs_from_root_to_parent() {
        assert_eq!(
            ancestor_chain(&p("/a"), &p("/a/b/c")),
            vec![p("/a"), p("/a/b")]
        );
        assert_eq!(ancestor_chain(&p("/a"), &p("/a/b")), vec![p("/a")]);
        assert!(ancestor_chain(&p("/a"), &p("/a")).is_empty());
        assert!(ancestor_chain(&p("/a"), &p("/b")).is_empty());
        assert_eq!(ancestor_chain(&p("/"), &p("/a")), vec![p("/")]);
    }

    #[test]
    fn set_location_scans_only_missing_columns() {
        let mut state = ColumnsState::default();
        state.set_location(&p("/a"));
        assert!(state.ancestors.is_empty());
        assert!(state.take_pending_scans().is_empty());

        // Moving right: the old active folder is handed over as a stashed listing
        state.stash(ColumnListing::new(p("/a"), Some(Vec::new())));
        state.set_location(&p("/a/b"));
        assert_eq!(state.root.as_deref(), Some(Path::new("/a")));
        assert_eq!(state.ancestors.len(), 1);
        assert!(state.ancestors[0].items.is_some());
        assert!(state.take_pending_scans().is_empty());

        // Jumping deeper scans only the folders not yet held
        state.set_location(&p("/a/b/c/d"));
        let paths: Vec<_> = state.ancestors.iter().map(|l| l.path.clone()).collect();
        assert_eq!(paths, vec![p("/a"), p("/a/b"), p("/a/b/c")]);
        assert_eq!(state.take_pending_scans(), vec![p("/a/b"), p("/a/b/c")]);
    }

    #[test]
    fn moving_left_keeps_listing_for_preview() {
        let mut state = ColumnsState::default();
        state.set_location(&p("/a"));
        state.stash(ColumnListing::new(p("/a"), Some(Vec::new())));
        state.set_location(&p("/a/b"));
        state.take_pending_scans();

        // Left: /a/b becomes the preview of /a and needs no rescan
        state.stash(ColumnListing::new(p("/a/b"), Some(Vec::new())));
        state.set_location(&p("/a"));
        assert!(state.ancestors.is_empty());
        state.set_preview(Some(&p("/a/b")));
        assert!(state.preview.as_ref().unwrap().items.is_some());
        assert!(state.take_pending_scans().is_empty());
    }

    #[test]
    fn preview_scan_dropped_when_preview_changes_first() {
        let mut state = ColumnsState::default();
        state.set_location(&p("/a"));
        state.set_preview(Some(&p("/a/x")));
        state.set_preview(Some(&p("/a/y")));
        assert_eq!(state.take_pending_scans(), vec![p("/a/y")]);
        state.set_preview(None);
        assert!(state.preview.is_none());
    }

    #[test]
    fn set_items_matches_by_path() {
        let mut state = ColumnsState {
            root: Some(p("/a")),
            ..Default::default()
        };
        state.set_location(&p("/a/b"));
        assert!(state.set_items(&p("/a"), Vec::new()));
        assert!(!state.set_items(&p("/elsewhere"), Vec::new()));
        assert_eq!(state.selected_child(0), Some(Path::new("/a/b")));
    }
}
