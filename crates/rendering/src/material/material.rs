use std::{any::TypeId, marker::PhantomData};

use assets::Assets;
use ecs::Scene;
use math::{Location, Rotation};
use world::{Resource, Storage, World};

use crate::{
    canvas::{BlendMode, Canvas, FaceCullMode, PrimitiveMode},
    context::{Context, Window},
    mesh::{Mesh, EnabledAttributes},
    others::Comparison,
    scene::{Camera, DirectionalLight, Renderer},
    shader::{Shader, Uniforms},
};

pub struct PipelineDefaultResources<'a> {
    camera: &'a Camera,
    camera_location: &'a Location,
    camera_rotation: &'a Rotation,
    directional_light: &'a DirectionalLight,
    directional_light_rotation: &'a Rotation,
}

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
        uniforms: &mut Uniforms,
        resources: &mut Self::Resources,
    );

    // Set the uniforms for this property block right before we render our surface
    fn set_render_properties(
        uniforms: &mut Uniforms,
        resources: &mut Self::Resources,
        renderer: &Renderer,
    );

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties(
        instance: &Self,
        uniforms: &mut Uniforms,
        resources: &mut Self::Resources,
    );
}
