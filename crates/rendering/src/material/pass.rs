use crate::{
    DefaultMaterialResources, Material, Renderer, SceneColorLayout, SceneDepthLayout,
    ShadowDepthLayout, Surface,
};

use graphics::{ColorLayout, DepthStencilLayout};

// Render pass that will render the scene
pub struct DeferredPass;

// Render pass that will render the shadows of the scene from the light POV
pub struct ShadowPass;

// Type of render pass
pub enum PassType {
    Deferred,
    Shadow,
}

// Stats about objects and surfaces drawn for any given pass
#[derive(Default, Clone, Copy)]
pub struct PassStats {
    pub material_instance_swap: usize,
    pub mesh_instance_swap: usize,
    pub rendered_direct_vertices_drawn: u64,
    pub rendered_direct_triangles_drawn: u64,
    pub culled_sub_surfaces: usize,
    pub rendered_sub_surfaces: usize,
}

// Generalized render pass from within the rendering system
// This will be implemented for the DeferredPass and ShadowPass structs
pub trait Pass {
    // Render-pass color and depth/stencil layouts
    type C: ColorLayout;
    type DS: DepthStencilLayout;

    // Check if the pass is the deferred pass
    fn is_deferred_pass() -> bool {
        matches!(Self::pass_type(), PassType::Deferred)
    }

    // Check if the pass is the shadow pass
    fn is_shadow_pass() -> bool {
        matches!(Self::pass_type(), PassType::Shadow)
    }

    // Set the cull state of a specific surface
    fn set_cull_state<M: Material>(surface: &mut Surface<M>, culled: bool);

    // Cull a specific surface when rendering the objects of this pass
    fn cull(
        defaults: &DefaultMaterialResources,
        aabb: math::Aabb<f32>,
        mesh: &vek::Mat4<f32>,
    ) -> bool;

    // Check if a surface should be visible
    fn is_surface_visible<M: Material>(surface: &Surface<M>, renderer: &Renderer) -> bool;

    // Get the pass type
    fn pass_type() -> PassType;
}

impl Pass for DeferredPass {
    type C = SceneColorLayout;
    type DS = SceneDepthLayout;

    #[inline(always)]
    fn pass_type() -> PassType {
        PassType::Deferred
    }

    #[inline(always)]
    fn set_cull_state<M: Material>(surface: &mut Surface<M>, culled: bool) {
        surface.culled = culled;
    }

    #[inline(always)]
    fn cull(
        defaults: &DefaultMaterialResources,
        aabb: math::Aabb<f32>,
        mesh: &vek::Mat4<f32>,
    ) -> bool {
        !crate::pipeline::intersects_frustum(&defaults.camera_frustum, aabb, mesh)
    }

    #[inline(always)]
    fn is_surface_visible<M: Material>(surface: &Surface<M>, renderer: &Renderer) -> bool {
        !surface.culled && surface.visible && renderer.visible
    }
}

impl Pass for ShadowPass {
    type C = ();
    type DS = ShadowDepthLayout;

    #[inline(always)]
    fn pass_type() -> PassType {
        PassType::Shadow
    }

    #[inline(always)]
    fn set_cull_state<M: Material>(surface: &mut Surface<M>, culled: bool) {
        surface.shadow_culled = culled;
    }

    #[inline(always)]
    fn cull(
        defaults: &DefaultMaterialResources,
        aabb: math::Aabb<f32>,
        mesh: &vek::Mat4<f32>,
    ) -> bool {
        !crate::pipeline::intersects_lightspace(defaults.lightspace.as_ref().unwrap(), aabb, mesh)
    }

    #[inline(always)]
    fn is_surface_visible<M: Material>(surface: &Surface<M>, renderer: &Renderer) -> bool {
        !surface.shadow_culled && surface.visible && renderer.visible
    }
}
