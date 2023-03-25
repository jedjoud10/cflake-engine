use ecs::{Scene, Position, Entity};
use graphics::{Graphics, XYZW, VertexBuffer, BufferMode, BufferUsage, Normalized, TriangleBuffer, DrawIndexedIndirectBuffer, DrawIndexedIndirect};
use rendering::{Mesh, Surface, Renderer};
use utils::{Storage, Time};
use world::{System, World, user};
use crate::{ChunkCoords, Terrain, TerrainMaterial, Chunk, ChunkState, ChunkViewer};

// Creates the default material storage for the terrain material
fn init(world: &mut World) {
    world.insert(Storage::<TerrainMaterial>::default());
}

// Creates a chunk entity and inserts it in the world
fn spawn_chunk(
    terrain: &Terrain,
    scene: &mut Scene,
    meshes: &mut Storage<Mesh>,
    indirects: &mut Storage<DrawIndexedIndirectBuffer>,
    graphics: &Graphics,
    coords: ChunkCoords,
) -> Entity {
    // Calculate the maximum number of vertices that we can store
    let vertex_count = (terrain.size as usize - 50).pow(3);
    let triangle_count = (terrain.size as usize - 1).pow(3) * 4;

    // Create the vertex buffer (make sure size can contain ALL possible vertices)
    let vertices = VertexBuffer::<XYZW<f32>>::zeroed(
        graphics, 
        vertex_count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE
    ).unwrap();

    // Create the normal buffer (make sure size can contain ALL possible normals)
    let normals = VertexBuffer::<XYZW<Normalized<i8>>>::zeroed(
        graphics, 
        vertex_count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE
    ).unwrap();
    
    // Create the triangle buffer (make sure size can contain ALL possible triangles)
    let triangles = TriangleBuffer::<u32>::zeroed(
        graphics, 
        triangle_count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE
    ).unwrap();

    // Create a mesh that uses the buffers
    let mesh = Mesh::from_buffers(
        Some(vertices),
        Some(normals),
        None,
        None,
        triangles
    ).unwrap();

    // Create an indexed indirect draw buffer
    let indirect = DrawIndexedIndirectBuffer::from_slice(
        graphics,
        &[DrawIndexedIndirect {
            vertex_count: 0,
            instance_count: 1,
            base_index: 0,
            vertex_offset: 0,
            base_instance: 0,
        }],
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE
    ).unwrap();

    // Add the values to the storages
    let mesh = meshes.insert(mesh);
    let indirect = indirects.insert(indirect);

    // Create the surface for rendering
    let surface = Surface::indirect(mesh, terrain.material.clone(), indirect, terrain.id.clone());
    let renderer = Renderer::default();
    let position = Position::from(coords.as_::<f32>() * (terrain.size as f32));

    // Create the chunk component
    let chunk = Chunk {
        state: ChunkState::Pending,
        coords,
    };

    // Add the terrain chunk entity
    scene.insert((position, renderer, surface, chunk))    
}

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {  
    // Tries to find a chunk viewer and the terrain generator 
    let time = world.get::<Time>().unwrap();
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let viewer = scene.find_mut::<(&Entity, &mut ChunkViewer, &Position)>();

    // If we don't have terrain, don't do shit
    let Ok(mut terrain) = terrain else {
        return;
    };

    // If we don't have a chunk viewer, don't do shit
    let Some((entity, _, position)) = viewer else {
        terrain.viewer = None;
        return;
    };
    
    // Set the main viewer location and fetches the oldvalue
    let new = (**position / vek::Vec3::broadcast(terrain.size as f32)).as_::<i32>();
    let old = if let Some((_, old)) = &mut terrain.viewer {
        std::mem::replace(old, new)
    } else {
        terrain.viewer = Some((*entity, new));
        new
    };
    
    // Check if it moved since last frame
    if new != old {
    }

    if time.frame_count() == 10 {
        let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
        let mut indirects = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
        let graphics = world.get::<Graphics>().unwrap();
        log::debug!("manager: chunk viewer moved with delta of {}", new - old);
        spawn_chunk(&mut terrain, &mut scene, &mut meshes, &mut indirects, &graphics, vek::Vec3::zero());
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
