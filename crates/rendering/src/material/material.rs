use std::{any::TypeId, marker::PhantomData};

use assets::Assets;
use ecs::EcsManager;
use math::Transform;

use world::{Resource, Storage, World};

use crate::{
    canvas::{BlendMode, Canvas, FaceCullMode, PrimitiveMode},
    context::{Context, Window},
    mesh::{Mesh},
    others::Comparison,
    scene::{Camera, Directional, Renderer, SceneSettings},
    shader::{Shader, Uniforms},
};

use super::Pipeline;


// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material<'w>: 'static + Sized {
    // The resources that we need to fetch from the world to set the uniforms
    type Resources: 'w;

    /*
    // Minimum mesh buffers required to render this material
    fn required() -> MeshBuffers {
        MeshBuffers::INDICES | MeshBuffers::POSITIONS
    }
    */

    // Load in the shader that we will use for our material pipeline
    fn shader(ctx: &mut Context, assets: &mut Assets) -> Shader;

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
        PrimitiveMode::Triangles { cull: Some(FaceCullMode::Back(true)) }
    }
    
    // Fetch the default rendering resources and the material property block resources as well
    fn fetch(
        world: &'w mut World,
    ) -> (
        &'w SceneSettings,
        &'w EcsManager,
        &'w Storage<Self>,
        &'w Storage<Mesh>,
        &'w mut Storage<Shader>,
        &'w mut Window,
        &'w mut Context,
        Self::Resources,
    );

    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties<'u>(
        _uniforms: &mut Uniforms<'u>,
        _resources: &mut Self::Resources,
        _canvas: &Canvas,
        _scene: &SceneSettings,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
    }

    // Set the uniforms for this property block right before we render our surface
    fn set_render_properties<'u>(
        _uniforms: &mut Uniforms<'u>,
        _resources: &mut Self::Resources,
        _renderer: &Renderer,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
    }

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties<'u>(
        &'w self,
        _uniforms: &mut Uniforms<'u>,
        _resources: &mut Self::Resources,
        _scene: &SceneSettings,
        _camera: (&Camera, &Transform),
        _light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
    }
}
