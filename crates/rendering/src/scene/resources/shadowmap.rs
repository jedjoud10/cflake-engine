use std::num::{NonZeroU32, NonZeroU8};

use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    ActiveGraphicsPipeline, BufferMode, BufferUsage, CompareFunction,
    Compiler, Depth, DepthConfig, FragmentModule, GpuPod,
    Graphics, LoadOp, ModuleVisibility, Operation, PrimitiveConfig,
    PushConstantLayout, RenderPass, RenderPipeline, SamplerSettings,
    Shader, StoreOp, Texture, Texture2D, TextureMipMaps, TextureMode,
    TextureUsage, UniformBuffer, VertexModule, WindingOrder, Face, LayeredTexture2D, Normalized,
};
use vek::FrustumPlanes;

use crate::MeshAttributes;

// This is what will write to the depth texture
pub type ShadowTexel = Depth<Normalized<u16>>;
pub type ShadowMap = LayeredTexture2D<ShadowTexel>;
pub type ShadowRenderPass = RenderPass<(), ShadowTexel>;
pub type ShadowGraphicsPipeline = RenderPipeline<(), ShadowTexel>;
pub type ActiveShadowGraphicsPipeline<'a, 'r, 't> =
    ActiveGraphicsPipeline<'a, 'r, 't, (), ShadowTexel>;

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub render_pass: ShadowRenderPass,
    pub pipeline: ShadowGraphicsPipeline,
    pub shader: Shader,
    
    // Multilayered shadow map texture
    pub depth_tex: ShadowMap,
    
    // Cached matrices
    pub view: vek::Mat4<f32>,
    pub projections: Vec<vek::Mat4<f32>>,

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
}

impl ShadowMapping {
    // Create a new shadow mapper that will create some nice crispy shadows
    pub(crate) fn new(
        depth: f32,
        resolution: u32,
        sizes: &[f32],
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

        // Contains the mesh matrix and the lightspace uniforms
        let layout = PushConstantLayout::single(
            <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size() * 2,
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
                load: LoadOp::Clear(u16::MAX),
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
                cull_face: Some(Face::Back),
                wireframe: false,
            },
            &shader,
        )
        .unwrap();

        // Create the depth textures that we will render to
        let depth_tex = ShadowMap::from_texels(
            graphics,
            None,
            (vek::Extent2::broadcast(resolution), sizes.len() as u32),
            TextureMode::Dynamic,
            TextureUsage::TARGET | TextureUsage::SAMPLED,
            Some(SamplerSettings::default()),
            TextureMipMaps::Disabled,
        )
        .unwrap();

        // Create a buffer that will contain shadow parameters
        let parameter_buffer = UniformBuffer::from_slice(
            graphics,
            &[ShadowUniform {
                strength: 1.0,
                spread: 0.01,
            }],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        ).unwrap();

        // We can initialize these to zero since the first frame would update the buffer anyways
        let lightspace_buffer = UniformBuffer::<vek::Vec4<vek::Vec4<f32>>>::zeroed(
            graphics,
            sizes.len(),
            BufferMode::Dynamic,
            BufferUsage::WRITE | BufferUsage::STORAGE,
        ).unwrap();

        // Pre-initialize the projection matrices
        let projections = sizes.iter().map(|&size| {
            // The shadow frustum is the cuboid that will contain the shadow map
            let frustum = FrustumPlanes::<f32> {
                left: -size,
                right: size,
                bottom: -size,
                top: size,
                near: -depth / 2.0,
                far: depth / 2.0,
            };

            // Create the projection matrix (orthographic)
            vek::Mat4::orthographic_rh_zo(frustum)
        }).collect::<Vec<_>>();

        Self {
            render_pass,
            shader,
            pipeline,
            depth_tex,
            view: vek::Mat4::identity(),
            resolution,
            parameter_buffer,
            lightspace_buffer,
            depth,
            projections,
        }
    }

    // Update the rotation of the sun shadows using a new rotation
    // Returns the newly created lightspace matrix (only one)
    pub(crate) fn update(
        &mut self,
        rotation: vek::Quaternion<f32>,
        camera: (vek::Vec3<f32>, vek::Quaternion<f32>),
        frustum: math::Frustum<f32>,
        i: u32,
    ) -> vek::Mat4<f32> {
        // Calculate a new view matrix and set it
        let rot = vek::Mat4::from(rotation);
        self.view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );
        let camera = vek::Mat4::<f32>::translation_3d(-camera.0);

        // TODO: Do funky shit with the matrix
        
        // Update ONE of the internally stored lightspace matrices
        let projection = &self.projections[i as usize];

        // Calculate light skin rizz (real) (I have gone insane)
        let lightspace = *projection * self.view * camera;

        // Update the internally stored buffer
        self.lightspace_buffer.write(&[lightspace.cols],i as usize).unwrap();
        lightspace
    }
}
