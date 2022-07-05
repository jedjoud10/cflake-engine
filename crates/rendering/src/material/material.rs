use std::{marker::PhantomData, rc::Rc};

use assets::Assets;
use ecs::EcsManager;
use math::Transform;
use world::{Handle, Storage, World};

use crate::{
    canvas::{Canvas, FaceCullMode, PrimitiveMode, RasterSettings, BlendMode},
    context::{Context, Device, Graphics},
    mesh::{SubMesh, Surface},
    others::Comparison,
    scene::{Camera, Directional, Renderer, SceneSettings},
    shader::{Shader, Uniforms},
};

use super::{Pipeline, Standard, Stats};

// This is an Instance ID that will be stored within the materials
// By itself it does nothing, but we use it internally to make sure that the underlying material was created through a material builder
pub struct InstanceID<M: Material>(PhantomData<M>);

impl<M: Material> InstanceID<M> {
    // This will create a new instance ID for a specific material by registering it's pipeline
    pub fn new(
        ctx: &mut Context,
        assets: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> InstanceID<M> {
        ctx.register_pipeline::<M>(assets, storage);
        InstanceID(Default::default())
    }
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: 'static + Sized + for<'a> PropertyBlock<'a> {
    // The material pipeline that this material will use
    type Pipeline: Pipeline;

    // Create a new material pipeline for this material type. This should be called once
    fn pipeline(
        ctx: &mut Context,
        assets: &mut Assets,
        storage: &mut Storage<Shader>,
    ) -> Self::Pipeline;

    // Get the instance ID of the material instance
    fn id(&self) -> &InstanceID<Self>;

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

    // Get the face culling mode 
    fn face_cull_mode() -> Option<FaceCullMode> {
        Some(FaceCullMode::Back(true))
    }
}

// A property block is an interface that tells us exactly we should set the material properties when using shader batching
// This will be implemented for ALL material types, since they all use shader batching
// TODO: Remove this whole trait by merging it into the material trait somehow...
pub trait PropertyBlock<'w>: Sized {
    // The resources that we need to fetch from the world to set the uniforms
    type Resources: 'w;

    // Fetch the default rendering resources and the material property block resources as well
    fn fetch(
        world: &'w mut World,
    ) -> (
        &'w SceneSettings,
        &'w EcsManager,
        &'w Storage<Self>,
        &'w Storage<SubMesh>,
        &'w mut Storage<Shader>,
        &'w mut Graphics,
        Self::Resources,
    );

    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        canvas: &Canvas,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
    }

    // Set the uniforms for this property block right before we render our surface
    fn set_render_properties<'u>(
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        renderer: &Renderer,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u,
    {
    }

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties<'u>(
        &'w self,
        uniforms: &mut Uniforms<'u>,
        resources: &mut Self::Resources,
        scene: &SceneSettings,
        camera: (&Camera, &Transform),
        light: (&Directional, &Transform),
    ) where
        'w: 'u;
}
