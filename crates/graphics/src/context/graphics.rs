use std::{ffi::{CStr, CString}, sync::Arc};
use super::{Window, FrameRateLimit, WindowSettings};
use wgpu::{*, util::*};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicSettings {
}

impl Default for GraphicSettings {
    fn default() -> Self {
        Self {
        }
    }
}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
#[derive(Clone)]
pub struct Graphics {
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,
}

impl Graphics {
    // Create a new wgpu graphics context based on the window wrapper
    pub(crate) fn new(
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
        window_settings: &WindowSettings,
    ) -> Graphics {
        // Create the wgpu instance and main surface
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        // Pick an appopriate adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).unwrap();       
        
        // Create the device and main queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),                
                label: None,
            },
            None,
        )).unwrap();

        // Create the surface that we will render to
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::Immediate,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        Self {
            surface: Arc::new(surface),
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
        }
    }
}