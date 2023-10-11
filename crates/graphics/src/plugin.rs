use winit::event_loop::EventLoop;
use world::{prelude::{Plugin, Init}, world::World, system::Registries};
use crate::context::WindowSettings;

/// Initialization function
pub fn init(world: &mut World, _: &Init) {
    /*
    let settings = world.remove::<WindowSettings>();
    let el = world.get::<EventLoop<()>>().unwrap();
    let (window, graphics) = crate::context::initialize_phobos_context(&el, settings);
    drop(el);
    world.insert(window);
    world.insert(graphics);
    */
}

/// Graphics plugin that will create the [phobos] context and [winit] window
pub fn plugin(registries: &mut Registries) {
    registries.init.insert(init);
}