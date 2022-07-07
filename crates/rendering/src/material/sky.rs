use super::{Material, Pipeline};

// This is the material that our skysphere/skybox will use for rendering
// TODO: Implemented HDRi sky material and sheit
pub struct Sky {}

impl<'w> Material<'w> for Sky {
    type Resources = ();

    fn fetch(
        world: &'w mut world::World,
    ) -> (
        &'w crate::scene::SceneSettings,
        &'w ecs::EcsManager,
        &'w world::Storage<Self>,
        &'w world::Storage<crate::mesh::SubMesh>,
        &'w mut world::Storage<crate::prelude::Shader>,
        &'w mut crate::context::Window,
        &'w mut crate::context::Context,
        Self::Resources,
    ) {
        todo!()
    }

    fn set_instance_properties<'u>(
        &'w self,
        _uniforms: &mut crate::prelude::Uniforms<'u>,
        _resources: &mut Self::Resources,
        _scene: &crate::scene::SceneSettings,
        _camera: (&crate::scene::Camera, &math::Transform),
        _light: (&crate::scene::Directional, &math::Transform),
    ) where
        'w: 'u,
    {
        todo!()
    }

    fn shader(ctx: &mut crate::context::Context, assets: &mut assets::Assets) -> crate::prelude::Shader {
        todo!()
    }
}
