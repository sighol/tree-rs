use std::fs::{self, DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::io;
use std::cmp::Ordering;

use globset::{GlobMatcher};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct IteratorItem {
    pub file_name: String,
    pub path: PathBuf,
    pub metadata: Metadata,
    pub level: i32,
    pub is_last_in_folder: bool,
}

pub fn path_to_str(path: &Path) -> &str {
    path.file_name()
        .and_then(|x| x.to_str())
        .or_else(|| path.to_str())
        .unwrap_or("")
}

impl IteratorItem {
    fn new(path: &Path, level: i32, is_last: bool) -> IteratorItem {
        
        let metadata = path.symlink_metadata().expect("symlink_metadata");

        IteratorItem {
            file_name: String::from(path_to_str(path)),
            path: path.to_owned(),
            metadata: metadata,
            level: level,
            is_last_in_folder: is_last,
        }
    }

    fn is_dir(&self) -> bool {
        self.metadata.is_dir()
    }

    pub fn to_string(&self) -> String {
        // format!("{}", self.path.display())
        format!("path={:60} level={} is_last={}", self.path.display(), self.level, self.is_last_in_folder)
    }
}

pub struct FileIteratorConfig {
    pub show_hidden: bool,
    pub max_level: usize,
    pub include_glob: Option<GlobMatcher>,
}

pub struct FileIterator {
    queue: VecDeque<IteratorItem>,
    config: FileIteratorConfig,
}

fn order_dir_entry(a: &DirEntry, b: &DirEntry) -> Ordering {
    b.file_name().cmp(&a.file_name())
}

fn get_sorted_dir_entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    fs::read_dir(path).map(|entries| {
        let mut dir_entries: Vec<DirEntry> = entries.filter_map(Result::ok).collect();
        dir_entries.sort_by(order_dir_entry);
        dir_entries
    })
}

impl FileIterator {
    fn new(path: &Path, config: FileIteratorConfig) -> FileIterator {
        let mut queue = VecDeque::new();
        queue.push_back(IteratorItem::new(path, 0, true));
        FileIterator {
            queue: queue,
            config: config,
        }
    }

    fn is_glob_included(&self, file_name: &str) -> bool {
        if let Some(ref glob) = self.config.include_glob {
            glob.is_match(file_name)
        } else {
            true
        }
    }

    fn is_included(&self, name: &str, is_dir: bool) -> bool {
        if !self.config.show_hidden && name.starts_with(".") {
            return false;
        }

        if is_dir {
            return true;
        } else {
            self.is_glob_included(&name)
        }
    }

    fn push_dir(&mut self, item: &IteratorItem) {
        let entries = get_sorted_dir_entries(&item.path);
        if let Ok(entries) = entries {

            let mut entries: Vec<IteratorItem> = entries.iter()
            .map(|e| IteratorItem::new(&e.path(), item.level + 1, false))
            .filter(|item| {
                self.is_included(&item.file_name, item.is_dir())
            }).collect();

            if let Some(item) = entries.first_mut() {
                item.is_last_in_folder = true;
            }

            for item in entries {
                self.queue.push_back(item);
            }
        }
    }
}

impl Iterator for FileIterator {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.queue.pop_back(){
            if item.metadata.is_dir() {
                self.push_dir(&item);
            }

            Some(item)
        } else {
            None
        }
    }
}

pub fn iterate(path: &Path, config: FileIteratorConfig) -> FileIterator {
    FileIterator::new(path, config)
}