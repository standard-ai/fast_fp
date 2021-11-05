use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fast_fp::{ff32, ff64, FF32, FF64};
use rand::{distributions::Standard, thread_rng, Rng};

fn sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum");
    for count in [2, 4, 8, 16, 64, 1024, 1 << 15] {
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
            b.iter(|| vals.iter().copied().fold(0.0, |acc, val| acc + val));
        });

        group.bench_with_input(BenchmarkId::new("FF32", count), &ff32_vals, |b, vals| {
            b.iter(|| vals.iter().copied().fold(ff32(0.0), |acc, val| acc + val));
        });

        let f64_vals = thread_rng()
            .sample_iter(Standard)
            .take(count)
            .collect::<Vec<f64>>();

        // use the same values for both benchmarks
        let ff64_vals = f64_vals
            .clone()
            .into_iter()
            .map(ff64)
            .collect::<Vec<FF64>>();

        group.bench_with_input(BenchmarkId::new("std::f64", count), &f64_vals, |b, vals| {
            b.iter(|| vals.iter().copied().fold(0.0, |acc, val| acc + val));
        });

        group.bench_with_input(BenchmarkId::new("FF64", count), &ff64_vals, |b, vals| {
            b.iter(|| vals.iter().copied().fold(ff64(0.0), |acc, val| acc + val));
        });
    }
    group.finish();
}

criterion_group!(benches, sum);
criterion_main!(benches);
