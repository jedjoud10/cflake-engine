use std::{
    thread::Thread,
    time::{Duration, Instant},
};

use crate::{FileManager, ThreadPool, Time};
use world::{post_user, user, System, World};

// Add the threadpool resource to the world
pub fn threadpool(system: &mut System) {
    // Main initialization event
    system
        .insert_init(|world: &mut World| {
            world.insert(ThreadPool::default())
        })
        .before(user);

    // Update event that check if any of the threads panicked
    system
        .insert_update(|world: &mut World| {
            let pool = world.get::<ThreadPool>().unwrap();
            if let Some(id) = pool.check_any_panicked() {
                panic!("WorkerThread {} panicked", id);
            }
        })
        .after(post_user);
}

// Add the IO path manager
pub fn io(system: &mut System, author: String, app: String) {
    system
        .insert_init(move |world: &mut World| {
            world.insert(FileManager::new(&author, &app))
        })
        .before(user);
}

// Add the Time manager
pub fn time(system: &mut System) {
    // Main initialization event
    system
        .insert_init(|world: &mut World| {
            world.insert(Time {
                delta: Duration::ZERO,
                frame_count: 0,
                startup: Instant::now(),
                frame_start: Instant::now(),
                average_delta: 1.0,
            });
        })
        .before(user);

    // Update event that will mutate the time fields
    system
        .insert_update(|world: &mut World| {
            let mut time = world.get_mut::<Time>().unwrap();
            let now = Instant::now();
            time.delta = now - time.frame_start;
            time.frame_start = now;
            time.frame_count += 1;
            let delta = time.delta.as_secs_f32();
            time.average_delta =
                time.average_delta * 0.8 + delta * 0.2;
        })
        .before(user);
}
