use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fast_fp::FF32;
use nalgebra_v029::{self as na, center, distance_squared, Point3, Vector3};

fn los_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("los_distance");

    // FF32 measures about 10% faster than f32 when tested on AMD 5950x with -Ctarget-cpu=native
    // los_distance/f32        time:   [14.967 ns 14.983 ns 15.001 ns]
    // los_distance/FF32       time:   [13.446 ns 13.451 ns 13.456 ns]

    group.bench_function("f32", |b| {
        let center_of_projection_a = Point3::new(4., 1., 0.).map(From::from);
        let unit_line_of_sight_a = Vector3::new(1., 5., 0.).normalize().map(From::from);
        let center_of_projection_b = Point3::new(3., 2., 0.).map(From::from);
        let unit_line_of_sight_b = Vector3::new(-8., 1., 0.).normalize().map(From::from);

        let sigma_o_squared = na::convert(6.84911050897861e-05_f64);
        let sigma_p_squared = na::convert(0.000625_f64);

        b.iter(|| {
            calc_los_distance::<f32>(
                black_box(center_of_projection_a),
                black_box(unit_line_of_sight_a),
                black_box(center_of_projection_b),
                black_box(unit_line_of_sight_b),
                black_box(sigma_o_squared),
                black_box(sigma_p_squared),
            )
        })
    });

    group.bench_function("FF32", |b| {
        let center_of_projection_a = Point3::new(4., 1., 0.).map(From::from);
        let unit_line_of_sight_a = Vector3::new(1., 5., 0.).normalize().map(From::from);
        let center_of_projection_b = Point3::new(3., 2., 0.).map(From::from);
        let unit_line_of_sight_b = Vector3::new(-8., 1., 0.).normalize().map(From::from);

        let sigma_o_squared = na::convert(6.84911050897861e-05_f64);
        let sigma_p_squared = na::convert(0.000625_f64);

        b.iter(|| {
            calc_los_distance::<FF32>(
                black_box(center_of_projection_a),
                black_box(unit_line_of_sight_a),
                black_box(center_of_projection_b),
                black_box(unit_line_of_sight_b),
                black_box(sigma_o_squared),
                black_box(sigma_p_squared),
            )
        })
    });
}

/// Given two line-of-sight rays, find the Mahalanobis distance between their nearest two points
/// (given some distribution parameters)
#[allow(non_snake_case)]
fn calc_los_distance<T>(
    center_of_projection_a: Point3<T>,
    unit_line_of_sight_a: Vector3<T>,
    center_of_projection_b: Point3<T>,
    unit_line_of_sight_b: Vector3<T>,
    sigma_o_squared: T,
    sigma_p_squared: T,
) -> T
where
    T: na::RealField + Copy,
{
    let A = center_of_projection_a;
    let B = center_of_projection_b;
    let a = unit_line_of_sight_a;
    let b = unit_line_of_sight_b;
    let c = B - A;

    let ab = a.dot(&b);
    let bc = b.dot(&c);
    let ac = a.dot(&c);
    let bb = b.dot(&b);
    let aa = a.dot(&a);
    let denom = aa * bb - ab * ab;
    let D = A + a * ((-ab * bc + ac * bb) / denom);
    let E = B + b * ((ab * ac - bc * aa) / denom);

    let dist_squared = distance_squared(&D, &E);
    let pos = center(&D, &E);

    let r_a_squared = distance_squared(&pos, &A);
    let r_b_squared = distance_squared(&pos, &B);

    let sigma_a_squared = r_a_squared * sigma_o_squared + sigma_p_squared;
    let sigma_b_squared = r_b_squared * sigma_o_squared + sigma_p_squared;

    let mahalanobis_distance = (dist_squared / (sigma_a_squared + sigma_b_squared)).sqrt();

    mahalanobis_distance
}

criterion_group!(benches, los_distance);
criterion_main!(benches);
