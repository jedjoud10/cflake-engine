mod graphics;
mod window;
use std::sync::Arc;

pub use graphics::*;
use phobos::{AppBuilder, GPURequirements, SurfaceSettings, QueueRequest, QueueType};
pub use window::*;
use winit::{window::WindowBuilder, event_loop::EventLoop};

/// Create a graphics context and winit window
pub fn initialize_phobos_context(el: &EventLoop<()>) -> (winit::window::Window, Graphics) {
    let window = WindowBuilder::new()
        .with_title("a")
        .build(&el)
        .unwrap();

    let settings = AppBuilder::new()
    .name("cFlake engine")
    .scratch_chunk_size(2048u64)
    .raytracing(false)
    .gpu(GPURequirements {
        queues: vec![
            QueueRequest { dedicated: false, queue_type: QueueType::Graphics },
            QueueRequest { dedicated: true, queue_type: QueueType::Transfer },
            QueueRequest { dedicated: true, queue_type: QueueType::Compute }
        ],
        ..Default::default()
    })
    .validation(true)
    .surface(Some(SurfaceSettings {
        surface_format: None,
        present_mode: None,
        window: &window,
    }));

    let (
        instance,
        physical_device,
        device,
        allocator,
        pool,
        exec,
        surface,
        frame,
        debug_messenger
    ) = phobos::init(&settings.build()).unwrap();
    
    (window, Graphics {
        instance,
        physical_device,
        device,
        allocator,
        pool,
        exec,
        debug_messenger,
        frame,
        surface,
    })
}