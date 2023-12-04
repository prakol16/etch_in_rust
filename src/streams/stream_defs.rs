use std::ops::Mul;

use num_traits::Zero;


pub trait StreamIterator {
    type I;
    type V;

    /// Determines if the stream has been exhausted.
    fn valid(&self) -> bool;

    /// Determines if the stream should yield an element in its current state
    fn ready(&self) -> bool;

    /// Requests the stream to advance as far as possible up to `index`
    /// If `strict` is true, skipping `index` itself is permissible
    /// INVARIANT: will only be called when `valid` is true
    /// RULE (for termination): whenever (index, strict) >= (self.index(), self.ready()),
    /// (in the lexicographic order with false < true), then progress is made
    fn skip(&mut self, index: &Self::I, strict: bool);

    /// Emit the current index of the stream
    /// INVARIANT: will only be called when `valid` is true
    fn index(&self) -> Self::I;

    /// Emit the current value of the stream
    /// INVARIANT: will only be called when `valid` and `ready` are true
    fn value(&self) -> Self::V;
}

pub trait IntoStreamIterator {
    /// The index type of the stream iterator that can produce T
    type IndexType;

    /// The value type of the stream iterator that can produce T
    type ValueType;

    /// The stream type
    type StreamType: StreamIterator<I=Self::IndexType, V=Self::ValueType>;

    fn into_stream_iterator(self) -> Self::StreamType;
}

impl<S: StreamIterator> IntoStreamIterator for S {
    type IndexType = S::I;
    type ValueType = S::V;
    type StreamType = S;

    fn into_stream_iterator(self) -> Self::StreamType {
        self
    }
}

pub trait FromStreamIterator {
    /// The index type of the stream iterator that can produce T
    type IndexType;

    /// The value type of the stream iterator that can produce T
    type ValueType;

    fn from_stream_iterator<I: StreamIterator<I=Self::IndexType, V=Self::ValueType>>(iter: I) -> Self;

    fn extend_from_stream_iterator<I: StreamIterator<I=Self::IndexType, V=Self::ValueType>>(&mut self, iter: I);
}

impl<I, V> FromStreamIterator for Vec<(I, V)> {
    type IndexType = I;
    type ValueType = V;

    fn from_stream_iterator<Iter: StreamIterator<I=I, V=V>>(iter: Iter) -> Self {
        let mut result = Vec::new();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: StreamIterator<I=I, V=V>>(&mut self, mut iter: Iter) {
        while iter.valid() {
            if iter.ready() {
                let ind = iter.index();
                let val = iter.value();
                iter.skip(&ind, true);
                self.push((ind, val));
            } else {
                iter.skip(&iter.index(), false);
            }
        }
    }
}

pub struct MulStream<L, R> {
    left: L,
    right: R,
}

impl<L, R> MulStream<L, R> {
    pub fn mul(
        left: impl IntoStreamIterator<StreamType = L>,
        right: impl IntoStreamIterator<StreamType = R>
    ) -> Self {
        MulStream {
            left: left.into_stream_iterator(),
            right: right.into_stream_iterator(),
        }
    }
}

impl<I, L, R> StreamIterator for MulStream<L, R> 
    where L: StreamIterator<I=I>,
          R: StreamIterator<I=I>,
          I: Ord,
          L::V: Mul<R::V>, {
    type I = I;
    type V = <L::V as Mul<R::V>>::Output;

    fn valid(&self) -> bool {
        self.left.valid() && self.right.valid()
    }

    fn ready(&self) -> bool {
        self.left.ready() && self.right.ready() && self.left.index() == self.right.index()
    }

    fn skip(&mut self, index: &I, strict: bool) {
        self.left.skip(index, strict);
        self.right.skip(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().max(self.right.index())
    }

    fn value(&self) -> Self::V {
        self.left.value() * self.right.value()
    }
}

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


