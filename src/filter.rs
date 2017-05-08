use std::collections::VecDeque;

use pathiterator::{IteratorItem, FileIterator};

pub struct FilteredIterator {
    pub source: FileIterator,
    cache: VecDeque<IteratorItem>,
    skip: bool,
}

impl FilteredIterator {
    pub fn new(iterator: FileIterator) -> Self {
        FilteredIterator {
            source: iterator,
            cache: VecDeque::new(),
            skip: false,
        }
    }

    pub fn skip_filter(&mut self) {
        self.skip = true;
    }

    /// Remove previous directories from cache that shouldn't be
    /// shown because they are empty.
    fn remove_empty_directories_from_cache(&mut self, item: &IteratorItem) {
        loop {
            if let Some(last) = self.cache.pop_back() {
                if last.level < item.level {
                    self.cache.push_back(last);
                    break;
                }
            } else {
                break;
            }
        }
    }
}

impl Iterator for FilteredIterator {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip {
            return self.source.next();
        }

        if let Some(cache_item) = self.cache.pop_front() {
            return Some(cache_item)
        }

        loop {
            if let Some(item) = self.source.next() {
                self.remove_empty_directories_from_cache(&item);
                
                if item.is_dir() {
                    self.cache.push_back(item)
                } else {
                    return Some(item)
                }
            } else {
                break;
            }
        }

        None
    }
}
