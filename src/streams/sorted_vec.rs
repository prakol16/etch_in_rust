use super::{binary_search::binary_search, stream_defs::IndexedStream};

#[derive(Debug, Clone)]
pub struct SortedVecGalloper<'a, T> {
    inds: &'a [T],
    cur: usize,
}

impl<'a, T> SortedVecGalloper<'a, T> {
    pub fn new(inds: &'a [T]) -> Self {
        SortedVecGalloper { inds, cur: 0 }
    }
}

impl<T: Ord + Copy> IndexedStream for SortedVecGalloper<'_, T> {
    type I = T;
    type V = ();

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: T, strict: bool) {
        self.cur += binary_search(&self.inds[self.cur..], &index, strict);
    }

    fn index(&self) -> T {
        self.inds[self.cur].clone()
    }

    fn value(&self) -> () {}
}