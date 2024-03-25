use crate::streams::{sorted_vec::SortedVecGalloper, sparse_vec::SparseVec, stream_defs::{collect_indices, IndexedStream}};

/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
pub fn triangle_query<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(s1: &[A], s2: &[B], s3: &[C]) -> 
    SparseVec<A, SparseVec<B, Vec<C>>> {
    fn join_1<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
            x: impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = ()>>,
            y: impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>> + Clone) -> 
              impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>> {
        x
        .map(move |_, a| a.zip_with(y.clone(), |_, b| b))
    }

    fn join_2<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(
        x: impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>>,
        y: impl IndexedStream<I = A, V = impl IndexedStream<I = C, V = ()> + Clone>
    ) -> impl IndexedStream<I = A, V = impl IndexedStream<I = B, V = impl IndexedStream<I = C, V = ()>>> {
        x
        .zip_with(y, |a, b| a.map(move |_, c| b.clone().zip_with(c, |_, _| ())))            
    }

    let t1 = SortedVecGalloper::new(s1)
        .map(|_, _| SortedVecGalloper::new(s2));
    let t2 = SortedVecGalloper::new(s2)
        .map(|_, _| SortedVecGalloper::new(s3));
    let t3 = SortedVecGalloper::new(s1)
        .map(|_, _| SortedVecGalloper::new(s3));
    let result = join_2(join_1(t1, t2), t3);
    
    result.map(|_, a| 
        a.map(|_, b|
                collect_indices(b)
        ).collect::<SparseVec<B, Vec<C>>>()
    ).collect()
}

#[test]
fn test_triangle_query() {
    let result = triangle_query(
        &["a", "b", "c", "d"],
        &["e", "f", "g", "h"],
        &["i", "j", "k", "l"]
    );
    for (k1, v1) in result.into_iter() {
        for (k2, v2) in v1.into_iter() {
            println!("{} -> {} -> {:?}", k1, k2, v2);
        }
    }
}
