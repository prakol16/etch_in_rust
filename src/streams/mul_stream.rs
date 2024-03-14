use super::{fun_stream::{Expand, FunStream}, stream_defs::{IntoStreamIterator, IndexedIterator}};


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

/// A copy of Mul to be used by streams (and base case values)
pub trait StreamMul<Rhs = Self> {
    type Output;

    /// The method for the multiplication of two values
    fn mul(self, rhs: Rhs) -> Self::Output;
}

/// A macro that implements StreamMul for a type given a Mul instance
macro_rules! impl_stream_mul {
    ($($t:ty),*) => {
        $(
            impl StreamMul for $t {
                type Output = $t;

                fn mul(self, rhs: $t) -> Self::Output {
                    self * rhs
                }
            }
        )*
    };
}

impl_stream_mul!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

impl<L, R> StreamMul<R> for L
    where L: IndexedIterator,
          R: IndexedIterator<I=L::I>,
          L::V: StreamMul<R::V> {
    type Output = MulStream<L, R>;

    fn mul(self, rhs: R) -> Self::Output {
        MulStream::mul(self, rhs)
    }
}

/// A stream that multiplies an ordinary stream and a functional (expanded) stream
/// by simply multiplying the values the ordinary stream produces with the right-hand stream
pub struct MulFunStream<L, R> {
    left: L,
    right: R,
}

impl<L, R> IndexedIterator for MulFunStream<L, R>
    where L: IndexedIterator,
          R: FunStream<I=L::I>,
          L::V: StreamMul<R::V> {
    type I = L::I;
    type V = <L::V as StreamMul<R::V>>::Output;

    fn valid(&self) -> bool {
        self.left.valid()    
    }

    fn ready(&self) -> bool {
        self.left.ready()
    }

    fn seek(&mut self, index: &Self::I, strict: bool) {
        self.left.seek(index, strict);
    }

    fn index(&self) -> Self::I {
        self.left.index()
    }

    fn value(&self) -> Self::V {
        self.left.value().mul(self.right.value(&self.index()))
    }
}

impl<L, R> StreamMul<Expand<L::I, R>> for L
    where L: IndexedIterator,
          L::V: StreamMul<R> {
    type Output = MulFunStream<L, Expand<L::I, R>>;

    fn mul(self, rhs: Expand<L::I, R>) -> Self::Output {
        MulFunStream {
            left: self,
            right: rhs,
        }
    }
}

// impl<L, R> StreamMul<R> for L
//     where L: StreamIterator,
//           R: FunStream<I=L::I>,
//           L::V: StreamMul<R::V> {
//     type Output = impl StreamIterator;

//     fn mul(self, rhs: R) -> Self::Output {
//         MappedStream::map(self, |i, v| v.mul(rhs.value(&i)))
//     }
// }

impl<I, L, R> IndexedIterator for MulStream<L, R> 
    where L: IndexedIterator<I=I>,
          R: IndexedIterator<I=I>,
          I: Ord,
          L::V: StreamMul<R::V>, {
    type I = I;
    type V = <L::V as StreamMul<R::V>>::Output;

    fn valid(&self) -> bool {
        self.left.valid() && self.right.valid()
    }

    fn ready(&self) -> bool {
        self.left.ready() && self.right.ready() && self.left.index() == self.right.index()
    }

    fn seek(&mut self, index: &I, strict: bool) {
        self.left.seek(index, strict);
        self.right.seek(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().max(self.right.index())
    }

    fn value(&self) -> Self::V {
        self.left.value().mul(self.right.value())
    }
}
