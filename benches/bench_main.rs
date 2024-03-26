use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use etch::examples::triangle_query::{all_combinations, create_all_pairs_table, triangle_query_fused};
use rand::prelude::SliceRandom;

fn gen_random_sorted_strings(n: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut numbers: Vec<u32> = (1..=2*n as u32).collect();
    numbers.shuffle(&mut rng);
    numbers.truncate(n);
    let mut strings: Vec<String> = numbers.iter().map(|&num| num.to_string()).collect();
    strings.sort_unstable();
    strings
}

fn criterion_benchmark(c: &mut Criterion) {
    let n = 100;
    let s1 = gen_random_sorted_strings(n);
    let s2 = gen_random_sorted_strings(n);
    let s3 = gen_random_sorted_strings(n);
    let s1_ref = s1.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let s2_ref = s2.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let s3_ref = s3.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    let mut group = c.benchmark_group("tri");
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("tri.fused", |b| 
        b.iter(|| black_box(triangle_query_fused(
            create_all_pairs_table(&s1_ref, &s2_ref), 
            create_all_pairs_table(&s2_ref, &s3_ref),
            create_all_pairs_table(&s1_ref, &s3_ref)
        ))));
    group.bench_function("tri.optimal", |b|
        b.iter(|| black_box(all_combinations(&s1_ref, &s2_ref, &s3_ref))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
