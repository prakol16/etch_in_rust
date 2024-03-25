use super::stream_defs::{FromStreamIterator, IntoStreamIterator, IndexedStream};


/// Helper function to compare with a strict parameter
#[inline]
pub(crate) fn lt_or_possibly_eq<I: Ord>(x: &I, y: &I, allow_eq: bool) -> bool {
    match x.cmp(y) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => allow_eq,
        std::cmp::Ordering::Greater => false,
    }
}

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
    #[allow(dead_code)]
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


impl<I: Ord + Clone, T: Clone> IndexedStream for SparseVecGalloper<'_, I, T> {
    type I = I;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: &I, strict: bool) {
        // Do a binary search to find the first element >= index (or > index if strict)
        let mut left = self.cur;
        let mut right = self.inds.len();
        while left < right {
            let mid = left + (right - left) / 2;
            if lt_or_possibly_eq(&self.inds[mid], index, strict){
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        self.cur = left;
    }

    fn index(&self) -> I {
        self.inds[self.cur].clone()
    }

    fn value(&self) -> T {
        self.vals[self.cur].clone()
    }
}

impl<I: Ord + Clone, T: Clone> IndexedStream for SparseVecIterator<'_, I, T> {
    type I = I;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn seek(&mut self, index: &I, strict: bool) {
        while self.inds[self.cur] < *index || (strict && self.inds[self.cur] == *index) {
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
        self.inds[self.cur].clone()
    }

    fn value(&self) -> T {
        self.vals[self.cur].clone()
    }
}

impl<I, T> SparseVec<I, T> {
    pub fn gallop(&self) -> SparseVecGalloper<'_, I, T> {
        SparseVecGalloper {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }
}

impl<'a, I: Ord + Clone, T: Clone> IntoStreamIterator for &'a SparseVec<I, T> {
    type IndexType = I;
    type ValueType = T;
    type StreamType = SparseVecIterator<'a, I, T>;

    fn into_stream_iterator(self) -> Self::StreamType {
        SparseVecIterator {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }
}

impl<I: Ord + Clone, T> FromStreamIterator for SparseVec<I, T> {
    type IndexType = I;
    type ValueType = T;

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
