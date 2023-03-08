use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{Texture2D, Normalized, Depth, GraphicsPipeline, RenderPass, Shader, Graphics, VertexModule, FragmentModule, Compiler, Operation, LoadOp, StoreOp, PrimitiveConfig, Texture, TextureMode, TextureUsage, SamplerSettings, TextureMipMaps, ActiveGraphicsPipeline};
use vek::FrustumPlanes;

use crate::EnabledMeshAttributes;

// This is what will write to the depth texture
pub type ShadowTexel = Depth<f32>;
pub type ShadowMap = Texture2D<ShadowTexel>;
pub type ShadowRenderPass = RenderPass<(), ShadowTexel>;
pub type ShadowGraphicsPipeline =
    GraphicsPipeline<(), ShadowTexel>;
pub type ActiveShadowGraphicsPipeline<'a, 'r, 't> 
    = ActiveGraphicsPipeline<'a, 'r, 't, (), ShadowTexel>;

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    // Everything required to render to the depth texture
    pub(crate) depth_tex: ShadowMap,
    pub(crate) render_pass: ShadowRenderPass,
    pub(crate) pipeline: ShadowGraphicsPipeline,
    pub(crate) shader: Shader,    

    // This is the corresponding data that must be sent to the shader
    pub(crate) view_matrix: vek::Mat4<f32>,
    pub(crate) proj_matrix: vek::Mat4<f32>,
}

// This is the uniform that is defined in the Vertex Module
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C, align(64))]
pub struct ShadowUniform {
    pub projection: vek::Vec4<vek::Vec4<f32>>,
    pub view: vek::Vec4<vek::Vec4<f32>>,
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
            .load::<VertexModule>("engine/shaders/scene/shadow/shadow.vert")
            .unwrap();

        // Load the fragment module for the shadowmap shader
        let fragment = assets
            .load::<FragmentModule>("engine/shaders/scene/shadow/shadow.frag").unwrap();

        // Create the bind layout for the shadow map shader
        let mut compiler = Compiler::new(assets);
        compiler.use_ubo::<ShadowUniform>("ubo");

        // Combine the modules to the shader
        let shader = Shader::new(
            graphics,
            vertex,
            fragment,
            compiler
        ).unwrap();

        // Create the shadow map render pass
        let render_pass = ShadowRenderPass::new(
            graphics,
            (),
            Operation {
                load: LoadOp::Clear(0.0),
                store: StoreOp::Store,
            },
        )
        .unwrap();

        // Create the shadow map graphics pipeline
        let pipeline = ShadowGraphicsPipeline::new(
            graphics,
            None,
            None,
            None,
            crate::attributes::enabled_to_vertex_config(
                EnabledMeshAttributes::POSITIONS,
            ),
            PrimitiveConfig::Triangles {
                winding_order: graphics::WindingOrder::Ccw,
                cull_face: None,
                wireframe: false,
            },
            &shader,
        ).unwrap();

        // Create the depth texture that we will render to
        let depth_tex = Texture2D::<Depth::<f32>>::from_texels(
            graphics,
            None,
            vek::Extent2::broadcast(resolution),
            TextureMode::Dynamic,
            TextureUsage::RENDER_TARGET | TextureUsage::SAMPLED,
            SamplerSettings::default(),
            TextureMipMaps::Disabled
        ).unwrap();

        // The shadow frustum is the cuboid that will contain the shadow map
        let frustum = FrustumPlanes::<f32> {
            left: -size,
            right: size,
            bottom: -size,
            top: size,
            near: -depth / 2.0,
            far: depth / 2.0,
        };

        // Create the projection matrix from the frustum
        let proj_matrix = vek::Mat4::orthographic_rh_no(frustum);

        Self {
            render_pass,
            shader,
            pipeline,
            depth_tex,
            view_matrix: vek::Mat4::identity(),
            proj_matrix,
        }
    }
}
