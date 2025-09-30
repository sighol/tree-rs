//! Directory traversal iterator with filtering and glob pattern matching.
//!
//! This module provides a recursive directory iterator that supports:
//! - Glob-based include/exclude patterns
//! - Hidden file filtering
//! - Depth limiting
//! - Directory-only mode
//!
//! Uses a breadth-first traversal strategy with `VecDeque` for efficient processing.

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use globset::GlobMatcher;

#[derive(Debug)]
pub struct IteratorItem {
    pub file_name: String,
    pub path: PathBuf,
    pub metadata: io::Result<Metadata>,
    pub level: usize,
    pub is_last: bool,
}

pub fn path_to_str(path: &Path) -> &str {
    path.file_name()
        .and_then(|x| x.to_str())
        .or_else(|| path.to_str())
        .unwrap_or("")
}

impl IteratorItem {
    fn new(path: &Path, level: usize, is_last: bool) -> Self {
        let metadata = path.symlink_metadata();

        Self {
            file_name: String::from(path_to_str(path)),
            path: path.to_owned(),
            metadata,
            level,
            is_last,
        }
    }

    fn from_dir_entry(entry: &DirEntry, level: usize, is_last: bool) -> Self {
        let path = entry.path();
        // Reuse metadata from DirEntry to avoid duplicate syscall
        let metadata = entry.metadata();

        Self {
            file_name: String::from(path_to_str(&path)),
            path,
            metadata,
            level,
            is_last,
        }
    }

    pub fn is_dir(&self) -> bool {
        self.metadata.as_ref().is_ok_and(Metadata::is_dir)
    }
}

#[derive(Debug)]
pub struct FileIteratorConfig {
    pub show_hidden: bool,
    pub show_only_dirs: bool,
    pub max_level: usize,
    pub include_globs: Arc<[GlobMatcher]>,
    pub exclude_globs: Arc<[GlobMatcher]>,
}

#[derive(Debug)]
pub struct FileIterator {
    queue: VecDeque<IteratorItem>,
    config: FileIteratorConfig,
}

/// Compares directory entries in reverse order for efficient `VecDeque` usage.
/// Sort is reversed because we use `pop_back()` in the iterator, ensuring
/// alphabetically first items are processed first (LIFO becomes FIFO).
fn order_dir_entry(a: &DirEntry, b: &DirEntry) -> Ordering {
    b.file_name().cmp(&a.file_name())
}

fn get_sorted_dir_entries(path: &Path, only_dirs: bool) -> io::Result<Vec<DirEntry>> {
    let entries = fs::read_dir(path)?;
    let mut dir_entries: Vec<DirEntry> = entries
        .into_iter()
        .filter(|entry| {
            entry.as_ref().is_ok_and(|entry| {
                entry
                    .metadata()
                    .is_ok_and(|meta| !only_dirs || meta.is_dir())
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    dir_entries.sort_by(order_dir_entry);
    Ok(dir_entries)
}

impl FileIterator {
    pub fn new(path: &Path, config: FileIteratorConfig) -> FileIterator {
        let mut queue = VecDeque::new();
        queue.push_back(IteratorItem::new(path, 0, true));
        FileIterator { queue, config }
    }

    fn is_glob_included(&self, file_name: &str) -> bool {
        let incl = &self.config.include_globs;
        let excl = &self.config.exclude_globs;

        let not_exclude = excl.is_empty() || excl.iter().all(|glob| !glob.is_match(file_name));
        let include = incl.is_empty() || incl.iter().any(|glob| glob.is_match(file_name));

        not_exclude && include
    }

    fn is_included(&self, name: &str, is_dir: bool) -> bool {
        (self.config.show_hidden || !name.starts_with('.'))
            && (is_dir || self.is_glob_included(name))
    }

    fn push_dir(&mut self, item: &IteratorItem) {
        let entries = match get_sorted_dir_entries(&item.path, self.config.show_only_dirs) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!(
                    "Warning: couldn't read directory {}: {}",
                    item.path.display(),
                    e
                );
                return;
            }
        };

        for (index, entry) in entries.iter().enumerate() {
            let item = IteratorItem::from_dir_entry(entry, item.level + 1, index == 0);
            if self.is_included(&item.file_name, item.is_dir()) {
                self.queue.push_back(item);
            }
        }
    }
}

impl Iterator for FileIterator {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_back().inspect(|item| {
            if item.is_dir() && item.level < self.config.max_level {
                self.push_dir(item);
            }
        })
    }
}
