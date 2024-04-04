use super::stream_defs::{IndexedStream, IntoStreamIterator, StreamResult};

// impl<I, L, R> IndexedStream for AddStream<L, R> 
//     where L: IndexedStream<I=I>,
//           R: IndexedStream<I=I>,
//           I: Ord + Copy,
//           L::V: Add<R::V>, {
//     type I = I;
//     type V = <L::V as Add<R::V>>::Output;

//     fn valid(&self) -> bool {
//         self.left.valid() || self.right.valid()
//     }

//     fn ready(&self) -> bool {
//         if self.left.valid() {
//             if self.right.valid() {
//                 self.left.index() == self.right.index()
//             } else {
//                 self.left.ready()
//             }
//         } else if self.right.valid() {
//             self.right.ready()
//         } else {
//             panic!("AddStream::ready called when neither stream is valid")
//         }
//     }

//     fn seek(&mut self, index: I, strict: bool) {
//         self.left.seek(index, strict);
//         self.right.seek(index, strict);
//     }

//     fn index(&self) -> I {
//         self.left.index().min(self.right.index())
//     }

//     fn value(&self) -> Self::V {
//         self.left.value() + self.right.value()
//     }
// }

#[derive(Debug, Clone)]
pub enum EitherOrBoth<A, B> {
    Left(A),
    Right(B),
    Both(A, B),
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

    fn seek(&mut self, index: I, strict: bool) {
        self.left.seek(index, strict);
        self.right.seek(index, strict);
    }

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        match (self.left.current(), self.right.current()) {
            (StreamResult::Yield { index: li, value: lv },
             StreamResult::Yield { index: ri, value: rv }) => {
                StreamResult::Yield {
                    index: std::cmp::min(li, ri),
                    value: (|| {
                        match li.cmp(&ri) {
                            std::cmp::Ordering::Less => Some((self.f)(EitherOrBoth::Left(lv?))),
                            std::cmp::Ordering::Equal => Some((self.f)(EitherOrBoth::Both(lv?, rv?))),
                            std::cmp::Ordering::Greater => Some((self.f)(EitherOrBoth::Right(rv?))),
                        }
                    })()
                }
            },
            _ => StreamResult::Done
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

