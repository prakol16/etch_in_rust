use crate::{cloneable_indexed_stream, streams::{
    sorted_vec::SortedVecGalloper, sparse_vec::SparseVec, stream_defs::{collect_indices, IndexedStream}
}};

fn join_1<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    x: cloneable_indexed_stream!(A, B, ()),
    y: cloneable_indexed_stream!(B, C, ()),
) -> cloneable_indexed_stream!(A, B, C, ())
{
    x.map(move |_, a| a.zip_with(y.clone(), |_, b| b))
}

fn join_2<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    x: cloneable_indexed_stream!(A, B, C, ()),
    y: cloneable_indexed_stream!(A, C, ()),
) -> cloneable_indexed_stream!(A, B, C, ())
{
    x.zip_with(y, |a, b| {
        a.map(move |_, c| b.clone().zip_with(c, |_, _| ()))
    })
}

pub fn create_all_pairs_table<'a, A: Ord + Clone, B: Ord + Clone>(
    s1: &'a [A],
    s2: &'a [B],
) -> impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = ()> + 'a + Clone> + Clone {
    SortedVecGalloper::new(s1).map(|_, _| SortedVecGalloper::new(s2))
}

/*
/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
/// This version is unfused
pub fn triangle_query_unfused<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    t1: impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = ()>>,
    t2: impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>> + Clone,
    t3: impl IndexedStream<I = A, V = impl IndexedStream<I = C, V = ()> + Clone>
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let tmp = join_1(t1, t2)
        .map(|_, a| a.map(|_, b| collect_indices(b))
            .collect::<SparseVec<B, Vec<C>>>())
        .collect::<SparseVec<A, SparseVec<B, Vec<C>>>>();
    let tmp_as_iter = tmp.into_stream_iterator()
        .map(|_, x|
            x.into_stream_iterator().map(|_, y|
                SortedVecGalloper::new(&y)));
    let result = join_2(tmp_as_iter,t3);

    result
        .map(|_, a| {
            a.map(|_, b| collect_indices(b))
                .collect::<SparseVec<B, Vec<C>>>()
        })
        .collect()
}
*/


/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
pub fn triangle_query_fused<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
    t1: cloneable_indexed_stream!(A, B, ()),
    t2: cloneable_indexed_stream!(B, C, ()),
    t3: cloneable_indexed_stream!(A, C, ())
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
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
        create_all_pairs_table(&s1, &s2),
        create_all_pairs_table(&s2, &s3),
        create_all_pairs_table(&s1, &s3),
    );
    assert_eq!(
        result,
        all_combinations(&s1, &s2, &s3)
    );
}
