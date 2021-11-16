use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fast_fp::{ff32, FF32};
use rand::{distributions::Standard, thread_rng, Rng};

fn min(c: &mut Criterion) {
    let mut group = c.benchmark_group("min");
    for count in [2, 8, 32, 1024] {
        group.throughput(Throughput::Elements(count as u64));

        let f32_vals = thread_rng()
            .sample_iter(Standard)
            .take(count)
            .collect::<Vec<f32>>();

        // use the same values for both benchmarks
        let ff32_vals = f32_vals
            .clone()
            .into_iter()
            .map(ff32)
            .collect::<Vec<FF32>>();

        group.bench_with_input(BenchmarkId::new("std::f32", count), &f32_vals, |b, vals| {
            b.iter(|| vals.iter().copied().fold(f32::MAX, |acc, val| acc.min(val)));
        });

        group.bench_with_input(BenchmarkId::new("FF32", count), &ff32_vals, |b, vals| {
            b.iter(|| {
                vals.iter()
                    .copied()
                    .fold(FF32::MAX, |acc, val| acc.min(val))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, min);
criterion_main!(benches);
