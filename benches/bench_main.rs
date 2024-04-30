use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use etch::{examples::{sorted_vec_intersect::{vec_intersect_manual, vec_intersect_streams_gallop, vec_intersect_streams_linear}, tree_iteration::{intersect2_iterators, intersect2_manual, intersect3_iterators, itersect3_manual}, triangle_query::{create_skewed_relation, triangle_query_fused, triangle_query_naive, triangle_query_unfused}}, streams::{sorted_vec::SortedVecGalloper, stream_defs::IndexedStream}};
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};

fn gen_random_sorted_strings(n: usize, sparsity: usize, seed: u64) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(1024 + seed);
    let mut numbers: Vec<usize> = (0..sparsity*n).collect();
    numbers.shuffle(&mut rng);
    numbers.truncate(n);
    let mut strings: Vec<String> = numbers.iter().map(|&num| num.to_string()).collect();
    strings.sort_unstable();
    strings
}

fn gen_random_sorted_ints(n: usize, sparsity: u32, seed: u64) -> Vec<u32> {
    let mut rng = StdRng::seed_from_u64(420 + seed);
    let mut numbers: Vec<u32> = (0..sparsity*n as u32).collect();
    numbers.shuffle(&mut rng);
    numbers.truncate(n);
    numbers.sort_unstable();
    numbers
}

fn triangle_query_benchmark(c: &mut Criterion) {
    let n = 1000;
    let s1 = gen_random_sorted_strings(n, 2, 0);
    let s2 = gen_random_sorted_strings(n, 2, 1);
    let s3 = gen_random_sorted_strings(n, 2, 2);
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
    group.bench_function("tri.naive", |b| {
        b.iter(|| black_box(triangle_query_naive(
            &r1,
            &r2,
            &r3
        )));
    });
}

fn sorted_vec_sparse_intersect_benchmark(c: &mut Criterion) {
    let s1 = gen_random_sorted_ints(100, 10_000, 0);
    let s2 = gen_random_sorted_ints(100_000, 10, 1);

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
    let s1 = gen_random_sorted_ints(1_000_000, 10, 0);
    let s2 = gen_random_sorted_ints(1_000_000, 10, 1);

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

fn rbtree_intersect_benchmark(c: &mut Criterion) {
    let tree_a = gen_random_sorted_ints(1_000_000, 10, 0).into_iter().map(|x| (x, ())).collect();
    let tree_b = gen_random_sorted_ints(1_000_000, 10, 1).into_iter().map(|x| (x, ())).collect();
    let tree_c = gen_random_sorted_ints(1_000_000, 10, 2).into_iter().map(|x| (x, ())).collect();

    let mut group = c.benchmark_group("rbtree");
    group.bench_function("rbtree.intersect2_iterators", |b| {
        b.iter(|| black_box(intersect2_iterators(
            &tree_a,
            &tree_b
        )))
    });
    group.bench_function("rbtree.intersect2_manual", |b| {
        b.iter(|| black_box(intersect2_manual(
            &tree_a,
            &tree_b
        )))
    });
    group.bench_function("rbtree.intersect3_iterators", |b| {
        b.iter(|| black_box(intersect3_iterators(
            &tree_a,
            &tree_b,
            &tree_c
        )))
    });
    group.bench_function("rbtree.intersect3_manual", |b| {
        b.iter(|| black_box(itersect3_manual(
            &tree_a,
            &tree_b,
            &tree_c
        )))
    });
    group.finish();
}

criterion_group!(benches, triangle_query_benchmark, sorted_vec_sparse_intersect_benchmark, sorted_vec_dense_intersect_benchmark, rbtree_intersect_benchmark);
criterion_main!(benches);
