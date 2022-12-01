use std::time::{Duration, Instant};

use crate::{FileManager, ThreadPool, Time};
use world::{user, System, World};

// Add the threadpool resource to the world
pub fn threadpool(system: &mut System) {
    system
        .insert_init(|world: &mut World| {
            world.insert(ThreadPool::default())
        })
        .before(user);
}

// Add the IO path manager
pub fn io(system: &mut System, author: String, app: String) {
    system
        .insert_init(move |world: &mut World| {
            world.insert(FileManager::new(&author, &app))
        })
        .before(user);
}

// The timer system will automatically insert the Time resource and will update it at the start of each frame
pub fn system(system: &mut System) {
    system.insert_init(|world: &mut World| {
        world.insert(Time {
            delta: Duration::ZERO,
            frame_count: 0,
            startup: Instant::now(),
            frame_start: Instant::now(),
        });
    }).before(user);
    system.insert_update(|world: &mut World| {
        let mut time = world.get_mut::<Time>().unwrap();
        let now = Instant::now();
        time.delta = now - time.frame_start;
        time.frame_start = now;
        time.frame_count += 1;
    }).before(user);
}
