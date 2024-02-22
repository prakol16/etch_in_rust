use std::ops::Mul;

use super::stream_defs::{IntoStreamIterator, StreamIterator};


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
    ($t:ty) => {
        impl StreamMul for $t {
            type Output = $t;

            fn mul(self, rhs: $t) -> Self::Output {
                self * rhs
            }
        }
    };
}

impl_stream_mul!(i8);
impl_stream_mul!(i16);
impl_stream_mul!(i32);
impl_stream_mul!(i64);
impl_stream_mul!(i128);
impl_stream_mul!(isize);
impl_stream_mul!(u8);
impl_stream_mul!(u16);
impl_stream_mul!(u32);
impl_stream_mul!(u64);
impl_stream_mul!(u128);
impl_stream_mul!(usize);
impl_stream_mul!(f32);
impl_stream_mul!(f64);

impl<I, L, R> StreamIterator for MulStream<L, R> 
    where L: StreamIterator<I=I>,
          R: StreamIterator<I=I>,
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

    fn skip(&mut self, index: &I, strict: bool) {
        self.left.skip(index, strict);
        self.right.skip(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().max(self.right.index())
    }

    fn value(&self) -> Self::V {
        self.left.value().mul(self.right.value())
    }
}
