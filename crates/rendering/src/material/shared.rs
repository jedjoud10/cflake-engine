use bytemuck::{Pod, Zeroable};
use graphics::{
    DrawCountIndirectBuffer, DrawIndexedIndirectBuffer, Normalized, Texture2D, TriangleBuffer,
    UniformBuffer, RG, RGBA, SRGBA,
};
use math::Frustum;
use utils::Storage;

use crate::{
    attributes, AttributeBuffer, EnvironmentMap, IndirectMesh, Mesh,
    MultiDrawIndirectCountMesh, MultiDrawIndirectMesh,
};

// These are the default settings that we pass to each material
pub struct DefaultMaterialResources<'a> {
    // Main scene uniform buffers
    // TODO: Make use of crevice to implement Std130, Std140
    pub camera_buffer: &'a CameraBuffer,
    pub timing_buffer: &'a TimingBuffer,
    pub scene_buffer: &'a SceneBuffer,

    // Main camera values
    pub camera: crate::Camera,
    pub camera_frustum: Frustum<f32>,
    pub camera_position: coords::Position,
    pub camera_rotation: coords::Rotation,

    // Main directional light values
    pub directional_light: crate::DirectionalLight,
    pub directional_light_rotation: coords::Rotation,

    // Main scene textures
    pub white: &'a AlbedoMap,
    pub black: &'a AlbedoMap,
    pub normal: &'a NormalMap,
    pub mask: &'a MaskMap,

    // Envinronment maps
    pub environment_map: &'a EnvironmentMap,

    // Common direct mesh storages
    pub meshes: &'a Storage<Mesh>,

    // Common indirect mesh storages
    pub indirect_meshes: &'a Storage<IndirectMesh>,
    pub multi_draw_indirect_meshes: &'a Storage<MultiDrawIndirectMesh>,
    pub multi_draw_indirect_count_meshes: &'a Storage<MultiDrawIndirectCountMesh>,

    // Common indirect attribute storages
    pub indirect_positions: &'a Storage<AttributeBuffer<attributes::Position>>,
    pub indirect_normals: &'a Storage<AttributeBuffer<attributes::Normal>>,
    pub indirect_tangents: &'a Storage<AttributeBuffer<attributes::Tangent>>,
    pub indirect_tex_coords: &'a Storage<AttributeBuffer<attributes::TexCoord>>,
    pub indirect_triangles: &'a Storage<TriangleBuffer<u32>>,

    pub draw_indexed_indirect_buffers: &'a Storage<DrawIndexedIndirectBuffer>,
    pub draw_count_indirect_buffer: &'a Storage<DrawCountIndirectBuffer>,

    // Lightspace matrix for shadows
    pub lightspace: Option<vek::Mat4<f32>>,

    /*
    pub drawn_unique_material_count: &'a mut u32,
    pub material_instances_count: &'a mut u32,
    pub rendered_direct_vertices_drawn: &'a mut u64,
    pub rendered_direct_triangles_drawn: &'a mut u64,
    pub culled_sub_surfaces: &'a mut u64,
    pub rendered_sub_surfaces: &'a mut u64,
    */
}

// Camera data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct CameraUniform {
    // Projection & view matrix
    pub projection: vek::Vec4<vek::Vec4<f32>>,
    pub view: vek::Vec4<vek::Vec4<f32>>,

    // Position and direction vectors
    pub position: vek::Vec4<f32>,
    pub forward: vek::Vec4<f32>,
    pub right: vek::Vec4<f32>,
    pub up: vek::Vec4<f32>,
}

// Timing data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TimingUniform {
    pub frame_count: u32,
    pub delta_time: f32,
    pub time_since_startup: f32,
}

// Scene data that will be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct SceneUniform {
    // Sun related parameters
    pub sun_direction: vek::Vec4<f32>,
    pub sun_color: vek::Rgba<f32>,

    // Ambient color of the environment
    pub ambient_color_strength: f32,

    // Procedural sun parameters
    pub sun_circle_strength: f32,
    pub sun_circle_size: f32,
    pub sun_circle_fade: f32,
}

// Window/monitor data thw ill be stored in a UBO
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C, align(4))]
pub struct WindowUniform {
    pub width: u32,
    pub height: u32,
}

// Type aliases for texels
//pub type AlbedoTexel = SRGBA<Normalized<graphics::UBC1>>;
pub type AlbedoTexel = SRGBA<Normalized<u8>>;
pub type NormalTexel = RG<Normalized<u8>>;
pub type MaskTexel = RGBA<Normalized<u8>>;

// Type aliases for textures
pub type AlbedoMap = Texture2D<AlbedoTexel>;
pub type NormalMap = Texture2D<NormalTexel>;
pub type MaskMap = Texture2D<MaskTexel>;

// Type aliases for buffers
pub type CameraBuffer = UniformBuffer<CameraUniform>;
pub type TimingBuffer = UniformBuffer<TimingUniform>;
pub type SceneBuffer = UniformBuffer<SceneUniform>;
pub type WindowBuffer = UniformBuffer<WindowUniform>;
