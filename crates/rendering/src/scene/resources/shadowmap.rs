use std::num::{NonZeroU32, NonZeroU8};

use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    ActiveGraphicsPipeline, BufferMode, BufferUsage, CompareFunction,
    Compiler, Depth, DepthConfig, FragmentModule, GpuPod,
    Graphics, LoadOp, ModuleVisibility, Operation, PrimitiveConfig,
    PushConstantLayout, RenderPass, RenderPipeline, SamplerSettings,
    Shader, StoreOp, Texture, Texture2D, TextureMipMaps, TextureMode,
    TextureUsage, UniformBuffer, VertexModule, WindingOrder, Face,
};
use vek::FrustumPlanes;

use crate::MeshAttributes;

// This is what will write to the depth texture
pub type ShadowTexel = Depth<f32>;
pub type ShadowMap = Texture2D<ShadowTexel>;
pub type ShadowRenderPass = RenderPass<(), ShadowTexel>;
pub type ShadowGraphicsPipeline = RenderPipeline<(), ShadowTexel>;
pub type ActiveShadowGraphicsPipeline<'a, 'r, 't> =
    ActiveGraphicsPipeline<'a, 'r, 't, (), ShadowTexel>;

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub depth_tex: ShadowMap,
    pub render_pass: ShadowRenderPass,
    pub pipeline: ShadowGraphicsPipeline,
    pub shader: Shader,

    // Cached matrices
    pub view: vek::Mat4<f32>,

    // Contains the frustum percentage of each cascade
    pub percentages: Vec<f32>,

    // Resolution of the base level
    pub resolution: u32,
    pub depth: f32,

    // Contains shadow parameters
    pub parameter_buffer: UniformBuffer<ShadowUniform>,

    // Contains the light space shadow matrices
    pub lightspace_buffer: UniformBuffer<vek::Vec4<vek::Vec4<f32>>>,
}

// This is the uniform that is defined in the Vertex Module
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ShadowUniform {
    pub strength: f32,
    pub spread: f32,
    pub resolution: u32,
}

impl ShadowMapping {
    // Create a new shadow mapper that will create some nice crispy shadows
    pub(crate) fn new(
        depth: f32,
        resolution: u32,
        percentages: &[f32],
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Self {
        // Load the vertex module for the shadowmap shader
        let vertex = assets
            .load::<VertexModule>(
                "engine/shaders/scene/shadow/shadow.vert",
            )
            .unwrap();

        // Load the fragment module for the shadowmap shader
        let fragment = assets
            .load::<FragmentModule>(
                "engine/shaders/scene/shadow/shadow.frag",
            )
            .unwrap();

        // Create the bind layout for the shadow map shader
        let mut compiler = Compiler::new(assets, graphics);
    
        // Shadow parameters
        compiler.use_uniform_buffer::<ShadowUniform>("shadow_parameters");
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("shadow_lightspace_matrices");

        // Contains the mesh matrix and the lightspace uniforms
        let layout = PushConstantLayout::single(
            <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size() + u32::size(),
            ModuleVisibility::Vertex,
        )
        .unwrap();
        compiler.use_push_constant_layout(layout);

        // Combine the modules to the shader
        let shader = Shader::new(vertex, fragment, compiler).unwrap();

        // Create the shadow map render pass
        let render_pass = ShadowRenderPass::new(
            graphics,
            (),
            Operation {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            },
        );

        // Create the shadow map graphics pipeline
        let pipeline = ShadowGraphicsPipeline::new(
            graphics,
            Some(DepthConfig {
                compare: CompareFunction::LessEqual,
                write_enabled: true,
                depth_bias_constant: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            None,
            None,
            crate::attributes::enabled_to_vertex_config(
                MeshAttributes::POSITIONS,
            ),
            PrimitiveConfig::Triangles {
                winding_order: WindingOrder::Ccw,
                cull_face: Some(Face::Front),
                wireframe: false,
            },
            &shader,
        )
        .unwrap();

        // Create the depth texture that we will render to
        let depth_tex = Texture2D::<Depth<f32>>::from_texels(
            graphics,
            None,
            vek::Extent2::broadcast(resolution),
            TextureMode::Dynamic,
            TextureUsage::TARGET | TextureUsage::SAMPLED,
            SamplerSettings::default(),
            TextureMipMaps::Zeroed { clamp: Some(NonZeroU8::new(percentages.len() as u8).unwrap()) },
        )
        .unwrap();

        let view = vek::Mat4::identity();

        // Create a buffer that will contain shadow parameters
        let parameter_buffer = UniformBuffer::from_slice(
            graphics,
            &[ShadowUniform {
                strength: 1.0,
                spread: 0.01,
                resolution,
            }],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        ).unwrap();

        // We can initialize these to zero since the first frame would update the buffer anyways
        let lightspace_buffer = UniformBuffer::<vek::Vec4<vek::Vec4<f32>>>::zeroed(
            graphics,
            percentages.len(),
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        ).unwrap();

        Self {
            render_pass,
            shader,
            pipeline,
            depth_tex,
            view,
            resolution,
            percentages: percentages.to_vec(),
            parameter_buffer,
            lightspace_buffer,
            depth,
        }
    }

    // Update the rotation of the sun shadows using a new rotation
    pub(crate) fn update(
        &mut self,
        rotation: vek::Quaternion<f32>,
        camera: vek::Vec3<f32>,
        frustum: math::Frustum<f32>,
    ) {
        // Calculate a new view matrix and set it
        let rot = vek::Mat4::from(rotation);
        self.view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );
        
        // Use the camera frustum to calculate it's corners to be able to make each cascade fit the camera nicely

        /*
        // Create some new lightspace matrices using new parameters
        let matrices = self.cascade_sizes.iter().map(|&size| {
            // The shadow frustum is the cuboid that will contain the shadow map
            let frustum = FrustumPlanes::<f32> {
                left: -size,
                right: size,
                bottom: -size,
                top: size,
                near: -self.depth / 2.0,
                far: self.depth / 2.0,
            };

            // Create the projection matrix (orthographic)
            let proj = vek::Mat4::orthographic_rh_zo(frustum);

            // Calculate light skin rizz (real) (I have gone insane)
            let mat = proj * self.view;

            // Convert to vec vec matrix
            mat.cols
        }).collect::<Vec<_>>();
        self.lightspace_buffer.write(&matrices, 0).unwrap()
        */
    }
}
