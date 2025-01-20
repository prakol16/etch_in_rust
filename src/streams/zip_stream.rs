use std::borrow::Borrow;

use super::stream_defs::{IndexedStream, IntoStreamIterator, StreamResult};


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

    fn seek(&mut self, index: impl Borrow<I>, strict: bool) {
        self.left.seek(index.borrow(), strict);
        self.right.seek(index.borrow(), strict);
    }

    fn current(&self) -> StreamResult<Self::I, Self::V> {
        match (self.left.current(), self.right.current()) {
            (StreamResult::Yield { index: li, value: lv }, StreamResult::Yield { index: ri, value: rv }) => {
                if li < ri {
                    StreamResult::Yield { index: ri, value: None  }
                } else if ri < li {
                    StreamResult::Yield { index: li, value: None }
                } else {
                    StreamResult::Yield { index: li, value: (|| Some((self.f)(lv?, rv?)))() }
                }
            }
            _ => StreamResult::Done
        }
    }
}
