use crate::{indexed_stream, streams::{
    sorted_vec::SortedVecGalloper, sparse_vec::SparseVec, stream_defs::IndexedStream
}};

fn join_1<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
    x: indexed_stream!(A, B, (); Clone),
    y: indexed_stream!(B, C, (); Clone),
) -> indexed_stream!(A, B, C, (); Clone)
{
    x.map(move |_, a| a.zip_with(y.clone(), |_, b| b))
}

fn join_2<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
    x: indexed_stream!(A, B, C, (); Clone),
    y: indexed_stream!(A, C, (); Clone),
) -> indexed_stream!(A, B, C, (); Clone)
{
    x.zip_with(y, |a, b| {
        a.map(move |_, c| b.clone().zip_with(c, |_, _| ()))
    })
}

pub fn create_all_pairs_table<'a, A: Ord + Copy, B: Ord + Copy>(
    s1: &'a [A],
    s2: &'a [B],
) -> indexed_stream!(A, B, (); Clone, 'a){
    SortedVecGalloper::new(s1).map(|_, _| SortedVecGalloper::new(s2))
}

/// Create a relation that maps the first `skew` elements of s1
/// to all elements of s2 and all other elements to the first `skew` elements of s2
pub fn create_skewed_relation<A: Ord + Clone, B: Ord + Clone>(
    s1: &[A],
    s2: &[B],
    skew: usize
) -> SparseVec<A, Vec<B>> {
    SparseVec {
        inds: s1.to_vec(),
        vals: (0..s1.len()).map(|i| {
            if i < skew {
                s2.to_vec()
            } else {
                s2[..skew].to_vec()
            }
        }).collect()
    }
}

pub fn triangle_query_naive<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
    r1: &SparseVec<A, Vec<B>>,
    r2: &SparseVec<B, Vec<C>>,
    r3: &SparseVec<A, Vec<C>>,
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let mut result = SparseVec::empty();
    for (a, bvec) in r1.iter() {
        let mut b_result: SparseVec<B, Vec<C>> = SparseVec::empty();
        for b in bvec.iter() {
            let mut c_result: Vec<C> = Vec::new();
            for c in r2.get(*b).unwrap_or(&vec![]).iter() {
                if let Some(cvec) = r3.get(*a) {
                    if cvec.binary_search(c).is_ok() {
                        c_result.push(*c);
                    }
                }
            }
            b_result.inds.push(*b);
            b_result.vals.push(c_result);
        }
        result.inds.push(*a);
        result.vals.push(b_result);
    }
    result
}

/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
/// This version is unfused
pub fn triangle_query_unfused<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
    t1: indexed_stream!(A, B, (); Clone),
    t2: indexed_stream!(B, C, (); Clone),
    t3: indexed_stream!(A, C, (); Clone)
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let tmp = join_1(t1, t2)
        .map(|_, a| a.map(|_, b| b.collect_indices())
            .collect::<SparseVec<B, Vec<C>>>())
        .collect::<SparseVec<A, SparseVec<B, Vec<C>>>>();
    let tmp_as_iter = tmp.stream_iter()
        .map(|_, x|
            x.stream_iter().map(|_, y|
                SortedVecGalloper::new(y)));
    let result = join_2(tmp_as_iter, t3);

    result
        .map(|_, a| {
            a.map(|_, b| b.collect_indices())
                .collect::<SparseVec<B, Vec<C>>>()
        })
        .collect()
}


/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
pub fn triangle_query_fused<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
    t1: indexed_stream!(A, B, (); Clone),
    t2: indexed_stream!(B, C, (); Clone),
    t3: indexed_stream!(A, C, (); Clone)
) -> SparseVec<A, SparseVec<B, Vec<C>>> {
    let result = join_2(join_1(t1, t2), t3);

    result
        .map(|_, a| {
            a.map(|_, b| b.collect_indices())
                .collect::<SparseVec<B, Vec<C>>>()
        })
        .collect()
}

pub fn all_combinations<A: Ord + Copy, B: Ord + Copy, C: Ord + Copy>(
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
    assert_eq!(
        result,
        triangle_query_unfused(
            create_all_pairs_table(&s1, &s2),
            create_all_pairs_table(&s2, &s3),
            create_all_pairs_table(&s1, &s3),
        )
    );
}
