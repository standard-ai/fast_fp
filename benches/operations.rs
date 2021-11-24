use criterion::{
    criterion_group, criterion_main, measurement::Measurement, BatchSize, BenchmarkGroup,
    BenchmarkId, Criterion, Throughput,
};
use fast_fp::{ff32, ff64, FF32, FF64};
use rand::{
    distributions::{self, Distribution},
    rngs::StdRng,
    Rng, SeedableRng,
};
use std::ops::{Add, Div, Mul};

fn add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");

    let rng = StdRng::from_entropy();
    let f32s = distributions::Uniform::<f32>::new(0.0, 1.0);
    let f64s = distributions::Uniform::<f64>::new(0.0, 1.0);

    // clone the rng for each benched type to keep the generated values identical
    fold(&mut group, "std::f32", f32::add, 0.0, rng.clone(), f32s);
    fold(&mut group, "FF32", FF32::add, ff32(0.0), rng.clone(), f32s);
    fold(&mut group, "std::f64", f64::add, 0.0, rng.clone(), f64s);
    fold(&mut group, "FF64", FF64::add, ff64(0.0), rng.clone(), f64s);
}

fn mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("mul");

    let rng = StdRng::from_entropy();

    // try to avoid subnormals/explosions by limiting the values near 1
    let f32s = distributions::Uniform::<f32>::new(0.9, 1.1);
    let f64s = distributions::Uniform::<f64>::new(0.9, 1.1);

    // clone the rng for each benched type to keep the generated values identical
    fold(&mut group, "std::f32", f32::mul, 0.0, rng.clone(), f32s);
    fold(&mut group, "FF32", FF32::mul, ff32(0.0), rng.clone(), f32s);
    fold(&mut group, "std::f64", f64::mul, 0.0, rng.clone(), f64s);
    fold(&mut group, "FF64", FF64::mul, ff64(0.0), rng.clone(), f64s);
}

fn div(c: &mut Criterion) {
    let mut group = c.benchmark_group("div");

    let rng = StdRng::from_entropy();

    // try to avoid subnormals/explosions by limiting the values near 1
    let f32s = distributions::Uniform::<f32>::new(0.9, 1.1);
    let f64s = distributions::Uniform::<f64>::new(0.9, 1.1);

    // clone the rng for each benched type to keep the generated values identical
    fold(&mut group, "std::f32", f32::div, 0.0, rng.clone(), f32s);
    fold(&mut group, "FF32", FF32::div, ff32(0.0), rng.clone(), f32s);
    fold(&mut group, "std::f64", f64::div, 0.0, rng.clone(), f64s);
    fold(&mut group, "FF64", FF64::div, ff64(0.0), rng.clone(), f64s);
}

fn min(c: &mut Criterion) {
    let mut group = c.benchmark_group("min");

    let rng = StdRng::from_entropy();
    let f32s = distributions::Uniform::<f32>::new(0.0, 1.0);
    let f64s = distributions::Uniform::<f64>::new(0.0, 1.0);

    // clone the rng for each benched type to keep the generated values identical
    fold(&mut group, "std::f32", f32::min, 0.0, rng.clone(), f32s);
    fold(&mut group, "FF32", FF32::min, ff32(0.0), rng.clone(), f32s);
    fold(&mut group, "std::f64", f64::min, 0.0, rng.clone(), f64s);
    fold(&mut group, "FF64", FF64::min, ff64(0.0), rng.clone(), f64s);
}

fn fold<T, S>(
    group: &mut BenchmarkGroup<'_, impl Measurement>,
    id: &str,
    op: impl Fn(T, T) -> T + Copy,
    init: T,
    mut rng: impl Rng,
    vals: impl Distribution<S> + Copy,
) where
    T: From<S> + Copy,
{
    fold_count([init; 1], group, id, op, init, &mut rng, vals);
    fold_count([init; 2], group, id, op, init, &mut rng, vals);
    fold_count([init; 4], group, id, op, init, &mut rng, vals);
    fold_count([init; 8], group, id, op, init, &mut rng, vals);
    fold_count([init; 64], group, id, op, init, &mut rng, vals);
    fold_count([init; 256], group, id, op, init, &mut rng, vals);
    fold_count([init; 1024], group, id, op, init, &mut rng, vals);
}

fn fold_count<T, S, const N: usize>(
    arr: [T; N],
    group: &mut BenchmarkGroup<'_, impl Measurement>,
    id: &str,
    op: impl Fn(T, T) -> T + Copy,
    init: T,
    mut rng: impl Rng,
    vals: impl Distribution<S> + Copy,
) where
    T: From<S> + Copy,
{
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function(BenchmarkId::new(id, N), |b| {
        b.iter_batched_ref(
            || {
                let mut inputs = arr;
                inputs
                    .iter_mut()
                    .zip((&mut rng).sample_iter(&vals))
                    .for_each(|(dst, val)| *dst = T::from(val));
                inputs
            },
            |vals| vals.iter().copied().fold(init, op),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, add, mul, div, min);
criterion_main!(benches);
