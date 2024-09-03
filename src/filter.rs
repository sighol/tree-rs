use std::collections::VecDeque;

use crate::pathiterator::{FileIterator, IteratorItem};

pub struct FilteredIterator {
    pub source: FileIterator,
    cache: VecDeque<IteratorItem>,
    skip: bool,
    next_item: Option<IteratorItem>,
}

impl FilteredIterator {
    pub fn new(iterator: FileIterator) -> Self {
        FilteredIterator {
            source: iterator,
            cache: VecDeque::new(),
            skip: false,
            next_item: None,
        }
    }

    pub fn skip_filter(&mut self) {
        self.skip = true;
    }
}

impl Iterator for FilteredIterator {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip {
            return self.source.next();
        }

        if let Some(cache_item) = self.cache.pop_front() {
            return Some(cache_item);
        }

        if let Some(next_item) = self.next_item.take() {
            return Some(next_item);
        }

        for item in self.source.by_ref() {
            if item.is_dir() {
                self.cache.push_back(item);
            } else {
                // If the cache already contains a folder, 
                // start emptying cache, and save the item.
                if let Some(cache_front) = self.cache.pop_front() {
                    self.next_item = Some(item);
                    return Some(cache_front);
                }
                return Some(item);
            }
        }

        None
    }
}
