use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use etch::{examples::{sorted_vec_intersect::{vec_intersect_manual, vec_intersect_streams_gallop, vec_intersect_streams_linear}, triangle_query::{all_combinations, create_all_pairs_table, create_skewed_relation, triangle_query_fused, triangle_query_unfused}}, streams::{sorted_vec::SortedVecGalloper, stream_defs::IndexedStream}};
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};

fn gen_random_sorted_strings(n: usize, sparsity: usize) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(1024);
    let mut numbers: Vec<usize> = (0..sparsity*n).collect();
    numbers.shuffle(&mut rng);
    numbers.truncate(n);
    let mut strings: Vec<String> = numbers.iter().map(|&num| num.to_string()).collect();
    strings.sort_unstable();
    strings
}

fn gen_random_sorted_ints(n: usize, sparsity: u32) -> Vec<u32> {
    let mut rng = StdRng::seed_from_u64(420);
    let mut numbers: Vec<u32> = (0..sparsity*n as u32).collect();
    numbers.shuffle(&mut rng);
    numbers.truncate(n);
    numbers.sort_unstable();
    numbers
}

fn triangle_query_benchmark(c: &mut Criterion) {
    let n = 500;
    let s1 = gen_random_sorted_strings(n, 2);
    let s2 = gen_random_sorted_strings(n, 2);
    let s3 = gen_random_sorted_strings(n, 2);
    let s1_ref = s1.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let s2_ref = s2.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let s3_ref = s3.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    let r1 = create_skewed_relation(&s1_ref, &s2_ref, 10);
    let r2 = create_skewed_relation(&s2_ref, &s3_ref, 10);
    let r3 = create_skewed_relation(&s1_ref, &s3_ref, 10);

    let mut group = c.benchmark_group("tri");
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("tri.fused", |b| 
        b.iter(|| black_box(triangle_query_fused(
            r1.stream_iter().map(|_, x| SortedVecGalloper::new(x)),
            r2.stream_iter().map(|_, x| SortedVecGalloper::new(x)),
            r3.stream_iter().map(|_, x| SortedVecGalloper::new(x))
        ))));
    group.bench_function("tri.unfused", |b| {
        b.iter(|| black_box(triangle_query_unfused(
            r1.stream_iter().map(|_, x| SortedVecGalloper::new(x)),
            r2.stream_iter().map(|_, x| SortedVecGalloper::new(x)),
            r3.stream_iter().map(|_, x| SortedVecGalloper::new(x))
        )));
    });
}

fn sorted_vec_sparse_intersect_benchmark(c: &mut Criterion) {
    let s1 = gen_random_sorted_ints(100, 10_000);
    let s2 = gen_random_sorted_ints(100_000, 10);

    let mut group = c.benchmark_group("inter.sparse");
    group.bench_function("inter.sparse.indexed_streams", |b|
        b.iter(|| black_box(vec_intersect_streams_gallop(&s1, &s2)))
    );
    group.bench_function("inter.sparse.indexed_streams_linear", |b|
        b.iter(|| black_box(vec_intersect_streams_linear(&s1, &s2)))
    );
    group.bench_function("inter.sparse.manual", |b|
        b.iter(|| black_box(vec_intersect_manual(&s1, &s2)))
    );
}

fn sorted_vec_dense_intersect_benchmark(c: &mut Criterion) {
    let s1 = gen_random_sorted_ints(1_000_000, 10);
    let s2 = gen_random_sorted_ints(1_000_000, 10);

    let mut group = c.benchmark_group("inter.dense");
    group.bench_function("inter.dense.indexed_streams", |b|
        b.iter(|| black_box(vec_intersect_streams_gallop(&s1, &s2)))
    );
    group.bench_function("inter.dense.indexed_streams_linear", |b|
        b.iter(|| black_box(vec_intersect_streams_linear(&s1, &s2)))
    );
    group.bench_function("inter.dense.manual", |b|
        b.iter(|| black_box(vec_intersect_manual(&s1, &s2)))
    );
}

criterion_group!(benches, triangle_query_benchmark, sorted_vec_sparse_intersect_benchmark, sorted_vec_dense_intersect_benchmark);
criterion_main!(benches);
