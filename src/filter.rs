use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::collections::VecDeque;


struct TreePrinterCacheItem {
    metadata: Metadata,
    levels: Vec<bool>,
    path: PathBuf,
}

impl TreePrinterCacheItem {
    fn new(path: PathBuf, metadata: Metadata, levels: Vec<bool>) -> TreePrinterCacheItem {
        TreePrinterCacheItem {
            path: path,
            metadata: metadata,
            levels: levels,
        }
    }
}

struct TreePrinterCache {
    items: VecDeque<TreePrinterCacheItem>,
}

impl TreePrinterCache {
    fn new() -> TreePrinterCache {
        TreePrinterCache { items: VecDeque::new() }
    }
}