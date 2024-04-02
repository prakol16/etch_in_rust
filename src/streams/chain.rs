use std::ops::ControlFlow;

use replace_with::{replace_with_or_abort, replace_with_or_abort_and_return};

use super::stream_defs::{IndexedStream, IntoStreamIterator};


/// A stream that chains two streams together, using the second stream when the first stream is exhausted.
/// Invariant: if the variant is `First`, then the first stream is valid i.e.,
/// after calling `next` or `seek` on the first stream, we must check if it is valid.
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
                    a.next();
                    if !a.valid() {
                        let b = f(a);
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
                ChainStream::First { mut stream, f: next } => {
                    match stream.try_fold(init, &mut f) {
                        ControlFlow::Continue(acc) => {
                            let mut b = next(stream);
                            let result = b.try_fold(acc, f);
                            (result, ChainStream::Second { stream: b })
                        },
                        ControlFlow::Break(acc) => {
                            // if the first stream returned 'break' on the last iteration
                            // it may be invalid despite acc being 'break' so we must check this case
                            if stream.valid() {
                                (ControlFlow::Break(acc), ChainStream::First { stream, f: next })
                            } else {
                                let b = next(stream);
                                (ControlFlow::Break(acc), ChainStream::Second { stream: b })
                            }
                        },
                    }
                },
                ChainStream::Second { mut stream } => {
                    let result = stream.try_fold(init, f);
                    (result, ChainStream::Second { stream })
                }
            }
        })
    }
}

