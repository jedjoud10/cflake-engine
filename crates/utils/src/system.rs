use crate::{ThreadPool, FileManager};
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
