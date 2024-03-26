use super::stream_defs::{IntoStreamIterator, IndexedStream};


#[derive(Debug, Clone)]
pub struct ZipStream<L, R, F> {
    left: L,
    right: R,
    f: F,
}

impl<L, R, F> ZipStream<L, R, F> {
    pub fn new(
        left: impl IntoStreamIterator<StreamType = L>,
        right: impl IntoStreamIterator<StreamType = R>,
        f: F,
    ) -> Self {
        ZipStream {
            left: left.into_stream_iterator(),
            right: right.into_stream_iterator(),
            f
        }
    }
}

impl<I, L, R, F, O> IndexedStream for ZipStream<L, R, F> 
    where L: IndexedStream<I=I>,
          R: IndexedStream<I=I>,
          I: Ord + Copy,
          F: Fn(L::V, R::V) -> O {
    type I = I;
    type V = O;

    fn valid(&self) -> bool {
        self.left.valid() && self.right.valid()
    }

    fn ready(&self) -> bool {
        self.left.ready() && self.right.ready() && self.left.index() == self.right.index()
    }

    fn seek(&mut self, index: I, strict: bool) {
        self.left.seek(index, strict);
        self.right.seek(index, strict);
    }

    fn index(&self) -> I {
        self.left.index().max(self.right.index())
    }

    fn value(&self) -> Self::V {
        (self.f)(self.left.value(), self.right.value())
    }
}
