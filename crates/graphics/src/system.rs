use crate::{Graphics, GraphicsStats, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.get::<WindowSettings>().unwrap().clone();

    // Initialize the WGPU context and create a winit Window
    let (graphics, window) =
        unsafe { crate::context::init_context_and_window(init, el) };

    // Add the resources to the world
    world.insert(window);
    world.insert(graphics);
    world.insert(GraphicsStats::default());
}

// Update the graphics stats based on the current frame data
fn update(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut stats = world.get_mut::<GraphicsStats>().unwrap();
    let cached = &graphics.0.cached;
    let report = graphics.instance().generate_report();
    let vulkan = report.vulkan.as_ref().unwrap();
    *stats = GraphicsStats {
        submissions: *graphics.0.submissions.lock() as usize,
        acquires: *graphics.0.acquires.lock() as usize,
        stalls: *graphics.0.stalls.lock() as usize,
        staging_buffers: graphics.0.staging.allocations.len(),
        cached_samplers: cached.samplers.len(),
        cached_bind_group_layouts: cached.bind_group_layouts.len(),
        cached_pipeline_layouts: cached.pipeline_layouts.len(),
        cached_bind_groups: cached.bind_groups.len(),

        adapters: vulkan.adapters.num_occupied,
        devices: vulkan.adapters.num_occupied,
        pipeline_layouts: vulkan.pipeline_layouts.num_occupied,
        shader_modules: vulkan.shader_modules.num_occupied,
        bind_group_layouts: vulkan.bind_group_layouts.num_occupied,
        bind_groups: vulkan.bind_groups.num_occupied,
        command_buffers: vulkan.command_buffers.num_occupied,
        render_pipelines: vulkan.render_pipelines.num_occupied,
        buffers: vulkan.buffers.num_occupied,
        textures: vulkan.textures.num_occupied,
        texture_views: vulkan.texture_views.num_occupied,
        samplers: vulkan.samplers.num_occupied,
    };
    *graphics.0.submissions.lock() = 0;
    *graphics.0.acquires.lock() = 0;
    *graphics.0.stalls.lock() = 0;
}

// Handle window quitting and resizing
fn event(world: &mut World, event: &mut WindowEvent) {
    match event {
        // Window has been resized
        WindowEvent::Resized(size) => {
            // Check if the size is valid
            if size.height == 0 || size.height == 0 {
                return;
            }

            // Handle resizing the window
            let size = vek::Extent2::new(size.width, size.height);
            let mut window = world.get_mut::<Window>().unwrap();
            let graphics = world.get::<Graphics>().unwrap();

            // Update the surface configuration and reconfigure the surface
            window.surface_config.width = size.w;
            window.surface_config.height = size.h;
            let config = &window.surface_config;
            window.surface.configure(graphics.device(), config);
            window.size = size;
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        }

        _ => (),
    }
}

// Common system will be responsible for calling the init event and window event
pub fn common(system: &mut System) {
    system
        .insert_update(update)
        .after(post_user)
        .after(acquire)
        .before(present);
    system.insert_init(init).before(user).after(assets::system);
    system.insert_window(event).before(user);
}

// Acquire system will acquire a valid texture to draw to at the start of every frame
pub fn acquire(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            // Acquire a new texture to render to
            let mut window = world.get_mut::<Window>().unwrap();
            let texture =
                window.surface.get_current_texture().unwrap();
            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Set the Window's texture view
            // TODO: Cache the texture views instead?
            window.presentable_texture = Some(texture);
            window.presentable_texture_view = Some(view);

            // Clear the window first, and save the command encoder
        })
        .after(post_user);
}

// Present system will present the currently acquired texture to the monitor
pub fn present(system: &mut System) {
    system
        .insert_update(|world: &mut World| {
            let mut window = world.get_mut::<Window>().unwrap();
            let graphics = world.get::<Graphics>().unwrap();
            graphics.submit_unused(false);
            window.presentable_texture.take().unwrap().present();
        })
        .after(post_user)
        .after(acquire);
}
