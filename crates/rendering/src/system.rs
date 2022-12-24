use graphics::{Graphics, Window};
use world::{post_user, System, World};

// Clear the window and render the entities
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let queue = graphics.queue();
    let device = graphics.device();
    let swapchain = graphics.swapchain();
    
    /*
    unsafe {
        let mut recorder = queue.acquire(device);
        /*
        if let Some((index, image)) = swapchain.acquire_next_image() {
            let recreate = swapchain.present(queue, (index, image));
        }
        */
        let submission = queue.submit(recorder).wait();
        log::info!("{:?}", submission);
    }
    */
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
