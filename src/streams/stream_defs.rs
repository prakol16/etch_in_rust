use std::ops::AddAssign;

use num_traits::Zero;

pub trait StreamIterator {
    type I;
    type V;

    /// Determines if the stream has been exhausted.
    fn valid(&self) -> bool;

    /// Determines if the stream should yield an element in its current state
    /// Will only be called when `valid` is true
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

pub trait ScalarStream: AddAssign + Zero {}

macro_rules! impl_scalar_stream {
    ($($t:ty),*) => {
        $(
            impl ScalarStream for $t {}
        )*
    }
}

impl_scalar_stream!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

impl<V: ScalarStream> FromStreamIterator for V {
    type IndexType = ();
    type ValueType = V;

    fn from_stream_iterator<I: StreamIterator<I=Self::IndexType, V=Self::ValueType>>(iter: I) -> Self {
        let mut result: Self = V::zero();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<I: StreamIterator<I=Self::IndexType, V=Self::ValueType>>(&mut self, mut iter: I) {
        while iter.valid() {
            if iter.ready() {
                let val = iter.value();
                iter.skip(&(), true);
                *self += val;
            } else {
                iter.skip(&(), false);
            }
        }
    }
}

impl<I: PartialEq, V: AddAssign> FromStreamIterator for Vec<(I, V)> {
    type IndexType = I;
    type ValueType = V;

    fn from_stream_iterator<Iter: StreamIterator<I=I, V=V>>(iter: Iter) -> Self {
        let mut result = Vec::new();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: StreamIterator<I=I, V=V>>(&mut self, mut iter: Iter) {
        let mut last_pair: Option<(I, V)> = None;

        while iter.valid() {
            if iter.ready() {
                let ind = iter.index();
                let val = iter.value();
                iter.skip(&ind, true);

                match &mut last_pair {
                    Some((last_ind, last_val)) if *last_ind == ind => {
                        *last_val += val;
                    },
                    _ => {
                        if let Some(pair) = last_pair.take() {
                            self.push(pair);
                        }
                        last_pair = Some((ind, val));
                    }
                }
            } else {
                iter.skip(&iter.index(), false);
            }
        }

        if let Some(pair) = last_pair {
            self.push(pair);
        }
    }
}


