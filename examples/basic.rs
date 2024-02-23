use cflake_engine::prelude::*;

fn main() {
    App::new()
        .insert_init(init)
        .insert_update(update)
        .execute().unwrap();
}

fn init(world: &mut World, _: &Init) {
    let pass: RenderPass<BGRA<Normalized<u8>>, ()> = RenderPass::<SwapchainFormat, ()>::new(Operation {
        load: LoadOp::Clear(Vec4::zero()),
        store: StoreOp::Store,
    }, ());
    world.insert(pass);
}

fn update(world: &mut World, _: &Update) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut pass = world.get_mut::<RenderPass<BGRA<Normalized<u8>>, ()>>().unwrap();
    let mut window = world.get_mut::<Window>().unwrap();
    let mut encoder = graphics.acquire();
    let active = pass.begin(&mut encoder, window.as_render_target().unwrap(), ());
    drop(active);
    graphics.reuse([encoder]);
}