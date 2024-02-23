use std::marker::PhantomData;

/// A functional stream is essentially a closure that returns values given
/// an arbitrary index
pub trait FunStream {
    type I;
    type V;

    /// The method for the value of the stream at a given index
    fn value(&self, index: &Self::I) -> Self::V;
}

#[derive(Debug, Clone)]
pub struct Expand<I, V> {
    index: PhantomData<I>,
    pub value: V
}

impl<I, V> Expand<I, V> {
    pub fn new(value: V) -> Self {
        Expand { value, index: PhantomData }
    }
}

impl<I, V: Clone> FunStream for Expand<I, V> {
    type I = I;
    type V = V;

    fn value(&self, _index: &Self::I) -> Self::V {
        self.value.clone()
    }
}


