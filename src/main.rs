use crate::streams::sparse_vec::SparseVec;

use crate::streams::{stream_defs::FromStreamIterator, mul_stream::MulStream};

mod streams;

fn main() {
    let v1 = SparseVec::from([(1, 4), (20, 2), (33, 3)]);
    let v2 = SparseVec::from([(1, 7), (2, -3), (5, 10), (33, 9)]);
    let prod = MulStream::mul(&v1, v2.gallop());
    let mut result = Vec::with_capacity(v1.len().min(v2.len()));
    result.extend_from_stream_iterator(prod);
    assert_eq!(result, vec![(1, 28), (33, 27)]);

    let v3 = SparseVec::from([(1, -2), (2, 3), (5, 7), (29, 4)]);
    let prod2 = MulStream::mul(MulStream::mul(&v1, &v2), &v3);
    let mut result2 = Vec::with_capacity(v1.len().min(v2.len()).min(v3.len()));
    result2.extend_from_stream_iterator(prod2);
    assert_eq!(result2, vec![(1, -56)]);

    let v4 = SparseVec::from([(3, -2), (10, 5), (20, 5), (33, 7)]);
    let prod3 = MulStream::mul(&v4, &v1);
    let mut result3 = SparseVec::with_capacity(v4.len().min(v1.len()));
    result3.extend_from_stream_iterator(prod3);
    assert_eq!(result3, SparseVec::from([(20, 10), (33, 21)]));
}
