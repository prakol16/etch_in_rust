use crate::streams::{
    sorted_vec::SortedVecGalloper,
    sparse_vec::SparseVec,
    stream_defs::{collect_indices, IndexedStream},
};

fn join_1<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    x: impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = ()>>,
    y: impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>> + Clone,
) -> impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>>
{
    x.map(move |_, a| a.zip_with(y.clone(), |_, b| b))
}

fn join_2<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    x: impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>>,
    y: impl IndexedStream<I = A, V = impl IndexedStream<I = C, V = ()> + Clone>,
) -> impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>>
{
    x.zip_with(y, |a, b| {
        a.map(move |_, c| b.clone().zip_with(c, |_, _| ()))
    })
}

/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
pub fn triangle_query_fused<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    s1: &[A],
    s2: &[B],
    s3: &[C],
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let t1 = SortedVecGalloper::new(s1).map(|_, _| SortedVecGalloper::new(s2));
    let t2 = SortedVecGalloper::new(s2).map(|_, _| SortedVecGalloper::new(s3));
    let t3 = SortedVecGalloper::new(s1).map(|_, _| SortedVecGalloper::new(s3));
    let result = join_2(join_1(t1, t2), t3);

    result
        .map(|_, a| {
            a.map(|_, b| collect_indices(b))
                .collect::<SparseVec<B, Vec<C>>>()
        })
        .collect()
}

pub fn all_combinations<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    s1: &[A],
    s2: &[B],
    s3: &[C],
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let tmp = SparseVec {
        inds: s2.to_vec(),
        vals: (0..s2.len()).map(|_| s3.to_vec()).collect(),
    };
    SparseVec {
        inds: s1.to_vec(),
        vals: (0..s1.len()).map(|_| tmp.clone()).collect(),
    }
}

#[test]
fn test_triangle_query() {
    let s1 = ["a", "b", "c", "d"];
    let s2 = ["e", "f", "g", "h"];
    let s3 = ["i", "j", "k", "l"];
    let result = triangle_query_fused(
        &s1,
        &s2,
        &s3,
    );
    assert_eq!(
        result,
        all_combinations(&s1, &s2, &s3)
    );
}
