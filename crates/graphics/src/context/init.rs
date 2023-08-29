use crate::{
    FrameRateLimit, Graphics, InternalGraphics, Window, WindowSettings,
};

use dashmap::DashMap;
use nohash_hasher::NoHashHasher;
use parking_lot::Mutex;
use phobos::{WindowedContext, ContextInit, AppSettings, AppBuilder, QueueRequest, GPURequirements, QueueType};
use std::sync::Arc;
use systemstat::Platform;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Create the Phobos context wrapper and a Window wrapper
pub(crate) unsafe fn init_phobos_context_and_window(
    settings: WindowSettings,
    el: &EventLoop<()>,
) -> (Graphics, Window) {
    // Create a winit window (but don't make it's wrapper)
    let window = init_window(el, &settings);
    let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(window.inner_size()));

    // Create some settings for our App
    let app_settings = AppBuilder::new()
        .version((1, 0, 0))
        .name("App")
        .validation(true)
        .window(&window)
        .scratch_chunk_size(2048u64)
        .gpu(GPURequirements {
            dedicated: true,
            min_video_memory: 1 * 1024 * 1024 * 1024, // 1 GiB.
            min_dedicated_video_memory: 1 * 1024 * 1024 * 1024,
            queues: vec![
                QueueRequest { dedicated: false, queue_type: QueueType::Graphics },
                QueueRequest { dedicated: true, queue_type: QueueType::Transfer },
                QueueRequest { dedicated: true, queue_type: QueueType::Compute }
            ],
            ..Default::default()
        })
        .build();

    // Fetch the phobos context types
    let (
        instance,
        physical_device,
        surface,
        device,
        allocator,
        pool,
        exec,
        frame,
        Some(debug_messenger)
    ) = WindowedContext::init(&app_settings).unwrap() else {
        panic!("Asked for debug messenger but didn't get one.")
    };

    // Create the window wrapper
    let window = Window {
        settings,
        raw: Arc::new(window),
        size,
    };

    // Create the graphics wrapper
    let internal = InternalGraphics {
        instance,
        physical_device,
        device,
        allocator,
        pool,
        exec,
        frame,
        debug_messenger,
    };
    let graphics = Graphics(Arc::new(internal));

    (graphics, window)
}

// Init a winit window
fn init_window(el: &EventLoop<()>, window_settings: &WindowSettings) -> winit::window::Window {
    WindowBuilder::default()
        .with_fullscreen(
            window_settings
                .fullscreen
                .then_some(Fullscreen::Borderless(None)),
        )
        .with_title(&window_settings.title)
        .build(el)
        .unwrap()
}
