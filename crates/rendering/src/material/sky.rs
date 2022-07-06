use super::{Material, BatchedPipeline, Pipeline};

// This is the material that our skysphere/skybox will use for rendering
// TODO: Implemented HDRi sky material and sheit
pub struct Sky {
}

impl<'w> Material<'w> for Sky {
    type Resources = ();

    type Pipeline = BatchedPipeline<Self>;

    fn pipeline(
        ctx: &mut crate::context::Context,
        assets: &mut assets::Assets,
        storage: &mut world::Storage<crate::prelude::Shader>,
    ) -> Self::Pipeline {
        <Self::Pipeline as Pipeline>::new(todo!())
    }

    fn fetch(
        world: &'w mut world::World,
    ) -> (
        &'w crate::scene::SceneSettings,
        &'w ecs::EcsManager,
        &'w world::Storage<Self>,
        &'w world::Storage<crate::mesh::SubMesh>,
        &'w mut world::Storage<crate::shader::Shader>,
        &'w mut crate::context::Graphics,
        Self::Resources,
    ) {
        todo!()
    }

    fn set_instance_properties<'u>(
        &'w self,
        uniforms: &mut crate::prelude::Uniforms<'u>,
        resources: &mut Self::Resources,
        scene: &crate::scene::SceneSettings,
        camera: (&crate::scene::Camera, &math::Transform),
        light: (&crate::scene::Directional, &math::Transform),
    ) where
        'w: 'u {
        todo!()
    }
}