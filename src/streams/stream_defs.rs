use std::{convert::Infallible, marker::PhantomData, ops::{AddAssign, ControlFlow}};

use num_traits::Zero;

pub trait IndexedIterator {
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
    fn seek(&mut self, index: &Self::I, strict: bool);

    /// Emit the current index of the stream
    /// INVARIANT: will only be called when `valid` is true
    fn index(&self) -> Self::I;

    /// Emit the current value of the stream
    /// INVARIANT: will only be called when `valid` and `ready` are true
    fn value(&self) -> Self::V;

    /// Get the value of the stream by folding over it
    /// A default implementation is given
    /// Stream combinators can override with more efficient implementations
    /// using child `try_fold` implementations.
    /// Note that `try_fold` should work even if the stream is resumed from
    /// some state other than the beginning, and it may not consume the entire stream either
    /// if it encounters an intermediate `break`.
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> ControlFlow<R, B> where
        F: FnMut(B, Self::I, Self::V) -> ControlFlow<R, B>
    {
        let mut acc = init;
        while self.valid() {
            let i = self.index();
            if self.ready() {
                let v = self.value();
                self.seek(&i, true);
                acc = f(acc, i, v)?;
            } else {
                self.seek(&i, false);
            }
        }
        ControlFlow::Continue(acc)
    }

    fn try_for_each<R>(&mut self, mut f: impl FnMut(Self::I, Self::V) -> ControlFlow<R>) -> ControlFlow<R>
    where
        Self: Sized
    {
        self.try_fold((), |(), i, v| f(i, v))
    }

    fn for_each(&mut self, mut f: impl FnMut(Self::I, Self::V))
    where
        Self: Sized
    {
        self.try_for_each(|i, v| {
            f(i, v);
            ControlFlow::<Infallible, ()>::Continue(())
        });
    }

    fn fold<B, F>(&mut self, init: B, f: F) -> B
    where
        F: Fn(B, Self::I, Self::V) -> B
    {
        match self.try_fold(init, |acc, i, v| 
            ControlFlow::<Infallible, B>::Continue(f(acc, i, v))) {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => match x {}
        }
    }

    fn contract(mut self) -> Self::V
    where
        Self: Sized,
        Self::V: AddAssign + Zero
    {
        self.fold(Self::V::zero(), |acc, _, v| acc + v)
    }

    fn any_nonzero(mut self) -> bool
    where
        Self: Sized,
        Self::V: Zero + PartialEq
    {
        self.try_for_each(|_, v| 
            if v != Self::V::zero() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        ).is_break()
    }

    fn map<O, F: Fn(Self::I, Self::V) -> O>(self, map: F) -> MappedStream<Self, F, O>
    where
        Self: Sized
    {
        MappedStream::map(self, map)
    }
}

pub trait IntoStreamIterator {
    /// The index type of the stream iterator that can produce T
    type IndexType;

    /// The value type of the stream iterator that can produce T
    type ValueType;

    /// The stream type
    type StreamType: IndexedIterator<I=Self::IndexType, V=Self::ValueType>;

    fn into_stream_iterator(self) -> Self::StreamType;
}

impl<S: IndexedIterator> IntoStreamIterator for S {
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

    fn from_stream_iterator<I: IndexedIterator<I=Self::IndexType, V=Self::ValueType>>(iter: I) -> Self;

    fn extend_from_stream_iterator<I: IndexedIterator<I=Self::IndexType, V=Self::ValueType>>(&mut self, iter: I);
}

impl<I, V> FromStreamIterator for Vec<(I, V)> {
    type IndexType = I;
    type ValueType = V;

    fn from_stream_iterator<Iter: IndexedIterator<I=I, V=V>>(iter: Iter) -> Self {
        let mut result = Vec::new();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: IndexedIterator<I=I, V=V>>(&mut self, mut iter: Iter) {
        iter.for_each(|i, v| {
            self.push((i, v));
        });
    }
}

pub struct MappedStream<S, F, O> {
    stream: S,
    map: F,
    _output: PhantomData<O>
}

impl<S, F, O> MappedStream<S, F, O>
        where S: IndexedIterator,
        F: Fn(S::I, S::V) -> O {
    pub fn map(stream: S, map: F) -> Self {
        MappedStream { stream, map, _output: PhantomData }
    }
}

impl<S, F, O> IndexedIterator for MappedStream<S, F, O>
    where S: IndexedIterator,
          F: Fn(S::I, S::V) -> O {
    type I = S::I;
    type V = O;

    fn valid(&self) -> bool {
        self.stream.valid()
    }

    fn ready(&self) -> bool {
        self.stream.ready()
    }

    fn seek(&mut self, index: &Self::I, strict: bool) {
        self.stream.seek(index, strict);
    }

    fn index(&self) -> Self::I {
        self.stream.index()
    }

    fn value(&self) -> Self::V {
        (self.map)(self.stream.index(), self.stream.value())
    }
}

/// A stream iterator that produces a dense stream of values at every index
pub struct DenseStreamIterator<S> {
    index: usize,
    stream: S
}

impl<S> DenseStreamIterator<S> {
    pub fn from_stream_iterator(stream: S) -> Self {
        DenseStreamIterator { index: 0, stream }
    }
}

impl<S> Iterator for DenseStreamIterator<S>
    where S: IndexedIterator<I = usize>,
          S::V: Zero
{
    type Item = S::V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.valid() {
            let i = self.stream.index();
            if self.index < i {
                self.index += 1;
                Some(S::V::zero())
            } else if self.stream.ready() {
                self.index += 1;
                let v = self.stream.value();
                self.stream.seek(&i, true);
                Some(v)
            } else {
                self.stream.seek(&self.index, false);
                None
            }
        } else {
            None
        }
    }    
}
