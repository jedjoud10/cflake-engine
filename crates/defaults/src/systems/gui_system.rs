use world::{
    ecs::component::{ComponentQueryParams, ComponentQuerySet},
    World,
};

use crate::components::{Light, Transform};

// The lights system update loop
fn run(world: &mut World, mut _data: ComponentQuerySet) {
    world.gui.draw_frame(&mut world.pipeline);
    let query = _data.get_mut(0).unwrap();
    query.all.iter_mut().for_each(|(_, linked)| {
        let transform = linked.get_mut::<Transform>().unwrap();
        transform.rotation = vek::Quaternion::<f32>::rotation_x(-world.time.elapsed.to_radians() as f32);
    });
}

// Create the GUI system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .event(run)
        .query(ComponentQueryParams::default().link::<Light>().link::<Transform>())
        .build();
}
