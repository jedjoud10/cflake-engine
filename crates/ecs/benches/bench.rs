use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use ecs::*;
use world::ThreadPool;

#[derive(Component, Debug, PartialEq, Eq, Clone, Default)]
struct Name(&'static str);
#[derive(Component, Debug, PartialEq, Eq, Clone, Default)]
struct Health(i32);
#[derive(Component, Debug, Clone, Copy, Default)]
struct Ammo(u32);
#[derive(Component, Debug, Clone, Copy, Default)]
struct Placeholder();


fn filtering_init(ecs: &mut Scene) {
    let filter = added::<Placeholder>() & removed::<Name>();
    ecs.query_mut_with::<(&mut Health, &Ammo)>(filter);
}

fn filtering(ecs: &mut Scene) {
    let filter = added::<Placeholder>();
    for (health, ammo) in ecs.query_mut_with::<(&mut Health, &Ammo)>(filter) {
        health.0 += ammo.0 as i32;
    }
}

fn filtering_threaded(ecs: &mut Scene, threadpool: &mut ThreadPool) {
    let filter = added::<Placeholder>() & removed::<Name>();
    ecs.query_mut_with::<(&mut Health, &Ammo)>(filter).for_each(threadpool, |(health, ammo)| {
        health.0 += ammo.0 as i32;
    }, 4096);
}

fn iteration(ecs: &mut Scene) {
    for (_, health, ammo) in ecs.query_mut::<(&Name, &mut Health, &Ammo)>() {
        health.0 += ammo.0 as i32;
    }
}

fn iteration_threaded(ecs: &mut Scene, threadpool: &mut ThreadPool) {
    ecs.query_mut::<(&Name, &mut Health, &Ammo)>().for_each(threadpool, |(_, health, ammo)| {
        health.0 += ammo.0 as i32;
    }, 4096);
}

fn init<B: Bundle + Default + Clone>(count: usize) -> (Scene, Vec<Entity>) {
    let mut scene = Scene::default();
    let entities = scene.extend_from_iter(std::iter::repeat(B::default()).take(count));
    let entities = entities.to_vec();
    (scene, entities)
}


fn cleanup(ecs: &mut Scene) {
    for (_, archetype) in ecs.archetypes_mut() {
        for (_, column) in archetype.state_table_mut().iter_mut() {
            column.clear();
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut threadpool = ThreadPool::new();

    let mut group = c.benchmark_group("test");
    for i in (1..2usize) {
        let num = 5000usize * i;
        let (mut scene, entities) = init::<(Name, Health, Ammo)>(num);
    
        group.throughput(Throughput::Elements(num as u64));
        group.bench_with_input(
            BenchmarkId::new("Single-threaded", num),
            &(num as u64), |b, &size| {
                b.iter(|| iteration(&mut scene));
            }
        );

        group.bench_with_input(
            BenchmarkId::new("Multi-threaded", num),
            &(num as u64), |b, &size| {
                b.iter(|| iteration_threaded(&mut scene, &mut threadpool));
            }
        );

        cleanup(&mut scene);
        for (i, id) in entities.into_iter().enumerate() {
            if i % 10 == 0 {
                let mut entry = scene.entry_mut(id).unwrap();
                //entry.remove_bundle::<Name>().unwrap();
                entry.insert_bundle::<Placeholder>(Placeholder()).unwrap();
            }
        }

        group.bench_with_input(
            BenchmarkId::new("Filtering Single-threaded", num),
            &(num as u64), |b, &size| {
                b.iter(|| filtering(&mut scene));
            }
        );

        group.bench_with_input(
            BenchmarkId::new("Filtering Multi-threaded", num),
            &(num as u64), |b, &size| {
                b.iter(|| filtering_threaded(&mut scene, &mut threadpool));
            }
        );

        group.bench_with_input(
            BenchmarkId::new("Filtering BitSet Init", num),
            &(num as u64), |b, &size| {
                b.iter(|| filtering_init(&mut scene));
            }
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);