use num_traits::Zero;

use super::{binary_search::binary_search, stream_defs::{FromStreamIterator, IndexedStream, IntoStreamIterator}};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SparseVec<I, T> {
    /// The data in the sparse vector
    /// Assumes that the indices are sorted in ascending order
    pub inds: Vec<I>,

    pub vals: Vec<T>,
}

impl<I, T> FromIterator<(I, T)> for SparseVec<I, T> {
    fn from_iter<V: IntoIterator<Item = (I, T)>>(v: V) -> Self {
        let (inds, vals) = v.into_iter().unzip();
        SparseVec { inds, vals }
    }
}

impl<I, T> SparseVec<I, T> {
    pub fn iter(&self) -> std::iter::Zip<std::slice::Iter<I>, std::slice::Iter<T>> {
        self.inds.iter().zip(self.vals.iter())
    }

    pub fn get(&self, index: I) -> Option<&T>
    where I: Ord
    {
        match self.inds.binary_search(&index) {
            Ok(i) => Some(&self.vals[i]),
            Err(_) => None,
        }
    }
}

impl<I: PartialEq, V: Zero + PartialEq> SparseVec<I, V> {
    pub fn eq_ignoring_zeros(&self, other: &Self) -> bool {
        self.iter()
            .filter(|(_, v)| !V::is_zero(v))
            .eq(other.iter().filter(|(_, v)| !V::is_zero(v)))
    }
}

impl<I, T> IntoIterator for SparseVec<I, T> {
    type Item = (I, T);
    type IntoIter = std::iter::Zip<std::vec::IntoIter<I>, std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inds.into_iter().zip(self.vals.into_iter())
    }
}


impl<I, T> SparseVec<I, T> {
    pub fn len(&self) -> usize {
        self.inds.len()
    }

    /// Creates an empty `SparseVec`.
    pub fn empty() -> Self {
        SparseVec {
            inds: Vec::new(),
            vals: Vec::new(),
        }
    }

    /// Creates a `SparseVec` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        SparseVec {
            inds: Vec::with_capacity(capacity),
            vals: Vec::with_capacity(capacity),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SparseVecGalloper<'a, I, T> {
    inds: &'a [I],
    vals: &'a [T],
    cur: usize
}

#[derive(Debug, Clone)]
pub struct SparseVecIterator<'a, I, T> {
    inds: &'a [I],
    vals: &'a [T],
    cur: usize
}

impl<'a, I, T> SparseVecIterator<'a, I, T> {
    pub fn new(inds: &'a [I], vals: &'a [T]) -> Self {
        SparseVecIterator { inds, vals, cur: 0 }
    }
}

impl<'a, I, T> SparseVecGalloper<'a, I, T> {
    #[allow(dead_code)]
    pub fn new(inds: &'a [I], vals: &'a [T]) -> Self {
        SparseVecGalloper { inds, vals, cur: 0 }
    }
}


impl<'a, I: Ord + Copy, T> IndexedStream for SparseVecGalloper<'a, I, T> {
    type I = I;
    type V = &'a T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: I, strict: bool) {
        self.cur += binary_search(&self.inds[self.cur..], &index, strict);
    }

    fn next(&mut self) {
        self.cur += 1;
    }

    fn index(&self) -> I {
        self.inds[self.cur]
    }

    fn value(&self) -> &'a T {
        &self.vals[self.cur]
    }
}

impl<'a, I: Ord + Copy, T> IndexedStream for SparseVecIterator<'a, I, T> {
    type I = I;
    type V = &'a T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: I, strict: bool) {
        while self.inds[self.cur] < index || (strict && self.inds[self.cur] == index) {
            self.cur += 1;
            if self.cur >= self.inds.len() {
                break;
            }
        }
    }

    fn next(&mut self) {
        self.cur += 1;
    }

    fn index(&self) -> I {
        self.inds[self.cur]
    }

    fn value(&self) -> &'a T {
        &self.vals[self.cur]
    }
}

impl<I, T> SparseVec<I, T> {
    pub fn stream_iter(&self) -> SparseVecGalloper<'_, I, T> {
        SparseVecGalloper {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }

    pub fn stream_iter_linear(&self) -> SparseVecIterator<'_, I, T> {
        SparseVecIterator {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }
}

impl<'a, I: Ord + Copy, T> IntoStreamIterator for &'a SparseVec<I, T> {
    type IndexType = I;
    type ValueType = &'a T;
    type StreamType = SparseVecGalloper<'a, I, T>;

    fn into_stream_iterator(self) -> Self::StreamType {
        self.stream_iter()
    }
}

impl<I: Ord + Clone, T> FromStreamIterator<I, T> for SparseVec<I, T> {
    fn from_stream_iterator<Iter: IndexedStream<I=I, V=T>>(iter: Iter) -> Self {
        let mut result = SparseVec {
            inds: Vec::new(),
            vals: Vec::new(),
        };
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: IndexedStream<I=I, V=T>>(&mut self, iter: Iter) {
        iter.for_each(|i, v| {
            self.inds.push(i);
            self.vals.push(v);
        });
    }
}
