# 🚧🚧🚧 **UNDER HEAVY DEVELOPMENT** 🚧🚧🚧


# cFlake Engine


cFlake Engine is a free and open-source Rust game engine that I have been working on for the past 2 years as a personal project.
Currently, cFlake engine is under heavy development (***very*** WIP) and is still in it's early stages, but pull requests are heavily appreciated (pls help me I am becoming insane)

# Main features of cFlake:
* 6 World Event Variants, Systems, and Resources all accessible within the **World** struct
* Deterministic event sorting through multiple stages 
* Archetypal multithreaded ECS built to be used in conjunction with the World Events and Systems
* Custom Graphics API built on top of WGPU
* GPU Voxel generation and Octree Terrain (disabled temporarily)
* Asynchronous asset loader using Rayon threads
* Input handling with gamepad support (gilrs) and keybinding serialization / deserialization
* Custom sound support through CPAL (kinda works)

# State of each crate:
## Legend
* ❌ = TODO
* 🚧 = WIP, not complete
* ⚠️ = Semi complete, needs finishing touches
* ✅ = complete

## Crates
* World: 🚧
* Utils: ⚠️
* Terrain: 🚧
* Graphics: 🚧
* Rendering: 🚧
* Networking: ❌
* Math: 🚧
* Input: ✅
* ECS: ✅
* Asset loader: ⚠️
* Audio: 🚧
* GUI (eGUI): ⚠️
  
# Main links
* [YouTube Development Channel](https://www.youtube.com/channel/UCaeZjQFw4QIi5vdfonAmsvA)
* [Trello dashboard](https://trello.com/b/9FsDb6Z1/cflake)

# Examples
Examples are in the **examples** folder, but here is a quick and dirty setup to get you started (literally just the ``hello`` example)
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

# Scene Architecture & Design
**cFlake engine** uses an ECS (Entity, Component, System) architecture. This architecture is mostly used by new and modern game engines to be able to easily use of highly parallel code that can benefit from multithreading. The same goes for cFlake engine. The ECS architectures is split into 3 main parts, **Entities**, **Components**, and **Systems**.

## Entities
Entities could be though of as a simple handle to some underlying data stored within the world scene. Entities by themselves do not store any data, and they can be represented using a u64 integer handle. 

They act a *pointer* of some sort that will fetch data from somewhere else. This alleviates the problem of having random data scattered in memory and having separate handles for them and such. 

Entities can be created and destroyed using the ``Scene`` resource that is automatically added into the world. More so, entities can have data linked to them through ``Component``s. An entity could have up to 32 components linked to it at the same time (if the "extend-bitmasks" feature is enabled, this gets bumped up to 64 max components)

## Components
Components are what actually store the data for a specified entity. Components are all stored within an ``Archetype``, which is a deisng/optimization that certain ECS implementation use for faster iteration speeds at the cost for slower insertion / removal speeds. Components can be inserted into the scene using the ``Scene::insert()`` function, which takes in a ``Bundle`` parameter that contains a tuple of different components. Components can also be removed from entities using ``EntryMut``, which is a mutable access to an entity's components directly.

## Systems
This is where things get tricky however, and this is where my implementation of ECS starts to differ from the pure ECS implementation. In this engine, ``System``s are just containers that store multiple ``Event``s, and these events get fired off whenever something interesting happens, like the start of a frame or during engine initialization. 

To actually handle modifying data related to components, one must use a scene ``Query`` that would iterate over all the given components of the given ``Bundle`` tuple type. These ``Queries`` are not related in any way to the ``Systems``, and they could be accessed as long as you have a mutable or immutable reference to the ``Scene`` resource (depending on what type of query you wish to use)

# Graphics (story time)
At the moment, cFlake uses a custom built graphics API abstraction that wraps over WGPU and ShaderC. This however, was a recent change (~4 months in the making) due to limitations with the original backend (OpenGL) the engine. I had realized that OpenGL was not going to scale well with all the new multi-threaded features that I've implemented like a multithreaded asset loader and multithreaded ECS system. After tinkering with Vulkan (raw Vulkan, Vulkano), I decided to not use it (even after I pathetically tried to implement it, which took me 2-3 months), I decided to use Wgpu, since I simply could not cope with the manual state tracking of Vulkan. Nonetheless, the new graphics API allows us to create objects (like textures/buffers/shaders) in other threads. THis allows us to load texture/mesh assets asynchronously. 

# Asset Managment
Currently, there are a few way to load in external assets (and to ship them) within your binary to be able to make your published executables more portable. There is a an ``Assets`` resource that is automatically added to the world that allows you to load and deserialize assets from the file system or pre-defined persistent assets. 

In this engine, assets are defined as structs that can be deserialized and loaded from raw binary data (that is most probably file binary data). You can customize how assets are loaded in within the engine using the ``Asset`` trait, and you can implement it on any structure that can be deserialized from a raw stream of bytes. 

You can define a "Context" and "Settings" that can be used to customize how each asset is loaded. Asset deserialization *must* be faillible, and due to that restriction, I made it so you *must* define an ``Error`` type that gets returned whenever asset conversion fails. There is also asnychronous asset loading supported within the engine, and this is implemented using the ``AsyncAsset`` trait that gets automatically gets implemented for ``Asset``s that are Sync + Send and have their Settings + Context be Sync and Send. So assets can be automatically loaded as asynchronous assets if they fit those requirements.

For now, these are the types of assets that are loadable/deseriazable by default and their respective extensions.
* Texture2D: .png, .jpg
* glTF scene: .gltf
* AudioClip: .wav, .mp3
* Vertex Shader: .vert
* Fragment Shader: .frag
* Compute Shader: .comp
* Raw GLSL (only for includes): .glsl
* Raw UTF8 text: .txt

# Thanks to:
* Lionel Stanway (MoldyToeMan)
* Logan McLennan (PigSteel)
* Dimev (Skythedragon)
* Poly Haven for their awesome textures and models (used in examples folder)
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
