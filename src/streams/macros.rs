

#[macro_export]
macro_rules! indexed_stream {
    ($I:ty, $V:ty) => {
        impl $crate::streams::stream_defs::stream_defs::IndexedStream<I=$I, V=$V>
    };
    ($I:ty, $($rest:ty),+) => {
        indexed_stream!($I, indexed_stream!($($rest),+))
    };
}

#[macro_export]
macro_rules! cloneable_indexed_stream {
    ($I:ty, $V:ty) => {
        impl $crate::streams::stream_defs::CloneableIndexedStream<I=$I, V=$V>
    };
    ($I:ty, $($rest:ty),+) => {
        cloneable_indexed_stream!($I, cloneable_indexed_stream!($($rest),+))
    };
}

