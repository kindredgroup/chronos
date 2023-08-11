#![allow(unused)]
use chronos_bin::runner::Runner;
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// // use hello_cargo::fibonacci;
//
// pub fn criterion_benchmark(c: &mut Criterion) {
//
//     c.bench_function("Bench Chronos", |b| b.iter(|| Runner::run()));
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);

use criterion::*;

async fn my_function() {
    let runner = Runner {};
    runner.run().await;
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    group.significance_level(0.1).sample_size(500);
    group.bench_function("my-function", |b| b.iter(|| my_function()));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
