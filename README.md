# cFlake Engine
cFlake Engine, is a free and open-source Rust game engine designed to be easily usable by beginners whilst giving some decent performance.
Currently, cFlake engine is under heavy development (***very*** WIP), but pull requests are heavily appreciated (pls help me I am becoming insane)


# Main features of cFlake:
* Custom World Event, Systems, and Resources all accessible within the **World** struct
* Archetypal ECS built to be used in conjunction with the World Events and Systems
* Custom Renderer built on OpenGL and Glutin (**WIP, currently switching to Ash and Winit**)
* GPU Voxel generation and Octree Terrain (disabled temporarily)
* Asynchronous asset loader and utility thread pool
* Input handling with gamepad support (gilrs) and keybinding serialization / deserialization
* Custom sound support through CPAL and directional audio through HRTF (TODO)
# Main links
* [YouTube Development Channel](https://www.youtube.com/channel/UCaeZjQFw4QIi5vdfonAmsvA)
* [Trello dashboard](https://trello.com/b/9FsDb6Z1/cflake)

# Examples
Examples are in the **examples** foldder, but here is a quick and dirty setup to get you started (literally just the ``hello`` example)
```rs
use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Function that gets executed when the engine starts
fn init(world: &mut World) {
    println!("Hello World!");
}

// Function that gets executed every frame
fn update(world: &mut World) {
    println!("Hello Frame!");
}
``` 

# Thanks to:
* Lionel Stanway (MoldyToeMan)
* Logan McLennan (PigSteel)
* Dimev (Skythedragon)

# LICENSE
Dual-Licensed under
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
