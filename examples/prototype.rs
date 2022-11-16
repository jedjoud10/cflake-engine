/*
use std::num::NonZeroU8;

use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_init(init)
        .execute();
}
fn init(world: &mut World) {
    let mut ctx = world.get_mut::<Context>().unwrap();
    let assets = world.get::<Assets>().unwrap();

    // Load the BRDF integration map
    let brdf_integration_map = assets
        .load::<IntegrationMap>(
            "engine/textures/integration.png",
            (
                &mut ctx,
                TextureImportSettings {
                    sampling: Sampling {
                        filter: Filter::Linear,
                        wrap: Wrap::ClampToEdge,
                        ..Default::default()
                    },
                    mode: TextureMode::Resizable,
                    mipmaps: MipMapSetting::Manual {
                        levels: NonZeroU8::new(3).unwrap(),
                    },
                },
            ),
        )
        .unwrap();
}
*/

fn main() {}
