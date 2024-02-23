use crate::streams::csr_mat::SparseCSRMat;
use crate::streams::fun_stream::Expand;
use crate::streams::mul_stream::StreamMul;
use crate::streams::sparse_vec::SparseVec;

use crate::streams::stream_defs::IntoStreamIterator;
use crate::streams::stream_defs::FromStreamIterator;
use crate::streams::stream_defs::MappedStream;
use crate::streams::stream_defs::StreamIterator;

mod streams;

fn main() {
    let v1 = SparseVec::from_iter([(1, 4), (20, 2), (33, 3)]);
    let v2 = SparseVec::from_iter([(1, 7), (2, -3), (5, 10), (33, 9)]);
    let prod = StreamMul::mul(v1.into_stream_iterator(), v2.gallop());
    let mut result = Vec::with_capacity(v1.len().min(v2.len()));
    result.extend_from_stream_iterator(prod);
    assert_eq!(result, vec![(1, 28), (33, 27)]);

    let v3 = SparseVec::from_iter([(1, -2), (2, 3), (5, 7), (29, 4)]);
    let prod2 = StreamMul::mul(StreamMul::mul(v1.into_stream_iterator(), v2.into_stream_iterator()), v3.into_stream_iterator());
    let mut result2 = Vec::with_capacity(v1.len().min(v2.len()).min(v3.len()));
    result2.extend_from_stream_iterator(prod2);
    assert_eq!(result2, vec![(1, -56)]);

    let v4 = SparseVec::from_iter([(3, -2), (10, 5), (20, 5), (33, 7)]);
    let prod3 = StreamMul::mul(v4.into_stream_iterator(), v1.into_stream_iterator());
    let mut result3 = SparseVec::with_capacity(v4.len().min(v1.len()));
    result3.extend_from_stream_iterator(prod3);
    assert_eq!(result3, SparseVec::from_iter([(20, 10), (33, 21)]));

    let result4: i32 = v1.into_stream_iterator().contract();
    assert_eq!(result4, 9);

    let mat1 = SparseCSRMat::from_iter([(0, 0, 1), (0, 1, 2), (1, 5, 3), (1, 29, 4)]);
    let prod4 = StreamMul::mul(
        mat1.into_stream_iterator(),
        Expand::new(v3.into_stream_iterator())        
    );
    let sum2 = MappedStream::map(prod4, |_i, v| v.contract());
    let result5: SparseVec<i32> = SparseVec::from_stream_iterator(sum2);
    println!("Result5: {:?}", result5);
}
