use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
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


fn filtering(ecs: &mut Scene) {
    let filter = added::<Placeholder>() & removed::<Name>();
    for (health, ammo) in ecs.query_mut_with::<(&mut Health, &Ammo)>(filter) {
        health.0 += ammo.0 as i32;
    }
}

fn filtering_threaded(ecs: &mut Scene, threadpool: &mut ThreadPool) {
    let filter = added::<Placeholder>() & removed::<Name>();
    ecs.query_mut_with::<(&mut Health, &Ammo)>(filter).for_each(threadpool, |(health, ammo)| {
        health.0 += ammo.0 as i32;
    }, 512);
}

fn iteration(ecs: &mut Scene) {
    for (_, health, ammo) in ecs.query_mut::<(&Name, &mut Health, &Ammo)>() {
        health.0 += ammo.0 as i32;
    }
}

fn iteration_threaded(ecs: &mut Scene, threadpool: &mut ThreadPool) {
    ecs.query_mut::<(&Name, &mut Health, &Ammo)>().for_each(threadpool, |(_, health, ammo)| {
        health.0 += ammo.0 as i32;
    }, 512);
}

fn init<B: Bundle + Default + Clone>(count: usize) -> (Scene, Vec<Entity>) {
    let mut scene = Scene::default();
    let entities = scene.extend_from_iter(std::iter::repeat(B::default()).take(count));
    let entities = entities.to_vec();
    (scene, entities)
}


fn cleanup(ecs: &mut Scene) {
    for (_, archetype) in ecs.archetypes_mut() {
        for state in archetype.states_mut().iter_mut() {
            state.update(|added, removed, mutated| {
                *added = Mask::zero();
                *mutated = Mask::zero();
                *removed = Mask::zero();
            });
        }
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    let mut threadpool = ThreadPool::new();
    
    for i in (0..5usize) {
        let num = 1000usize * 10usize.pow(i as u32);
        let name1 = format!("Iteration {}k", num / 1000);
        let (mut scene, entities) = init::<(Name, Health, Ammo)>(num);
        let mut group = c.benchmark_group(name1);
        group.bench_function("Single-threaded", |b| b.iter(|| iteration(&mut scene)));
        group.bench_function("Multi-threaded", |b| b.iter(|| iteration_threaded(&mut scene, &mut threadpool)));
        cleanup(&mut scene);
        for (i, id) in entities.into_iter().enumerate() {
            if i % 10 == 0 {
                let mut entry = scene.entry_mut(id).unwrap();
                entry.remove_bundle::<Name>().unwrap();
                entry.insert_bundle::<Placeholder>(Placeholder()).unwrap();
            }
        }
        
        group.bench_function("Filtering Single-threaded", |b| b.iter(|| filtering(&mut scene)));
        group.bench_function("Filtering Multi-threaded", |b| b.iter(|| filtering_threaded(&mut scene, &mut threadpool)));
    
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);