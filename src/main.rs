use streams::stream_defs::{SparseVec, MulStream};

use crate::streams::stream_defs::FromStreamIterator;

mod streams;

fn main() {
    let v1 = SparseVec::from([(1, 4), (20, 2), (33, 3)]);
    let v2 = SparseVec::from([(1, 7), (2, -3), (5, 10), (33, 9)]);
    let prod = MulStream::mul(&v1, v2.gallop());
    let mut result = Vec::with_capacity(v1.len().min(v2.len()));
    result.extend_from_stream_iterator(prod);
    println!("{:?}", result);

    let v3 = SparseVec::from([(1, -2), (2, 3), (5, 7), (29, 4)]);
    let prod2 = MulStream::mul(MulStream::mul(&v1, &v2), &v3);
    let mut result2 = Vec::with_capacity(v1.len().min(v2.len()).min(v3.len()));
    result2.extend_from_stream_iterator(prod2);
    println!("{:?}", result2);
}
