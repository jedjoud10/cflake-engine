use super::{Camera, SceneRenderer};
use crate::{
    context::{Context, Graphics, GraphicsSetupSettings},
    material::{AlbedoMap, MaskMap, Material, NormalMap, Standard},
    prelude::{
        Filter, MipMaps, Ranged, Sampling, Texel, Texture, Texture2D, TextureMode, Wrap, RG, RGB,
        RGBA,
    },
    shader::Shader,
};

use assets::Assets;
use ecs::{added, modified, or, EcsManager};
use glutin::event_loop::EventLoop;
use math::Transform;
use world::{Events, Init, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) {
    // During initialization, the world always contains the Init resource
    // This resource contains the global event loop and all informations that are needed for

    // Create a new graphics pipeline and insert it
    let Graphics(device, ctx) = world.entry().or_insert(Graphics::new(settings, el));

    // This function creates a 1x1 Texture2D wit default settings that we can store within the scene renderer
    fn create<T: Texel>(ctx: &mut Context, texel: T::Storage) -> Texture2D<T> {
        Texture2D::<T>::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::new(Filter::Nearest, Wrap::Repeat),
            MipMaps::Disabled,
            &[texel],
        )
        .unwrap()
    }

    // Create the default black texture
    let black = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::zero());

    // Create the default white texture
    let white = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::one());

    // Create the default PBR textures (normal map, mask map)
    let normal_map = create::<RGB<Ranged<u8>>>(ctx, vek::Vec3::new(128, 128, 255));
    let mask_map = create::<RG<Ranged<u8>>>(ctx, vek::Vec2::new(255, 51));

    // Insert all of the textures into their corresponding storages
    let (albedo_maps, normal_maps, mask_maps) = world
        .get_mut::<(
            &mut Storage<AlbedoMap>,
            &mut Storage<NormalMap>,
            &mut Storage<MaskMap>,
        )>()
        .unwrap();
    let black = albedo_maps.insert(black);
    let white = albedo_maps.insert(white);
    let normal_map = normal_maps.insert(normal_map);
    let mask_map = mask_maps.insert(mask_map);

    // Load the default PBR material (refetch the resources since we need storage and asset loader)
    let (Graphics(_, ctx), assets, storage) = world
        .get_mut::<(&mut Graphics, &mut Assets, &mut Storage<Shader>)>()
        .unwrap();
    let material = Standard::builder()
        .albedo(&white)
        .normal(&normal_map)
        .mask(&mask_map)
        .metallic(0.2)
        .roughness(1.0)
        .build(ctx, assets, storage);

    // Insert el material and get it's handle
    let storage = world.get_mut::<&mut Storage<Standard>>().unwrap();
    let material = storage.insert(material);

    // Create the new scene renderer from these values and insert it into the world
    let scene = SceneRenderer::new(
        black,
        white.clone(),
        white.clone(),
        normal_map,
        mask_map,
        material,
    );
    world.insert(scene);
}

// Rendering system that will try to render the 3D scene each frame
// I am pretty proud of my material system tbh. Sick as hell fr fr
fn rendering(world: &mut World) {
    // Get the graphics context, ecs, and the main scene renderer
    let (graphics, renderer) = world.get_mut::<(&mut Graphics, &SceneRenderer)>().unwrap();
    let Graphics(_device, context) = graphics;

    // Can we render the scene? (cause if we can't then we have a big problemo)
    if !renderer.can_render() {
        return;
    }
    let settings = renderer.clone();

    // Update all the renderer components
    let renderers = context.extract_material_renderers();

    // Render all the material surfaces
    let _stats = renderers
        .into_iter()
        .map(|elem| elem.render(world, &settings))
        .collect::<Vec<_>>();
}

// Update system that will update the view matrix of the main perspective camera
// The main camera entity is stored in the Scene renderer
fn main_camera(world: &mut World) {
    // Get the ecs, window, and scene renderer
    let (ecs, Graphics(device, _), scene) = world
        .get_mut::<(&mut EcsManager, &Graphics, &SceneRenderer)>()
        .unwrap();

    // Fetch the main perspective camera from the scene renderer
    let entity = scene.main_camera().unwrap();
    let mut entry = ecs.try_entry(entity).unwrap();

    // Fetch it's components, and update them
    let (camera, transform) = entry.get_mut_layout::<(&mut Camera, &Transform)>().unwrap();
    camera.update(transform);
}

// Main rendering system that will register the appropriate events
pub fn system(events: &mut Events, settings: GraphicsSetupSettings) {
    events
        .registry::<Init>()
        .insert(|world: &mut World, el: &EventLoop<()>| init(world, settings, el));
    //events.register::<Update>(main_camera);
    //events.register_with::<Update>(rendering, Stage::after("rendering"))
}
