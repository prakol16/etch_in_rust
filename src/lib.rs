pub mod streams;
pub mod examples;

#[cfg(test)]
mod test {
    use crate::streams::csr_mat::SparseCSRMat;
    use crate::streams::sorted_vec::SortedVecGalloper;
    use crate::streams::sparse_vec::SparseVec;
    
    use crate::streams::stream_defs::DenseStreamIterator;
    use crate::streams::stream_defs::IntoStreamIterator;
    use crate::streams::stream_defs::FromStreamIterator;
    use crate::streams::stream_defs::IndexedStream;
    
    fn mul(i: i32, j: i32) -> i32 {
        i * j
    }

    #[test]
    fn test_gallop() {
        let v1 = SparseVec::from_iter([(1, 4), (20, 2), (33, 3)]);
        let v2 = SparseVec::from_iter([(1, 7), (2, -3), (5, 10), (33, 9)]);
        let prod = v1.stream_iter_linear().cloned().zip_with(v2.stream_iter().cloned(), mul);
        let mut result = Vec::with_capacity(v1.len().min(v2.len()));
        result.extend_from_stream_iterator(prod);
        assert_eq!(result, vec![(1, 28), (33, 27)]);
    }

    #[test]
    fn test_mul3() {
        let v1 = SparseVec::from_iter([(1, 4), (20, 2), (33, 3)]);
        let v2 = SparseVec::from_iter([(1, 7), (2, -3), (5, 10), (33, 9)]);
        let v3 = SparseVec::from_iter([(1, -2), (2, 3), (5, 7), (29, 4)]);
        let prod2 = v1.stream_iter_linear().cloned().zip_with(v2.stream_iter_linear().cloned(), mul).zip_with(v3.stream_iter_linear().cloned(), mul);
        let mut result2 = Vec::with_capacity(v1.len().min(v2.len()).min(v3.len()));
        result2.extend_from_stream_iterator(prod2);
        assert_eq!(result2, vec![(1, -56)]);
    }

    #[test]
    fn test_sparse_vec_from_iter() {
        let v1 = SparseVec::from_iter([(1, 4), (20, 2), (33, 3)]);
        let v4 = SparseVec::from_iter([(3, -2), (10, 5), (20, 5), (33, 7)]);
        let prod3 = v4.stream_iter().cloned().zip_with(v1.stream_iter().cloned(), mul);
        let mut result3 = SparseVec::with_capacity(v4.len().min(v1.len()));
        result3.extend_from_stream_iterator(prod3);
        assert_eq!(result3, SparseVec::from_iter([(20, 10), (33, 21)]));
    }

    #[test]
    fn test_contract() {
        let v1 = SparseVec::from_iter([(1, 4), (20, 2), (33, 3)]);
        let result4: i32 = v1.stream_iter_linear().cloned().contract();
        assert_eq!(result4, 9);
    }

    #[test]
    fn test_matvecmul() {
        let v3 = SparseVec::from_iter([(1, -2), (2, 3), (5, 7), (29, 4)]);
        let mat1 = SparseCSRMat::from_iter([(0, 0, 1), (0, 1, 2), (1, 5, 3), (1, 29, 4)]);
        let prod4 = mat1.into_stream_iterator()
            .map(|_, v| v.cloned().zip_with(v3.stream_iter().cloned(), mul));
        let sum2 = prod4.map(|_i, v| v.contract());
        let result5: Vec<i32> = DenseStreamIterator::from_stream_iterator(sum2).into_iter().collect();
        assert_eq!(result5, vec![-4, 37]);
    }


    #[test]
    fn sorted_vec_galloper() {
        let v1: Vec<i32> = vec![1, 2, 5, 10, 20, 33];
        let v2: Vec<i32> = vec![1, 2, 6, 8, 10, 25, 33];
        let intersection = SortedVecGalloper::new(&v1)
            .zip_with(SortedVecGalloper::new(&v2), |_, _| ());
        assert_eq!(intersection.collect_indices(), vec![1, 2, 10, 33]);
    }

    // fn nested_sparse_vec() {
    //     let nested_vec = SparseVec::from_iter(
    //         [(1, SparseVec::from_iter([(1, 2), (2, 3)]))]
    //     );
    // }
}
