use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Insert some entities and link 'em together
fn init(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let parent = scene.insert((
        Position::at_y(1.0),
        Rotation::rotation_x(90.0f32.to_radians()),
    ));
    let child =
        scene.insert((Position::default(), LocalPosition::default()));
    scene.attach(child, parent);
}

// Check the position of the child
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let (_pos, _) =
        scene.find_mut::<(&mut Position, &Child)>().unwrap();
}
