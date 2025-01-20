use std::{convert::Infallible, marker::PhantomData, ops::{AddAssign, ControlFlow}};

use num_traits::Zero;

use super::{chain::{ChainStream, FixedChainStream}, zip_stream::ZipStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamResult<I, V> {
    Done,
    Yield { index: I, value: Option<V> },
}

pub trait IndexedStream {
    type I: Copy;
    type V;

    fn current(&self) -> StreamResult<Self::I, Self::V>;

    fn valid(&self) -> bool {
        match self.current() {
            StreamResult::Done => false,
            StreamResult::Yield { .. } => true
        }
    }

    fn index(&self) -> Option<Self::I> {
        match self.current() {
            StreamResult::Done => None,
            StreamResult::Yield { index, .. } => Some(index)
        }
    }

    /// Requests the stream to advance as far as possible up to `index`
    /// If `strict` is true, skipping `index` itself is permissible
    /// Will only be called when `valid` is true
    /// RULE (for termination): whenever (index, strict) >= (self.index(), self.ready()),
    /// (in the lexicographic order with false < true), then progress is made
    fn seek(&mut self, index: Self::I, strict: bool);

    /// Like `seek`, but guarantees that current() == Yield(index, value),
    /// where value.is_some() iff strict is true.
    /// Should be equivalent to calling seek with those parameters.
    /// Some stream implementations may choose to override this with a more efficient implementation.
    #[inline]
    fn next(&mut self, index: Self::I, strict: bool) {
        self.seek(index, strict);
    }

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
        while let StreamResult::Yield { index, value } = self.current() {
            if let Some(value) = value {
                self.next(index, true);
                acc = f(acc, index, value)?;
            } else {
                self.next(index, false);
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
    fn collect_indices<'a, I>(self) -> Vec<I>
    where
        Self: Sized + IndexedStream<I = &'a I>,
        I: 'a + Clone
    {
        let mut indices = Vec::new();
        self.for_each(|i, _| indices.push(i.clone()));
        indices
    }

    
    /// Collect the indices of this iterator as a Vec of (copied) indices
    /// TODO: turn this into an iterator
    fn collect_indices_ref(self) -> Vec<Self::I>
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

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        match self.stream.current() {
            StreamResult::Done => StreamResult::Done,
            StreamResult::Yield { index, value } => StreamResult::Yield { index, value: value.map(|v| (self.map)(index, v)) }
        }
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.stream.seek(index, strict);
    }

    fn next(&mut self, index: Self::I, strict: bool) {
        self.stream.next(index, strict);
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

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        match self.stream.current() {
            StreamResult::Done => StreamResult::Done,
            StreamResult::Yield { index, value } => StreamResult::Yield { index, value: value.cloned() }
        }
    }

    fn seek(&mut self, index: Self::I, strict: bool) {
        self.stream.seek(index, strict);
    }

    fn next(&mut self, index: Self::I, strict: bool) {
        self.stream.next(index, strict);
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
        loop {
            match self.stream.current() {
                StreamResult::Done => return None,
                StreamResult::Yield { index, value } => {
                    if self.index < index {
                        self.index += 1;
                        return Some(S::V::zero());
                    } else {
                        match value {
                            Some(v) => {
                                self.stream.next(index, true);
                                self.index += 1;
                                return Some(v);
                            },
                            None => {
                                self.stream.seek(index, false);
                            }
                        }
                    }
                },
            }
        }
    }    
}
