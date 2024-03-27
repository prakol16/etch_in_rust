

#[macro_export]
macro_rules! indexed_stream {
    ($I:ty, $V:ty ; $($Trait:tt),*) => {
        impl $crate::streams::stream_defs::IndexedStream<I=$I, V=$V> $(+ $Trait)*
    };
    ($I:ty, $($rest:ty),+ ; $($Trait:tt),*) => {
        indexed_stream!($I, indexed_stream!($($rest),+ ; $($Trait),*) ; $($Trait),*)
    };
}
