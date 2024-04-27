pub mod streams;
pub mod rbtree;
pub mod examples;

#[cfg(test)]
mod test {
    use std::iter;

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
    fn test_matvecmul2() {
        let mat = SparseCSRMat::from_iter([(1, 0, 1)]);
        let vec = SparseVec::from_iter([(0, 1)]);
        let prod = mat.into_stream_iterator()
            .map(|_, v| v.cloned().zip_with(vec.stream_iter().cloned(), mul).contract());
        let result: Vec<i32> = DenseStreamIterator::from_stream_iterator(prod).into_iter().collect();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_csrmat_empty() {
        let mat: SparseCSRMat<i32> = SparseCSRMat::from_iter(iter::empty());
        assert_eq!(mat, SparseCSRMat::empty());
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

#[cfg(test)]
mod proptests {
    use std::{collections::BTreeMap, num::Wrapping, ops::{Add, AddAssign, Mul}};

    use num_traits::Zero;
    use quickcheck_macros::quickcheck;

    use crate::streams::{add_stream::EitherOrBoth, csr_mat::SparseCSRMat, sparse_vec::SparseVec, stream_defs::{IndexedStream, IntoStreamIterator}};

    fn intersect_maps<I, V1, V2>(a: BTreeMap<I, V1>, b: BTreeMap<I, V2>) -> BTreeMap<I, (V1, V2)>
    where
        I: Ord + Clone + PartialEq,
        V1: Clone,
        V2: Clone
    {
        let mut result = BTreeMap::new();
        for (k, v) in a.iter() {
            if let Some(v2) = b.get(k) {
                result.insert(k.clone(), (v.clone(), v2.clone()));
            }
        }
        result
    }

    #[quickcheck]
    fn test_zip(a: BTreeMap<u8, usize>, b: BTreeMap<u8, usize>) {
        let vec_a = a.iter().map(|(k, v)| (*k, *v)).collect::<SparseVec<_, _>>();
        let vec_b = b.iter().map(|(k, v)| (*k, *v)).collect::<SparseVec<_, _>>();
        let zipped = vec_a.stream_iter()
            .zip_with(vec_b.stream_iter(), |a, b| (*a, *b));
        let result: SparseVec<u8, (usize, usize)> = zipped.collect();
        let expected = intersect_maps(a, b);
        assert_eq!(result, expected.iter().map(|(k, (v1, v2))| (*k, (*v1, *v2))).collect());
    }

    #[quickcheck]
    fn test_matvecmul(a: Vec<BTreeMap<u8, Wrapping<i64>>>, b: BTreeMap<u8, Wrapping<i64>>) {
        let csr_a = a.iter()
            .enumerate()
            .flat_map(|(i, row )| row.iter().map(move |(k, v)| (i, *k as usize, *v)))
            .collect::<SparseCSRMat<_>>();
        let vec_b = b.iter().map(|(k, v)| (*k as usize, *v)).collect::<SparseVec<_, _>>();
        let prod = csr_a.into_stream_iterator()
            .map(|_, v| 
                v.zip_with(vec_b.stream_iter(), |a, b| a * b)
                .contract()
            );
        let result: SparseVec<usize, _> = prod.collect::<SparseVec<usize, _>>();
        let expected: SparseVec<usize, _> = a.iter()
            .map(|row| 
                row.iter()
                    .map(|(k, v)| v * *b.get(k).unwrap_or(&Wrapping(0)))
                    .sum()
            )
            .enumerate()
            .collect();
        assert!(result.eq_ignoring_zeros(&expected), "result: {:?}, expected: {:?}", result, expected);
    }

    #[quickcheck]
    fn test_csr_roundtrip(a: Vec<BTreeMap<u8, i64>>) {
        let csr_a = a.iter()
            .enumerate()
            .flat_map(|(i, row )| row.iter().map(move |(k, v)| (i, *k as usize, *v)))
            .collect::<SparseCSRMat<_>>();
        let round_trip = csr_a.into_stream_iterator()
            .map(|_, s| s.cloned())
            .collect::<SparseCSRMat<_>>();
        assert_eq!(csr_a, round_trip);
    }

    fn sum_matmul_maps<I, J, V>(m1: Vec<BTreeMap<I, V>>, m2: Vec<BTreeMap<J, V>>) -> V
    where
        I: Ord + Clone,
        J: Ord + Clone,
        V: Add + Mul<V, Output = V> + Clone + AddAssign + Zero,
    {
        let mut result = V::zero();
        for (i, row) in m1.iter().enumerate() {
            for (_, v1) in row.iter() {
                if let Some(m2_row) = m2.get(i) {
                    for (_, v2) in m2_row.iter() {
                        result += v1.clone() * v2.clone();
                    }
                }
            }
        }
        result
    }

    #[quickcheck]
    fn test_matmul(a: Vec<BTreeMap<u8, Wrapping<i64>>>, b: Vec<BTreeMap<u8, Wrapping<i64>>>) {
        let csr_a = a.iter().enumerate()
            .flat_map(|(i, row)| row.iter().map(move |(k, v)| (i as usize, *k as usize, *v)))
            .collect::<SparseCSRMat<_>>();
        let csr_b = b.iter().enumerate()
            .flat_map(|(i, row)| row.iter().map(move |(k, v)| (i as usize, *k as usize, *v)))
            .collect::<SparseCSRMat<_>>();
        let result = csr_a.into_stream_iterator()
            .zip_with(csr_b.into_stream_iterator(), |r1, r2| {
            r1.map(move |_, v1| r2.clone().map(|_, v2| v1 * v2).contract()).contract()
        }).contract();
        let expected = sum_matmul_maps(a, b);
        assert_eq!(result, expected);
    }

    fn union_maps<I, V>(a: &BTreeMap<I, V>, b: &BTreeMap<I, V>) -> BTreeMap<I, EitherOrBoth<V, V>>
    where
        I: Ord + Clone,
        V: Clone
    {
        let mut result: BTreeMap<I, EitherOrBoth<V, V>> = a.iter().map(|(k, v)| (k.clone(), EitherOrBoth::Left(v.clone()))).collect();
        for (k, v) in b.iter() {
            match result.get_mut(k) {
                Some(EitherOrBoth::Left(left_val)) => {
                    *result.get_mut(k).unwrap() = EitherOrBoth::Both(left_val.clone(), v.clone());
                },
                _ => {
                    result.insert(k.clone(), EitherOrBoth::Right(v.clone()));
                }
            }
        }   
        result
    }

    #[quickcheck]
    fn test_union_vec(a: BTreeMap<u8, usize>, b: BTreeMap<u8, usize>) {
        let vec_a = a.iter().map(|(k, v)| (*k, *v)).collect::<SparseVec<_, _>>();
        let vec_b = b.iter().map(|(k, v)| (*k, *v)).collect::<SparseVec<_, _>>();
        let union_ab = 
            crate::streams::add_stream::union(vec_a.stream_iter(), vec_b.stream_iter(),
            |x| x.map(|v| *v, |v| *v))
            .collect::<SparseVec<_, _>>();
        let expected = union_maps(&a, &b)
            .into_iter().collect::<SparseVec<_, _>>();
        assert_eq!(union_ab, expected);
    }
}
