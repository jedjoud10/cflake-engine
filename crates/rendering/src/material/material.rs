use std::{any::TypeId, marker::PhantomData};

use assets::Assets;
use ecs::EcsManager;
use math::{Location, Rotation};
use world::{Resource, Storage, World};

use crate::{
    canvas::{BlendMode, Canvas, FaceCullMode, PrimitiveMode},
    context::{Context, Window},
    mesh::{Mesh, EnabledAttributes},
    others::Comparison,
    scene::{Camera, Directional, Renderer, SceneSettings},
    shader::{Shader, Uniforms},
};

use super::Pipeline;

/*
pub struct PipelineStaticData<'a> {}
pub struct PipelineRenderData<'a> {}
pub struct PipelineInstanceData<'a> {}
*/

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material<'w>: 'static + Sized {
    // The resources that we need to fetch from the world to set the uniforms
    type Resources: 'w;

    // Load in the shader that we will use for our material pipeline
    fn shader(ctx: &mut Context, assets: &mut Assets) -> Shader;

    // These are the minimum mesh attributes that must be enabled to be able to render the surface
    // The EnabledAttributes::POSITIONS attribute will always be required 
    fn requirements() -> EnabledAttributes {
        EnabledAttributes::POSITIONS
    }

    // Get the depth comparison setting
    fn depth_comparison() -> Option<Comparison> {
        Some(Comparison::Less)
    }

    // Get the sRGB framebuffer setting
    fn srgb() -> bool {
        false
    }

    // Get the transparency setting
    fn blend_mode() -> Option<BlendMode> {
        None
    }

    // Get the rasterizer primitive mode
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Triangles {
            cull: Some(FaceCullMode::Back(true)),
        }
    }

    // Fetch the property block resources
    fn fetch(world: &'w World) -> Self::Resources;

    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties(
        _uniforms: &mut Uniforms,
        _resources: &mut Self::Resources,
        _viewport: vek::Extent2<u16>,
        _scene: &SceneSettings,
        _camera: (&Camera, &Location, &Rotation),
        _light: (&Directional, &Rotation),
    ) {
    }

    // Set the uniforms for this property block right before we render our surface
    fn set_render_properties(
        _uniforms: &mut Uniforms,
        _resources: &mut Self::Resources,
        _renderer: &Renderer,
        _camera: (&Camera, &Location, &Rotation),
        _light: (&Directional, &Rotation),
    ) {
    }

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties(
        &self,
        _uniforms: &mut Uniforms,
        _resources: &mut Self::Resources,
        _scene: &SceneSettings,
        _camera: (&Camera, &Location, &Rotation),
        _light: (&Directional, &Rotation),
    ) {
    }
}
