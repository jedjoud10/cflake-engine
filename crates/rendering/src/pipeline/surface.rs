use crate::{
    context::{Context, Window},
    display::{RasterSettings, Rasterizer},
    material::{DefaultMaterialResources, Material},
    mesh::{Mesh, MeshUtils, Surface},
    painter::{ScopedPainter, SingleLayerIntoTarget},
    prelude::{self, Depth, Display, Ranged, Shader, Texture, RGB},
    scene::{
        Camera, ClusteredShading, DirectionalLight, FrustumPlane, RenderedFrameStats, Renderer,
    },
    shader::Uniforms,
};

use ecs::Scene;
use math::{Location, Rotation, SharpVertices, AABB};

use world::{Handle, Read, Storage, World, Write};

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Use multithreading to make it faster as well
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub fn intersects_frustum(planes: &[FrustumPlane; 6], aabb: AABB, matrix: &vek::Mat4<f32>) -> bool {
    let mut corners = aabb.points();

    for corner in corners.iter_mut() {
        *corner = matrix.mul_point(*corner);
    }

    let aabb = MeshUtils::aabb_from_points(&corners).unwrap();

    let corners = [aabb.min, aabb.max];

    planes.iter().all(|plane| {
        let mut furthest = vek::Vec3::zero();
        furthest.iter_mut().enumerate().for_each(|(i, e)| {
            *e = corners[(plane.normal[i] > 0.0) as usize][i];
        });
        let signed = furthest.dot(plane.normal) + plane.distance;

        signed > 0.0
    })
}

// Gets an ECS query of all the visible surfaces and their renderers
fn get_surfaces_query<'a, M: for<'w> Material<'w>>(
    ecs: &'a Scene,
    meshes: &'a Storage<Mesh>,
    planes: [FrustumPlane; 6],
) -> impl Iterator<Item = (&'a Renderer, &'a Surface<M>)> + 'a {
    let planes = planes.clone();
    let query =
        ecs.view::<(&Renderer, &Surface<M>)>()
            .unwrap()
            .filter(move |(renderer, surface)| {
                // Check if the renderer is even enabled
                let enabled = renderer.visible;

                // Check if the mesh meets the material requirements
                let mesh = meshes.get(&surface.mesh);
                let buffers = mesh.vertices().layout().contains(M::requirements())
                    && mesh.vertices().len().is_some();

                // Check if the surface is visible inside the camera's frustum
                let aabb = !M::should_use_frustum_culling()
                    || if let Some(aabb) = mesh.aabb() {
                        intersects_frustum(&planes, aabb, &renderer.matrix)
                    } else {
                        false
                    };

                enabled && buffers && aabb
            });
    query
}

// Render all the surfaces that are part of the Z-prepass ECS query
fn render_prepass_query_surfaces<'a, M: for<'w> Material<'w>>(
    query: impl Iterator<Item = (&'a Renderer, &'a Surface<M>)>,
    mut uniforms: Uniforms,
    meshes: &Storage<Mesh>,
    main: DefaultMaterialResources,
    mut rasterizer: Rasterizer<ScopedPainter<RGB<f32>, Depth<Ranged<u32>>, ()>>,
) {
    for (_, surface) in query {
        // Draw the surface object using the current rasterizer pass
        let mesh = meshes.get(&surface.mesh);

        // Validate the uniforms
        uniforms.set_mat4x4("matrix", *main.camera.projection_matrix() * *main.camera.view_matrix());
        let validated = unsafe { uniforms.assume_valid() };

        // Render the object
        rasterizer.draw(mesh, validated);
    }
}

// Render all the surfaces that are part of a specific ECS query and that are rendered onto a specific rasterizer
fn render_query_surfaces<'a, M: for<'w> Material<'w>>(
    query: impl Iterator<Item = (&'a Renderer, &'a Surface<M>)>,
    mut stats: Write<RenderedFrameStats>,
    materials: &Storage<M>,
    mut uniforms: Uniforms,
    main: DefaultMaterialResources,
    mut property_block_resources: <M as Material>::Resources,
    meshes: &Storage<Mesh>,
    mut rasterizer: Rasterizer<ScopedPainter<RGB<f32>, Depth<Ranged<u32>>, ()>>,
) {
    let mut old: Option<Handle<M>> = None;
    for (renderer, surface) in query {
        // Check if we changed material instances
        if old != Some(surface.material.clone()) {
            stats.material_instances += 1;
            old = Some(surface.material.clone());
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
        let mesh = meshes.get(&surface.mesh);

        // Validate the uniforms
        let validated = unsafe {
            if M::should_assume_valid() {
                uniforms.assume_valid()
            } else {
                uniforms.validate().unwrap()
            }
        };

        rasterizer.draw(mesh, validated);
        stats.rendered_surfaces += 1;
        stats.verts += mesh.vertices().len().unwrap() as u32;
        stats.tris += mesh.triangles().len() as u32;
    }
}

// Render all the visible surfaces of a specific material type
pub(crate) fn render_surfaces<M: for<'w> Material<'w>>(world: &mut World, shader: Handle<Shader>) {
    let mut property_block_resources = M::fetch_resources(world);
    let ecs = world.get::<Scene>().unwrap();
    let materials = world.get::<Storage<M>>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut _shading = world.get_mut::<ClusteredShading>().unwrap();
    let shading = &mut *_shading;
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut stats = world.get_mut::<RenderedFrameStats>().unwrap();
    stats.unique_materials += 1;

    // How exactly we should rasterize the surfaces
    let settings = RasterSettings {
        depth_test: M::depth_comparison(),
        scissor_test: None,
        primitive: M::primitive_mode(),
        srgb: M::srgb(),
        blend: None,
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

    // Create the default resources that will be passed to the material
    let main = DefaultMaterialResources {
        camera,
        point_lights: &shading.point_lights,
        clusters: &shading.clusters,
        cluster_size: &shading.cluster_size,
        camera_location,
        camera_rotation,
        directional_light,
        directional_light_rotation,
        window: &window,
    };

    // Create the painter that will draw our bozos
    let color = shading.color_tex.mip_mut(0).unwrap();
    let depth = shading.depth_tex.mip_mut(0).unwrap();
    let mut scoped = shading
        .painter
        .scope(window.viewport(), color, depth, ())
        .unwrap();

    // Create a new rasterizer that will serve as our Z-preprass rasterizer
    let (prepass, uniforms) =
        scoped.rasterizer(&mut ctx, &mut shading.prepass_shader, settings);

    // Render out the scene, but only depth only
    let query = get_surfaces_query::<M>(&ecs, &meshes, planes);
    render_prepass_query_surfaces(query, uniforms, &meshes, main, prepass);

    // Create a new rasterizer so we can draw the objects onto the painter
    let shader = shaders.get_mut(&shader);
    let (rasterizer, mut uniforms) = scoped.rasterizer(&mut ctx, shader, settings);

    // Set global properties
    M::set_static_properties(&mut uniforms, &main, &mut property_block_resources);

    // Render the surfaces that are part of the query
    let query = get_surfaces_query::<M>(&ecs, &meshes, planes);
    render_query_surfaces(
        query,
        stats,
        &materials,
        uniforms,
        main,
        property_block_resources,
        &meshes,
        rasterizer,
    );
}
