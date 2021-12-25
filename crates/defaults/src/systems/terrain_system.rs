use others::{Instance, callbacks::RefCallback};
use rendering::pipec;
use core::global::callbacks::CallbackType;
use std::{rc::Rc, time::Instant};
use terrain::{ChunkCoords, TerrainStats, Terrain};
use crate::components::Chunk;
use ecs::*;
use math::octrees::*;

// Whenever the terrain system updates
fn system_prefire(terrain: &mut SystemData<Terrain>) {
    // Timing
    let t = Instant::now();
    // Get the camera location
    let camera_entity = core::global::ecs::entity(core::global::main::world_data().main_camera_entity_id).unwrap();
    let camera_transform = core::global::ecs::component::<crate::components::Transform>(&camera_entity).unwrap();
    // Get the camera transform values
    let camera_position = camera_transform.position;
    let camera_forward_vector = camera_transform.get_forward_vector();

    // Get the terrain data
    let material = terrain.settings.material.clone();
    let octree_size = terrain.octree.internal_octree.size;
    let octree_depth = terrain.settings.octree_depth;

    // Generate the octree each frame and generate / delete the chunks
    if terrain.chunk_manager.octree_update_valid() {
        match terrain.octree.generate_incremental_octree(&camera_position, terrain::DEFAULT_LOD_FACTOR) {
            Some((mut added, removed, total)) => {
                // Filter first
                added.retain(|node| node.children_indices.is_none() && math::Intersection::csgtree_aabb(&terrain.csgtree, &node.get_aabb()));
                // Turn all the newly added nodes into chunks and instantiate them into the world
                for octree_node in added {
                    // Add the chunk in the chunk manager
                    terrain.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                }
                // Delete all the removed octree nodes from the world
                for octree_node in removed {
                    let chunk_coords = ChunkCoords::new(&octree_node);
                    // Remove the chunk from the chunk manager
                    match terrain.chunk_manager.remove_chunk(&chunk_coords) {
                        Some(_) => {
                            // Get the entity id
                            terrain.chunk_manager.remove_chunk_entity(&chunk_coords);
                        }
                        None => {}
                    }
                }
            }
            None => { /* Nothing happened */ }
        }
        terrain.chunk_manager.update_camera_view(camera_position, camera_forward_vector);
    }

    // Update the chunk manager
    let cloned = terrain.clone();
    let terrain = &mut **terrain;
    match terrain.chunk_manager.update(&mut terrain.voxel_generator, core::global::timings::frame_count()) {
        Some((added_chunks, removed_chunks)) => {
            // Add the entities to the entity manager
            for (coords, tmodel) in added_chunks {
                // Add the entity
                let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
                let entity = Entity::new(name.as_str());

                // Create the chunk component
                let chunk = Chunk { coords: coords.clone() };
                
                // Link the components
                let mut linkings = ComponentLinkingGroup::new();
                linkings.link::<Chunk>(chunk).unwrap();
                // Transform
                linkings
                    .link::<crate::components::Transform>(crate::components::Transform::default()
                        .with_position(coords.position.into())
                        .with_scale(veclib::Vector3::new((coords.size / octree_size) as f32, (coords.size / octree_size) as f32, (coords.size / octree_size) as f32))
                    ).unwrap();
                // Turn the model into a GPU model
                let model = pipec::model(tmodel.model);
                let renderer = crate::components::Renderer::default().set_material(material.clone()).set_wireframe(true).set_model(model);
                linkings.link::<crate::components::Renderer>(renderer).unwrap();                
                let cloned = cloned.clone();
                core::global::ecs::entity_add(entity, linkings).with_callback(CallbackType::EntityCreatedCallback(RefCallback::new(move |entity: &ecs::Entity| {
                    let mut terrain = cloned.clone();
                    // Add the chunk entity to the terrain
                    terrain.chunk_manager.add_chunk_entity(&coords, entity.entity_id);
                })).create());
            }
            // Remove the entities
            for entity_id in removed_chunks {
                core::global::ecs::entity_remove(entity_id);
            }
        }
        None => {}
    }
}

// Create the terrain system
pub fn system() {
    core::global::ecs::add_system(|| {
        // Create a system

        // The terrain shader
        let terrain_shader = pipec::shader(
            rendering::Shader::default()
                .load_shader(vec![
                    "defaults\\shaders\\rendering\\default.vrsh.glsl",
                    "defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl",
                ])
                .unwrap(),
        );
        // Material
        let mut material = rendering::Material::new("Terrain material").set_shader(terrain_shader);
        // Create the diffuse texture array
        let texture = pipec::texturec(
            assets::cachec::cache_l(
                "terrain_diffuse_texture",
                rendering::Texture::create_texturearray(vec!["defaults\\textures\\rock_diffuse.png", "defaults\\textures\\missing_texture.png"], 256, 256)
                    .apply_texture_load_options(None)
                    .enable_mipmaps(),
            )
            .unwrap(),
        );
        // Create the normalmap texture array
        let texture2 = pipec::texturec(
            assets::cachec::cache_l(
                "terrain_normal_map_texture",
                rendering::Texture::create_texturearray(vec!["defaults\\textures\\rock_normal.png", "defaults\\textures\\missing_texture.png"], 256, 256)
                    .apply_texture_load_options(None)
                    .enable_mipmaps(),
            )
            .unwrap(),
        );
        let group = &mut material.uniforms;
        group.set_f32("normals_strength", 2.0);
        group.set_vec2f32("uv_scale", veclib::Vector2::ONE * 0.7);
        group.set_t2da("diffuse_textures", &texture, 0);
        group.set_t2da("normals_textures", &texture2, 1);
        group.set_i32("material_id", 0);
        let material = pipec::material(material);

        let settings = terrain::TerrainSettings {
            octree_depth: 8,
            material,
            voxel_generator_interpreter: terrain::interpreter::Interpreter::new_pregenerated(),
        };

        let mut system = ecs::System::new(Terrain::new(settings));
        // Link some components to the system
        system.link::<crate::components::Chunk>();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
