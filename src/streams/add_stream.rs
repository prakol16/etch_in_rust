use std::ops::Add;

use super::stream_defs::{IntoStreamIterator, StreamIterator};


pub struct AddStream<L, R> {
    left: L,
    right: R,
}

impl<L, R> AddStream<L, R> {
    #[allow(dead_code)]
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

impl<I, L, R> StreamIterator for AddStream<L, R> 
    where L: StreamIterator<I=I>,
          R: StreamIterator<I=I>,
          I: Ord,
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

    fn seek(&mut self, index: &I, strict: bool) {
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
