use std::borrow::Borrow;

use super::{binary_search::binary_search, stream_defs::{IndexedStream, StreamResult}};

#[derive(Debug, Clone)]
pub struct SortedVecGalloper<'a, T> {
    inds: &'a [T],
    cur: usize,
}

#[derive(Debug, Clone)]
pub struct SortedVecLinear<'a, T> {
    inds: &'a [T],
    cur: usize,
}

impl<'a, T> SortedVecGalloper<'a, T> {
    pub fn new(inds: &'a [T]) -> Self {
        SortedVecGalloper { inds, cur: 0 }
    }
}

impl<'a, T> SortedVecLinear<'a, T> {
    pub fn new(inds: &'a [T]) -> Self {
        SortedVecLinear { inds, cur: 0 }
    }
}

impl<'a, T: Ord> IndexedStream for SortedVecGalloper<'a, T> {
    type I = &'a T;
    type V = ();

    fn seek(&mut self, index: impl Borrow<Self::I>, strict: bool) {
        self.cur += binary_search(&self.inds[self.cur..], index.borrow(), strict);
    }

    fn next(&mut self, _index: impl Borrow<Self::I>, _strict: bool) {
        self.cur += 1;
    }

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        if self.cur < self.inds.len() {
            StreamResult::Yield {
                index: &self.inds[self.cur],
                value: Some(()),
            }
        } else {
            StreamResult::Done
        }
    }
}

impl<'a, T: Ord> IndexedStream for SortedVecLinear<'a, T> {
    type I = &'a T;
    type V = ();

    fn seek(&mut self, index: impl Borrow<Self::I>, strict: bool) {
        if (strict && self.inds[self.cur] <= **index.borrow()) ||
            (!strict && self.inds[self.cur] < **index.borrow()) {
            self.cur += 1;
        }
    }

    fn next(&mut self, _index: impl Borrow<Self::I>, _strict: bool) {
        self.cur += 1;
    }

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        if self.cur < self.inds.len() {
            StreamResult::Yield {
                index: &self.inds[self.cur],
                value: Some(()),
            }
        } else {
            StreamResult::Done
        }
    }
}

