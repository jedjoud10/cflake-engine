use std::iter::repeat;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use ecs::prelude::*;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator, IntoParallelIterator, IndexedParallelIterator};

#[derive(Default, Component, Clone)]
struct Position([f32; 3]);

#[derive(Default, Component, Clone)]
struct Matrix(vek::Mat4<f32>);

#[derive(Default, Component, Clone)]
struct BigHeap(Box::<[u128; 32]>);

#[derive(Default, Component, Clone)]
struct BigStack([u128; 32]);

const ITEMS_PER_BATCH: u64 = 5000;
const MIN_BATCH: u64 = 1;
const MAX_BATCH: u64 = 20;

fn benchmark(c: &mut Criterion) {
    /*
    let mut group = c.benchmark_group("Insertion (1 comp)");

    for x in MIN_BATCH..MAX_BATCH {
        let count = x * ITEMS_PER_BATCH;
        group.throughput(criterion::Throughput::Elements(count));

        group.bench_function(
            BenchmarkId::new("ECS Batch", count),
            |b| {
                b.iter(|| {
                    let mut scene = Scene::default();
                    scene.extend(repeat(Position::default()).take(count as usize));
                });
            }
        );

        group.bench_function(
            BenchmarkId::new("ECS Insert", count),
            |b| {
                b.iter(|| {
                    let mut scene = Scene::default();

                    for _ in 0..count {
                        scene.insert(Position::default());
                    }
                });
            }
        );

        group.bench_function(
            BenchmarkId::new("Vec Batch", count),
            |b| {
                b.iter(|| {
                    let mut scene = Vec::<Position>::default();
                    scene.extend(repeat(Position::default()).take(count as usize));
                });
            }
        );

        group.bench_function(
            BenchmarkId::new("Vec Insert", count),
            |b| {
                b.iter(|| {
                    let mut scene = Vec::<Position>::default();
                    
                    for _ in 0..count {
                        scene.push(Position::default());
                    }
                });
            }
        );
    }

    drop(group);
    */
    let mut group = c.benchmark_group("Query (2 comp + matrix calc)");

    for x in MIN_BATCH..MAX_BATCH {
        let count = x * ITEMS_PER_BATCH;
        group.throughput(criterion::Throughput::Elements(count));

        // ECS
        let mut scene = Scene::default();
        scene.extend(repeat((Position::default(), Matrix::default())).take(count as usize));
        let mut scene = black_box(scene);
        
        // SoA
        let mut positions = Vec::<Position>::default();
        positions.extend(repeat(Position::default()).take(count as usize));
        let mut matrices = Vec::<Matrix>::default();
        matrices.extend(repeat(Matrix::default()).take(count as usize));
        let mut matrices = black_box(matrices);
        let mut positions = black_box(positions);

        // AoS
        let mut comps = Vec::<(Position, Matrix)>::default();
        comps.extend(repeat((Position::default(), Matrix::default())).take(count as usize));
        let mut comps = black_box(comps);

        // Single threaded ECS
        group.bench_function(
            BenchmarkId::new("ECS Single-threaded", count),
            |b| {
                b.iter(|| {
                    for (pos, matrix) in scene.query_mut::<(&Position, &mut Matrix)>() {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    }
                });
            }
        );

        /*
        // Single threaded SoA
        group.bench_function(
            BenchmarkId::new("Vec SoA Single-threaded", count),
            |b| {
                b.iter(|| {
                    for (pos, matrix) in positions.iter().zip(matrices.iter_mut()) {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    }
                });
            }
        );

        // Single threaded AoS
        group.bench_function(
            BenchmarkId::new("Vec AoS Single-threaded", count),
            |b| {
                b.iter(|| {
                    for (pos, matrix) in comps.iter_mut() {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    }
                });
            }
        );
        */

        // Multithreaded threaded ECS
        group.bench_function(
            BenchmarkId::new("ECS Mutli-threaded", count),
            |b| {
                b.iter(|| {
                    let query = scene.query_mut::<(&Position, &mut Matrix)>();
                    let collected = query.into_iter().collect::<Vec<_>>();
                    collected.into_par_iter().for_each(|(pos, matrix)| {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    })
                });
            }
        );

        /*
        // Multithreaded threaded SoA
        group.bench_function(
            BenchmarkId::new("Vec SoA Mutli-threaded", count),
            |b| {
                b.iter(|| {
                    positions.par_iter_mut().zip(matrices.par_iter_mut()).for_each(|(pos, matrix)| {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    })
                });
            }
        );

        // Multithreaded threaded AoS
        group.bench_function(
            BenchmarkId::new("Vec AoS Mutli-threaded", count),
            |b| {
                b.iter(|| {
                    comps.par_iter_mut().for_each(|(pos, matrix)| {
                        matrix.0 = vek::Mat4::<f32>::translation_3d(vek::Vec3::from_slice(&pos.0));
                    })
                });
            }
        );
        */
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
