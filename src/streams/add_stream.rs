use std::ops::Add;

use super::stream_defs::{IntoStreamIterator, IndexedStream};


pub struct AddStream<L, R> {
    left: L,
    right: R,
}

impl<L, R> AddStream<L, R> {
    pub fn add(
        left: impl IntoStreamIterator<StreamType = L>,
        right: impl IntoStreamIterator<StreamType = R>
    ) -> Self {
        AddStream {
            left: left.into_stream_iterator(),
            right: right.into_stream_iterator(),
        }
    }
}

impl<I, L, R> IndexedStream for AddStream<L, R> 
    where L: IndexedStream<I=I>,
          R: IndexedStream<I=I>,
          I: Ord + Copy,
          L::V: Add<R::V>, {
    type I = I;
    type V = <L::V as Add<R::V>>::Output;

    fn valid(&self) -> bool {
        self.left.valid() || self.right.valid()
    }

    fn ready(&self) -> bool {
        if self.left.valid() {
            if self.right.valid() {
                self.left.index() == self.right.index()
            } else {
                self.left.ready()
            }
        } else if self.right.valid() {
            self.right.ready()
        } else {
            panic!("AddStream::ready called when neither stream is valid")
        }
    }

    fn seek(&mut self, index: I, strict: bool) {
        self.left.seek(index, strict);
        self.right.seek(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().min(self.right.index())
    }

    fn value(&self) -> Self::V {
        self.left.value() + self.right.value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EitherOrBoth<A, B> {
    Left(A),
    Right(B),
    Both(A, B),
}

impl<A, B> EitherOrBoth<A, B> {
    pub fn map<Fa, Fb, Oa, Ob>(self, fa: Fa, fb: Fb) -> EitherOrBoth<Oa, Ob>
    where
        Fa: FnOnce(A) -> Oa,
        Fb: FnOnce(B) -> Ob,
    {
        match self {
            EitherOrBoth::Left(a) => EitherOrBoth::Left(fa(a)),
            EitherOrBoth::Right(b) => EitherOrBoth::Right(fb(b)),
            EitherOrBoth::Both(a, b) => EitherOrBoth::Both(fa(a), fb(b)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntersectingUnionStream<L, R, F> {
    left: L,
    right: R,
    f: F,
}

impl<L, R, F> IntersectingUnionStream<L, R, F> {
    pub fn new(
        left: impl IntoStreamIterator<StreamType = L>,
        right: impl IntoStreamIterator<StreamType = R>,
        f: F,
    ) -> Self {
        IntersectingUnionStream {
            left: left.into_stream_iterator(),
            right: right.into_stream_iterator(),
            f
        }
    }
}

impl<I, V, L, R, F> IndexedStream for IntersectingUnionStream<L, R, F> 
    where L: IndexedStream<I=I>,
          R: IndexedStream<I=I>,
          I: Ord + Copy,
          F: Fn(EitherOrBoth<L::V, R::V>) -> V,
{
    type I = I;
    type V = V;

    fn valid(&self) -> bool {
        self.left.valid() && self.right.valid()
    }

    fn ready(&self) -> bool {
        match self.left.index().cmp(&self.right.index()) {
            std::cmp::Ordering::Less => self.left.ready(),
            std::cmp::Ordering::Equal => self.left.ready() && self.right.ready(),
            std::cmp::Ordering::Greater => self.right.ready(),
        }
    }

    fn seek(&mut self, index: I, strict: bool) {
        self.left.seek(index, strict);
        self.right.seek(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().min(self.right.index())
    }

    fn value(&self) -> Self::V {
        match self.left.index().cmp(&self.right.index()) {
            std::cmp::Ordering::Less => (self.f)(EitherOrBoth::Left(self.left.value())),
            std::cmp::Ordering::Equal => (self.f)(EitherOrBoth::Both(self.left.value(), self.right.value())),
            std::cmp::Ordering::Greater => (self.f)(EitherOrBoth::Right(self.right.value())),
        }
    }
}

pub fn union<I, V, A, B, F>(x: A, y: B, f: F) -> impl IndexedStream<I = I, V = V>
where
    A: IndexedStream<I = I>,
    B:  IndexedStream<I = I>,
    F: Fn(EitherOrBoth<A::V, B::V>) -> V,
    I: Ord + Copy,
{
    IntersectingUnionStream::new(x, y, f)
    .and_then_chain(|stream| {
        stream.left.map(|_, x| EitherOrBoth::Left(x))
        .chain(stream.right.map(|_, y| EitherOrBoth::Right(y)))
        .map(move |_, either| (stream.f)(either))
    })
}

