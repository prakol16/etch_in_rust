use criterion::{black_box, criterion_group, criterion_main, Criterion};


fn triangle_query<'a>(s1: &[&'a str], s2: &[&'a str], s3: &[&'a str]) {
    
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tri.fused abc", |b| b.iter(|| 
        triangle_query(&["a"], &["b"], &["c"])));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
