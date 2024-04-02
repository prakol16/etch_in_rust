use crate::streams::{sorted_vec::{SortedVecGalloper, SortedVecLinear}, stream_defs::IndexedStream};

/// Given two strictly sorted vectors, compute their intersection
pub fn vec_intersect_manual<I: Ord + Copy>(x: &Vec<I>, y: &Vec<I>) -> Vec<I> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < x.len() && j < y.len() {
        if x[i] < y[j] {
            i += 1;
        } else if x[i] > y[j] {
            j += 1;
        } else {
            result.push(x[i]);
            i += 1;
            j += 1;
        }
    }
    result
}

/// Given two strictly sorted vectors, compute their intersection
/// This uses galloping indexed streams
pub fn vec_intersect_streams_gallop<I: Ord + Copy>(x: &Vec<I>, y: &Vec<I>) -> Vec<I> {
     SortedVecGalloper::new(x)
        .zip_with(SortedVecGalloper::new(y), |_, _| ())
        .collect_indices()
}


/// Given two strictly sorted vectors, compute their intersection
/// This uses galloping indexed streams
pub fn vec_intersect_streams_linear<I: Ord + Copy>(x: &Vec<I>, y: &Vec<I>) -> Vec<I> {
     SortedVecLinear::new(x)
        .zip_with(SortedVecLinear::new(y), |_, _| ())
        .collect_indices()
}

#[test]
fn test_vec_intersect() {
    let x = vec![1, 2, 3, 4, 5];
    let y = vec![3, 4, 5, 6, 7];
    assert_eq!(vec_intersect_manual(&x, &y), vec![3, 4, 5]);
    assert_eq!(vec_intersect_streams_gallop(&x, &y), vec![3, 4, 5]);
    assert_eq!(vec_intersect_streams_linear(&x, &y), vec![3, 4, 5]);
}

