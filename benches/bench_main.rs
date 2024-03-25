use criterion::{black_box, criterion_group, criterion_main, Criterion};
use etch::streams::{sorted_vec::SortedVecGalloper, stream_defs::IndexedStream};

/// Perform the triangle query on s1, s2, s3
/// Assumes s1, s2, s3 are sorted
fn triangle_query<A: Ord + Clone, B: Ord + Clone, C: Ord + Clone>(s1: &[A], s2: &[B], s3: &[C]) {
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
    
    

}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tri.fused abc", |b| b.iter(|| 
        triangle_query(&["a"], &["b"], &["c"])));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
