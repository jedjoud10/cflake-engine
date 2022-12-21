use graphics::{Graphics, Window};
use world::{World, post_user, System};

// Clear the window and render the entities
fn update(world: &mut World) {
    /*
    let graphics = world.get::<Graphics>().unwrap();
    let window = world.get::<Window>().unwrap();
    let queue = graphics.queue();
    let swapchain = graphics.swapchain();
    let mut recorder = graphics.acquire();

    unsafe {
        /*
        if let Some((index, image)) = swapchain.acquire_next_image() {
            let recreate = swapchain.present(queue, (index, image));
        }
        */
    }

    let submission = graphics.submit(recorder);
    submission.wait();
    */
}

// Rendering system to clear the window and render the entities
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
