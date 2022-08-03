use super::{Camera, Renderer, ClusteredShading, PostProcessing, RenderedFrameStats};
use crate::{
    buffer::BufferMode,
    context::{Context, GraphicsSetupSettings, Window},
    material::{AlbedoMap, MaskMap, Material, NormalMap, Sky, Standard},
    pipeline::{PipeId, Pipeline, SpecializedPipeline},
    mesh::{Mesh, MeshImportSettings, Surface},
    prelude::{
        Filter, MipMaps, Ranged, Sampling, Texel, Texture, Texture2D, TextureImportSettings,
        TextureMode, Wrap, RG, RGB, RGBA, R, Depth,
    },
    shader::Shader, canvas::{Canvas, ColorAttachment, DepthAttachment, ToCanvasAttachment},
};

use assets::Assets;
use ecs::{added, modified, or, Scene};
use glutin::{event::WindowEvent, event_loop::EventLoop};
use math::{Scale, Location, Rotation, IntoMatrix};
use world::{Events, Init, Stage, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) {
    // Insert the default storages
    world.insert(Storage::<AlbedoMap>::default());
    world.insert(Storage::<NormalMap>::default());
    world.insert(Storage::<MaskMap>::default());
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Shader>::default());
    world.insert(Storage::<Standard>::default());
    world.insert(Storage::<Sky>::default());
    world.insert(Storage::<Canvas>::default());
    world.insert(Storage::<ColorAttachment>::default());
    world.insert(Storage::<DepthAttachment>::default());

    // Get mutable references to the data that we must use
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut color_attachments = world.get_mut::<Storage<ColorAttachment>>().unwrap();
    let mut depth_attachments = world.get_mut::<Storage<DepthAttachment>>().unwrap();

    // Create the window and graphical context
    let (mut window, mut context) = crate::context::new(settings, el);
    let ctx = &mut context;

    // Settings for framebuffer textures
    let sampling = Sampling {
        filter: Filter::Linear,
        wrap: Wrap::ClampToEdge,
    };
    let mipmaps = MipMaps::Disabled;

    // Create the render color texture
    let color: ColorAttachment = <ColorAttachment as Texture>::new(ctx, TextureMode::Resizable, window.canvas().size(), sampling, mipmaps, &[]).unwrap();
    let color = color_attachments.insert(color);
    let t1 = (&*color_attachments, color);

    // Create the render depth texture
    let depth: DepthAttachment = <DepthAttachment as Texture>::new(ctx, TextureMode::Resizable, window.canvas().size(), sampling, mipmaps, &[]).unwrap();
    let depth = depth_attachments.insert(depth);
    let t2 = (&*depth_attachments, depth);
    
    // Create the canvas that we will draw our 3D objects onto
    let targets: Vec<&dyn ToCanvasAttachment> = vec![&t1, &t2];
    let canvas = Canvas::new(ctx, window.canvas().size(), targets).unwrap();

    // Create the default pipelines
    let mut init = (&mut *shaders, &mut *assets);
    ctx.init_pipe_id::<SpecializedPipeline<Standard>>(&mut init);
    ctx.init_pipe_id::<SpecializedPipeline<Standard>>(&mut init);

    // Create the clustered shading rendererer
    let clustered_shading = ClusteredShading {
        main_camera: None,
        main_directional_light: None,
        canvas,
    };

    // Create the post-processing settings
    let postprocessing = PostProcessing {
        tonemapping_strength: 1.0,
        exposure: 1.0,
        vignette_strength: 1.0,
        vignette_size: 1.0,
    };
    
    // Create the frame-to-frame basis stats
    let stats = RenderedFrameStats {
        tris: 0,
        verts: 0,
        unique_materials: 0,
        material_instances: 0,
        surfaces: 0,
        current: false,
    };
    
    // Drop the old write/read handles
    drop(shaders);
    drop(assets);
    drop(color_attachments);
    drop(depth_attachments);

    // Insert the newly made resources
    world.insert(window);
    world.insert(context);
    world.insert(clustered_shading);
    world.insert(postprocessing);
    world.insert(stats);
}

// Update the global mesh matrices of objects that have been modified
fn update_matrices(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    
    // TODO: Add filter
    let query = ecs
        .query::<(&mut Renderer, Option<&Location>, Option<&Rotation>, Option<&Scale>)>()
        .unwrap();

    // Update the matrices of objects that might contain location, rotation, or scale
    for (renderer, location, rotation, scale) in query {
        let mut matrix = vek::Mat4::<f32>::identity();
        matrix = location.map_or(matrix, |l| matrix * l.into_matrix());
        matrix *= rotation.map_or(matrix, |r| matrix * r.into_matrix());
        matrix *= scale.map_or(matrix, |s| matrix * s.into_matrix());
        renderer.set_matrix(matrix);
    }
}

// Rendering event that will try to render the 3D scene each frame
fn rendering(world: &mut World) {
    
    // Check if we can even render the scene in the first place
    let shading = world.get::<ClusteredShading>().unwrap();
    if shading.main_camera().is_none() || shading.main_directional_light().is_none() {
        return;
    } drop(shading);
    
    // Extract the pipelines and render them
    let mut stats = *world.get::<RenderedFrameStats>().unwrap();
    let pipelines = world
        .get::<Context>()
        .unwrap()
        .extract_pipelines()
        .into_iter();

    // Render the pipelines one by one
    for render in pipelines {
        render.render(world, &mut stats);
    }

    let mut old_stats = world.get_mut::<RenderedFrameStats>().unwrap();
    *old_stats = stats; 
    old_stats.current = true;
}

// Window event for updating the current main canvas and world state if needed
fn window(world: &mut World, event: &mut WindowEvent) {
    match event {
        WindowEvent::Resized(size) => {
            // We might get null dimensions when the user minimizes the window
            let extent = if size.height > 0 && size.width > 0 {
                vek::Extent2::new(size.width as u16, size.height as u16)
            } else {
                return;
            };

            // Resize the default canvas when we resize the window
            let mut window = world.get_mut::<Window>().unwrap();
            window.canvas_mut().resize(extent);
        }
        WindowEvent::CloseRequested => {
            *world.get_mut::<world::State>().unwrap() = world::State::Stopped;
        }
        _ => {}
    }
}

// Frame startup (clearing the frame at the start of the frame)
fn clear(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    window
        .canvas_mut()
        .clear(Some(vek::Rgb::black()), Some(1.0), None);
}

// Frame cleanup event that will just swap the front and back buffers of the current context
fn swap(world: &mut World) {
    let ctx = world.get_mut::<Context>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}

// Update event that will update the view matrix of the main perspective camera
// The main camera entity is stored in the Scene renderer
fn main_camera(world: &mut World) {
    // Get the ecs, window, and scene renderer
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let shading = world.get::<ClusteredShading>().unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = shading.main_camera {
        let mut entry = ecs.entry_mut(entity).unwrap();

        // Fetch it's components, and update them
        let (camera, location, rotation) = entry.as_query::<(&mut Camera, &Location, &Rotation)>().unwrap();
        camera.update(location, rotation);
    }
}

// Main rendering/graphics system that will register the appropriate events
pub fn system(events: &mut Events, settings: GraphicsSetupSettings) {
    // Insert graphics init event
    events
        .registry::<Init>()
        .insert_with(
            |world: &mut World, el: &EventLoop<()>| init(world, settings, el),
            Stage::new("graphics insert")
                .after("asset loader insert")
                .before("user"),
        )
        .unwrap();

    // Insert update events (fetch the registry)
    let reg = events.registry::<Update>();
    reg.insert_with(clear, Stage::new("window clear").before("user"))
        .unwrap();

    // Insert camera update event
    reg.insert_with(
        main_camera,
        Stage::new("main camera update")
            .after("post user"),
    )
    .unwrap();

    // Insert update renderer event
    reg.insert_with(
        update_matrices,
        Stage::new("update renderer matrices")
            .after("post user"),
    )
    .unwrap();

    // Insert scene renderer event
    reg.insert_with(
        rendering,
        Stage::new("scene rendering")
            .after("main camera update")
            .after("update renderer matrices")
    )
    .unwrap();

    // Insert window buffer swap event
    reg.insert_with(
        swap,
        Stage::new("window back buffer swap").after("scene rendering"),
    )
    .unwrap();

    // Insert window event
    events.registry::<WindowEvent>().insert(window);
}
