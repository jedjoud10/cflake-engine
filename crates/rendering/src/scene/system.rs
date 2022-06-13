use std::any::Any;

use super::SceneRenderer;
use crate::{
    context::{Context, Graphics},
    material::{AlbedoMap, MaskMap, NormalMap},
    mesh::SubMesh,
    object::ToGlName,
    prelude::{
        Filter, Ranged, Sampling, Texel, Texture, Texture2D, TextureMode, Wrap, RG, RGB, RGBA,
    },
    shader::Shader,
};
use ecs::EcsManager;
use world::{resources::Storage, World};

// Initialization system that will setup the default textures and objects
pub fn init(world: &mut World) {
    // Get the storages and graphical context
    let Graphics(device, ctx) = world.get_mut::<&mut Graphics>().unwrap();

    // This function creates a 1x1 Texture2D wit default settings that we can store within the scene renderer
    fn create<T: Texel>(ctx: &mut Context, texel: T::Storage) -> Texture2D<T> {
        Texture2D::<T>::new(
            ctx,
            TextureMode::Dynamic,
            vek::Extent2::one(),
            Sampling::new(Filter::Nearest, Wrap::Repeat),
            false,
            &[texel],
        )
        .unwrap()
    }

    // Create the default black texture
    let black = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::zero());

    // Create the default white texture
    let white = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::one());

    // Create the default PBR textures (albedo map, normal map, mask map)
    let albedo_map = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::zero());
    let normal_map = create::<RGB<Ranged<u8>>>(ctx, vek::Vec3::new(128, 128, 255));
    let mask_map = create::<RG<Ranged<u8>>>(ctx, vek::Vec2::new(255, 51));

    // Insert all of the textures into their corresponding storages
    let mut set = world.storages();
    let black = set.insert(black);
    let white = set.insert(white);
    let albedo_map = set.insert(albedo_map);
    let normal_map = set.insert(normal_map);
    let mask_map = set.insert(mask_map);

    // Create the new scene renderer from these values and insert it into the world
    let renderer = SceneRenderer::new(black, white, albedo_map, normal_map, mask_map);
    world.insert(renderer);
}

// Update system that will execute each frame to try to render the scene
pub fn rendering(world: &mut World) {
    // Get the graphics context, ecs, and the main scene renderer
    let (graphics, renderer) = world
        .get_mut::<(&mut Graphics, &SceneRenderer)>()
        .unwrap();
    let Graphics(device, context) = graphics;

    // Can we render the scene? (cause if we can't then we have a big problemo)
    if !renderer.can_render() {
        return;
    }
    let settings = renderer.clone();

    // Update all the renderer components
    let renderers = context.extract_material_renderer();

    // Render all the material surfaces
    let stats = renderers
        .into_iter()
        .map(|elem| elem.render(world, &settings))
        .collect::<Vec<_>>();
}