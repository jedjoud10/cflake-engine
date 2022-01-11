// A camera object that can be sent to the render thread to update the main camera that is rendering the world
#[derive(Default)]
pub struct Camera {
    position: veclib::Vector3<f32>,
    rotation: veclib::Quaternion<f32>,
    clip_planes: veclib::Vector2<f32>,
    viewm: veclib::Matrix4x4<f32>,
    projm: veclib::Matrix4x4<f32>,    
}