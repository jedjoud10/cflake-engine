use super::{CreatePipeline, Pipeline};
use crate::{
    context::{Context, Window},
    display::RasterSettings,
    material::{DefaultMaterialResources, Material},
    mesh::{Mesh, Surface},
    painter::Painter,
    prelude::{Display, Region, Shader, Texture, Viewport},
    scene::{Camera, ClusteredShading, DirectionalLight, RenderedFrameStats, Renderer, FrustumPlane},
};
use assets::Assets;
use ecs::Scene;
use math::{Location, Rotation, AABB, SharpVertices};
use std::{marker::PhantomData, any::type_name};
use world::{Handle, Storage, World};

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Fix this and actually do it properly
// TODO: Use multithreading to make it faster as well
pub fn intersects_frustum(planes: &[FrustumPlane; 6], aabb: AABB) -> bool {
    false
}

// The standardized pipeline will use the default rendering canvas to render to
pub struct SpecializedPipeline<M: for<'w> Material<'w>> {
    pub(crate) shader: Handle<Shader>,
    pub(crate) _phantom: PhantomData<M>,
}

impl<M: for<'w> Material<'w>> Pipeline for SpecializedPipeline<M> {
    fn render(&self, world: &mut World, stats: &mut RenderedFrameStats) {
        let mut property_block_resources = M::fetch_resources(world);
        let ecs = world.get::<Scene>().unwrap();
        let materials = world.get::<Storage<M>>().unwrap();
        let meshes = world.get::<Storage<Mesh>>().unwrap();
        let window = world.get::<Window>().unwrap();
        let mut _shading = world.get_mut::<ClusteredShading>().unwrap();
        let shading = &mut *_shading;
        let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
        let mut ctx = world.get_mut::<Context>().unwrap();

        // How exactly we should rasterize the surfaces
        let settings = RasterSettings {
            depth_test: M::depth_comparison(),
            scissor_test: None,
            primitive: M::primitive_mode(),
            srgb: M::srgb(),
            blend: M::blend_mode(),
        };
        
        // Get the main camera component (there has to be one for us to render)
        let camera_entry = ecs.entry(shading.main_camera.unwrap()).unwrap();
        let (camera, camera_location, camera_rotation) = camera_entry
            .as_view::<(&Camera, &Location, &Rotation)>()
            .unwrap();
        let planes = camera.frustum();

        // Get the main directional light (there has to be one for us to render)
        let light_entry = ecs.entry(shading.main_directional_light.unwrap()).unwrap();
        let (directional_light, directional_light_rotation) = light_entry
            .as_view::<(&DirectionalLight, &Rotation)>()
            .unwrap();

        // Find all the surfaces that use this material type (and that have a valid renderer and valid mesh)
        let query = ecs.view::<(&Renderer, &Surface<M>)>().unwrap();
        let query = query.filter(|(renderer, surface)| {            
            // Check if the renderer is even enabled
            let enabled = renderer.enabled;

            // Check if the mesh meets the material requirements
            let mesh = meshes.get(&surface.mesh());
            let buffers = mesh.vertices().layout().contains(M::requirements())
                && mesh.vertices().len().is_some();

            // Check if the surface is visible inside the camera's frustum
            let aabb = if let Some(mut aabb) = mesh.aabb() {
                //aabb.min = renderer.matrix.mul_point(aabb.min);
                //aabb.max = renderer.matrix.mul_point(aabb.max);
                intersects_frustum(&planes, aabb)
            } else {
                false
            };
            
            enabled && buffers && aabb
        });

        // Create a new rasterizer so we can draw the objects onto the painter
        let shader = shaders.get_mut(&self.shader);
        let color = shading.color_tex.mip_mut(0).unwrap();
        let depth = shading.depth_tex.mip_mut(0).unwrap();
        let mut scoped = shading
            .painter
            .scope(window.viewport(), color, depth, ())
            .unwrap();
        let (mut rasterizer, mut uniforms) = scoped.rasterizer(&mut ctx, shader, settings);

        let main = DefaultMaterialResources {
            camera,
            point_lights: &shading.point_lights,
            camera_location,
            camera_rotation,
            directional_light,
            directional_light_rotation,
            window: &window,
        };

        // Set global properties
        M::set_static_properties(&mut uniforms, &main, &mut property_block_resources);

        let mut old: Option<Handle<M>> = None;
        for (renderer, surface) in query {
            // Check if we changed material instances
            if old != Some(surface.material().clone()) {
                old = Some(surface.material().clone());
                let instance = materials.get(old.as_ref().unwrap());

                // Update the material property block uniforms
                M::set_instance_properties(
                    &mut uniforms,
                    &main,
                    &mut property_block_resources,
                    instance,
                );
            }

            // Set the uniforms per renderer
            M::set_surface_properties(
                &mut uniforms,
                &main,
                &mut property_block_resources,
                renderer,
            );

            // Draw the surface object using the current rasterizer pass
            let mesh = meshes.get(&surface.mesh());

            // Sometimes, meshes can be invalid
            if let Some(len) = mesh.vertices().len() {
                // Validate the uniforms
                let validated = unsafe {
                    if M::should_assume_valid() {
                        uniforms.assume_valid()
                    } else {
                        uniforms.validate().unwrap()
                    }
                };

                rasterizer.draw(mesh, validated);
                stats.surfaces += 1;
                stats.verts += len as u32;
                stats.tris += mesh.triangles().len() as u32;
            }
        }
    }
}

impl<'a, M: for<'w> Material<'w>> CreatePipeline<'a> for SpecializedPipeline<M> {
    type Args = (&'a mut Storage<Shader>, &'a mut Assets);

    fn init(ctx: &mut Context, args: &mut Self::Args) -> Self {
        let shader = M::shader(ctx, args.1);
        let handle = args.0.insert(shader);
        Self {
            shader: handle,
            _phantom: PhantomData::default(),
        }
    }
}
