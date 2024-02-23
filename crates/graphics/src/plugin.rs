use crate::context::{Graphics, GraphicsStats, Window, WindowSettings};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::prelude::{post_user, pre_user, State, System, World, Registries, Update, Init};

/// Insert the required graphics resources
pub fn init(world: &mut World, _: &Init) {
    // Initialization resource
    let el = world.get::<EventLoop<()>>().unwrap();
    let init = world.get::<WindowSettings>().unwrap().clone();

    // Initialize the WGPU context and create a winit Window
    let (graphics, window) = unsafe { crate::context::init_context_and_window(init, &el) };
    drop(el);

    // Add the resources to the world
    world.insert(window);
    world.insert(graphics);
    world.insert(GraphicsStats::default());
}

/// Update the graphics stats based on the current frame data
pub fn update(world: &mut World, _: &Update) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut stats = world.get_mut::<GraphicsStats>().unwrap();
    let report = graphics.instance().generate_report();
    let vulkan = report.vulkan.as_ref().unwrap();
    *stats = GraphicsStats {
        adapters: vulkan.adapters.num_occupied,
        devices: vulkan.adapters.num_occupied,
        pipeline_layouts: vulkan.pipeline_layouts.num_occupied,
        shader_modules: vulkan.shader_modules.num_occupied,
        bind_group_layouts: vulkan.bind_group_layouts.num_occupied,
        bind_groups: vulkan.bind_groups.num_occupied,
        command_buffers: vulkan.command_buffers.num_occupied,
        render_pipelines: vulkan.render_pipelines.num_occupied,
        compute_pipelines: vulkan.compute_pipelines.num_occupied,
        buffers: vulkan.buffers.num_occupied,
        textures: vulkan.textures.num_occupied,
        texture_views: vulkan.texture_views.num_occupied,
        samplers: vulkan.samplers.num_occupied,
    };
}

/// Handle window quitting and resizing
pub fn event(world: &mut World, event: &WindowEvent) {
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

/// Acquire system will acquire a valid texture to draw to at the start of every frame
pub fn acquire(world: &mut World, _: &Update) {
    let mut window = world.get_mut::<Window>().unwrap();

    if let Ok(texture) = window.surface.get_current_texture() {
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Set the Window's texture view
        //log::trace!("acquire current texture");
        window.presentable_texture = Some(texture);
        window.presentable_texture_view = Some(view);
    } else {
        window.presentable_texture = None;
        window.presentable_texture_view = None;
    }
}

/// Present system will present the currently acquired texture to the monitor
pub fn present(world: &mut World, _: &Update) {
    let mut window = world.get_mut::<Window>().unwrap();
    let graphics = world.get_mut::<Graphics>().unwrap();
    graphics.submit(false);
    if let Some(texture) = window.presentable_texture.take() {
        texture.present();
    }
}


/// Plugin that will add all required systems for common behavior, acquiring, and presenting to the window
pub fn plugin(registries: &mut Registries) {
    registries.init.insert(init).before(pre_user).after(assets::init);
    registries.window_event.insert(event).before(pre_user);
    
    
    registries.update.insert(acquire).before(pre_user);
    
    registries.update
        .insert(update)
        .after(post_user)
        .after(acquire)
        .before(present);
    
    
    registries.update.insert(present).after(post_user);
}