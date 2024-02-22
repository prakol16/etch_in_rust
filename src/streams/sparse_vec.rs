use super::stream_defs::{FromStreamIterator, IntoStreamIterator, StreamIterator};


#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SparseVec<T> {
    /// The data in the sparse vector
    /// Assumes that the indices are sorted in ascending order
    pub inds: Vec<usize>,

    pub vals: Vec<T>,
}

impl<T, V: IntoIterator<Item = (usize, T)>> From<V> for SparseVec<T> {
    fn from(v: V) -> Self {
        let (inds, vals) = v.into_iter().unzip();
        SparseVec {
            inds,
            vals,
        }
    }
}

impl<T> SparseVec<T> {
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

pub struct SparseVecGalloper<'a, T> {
    inds: &'a [usize],
    vals: &'a [T],
    cur: usize
}

pub struct SparseVecIterator<'a, T> {
    inds: &'a [usize],
    vals: &'a [T],
    cur: usize
}

impl<'a, T> SparseVecIterator<'a, T> {
    pub fn new(inds: &'a [usize], vals: &'a [T]) -> Self {
        SparseVecIterator { inds, vals, cur: 0 }
    }
}

impl<'a, T> SparseVecGalloper<'a, T> {
    #[allow(dead_code)]
    pub fn new(inds: &'a [usize], vals: &'a [T]) -> Self {
        SparseVecGalloper { inds, vals, cur: 0 }
    }
}


impl<T: Clone> StreamIterator for SparseVecGalloper<'_, T> {
    type I = usize;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn skip(&mut self, index: &usize, strict: bool) {
        // Do a binary search to find the first element >= index (or > index if strict)
        let mut left = self.cur;
        let mut right = self.inds.len();
        while left < right {
            let mid = left + (right - left) / 2;
            if self.inds[mid] < *index || (strict && self.inds[mid] == *index) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        self.cur = left;
    }

    fn index(&self) -> usize {
        self.inds[self.cur]
    }

    fn value(&self) -> T {
        self.vals[self.cur].clone()
    }
}

impl<T: Clone> StreamIterator for SparseVecIterator<'_, T> {
    type I = usize;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.inds.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn skip(&mut self, index: &usize, strict: bool) {
        while self.inds[self.cur] < *index || (strict && self.inds[self.cur] == *index) {
            self.cur += 1;
            if self.cur >= self.inds.len() {
                break;
            }
        }
    }

    fn index(&self) -> usize {
        self.inds[self.cur]
    }

    fn value(&self) -> T {
        self.vals[self.cur].clone()
    }
}

impl<T> SparseVec<T> {
    pub fn gallop(&self) -> SparseVecGalloper<'_, T> {
        SparseVecGalloper {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }
}

impl<'a, T: Clone> IntoStreamIterator for &'a SparseVec<T> {
    type IndexType = usize;
    type ValueType = T;
    type StreamType = SparseVecIterator<'a, T>;

    fn into_stream_iterator(self) -> Self::StreamType {
        SparseVecIterator {
            inds: &self.inds,
            vals: &self.vals,
            cur: 0
        }
    }
}

impl<T: std::ops::AddAssign + Default + Clone> FromStreamIterator for SparseVec<T> {
    type IndexType = usize;
    type ValueType = T;

    fn from_stream_iterator<Iter: StreamIterator<I=usize, V=T>>(iter: Iter) -> Self {
        let mut result = SparseVec {
            inds: Vec::new(),
            vals: Vec::new(),
        };
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: StreamIterator<I=usize, V=T>>(&mut self, mut iter: Iter) {
        while iter.valid() {
            if iter.ready() {
                let ind = iter.index();
                let val = iter.value();
                iter.skip(&ind, true);

                // Check if the current index matches the last index in the SparseVec
                if let Some(last_ind) = self.inds.last() {
                    if *last_ind == ind {
                        // If so, add the value to the last value in the SparseVec
                        if let Some(last_val) = self.vals.last_mut() {
                            *last_val += val;
                        }
                    } else {
                        // Otherwise, push the new index and value
                        self.inds.push(ind);
                        self.vals.push(val);
                    }
                } else {
                    // If the SparseVec is empty, just push the new index and value
                    self.inds.push(ind);
                    self.vals.push(val);
                }
            } else {
                iter.skip(&iter.index(), false);
            }
        }
    }
}
