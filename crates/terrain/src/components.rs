use ecs::Component;

// This is a terrain chunk component that will be automatically added 
// on entities that are generated from the terrain generator
#[derive(Component)]
pub struct Chunk {
}