use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    ActiveGraphicsPipeline, BufferMode, BufferUsage, CompareFunction,
    Compiler, Depth, DepthConfig, FragmentModule, Graphics,
    GraphicsPipeline, LoadOp, Operation, PrimitiveConfig, RenderPass,
    SamplerSettings, Shader, StoreOp, Texture, Texture2D,
    TextureMipMaps, TextureMode, TextureUsage, UniformBuffer,
    VertexModule, WindingOrder,
};
use vek::FrustumPlanes;

use crate::EnabledMeshAttributes;

// This is what will write to the depth texture
pub type ShadowTexel = Depth<f32>;
pub type ShadowMap = Texture2D<ShadowTexel>;
pub type ShadowRenderPass = RenderPass<(), ShadowTexel>;
pub type ShadowGraphicsPipeline = GraphicsPipeline<(), ShadowTexel>;
pub type ActiveShadowGraphicsPipeline<'a, 'r, 't> =
    ActiveGraphicsPipeline<'a, 'r, 't, (), ShadowTexel>;

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub(crate) depth_tex: ShadowMap,
    pub(crate) render_pass: ShadowRenderPass,
    pub(crate) pipeline: ShadowGraphicsPipeline,
    pub(crate) shader: Shader,

    // Cached matrices
    pub(crate) projection: vek::Mat4<f32>,
    pub(crate) view: vek::Mat4<f32>,
    pub(crate) resolution: u32,

    // This is the corresponding data that must be sent to the shader
    pub(crate) buffer: UniformBuffer<ShadowUniform>,
}

// This is the uniform that is defined in the Vertex Module
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ShadowUniform {
    pub lightspace: vek::Vec4<vek::Vec4<f32>>,
    pub strength: f32,
    pub spread: f32,
    pub resolution: u32,
    _padding: f32,
}

impl ShadowMapping {
    // Create a new shadow mapper that will create some nice crispy shadows
    pub(crate) fn new(
        size: f32,
        depth: f32,
        resolution: u32,
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
        let mut compiler = Compiler::new(assets);
        compiler.use_uniform_buffer::<ShadowUniform>("shadow");

        // Combine the modules to the shader
        let shader =
            Shader::new(graphics, vertex, fragment, compiler)
                .unwrap();

        // Create the shadow map render pass
        let render_pass = ShadowRenderPass::new(
            graphics,
            (),
            Operation {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            },
        )
        .unwrap();

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
                EnabledMeshAttributes::POSITIONS,
            ),
            PrimitiveConfig::Triangles {
                winding_order: WindingOrder::Ccw,
                cull_face: None,
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
            TextureUsage::RENDER_TARGET | TextureUsage::SAMPLED,
            SamplerSettings::default(),
            TextureMipMaps::Disabled,
        )
        .unwrap();

        // The shadow frustum is the cuboid that will contain the shadow map
        let frustum = FrustumPlanes::<f32> {
            left: -size,
            right: size,
            bottom: -size,
            top: size,
            near: -depth / 2.0,
            far: depth / 2.0,
        };

        // Create the projection matrix and the view matrix (identity)
        let projection = vek::Mat4::orthographic_rh_zo(frustum);
        let view = vek::Mat4::identity();
        let lightspace = projection * view;

        Self {
            render_pass,
            shader,
            pipeline,
            depth_tex,
            buffer: UniformBuffer::from_slice(
                graphics,
                &[ShadowUniform {
                    lightspace: lightspace.cols,
                    strength: 1.0,
                    spread: 0.01,
                    resolution,
                    _padding: 0.0f32,
                }],
                BufferMode::Dynamic,
                BufferUsage::WRITE,
            )
            .unwrap(),
            projection,
            view,
            resolution,
        }
    }

    // Update the rotation of the sun shadows using a new rotation
    pub(crate) fn update(&mut self, rotation: vek::Quaternion<f32>) {
        let rot = vek::Mat4::from(rotation);
        let view = vek::Mat4::<f32>::look_at_rh(
            vek::Vec3::zero(),
            rot.mul_point(-vek::Vec3::unit_z()),
            rot.mul_point(-vek::Vec3::unit_y()),
        );
        self.view = view;
        let lightspace = self.projection * self.view;

        self.buffer
            .write(
                &[ShadowUniform {
                    lightspace: lightspace.cols,
                    strength: 1.0,
                    spread: 0.01,
                    resolution: self.resolution,
                    _padding: 0.0f32,
                }],
                0,
            )
            .unwrap();
    }
}
