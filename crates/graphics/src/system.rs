use crate::{GraphicSettings, WindowSettings};
use pollster::FutureExt;
use wgpu::RenderPassDescriptor;
use winit::{event::WindowEvent, event_loop::EventLoop};
use world::{post_user, user, State, System, World};

// Insert the required resources
fn init(
    world: &mut World,
    el: &EventLoop<()>,
    window_settings: WindowSettings,
    graphic_settings: GraphicSettings,
) {
    // Instantiate a new window wrapper
    let window = crate::context::Window::new(
        window_settings.clone(),
        el,
    );

    // Create a new wgpu context
    let graphics = crate::context::Graphics::new(
        &window.raw,
        &graphic_settings,
        &window_settings,
    );
    
    // Add the resources into the world
    world.insert(window);
    world.insert(graphics);
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

            // Resize the window by re-configuring WGPU 
            let graphics = world.get::<crate::context::Graphics>().unwrap();
            let config = graphics.config();
            let mut lock = config.lock();
            lock.width = size.width;
            lock.height = size.height;
            graphics.surface().configure(graphics.device(), &*lock);
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        },

        _ => (),
    }
}

// Clear the window at the start of every frame
fn update(world: &mut World) {
    let graphics = world.get::<crate::context::Graphics>().unwrap();
    let time = world.get::<time::Time>().unwrap();

    // Get the current texture we will write to
    let texture = graphics.surface().get_current_texture().unwrap();
    let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Default clear color
    let color = wgpu::Color::default();

    // Create a simple command encoder 
    let mut encoder = graphics.device().create_command_encoder(&Default::default());

    // Create a render pass to clear the image
    let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: true,
                },
            })
        ],
        depth_stencil_attachment: None,
    });
    drop(render_pass);

    // Submit the render pass and present the image
    graphics.queue().submit([encoder.finish()]);
    graphics.instance().poll_all(false);
    texture.present();
}

// Context system will just register the wgpu context and create a simple window
// This system will also handle window events like exiting
pub fn system(
    system: &mut System,
    window_settings: WindowSettings,
    graphic_settings: GraphicSettings,
) {
    system
        .insert_init(move |world: &mut World, el: &EventLoop<()>| {
            init(world, el, window_settings, graphic_settings)
        })
        .before(user);

    system.insert_update(update).before(user);
    system.insert_window(event);
}
