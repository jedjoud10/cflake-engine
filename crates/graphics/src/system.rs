use crate::{Window, WindowSettings, Graphics};
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required graphics resources
fn init(world: &mut World, el: &EventLoop<()>) {
    // Initialization resource
    let init = world.remove::<WindowSettings>().unwrap();

    // Initialize the WGPU context and create a winit Window
    let (graphics, window) =
        unsafe { crate::context::init_context_and_window(init, el) };

    // Add the resources to the world
    world.insert(window);
    world.insert(graphics);
}

// Reset the dirty state of the window at the end of each frame
fn update(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    
    let mut graphics = world.get_mut::<Graphics>().unwrap();
    let texture = window.surface.get_current_texture().unwrap();
    let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = graphics.device().create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(
            wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }
        )],
        depth_stencil_attachment: None,
    });
    drop(render_pass);

    graphics.queue().submit(Some(encoder.finish()));
    texture.present();
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

// Context system will just register the wgpu context and create a simple window
// This system will also handle window events like exiting
pub fn system(system: &mut System) {
    system
        .insert_init(init)
        .after(utils::threadpool)
        .before(user);

    system.insert_window(event);
    system.insert_update(update).after(post_user);
}
