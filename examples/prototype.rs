use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Sample test resource added into the world
struct TestResource {
    sync_read: bool,
    async_read: bool,
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();

    let buffer =
        Buffer::<i32, 0>::from_slice(&graphics, &vec![0i32; 1024*1024*10], BufferMode::Dynamic, BufferUsage::READ)
            .unwrap();

    drop(graphics);
    world.insert(buffer);
    world.insert(TestResource {
        sync_read: false,
        async_read: false,
    });
}

// Camera controller update executed every tick
fn update(world: &mut World) {
    let gui = world.get_mut::<Interface>().unwrap();
    let buffer = world.get::<Buffer::<i32, 0>>().unwrap();
    let mut test = world.get_mut::<TestResource>().unwrap();

    egui::Window::new("Prototyping").show(&gui, |ui| {
        ui.toggle_value(&mut test.sync_read, "Test buffer readback (synchronous)");
        ui.toggle_value(&mut test.async_read, "Test buffer readback (asynchronous)");

        if test.sync_read {
            let values = buffer.as_view(..).unwrap();
            let sum = (&*values).iter().sum::<i32>();
        } else if test.async_read {
            buffer.async_read(.., |data| {
                let sum = (&*data).iter().sum::<i32>();
            }).unwrap();
        }
    });
}
