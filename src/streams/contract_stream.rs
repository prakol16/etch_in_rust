use super::stream_defs::{IntoStreamIterator, StreamIterator};

pub struct ContractStream<S> {
    stream: S
}

impl<S> ContractStream<S> {
    pub fn contract(s: impl IntoStreamIterator<StreamType = S>) -> Self {
        ContractStream { stream: s.into_stream_iterator() }
    }
}

impl<S> StreamIterator for ContractStream<S> where S: StreamIterator {
    type I = ();

    type V = S::V;

    fn valid(&self) -> bool {
        self.stream.valid()
    }

    fn ready(&self) -> bool {
        self.stream.ready()
    }

    fn skip(&mut self, _index: &Self::I, strict: bool) {
        self.stream.skip(&self.stream.index(), strict)
    }

    fn index(&self) -> Self::I {
        return;
    }

    fn value(&self) -> Self::V {
        self.stream.value()
    }
}

