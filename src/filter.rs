use std::collections::VecDeque;

use pathiterator::{IteratorItem, FileIterator};

pub fn filter(iterator: FileIterator) -> Vec<IteratorItem> {
    let mut items: Vec<IteratorItem> = Vec::new();
    let mut cache: VecDeque<IteratorItem> = VecDeque::new();

    for mut item in iterator {
        let is_dir = item.is_dir();
        if is_dir {
            loop {
                if let Some(last) = cache.pop_back() {
                    if last.level < item.level
                    {
                        cache.push_back(last);
                        break;
                    }
                } else {
                    break;
                }
            }
            cache.push_back(item);
        } else {
            while let Some(cache_item) = cache.pop_front() {
                items.push(cache_item);
            }

            cache.clear();
            items.push(item);
        }
    }

    items    
}
