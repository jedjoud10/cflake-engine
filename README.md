# cFlake Engine
cFlake Engine, in it's current state, is a free and open-source Rust game engine that I have created with the help of certain friends and mates.
Currently, cFlake engine is under heavy development (***very*** WIP), but pull requests are heavily appreciated (pls help me I am becoming insane)

# Main features of cFlake:
* Custom World Events, Systems, and Resources all accessible within the **World** struct
* Deterministic event sorting through multiple stages 
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

# Architecture & Design
**cFlake engine** uses an ECS (Entity, Component, System) architecture. This architecture is mostly used by new and modern game engines to be able to easily use of highly parallel code that can benefit from multithreading. The same goes for cFlake engine. The ECS architectures is split into 3 main parts, **Entities**, **Components**, and **Systems**.

## Entities
Entities could be though of as a simple handle to some underlying data stored within the world scene. Entities by themselves do not store any data, and they can be represented using a u64 integer handle. 

They act a *pointer* of some sort that will fetch data from somewhere else. This alleviates the problem of having random data scattered in memory and having separate handles for them and such. 

Entities can be created and destroyed using the ``Scene`` resource that is automatically added into the scene if the ``scene`` system is enabled (check ``EnabledSystems``). More so, entities can have data linked to them through ``Component``s. An entity could have up to 32 components linked to it at the same time (if the "extend-bitmasks" feature is enabled, this gets bumped up to 64 max components)

## Components
Components are what actually store the data for a specified entity. Components are all stored within an ``Archetype``, which is a deisng/optimization that certain ECS implementation use for faster iteration speeds at the cost for slower insertion / removal speeds. Components can be inserted into the scene using the ``Scene::insert()`` function, which takes in a ``Bundle`` parameter that contains a tuple of different components. Components can also be removed from entities using ``EntryMut``, which is a mutable access to an entity's components directly.

## Systems
This is where things get tricky however, and this is where my implementation of ECS starts to differ from the pure ECS implementation. In this engine, ``System``s are just containers that store multiple ``Event``s, and these events get fired off whenever something interesting happens, like the start of a frame or during engine initialization. 

To actually handle modifying data related to components, one must use a scene ``Query`` that would iterate over all the given components of the given ``Bundle`` tuple type. These ``Queries`` are not related in any way to the ``Systems``, and they could be accessed as long as you have a mutable or immutable reference to the ``Scene`` resource (depending on what type of query you wish to use)



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
