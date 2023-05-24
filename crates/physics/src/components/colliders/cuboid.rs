use ecs::Component;

// Cuboid colliders represent a cuboid in 3D space
// The position and rotation of the cuboid will be fetched from it's Position component and Rotation component
#[derive(Component)]
pub struct CuboidCollider {
    // Full extent of the cuboid
    pub extent: vek::Extent3<f32>,
}