use super::stream_defs::{IntoStreamIterator, StreamIterator};


#[derive(Debug, Clone)]
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
    pub fn get_index(&self, i: usize) -> usize {
        self.inds[i]
    }

    pub fn get_value(&self, i: usize) -> &T {
        &self.vals[i]
    }

    pub fn len(&self) -> usize {
        self.inds.len()
    }
}

pub struct SparseVecGalloper<'a, T> {
    data: &'a SparseVec<T>,
    cur: usize
}

pub struct SparseVecIterator<'a, T> {
    data: &'a SparseVec<T>,
    cur: usize
}

impl<T: Clone> StreamIterator for SparseVecGalloper<'_, T> {
    type I = usize;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.data.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn skip(&mut self, index: &usize, strict: bool) {
        // Do a binary search to find the first element >= index (or > index if strict)
        let mut left = self.cur;
        let mut right = self.data.len();
        while left < right {
            let mid = left + (right - left) / 2;
            if self.data.get_index(mid) < *index || (strict && self.data.get_index(mid) == *index) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        self.cur = left;
    }

    fn index(&self) -> usize {
        self.data.get_index(self.cur)
    }

    fn value(&self) -> T {
        self.data.get_value(self.cur).clone()
    }
}

impl<T: Clone> StreamIterator for SparseVecIterator<'_, T> {
    type I = usize;
    type V = T;

    fn valid(&self) -> bool {
        self.cur < self.data.len()
    }

    fn ready(&self) -> bool {
        true
    }

    fn skip(&mut self, index: &usize, strict: bool) {
        while self.data.get_index(self.cur) < *index || (strict && self.data.get_index(self.cur) == *index) {
            self.cur += 1;
            if self.cur >= self.data.len() {
                break;
            }
        }
    }

    fn index(&self) -> usize {
        self.data.get_index(self.cur)
    }

    fn value(&self) -> T {
        self.data.get_value(self.cur).clone()
    }
}

impl<T> SparseVec<T> {
    pub fn gallop(&self) -> SparseVecGalloper<'_, T> {
        SparseVecGalloper {
            data: self,
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
            data: self,
            cur: 0
        }
    }
}
