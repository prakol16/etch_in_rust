use std::ops::ControlFlow;

use replace_with::{replace_with_or_abort, replace_with_or_abort_and_return};

use super::stream_defs::{IndexedStream, IntoStreamIterator};


/// A stream that chains two streams together, using the second stream when the first stream is exhausted.
/// Invariant: if the variant is `First`, then the first stream is valid i.e.,
/// after calling `next` or `seek` on the first stream, we must check if it is valid.
#[derive(Clone)]
pub enum ChainStream<A, B, F> {
    First { stream: A, f: F },
    Second { stream: B },
}


impl<A, B, F> ChainStream<A, B, F> {
    pub fn chain<X, I, V>(first: X, f: F) -> Self 
    where
        A: IndexedStream<I = I, V = V>,
        X: IntoStreamIterator<IndexType = I, ValueType = V, StreamType = A>,
        B: IndexedStream<I = I, V = V>,
        F: FnOnce(A) -> B,
    {
        let first_stream = first.into_stream_iterator();
        if first_stream.valid() {
            ChainStream::First { stream: first_stream, f }
        } else {
            let second_stream = (f)(first_stream);
            ChainStream::Second { stream: second_stream }
        }
    }
}

impl<I, A, B, V, F> IndexedStream for ChainStream<A, B, F>
where
    I: Ord + Copy,
    A: IndexedStream<I = I, V = V>,
    B: IndexedStream<I = I, V = V>,
    F: FnOnce(A) -> B,
{
    
    type I = I;
    type V = V;
    
    fn valid(&self) -> bool {
        match &self {
            ChainStream::First { .. } => true,
            ChainStream::Second { stream: b } => b.valid(),
        }
    }
    
    fn ready(&self) -> bool {
        match &self {
            ChainStream::First { stream: a, .. } => a.ready(),
            ChainStream::Second { stream: b } => b.ready(),
        }
    }
    
    fn seek(&mut self, index: Self::I, strict: bool) {
        replace_with_or_abort(self, |self_| {
            match self_ {
                ChainStream::First { stream: mut a, f } => {
                    let old_index = a.index();
                    a.seek(index, strict);
                    if !a.valid() {
                        let b = f(a);
                        debug_assert!(!b.valid() || old_index <= b.index());
                        ChainStream::Second { stream: b }
                    } else {
                        ChainStream::First { stream: a, f }
                    }
                },
                ChainStream::Second { stream: mut b } => {
                    b.seek(index, strict);
                    ChainStream::Second { stream: b }
                }
            }
        });
    }

    fn next(&mut self) {
        replace_with_or_abort(self, |self_| {
            match self_ {
                ChainStream::First { stream: mut a, f } => {
                    let old_index = a.index();
                    a.next();
                    if !a.valid() {
                        let b = f(a);
                        debug_assert!(!b.valid() || old_index <= b.index());
                        ChainStream::Second { stream: b }
                    } else {
                        ChainStream::First { stream: a, f }
                    }
                },
                ChainStream::Second { stream: mut b } => {
                    b.next();
                    ChainStream::Second { stream: b }
                }
            }
        });
    }
    
    fn index(&self) -> Self::I {
        match &self {
            ChainStream::First { stream: a, .. } => a.index(),
            ChainStream::Second { stream: b } => b.index(),
        }
    }
    
    fn value(&self) -> Self::V {
        match &self {
            ChainStream::First { stream: a, .. } => a.value(),
            ChainStream::Second { stream: b } => b.value(),
        }
    }

    fn try_fold<BB, FF, R>(&mut self, init: BB, mut f: FF) -> ControlFlow<R, BB> where
            FF: FnMut(BB, Self::I, Self::V) -> ControlFlow<R, BB> {
        replace_with_or_abort_and_return(self, |self_| {
            match self_ {
                ChainStream::First { stream: mut a, f: next } => {
                    match a.try_fold(init, &mut f) {
                        ControlFlow::Continue(acc) => {
                            let mut b = next(a);
                            let result = b.try_fold(acc, f);
                            (result, ChainStream::Second { stream: b })
                        },
                        ControlFlow::Break(acc) => {
                            // if the first stream returned 'break' on the last iteration
                            // it may be invalid despite acc being 'break' so we must check this case
                            if a.valid() {
                                (ControlFlow::Break(acc), ChainStream::First { stream: a, f: next })
                            } else {
                                let b = next(a);
                                (ControlFlow::Break(acc), ChainStream::Second { stream: b })
                            }
                        },
                    }
                },
                ChainStream::Second { stream:mut b } => {
                    let result = b.try_fold(init, f);
                    (result, ChainStream::Second { stream: b })
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct FixedChainStream<A, B> {
    first: A,
    second: B,
}

impl<A, B> FixedChainStream<A, B> {
    pub fn new(first: A, second: B) -> Self {
        FixedChainStream { first, second }
    }
}

impl<A, B> IndexedStream for FixedChainStream<A, B>
where
    A: IndexedStream,
    B: IndexedStream<I = A::I, V = A::V>,
    A::I: Ord + Copy,
{
    type I = A::I;
    type V = A::V;
    
    fn valid(&self) -> bool {
        self.first.valid() || self.second.valid()
    }
    
    fn ready(&self) -> bool {
        if self.first.valid() {
            self.first.ready()
        } else {
            self.second.ready()
        }
    }
    
    fn seek(&mut self, index: Self::I, strict: bool) {
        if self.first.valid() {
            let old_index = self.first.index();
            self.first.seek(index, strict);
            debug_assert!(self.first.valid() || !self.second.valid() || old_index <= self.second.index());
        } else {
            self.second.seek(index, strict);
        }
    }

    fn next(&mut self) {
        if self.first.valid() {
            let old_index = self.first.index();
            self.first.next();
            debug_assert!(self.first.valid() || !self.second.valid() || old_index <= self.second.index());
        } else {
            self.second.next();
        }
    }
    
    fn index(&self) -> Self::I {
        if self.first.valid() {
            self.first.index()
        } else {
            self.second.index()
        }
    }
    
    fn value(&self) -> Self::V {
        if self.first.valid() {
            self.first.value()
        } else {
            self.second.value()
        }
    }

    fn try_fold<BB, F, R>(&mut self, init: BB, mut f: F) -> ControlFlow<R, BB> where
            F: FnMut(BB, Self::I, Self::V) -> ControlFlow<R, BB> {
        let acc = self.first.try_fold(init, &mut f)?;
        self.second.try_fold(acc, &mut f)
    }
}

#[cfg(test)]

mod chain_test {
    use crate::streams::{chain::ChainStream, sorted_vec::SortedVecGalloper, stream_defs::IndexedStream};

    use super::FixedChainStream;


    #[test]
    fn basic_chain_test() {
        let stream = FixedChainStream::new(
            SortedVecGalloper::new(&[1, 2, 3, 4, 5]),
            SortedVecGalloper::new(&[6, 7, 8, 9, 10]),
        );
        assert_eq!(stream.clone().collect_indices(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn chain_seek_test() {
        let mut stream = FixedChainStream::new(
            SortedVecGalloper::new(&[1, 2, 3, 4, 5]),
            SortedVecGalloper::new(&[6, 7, 8, 9, 10]),
        );
        stream.seek(3, false);
        assert_eq!(stream.index(), 3);
        stream.seek(3, true);
        assert_eq!(stream.index(), 4);
        stream.seek(5, true);
        assert_eq!(stream.index(), 6);
        stream.seek(6, false);
        assert_eq!(stream.index(), 6);
        stream.seek(4, false);
        assert_eq!(stream.index(), 6);
    }


    
    #[test]
    fn basic_and_then_chain_test() {
        let stream = ChainStream::chain(
            SortedVecGalloper::new(&[1, 2, 3, 4, 5]),
            |_| SortedVecGalloper::new(&[6, 7, 8, 9, 10]),
        );
        assert_eq!(stream.clone().collect_indices(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn and_then_chain_seek_test() {
        let mut stream = ChainStream::chain(
            SortedVecGalloper::new(&[1, 2, 3, 4, 5]),
            |_| SortedVecGalloper::new(&[6, 7, 8, 9, 10]),
        );
        stream.seek(3, false);
        assert_eq!(stream.index(), 3);
        stream.seek(3, true);
        assert_eq!(stream.index(), 4);
        stream.seek(5, true);
        assert_eq!(stream.index(), 6);
        stream.seek(6, false);
        assert_eq!(stream.index(), 6);
        stream.seek(4, false);
        assert_eq!(stream.index(), 6);
    }
}
