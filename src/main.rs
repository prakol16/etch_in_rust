use streams::{csr_mat::SparseCSRMat, mul_stream::StreamMul, stream_defs::IntoStreamIterator};

mod streams;

fn main() {
    let mat1 = SparseCSRMat::from_iter([(0, 0, 1), (0, 20, 2), (0, 30, 100), (1, 0, 3), (1, 20, 4)]);
    let mat2 = SparseCSRMat::from_iter([(0, 0, 5), (0, 20, 6), (1, 0, 7), (1, 20, 8), (2, 40, -1)]);
    let prod4 = mat1.into_stream_iterator().mul(mat2.into_stream_iterator()); //.mul(Expand::new(v3.into_stream_iterator()));
    // let sum2 = prod4.map(|_i, v| v.contract());
    // let result5: Vec<i32> = DenseStreamIterator::from_stream_iterator(sum2).into_iter().collect();
}
