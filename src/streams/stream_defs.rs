use std::{convert::Infallible, marker::PhantomData, ops::{AddAssign, ControlFlow}};

use num_traits::Zero;

use super::{chain::{ChainStream, FixedChainStream}, zip_stream::ZipStream};

pub trait IndexedStream {
    type I: Copy;
    type V;

    /// Determines if the stream has been exhausted.
    fn valid(&self) -> bool;

    /// Determines if the stream should yield an element in its current state.
    /// Will only be called when `valid` is true
    fn ready(&self) -> bool;

    /// Requests the stream to advance as far as possible up to `index`
    /// If `strict` is true, skipping `index` itself is permissible
    /// Will only be called when `valid` is true
    /// RULE (for termination): whenever (index, strict) >= (self.index(), self.ready()),
    /// (in the lexicographic order with false < true), then progress is made
    fn seek(&mut self, index: Self::I, strict: bool);

    /// Should be equivalent to seek(index(), ready()).
    /// Will only be called when `valid` is true.
    /// Some stream implementations may choose to override this with a more efficient implementation.
    #[inline]
    fn next(&mut self) {
        self.seek(self.index(), self.ready());
    }

    /// Emit the current index of the stream.
    /// Will only be called when `valid` is true
    fn index(&self) -> Self::I;

    /// Emit the current value of the stream.
    /// Will only be called when `valid` and `ready` are true
    fn value(&self) -> Self::V;

    /// Get the value of the stream by folding over it.
    /// A default implementation is given.
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
            if self.ready() {
                let i = self.index();
                let v = self.value();
                self.next();
                acc = f(acc, i, v)?;
            } else {
                self.next();
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

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::I, Self::V) -> B
    {
        match self.try_fold(init, 
            |acc, i, v| ControlFlow::<Infallible, B>::Continue(f(acc, i, v))
        ) {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(x) => match x {}
        }
    }

    fn for_each(self, mut f: impl FnMut(Self::I, Self::V))
    where
        Self: Sized
    {
        self.fold((), |(), i, v| f(i, v))
    }

    fn contract(self) -> Self::V
    where
        Self: Sized,
        Self::V: AddAssign + Zero
    {
        self.fold(Self::V::zero(), |acc, _, v| acc + v)
    }

    /// Collect the indices of this iterator as a Vec
    /// TODO: turn this into an iterator
    fn collect_indices(self) -> Vec<Self::I>
    where
        Self: Sized
    {
        let mut indices = Vec::new();
        self.for_each(|i, _| indices.push(i));
        indices
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

    fn cloned<'a, V>(self) -> ClonedStream<Self>
    where
        Self: Sized + IndexedStream<V = &'a V>,
        V: Clone + 'a
    {
        ClonedStream::new(self)
    }

    fn zip_with<R: IndexedStream<I = Self::I>, O, F: Fn(Self::V, R::V) -> O>(self, right: R, f: F) -> ZipStream<Self, R, F>
    where
        Self: Sized
    {
        ZipStream::new(self, right, f)
    }

    fn collect<O: FromStreamIterator<Self::I, Self::V>>(self) -> O
    where
        Self: Sized
    {
        O::from_stream_iterator(self)
    }

    fn and_then_chain<B, F>(self, second: F) -> ChainStream<Self, B, F>
    where
        Self: Sized,
        B: IndexedStream<I = Self::I, V = Self::V>,
        F: FnOnce(Self) -> B,
    {
        ChainStream::chain(self, second)
    }

    fn chain<B>(self, second: B) -> FixedChainStream<Self, B>
    where
        Self: Sized,
        B: IndexedStream<I = Self::I, V = Self::V>,
    {
        FixedChainStream::new(self, second)
    }
}

pub trait IntoStreamIterator {
    /// The index type of the stream iterator that can produce T
    type IndexType;

    /// The value type of the stream iterator that can produce T
    type ValueType;

    /// The stream type
    type StreamType: IndexedStream<I=Self::IndexType, V=Self::ValueType>;

    fn into_stream_iterator(self) -> Self::StreamType;
}

impl<S: IndexedStream> IntoStreamIterator for S {
    type IndexType = S::I;
    type ValueType = S::V;
    type StreamType = S;

    fn into_stream_iterator(self) -> Self::StreamType {
        self
    }
}

pub trait FromStreamIterator<I, V> {

    fn from_stream_iterator<S: IndexedStream<I=I, V=V>>(iter: S) -> Self;

    fn extend_from_stream_iterator<S: IndexedStream<I=I, V=V>>(&mut self, iter: S);
}

impl<I, V> FromStreamIterator<I, V> for Vec<(I, V)> {
    fn from_stream_iterator<Iter: IndexedStream<I=I, V=V>>(iter: Iter) -> Self {
        let mut result = Vec::new();
        result.extend_from_stream_iterator(iter);
        result
    }

    fn extend_from_stream_iterator<Iter: IndexedStream<I=I, V=V>>(&mut self, iter: Iter) {
        iter.for_each(|i, v| {
            self.push((i, v));
        });
    }
}

#[derive(Debug, Clone)]
pub struct MappedStream<S, F, O> {
    stream: S,
    map: F,
    _output: PhantomData<O>
}

impl<S, F, O> MappedStream<S, F, O>
        where S: IndexedStream,
        F: Fn(S::I, S::V) -> O {
    pub fn map(stream: S, map: F) -> Self {
        MappedStream { stream, map, _output: PhantomData }
    }
}

impl<S, F, O> IndexedStream for MappedStream<S, F, O>
    where S: IndexedStream,
          F: Fn(S::I, S::V) -> O {
    type I = S::I;
    type V = O;

    fn valid(&self) -> bool {
        self.stream.valid()
    }

    fn ready(&self) -> bool {
        self.stream.ready()
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.stream.seek(index, strict);
    }

    fn next(&mut self) {
        self.stream.next();
    }

    fn index(&self) -> Self::I {
        self.stream.index()
    }

    fn value(&self) -> Self::V {
        (self.map)(self.stream.index(), self.stream.value())
    }

    fn try_fold<B, FF, R>(&mut self, init: B, mut f: FF) -> ControlFlow<R, B> where
            FF: FnMut(B, Self::I, Self::V) -> ControlFlow<R, B> {
        self.stream.try_fold(init, |acc, i, v| f(acc, i, (self.map)(i, v)))
    }
}

#[derive(Debug, Clone)]
pub struct ClonedStream<S> 
where
    S: IndexedStream
{
    stream: S,
}

impl<S> ClonedStream<S>
where
    S: IndexedStream
{
    pub fn new(stream: S) -> Self {
        ClonedStream { stream }
    }
}

impl<'a, V, S> IndexedStream for ClonedStream<S>
where
    S: IndexedStream<V = &'a V>,
    V: Clone + 'a,
{
    type I = S::I;
    type V = V;

    fn valid(&self) -> bool {
        self.stream.valid()
    }

    fn ready(&self) -> bool {
        self.stream.ready()
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.stream.seek(index, strict);
    }

    fn next(&mut self) {
        self.stream.next();
    }

    fn index(&self) -> Self::I {
        self.stream.index()
    }

    fn value(&self) -> Self::V {
        self.stream.value().clone()
    }

    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> ControlFlow<R, B>
    where
        F: FnMut(B, Self::I, Self::V) -> ControlFlow<R, B>
    {
        self.stream.try_fold(init, |acc, i, v| f(acc, i, v.clone()))
    }
}

/// A stream iterator that produces a dense stream of values at every index
/// filling in values with a default zero value if now value is provided
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
    where S: IndexedStream<I = usize>,
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
                self.stream.next();
                Some(v)
            } else {
                self.stream.seek(self.index, false);
                None
            }
        } else {
            None
        }
    }    
}


pub trait CloneableIndexedStream: IndexedStream + Clone {}

impl<S> CloneableIndexedStream for S where S: IndexedStream + Clone {}
