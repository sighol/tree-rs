use crate::pathiterator::{FileIterator, IteratorItem};

pub struct FilteredIterator {
    pub source: FileIterator,
}

impl FilteredIterator {
    pub fn new(iterator: FileIterator) -> Self {
        FilteredIterator { source: iterator }
    }
}

impl Iterator for FilteredIterator {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}
