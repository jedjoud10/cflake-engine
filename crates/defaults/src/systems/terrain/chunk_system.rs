use ecs::SystemData;
use math::octrees::OctreeNode;
use terrain::ChunkCoords;
use super::ChunkSystem;
ecs::impl_systemdata!(ChunkSystem);

pub fn system_prefire(data: &mut SystemData<ChunkSystem>) {
    // We must add all the chunks that the octree tells us to add
    // First of all, we get the camera data
    let camera = core::global::ecs::entity(core::global::main::world_data().main_camera_entity_id).unwrap();
    let camera_transform = core::global::ecs::component::<crate::components::Transform>(&camera).unwrap();
    let camera_pos = camera_transform.position;

    // Update the octree
    let octree = &mut data.octree;
    let octree_size = octree.internal_octree.size;
    let i = std::time::Instant::now();
    if let Option::Some((added, removed, total)) = octree.generate_incremental_octree(&camera_pos, terrain::DEFAULT_LOD_FACTOR) {
        // We successfully updated the octree 
        
        // Add the chunks into the world
        for x in added {
            let octree_node: OctreeNode = x;
            let coords = ChunkCoords::new(&octree_node);
            // Add the entity
            create_chunk_entity(coords, octree_size, data);
        }

        // Remove the chunks as well
        for x in removed {
            let octree_node: OctreeNode = x;

        }
    }
}

// Create the chunk entity and add it to the world
fn create_chunk_entity(coords: ChunkCoords, octree_size: u64, data: &mut SystemData<ChunkSystem>) {
    // Create the entity
    let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
    let entity = ecs::Entity::new(name.as_str());

    // Create the chunk component
    let chunk = terrain::Chunk::new(coords);
    // Link the components
    let mut linkings = ecs::ComponentLinkingGroup::new();
    linkings.link::<terrain::Chunk>(chunk).unwrap();
    
    // Transform
    linkings
        .link::<crate::components::Transform>(crate::components::Transform::default()
            .with_position(coords.position.into())
            .with_scale(veclib::Vector3::new((coords.size / octree_size) as f32, (coords.size / octree_size) as f32, (coords.size / octree_size) as f32))
        ).unwrap();
    // Create a renderer with an empty model
    let renderer = crate::components::Renderer::default().set_material(data.material).set_wireframe(true);
    linkings.link::<crate::components::Renderer>(renderer).unwrap();
    println!("Add chunk {}", name);
    // Add the entity
    let result = core::global::ecs::entity_add(entity, linkings);
    // Callback to run after the entity is added into the world
    let mut data = data.clone();
    result.with_callback(core::global::callbacks::CallbackType::EntityCreatedCallback(others::callbacks::RefCallback::new(move |entity: &ecs::Entity| {
        // Add the chunk to our total chunks
        data.chunks.insert(coords, entity.entity_id);
    })).create());
}

// Create the Chunk Manager system
// This system will add / remove chunks from the world and nothing else
pub fn system(depth: u8, csgtree: math::csg::CSGTree, material: rendering::GPUObjectID) {    
    // Check if a an already existing node could be subdivided even more
    fn can_node_subdivide_twin(node: &math::octrees::OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let c: veclib::Vector3<f32> = node.get_center().into();
        let max = node.depth == 1 || node.depth == 2;
        let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
        node.children_indices.is_none() && node.depth < max_depth && result
    }
    // Create a new octree
    let internal_octree = math::octrees::Octree::new(depth, (terrain::MAIN_CHUNK_SIZE) as u64);
    let octree = math::octrees::AdvancedOctree::new(internal_octree, can_node_subdivide_twin);

    // Create the system data from the given terrain settings
    let chunk_system_data = ChunkSystem {
        octree,
        csgtree,
        material,
        ..ChunkSystem::default()
    };
    core::global::ecs::add_system(chunk_system_data, || {
        // Create a system
        let mut system = ecs::System::new();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
