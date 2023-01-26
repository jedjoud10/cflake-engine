use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default().insert_tick(tick).set_frame_rate_limit(FrameRateLimit::Limited(30)).execute();
}

fn tick(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    log::info!("Frame count: {}, tick count: {}", time.frame_count(), time.tick_count());
}