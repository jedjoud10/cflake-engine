use assets::Assets;

use math::{Location, Rotation};
use world::World;

use crate::{
    buffer::UniformBuffer,
    context::{Context, Window},
    display::{BlendMode, FaceCullMode, PrimitiveMode},
    mesh::EnabledAttributes,
    others::Comparison,
    scene::{Camera, DirectionalLight, PointLight, Renderer},
    shader::{Shader, Uniforms},
};

// These are the default resources that we pass to any/each material
pub struct DefaultMaterialResources<'a> {
    pub(crate) camera: &'a Camera,
    pub(crate) point_lights: &'a UniformBuffer<(PointLight, Location)>,
    pub(crate) camera_location: &'a Location,
    pub(crate) camera_rotation: &'a Rotation,
    pub(crate) directional_light: &'a DirectionalLight,
    pub(crate) directional_light_rotation: &'a Rotation,
    pub(crate) window: &'a Window,
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

    // Should we assume that the shader instance is always valid?
    unsafe fn should_assume_valid() -> bool {
        false
    }

    // Should we use frustum culling when rendering surfaces of this type?
    fn should_use_frustum_culling() -> bool {
        true
    }

    // Fetch the property block resources
    fn fetch_resources(world: &'w World) -> Self::Resources;

    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
    );

    // Set the uniforms for this property block right before we render our surface
    fn set_surface_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        renderer: &Renderer,
    );

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        instance: &Self,
    );
}
