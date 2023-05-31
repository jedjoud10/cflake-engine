use assets::Assets;
use graphics::{ColorLayout, DepthStencilLayout, RenderPass, BindGroup, Graphics, Shader, PrimitiveConfig, WindingOrder, PushConstants, ActiveRenderPipeline, ActivePipeline, Depth, RenderPipeline};
use world::World;
use crate::{RenderPath, DefaultMaterialResources, SceneColorLayout, MeshAttributes, Renderer, SceneDepthLayout};

// Render pass that will render the scene
pub struct DeferredPass;

// Render pass that will render the shadows of the scene from the light POV
pub struct ShadowPass;

// Generalized render pass from within the rendering system
// This will be implemented for the DeferredPass and ShadowPass structs
pub trait Pass {
    // Render-pass color and depth/stencil layouts
    type C: ColorLayout;
    type DS: DepthStencilLayout;

    // Check if the pass is the deferred pass
    fn is_deferred_pass() -> bool;

    // Check if the pass is the shadow pass
    fn is_shadow_pass() -> bool;
}

impl Pass for DeferredPass {
    type C = SceneColorLayout;
    type DS = SceneDepthLayout;

    fn is_deferred_pass() -> bool {
        true
    }

    fn is_shadow_pass() -> bool {
        false
    }
}

impl Pass for ShadowPass {
    type C = ();
    type DS = Depth<f32>;

    fn is_deferred_pass() -> bool {
        false
    }

    fn is_shadow_pass() -> bool {
        true
    }
}