

use arrayvec::ArrayVec;
use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, CompareFunction, Compiler,
    Depth, DepthConfig, Face, FragmentModule, GpuPod, Graphics, LayeredTexture2D, LoadOp,
    ModuleVisibility, Operation, PrimitiveConfig, PushConstantLayout, RenderPass,
    RenderPipeline, SamplerSettings, Shader, StoreOp, Texture, TextureMipMaps,
    TextureMode, TextureUsage, UniformBuffer, VertexModule, WindingOrder, Buffer,
};
use math::ExplicitVertices;
use vek::FrustumPlanes;

use crate::{MeshAttributes, create_uniform_buffer};

// This is what will write to the depth texture
pub type ShadowDepthLayout = Depth<f32>;
pub type ShadowMap = LayeredTexture2D<ShadowDepthLayout>;

// Create a cascaded depth texture with 4 layers
fn create_depth_texture(graphics: &Graphics, resolution: u32) -> LayeredTexture2D<Depth<f32>> {
    ShadowMap::from_texels(
        graphics,
        None,
        (vek::Extent2::broadcast(resolution), 4),
        TextureMode::Dynamic,
        TextureUsage::TARGET | TextureUsage::SAMPLED,
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub(crate) render_pass: RenderPass<(), ShadowDepthLayout>,

    // Multilayered shadow map texture
    pub(crate) depth_tex: ShadowMap,

    // Cached matrices
    pub percents: [f32; 4],
    pub distance: f32,

    // Resolution of the base level
    pub(crate) resolution: u32,
    pub(crate) depth: f32,

    pub(crate) last_camera_position: vek::Vec3<f32>,

    // Contains shadow parameters
    pub parameters: ShadowUniform,
    pub(crate) parameter_buffer: UniformBuffer<ShadowUniform>,

    // Contains the light space shadow matrices
    pub(crate) lightspace_buffer: UniformBuffer<vek::Vec4<vek::Vec4<f32>>>,
}

// This is the uniform that is defined in the Vertex Module
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ShadowUniform {
    pub strength: f32,
    pub spread: f32,
    pub base_bias: f32,
	pub bias_bias: f32,
	pub bias_factor_base: f32,
	pub normal_offset: f32,
}

impl ShadowMapping {
    // Create a new shadow mapper that will create some nice crispy shadows
    pub(crate) fn new(
        depth: f32,
        resolution: u32,
        percents: [f32; 4],
        distance: f32,
        graphics: &Graphics,
    ) -> Self {
        // Create the shadow map render pass
        let render_pass = RenderPass::<(), ShadowDepthLayout>::new(
            graphics,
            (),
            Operation {
                load: LoadOp::Clear(f32::MAX),
                store: StoreOp::Store,
            },
        );

        let depth_tex = create_depth_texture(graphics, resolution);

        // Default shadow parameters
        let parameters = ShadowUniform {
            strength: 1.0,
            spread: 0.47,
            base_bias: -0.0001,
            bias_bias: 0.0,
            bias_factor_base: 1.50,
            normal_offset: 0.0,
        };

        // Create a buffer that will contain shadow parameters
        let parameter_buffer = UniformBuffer::from_slice(
            graphics,
            &[parameters],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        )
        .unwrap();

        let lightspace_buffer = create_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>, 4>(graphics, BufferUsage::WRITE);

        Self {
            render_pass,
            depth_tex,
            resolution,
            parameter_buffer,
            lightspace_buffer,
            depth,
            distance,
            percents,
            parameters,
            last_camera_position: Default::default(),
        }
    }

    // Update the lightspace matrix of a single cascade within the shadowmap
    // This will update only one of the two buffers if ShadowMappingRefreshRate is not set to WholeEveryFrame
    pub(crate) fn update(
        &mut self,
        light_rotation: vek::Quaternion<f32>,
        camera_position: vek::Vec3<f32>,
        cascade: usize,
    ) -> vek::Mat4<f32> {
        if self.last_camera_position.distance(camera_position) > 1.0 {
            self.last_camera_position = camera_position;
        }

        let val = self.percents[cascade] * self.distance;
        let frustum = FrustumPlanes::<f32> {
            left: -val,
            right: val,
            bottom: -val,
            top: val,
            near: -self.depth,
            far: self.depth,
        };
        
        // Calculate a new view matrix and set it
        let rot = vek::Mat4::from(light_rotation);

        // Calculate light view matrix
        let view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );

        // Create the projection matrix (orthographic)
        let projection = vek::Mat4::orthographic_rh_zo(frustum);

        // Calculate light skin rizz (real) (I have gone insane)
        let lightspace = projection * view * vek::Mat4::translation_3d(-self.last_camera_position);
        self.lightspace_buffer.write(&[lightspace.cols], cascade).unwrap();
        self.parameter_buffer.write(&[self.parameters], 0).unwrap();
        lightspace
    }
}