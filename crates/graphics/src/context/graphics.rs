use std::{ffi::{CStr, CString}, sync::{Arc}};
use super::{Window, FrameRateLimit, WindowSettings};
use bytemuck::{Zeroable, Pod};
use parking_lot::Mutex;
use wgpu::{*, util::*, Instance};
use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone, Default)]
pub struct GraphicSettings {
}

// Plain old data type internally used by buffers and other types
pub trait Content: Zeroable + Pod + Clone + Copy + Sync + Send + 'static {}
impl<T: Clone + Copy + Sync + Send + Zeroable + Pod + 'static> Content for T {}

// Internal context so we don't make multiple allocations
struct Internal {
    instance: Instance,
    surface: Surface,
    device: Device,
    queue: Queue,
    config: Mutex<SurfaceConfiguration>,
}

// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(Arc<Internal>);

impl Graphics {
    // Create a new wgpu graphics context based on the window wrapper
    pub(crate) fn new(
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
        window_settings: &WindowSettings,
    ) -> Graphics {
        // Create the wgpu instance and main surface
        env_logger::init();
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

        // Pick the surface present mode
        let present_mode = if matches!(window_settings.limit, FrameRateLimit::VSync) {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };

        // Create the surface that we will render to
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode,
            alpha_mode: CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        Self(Arc::new(Internal {
            instance,
            surface,
            device,
            queue,
            config: Mutex::new(config),
        }))
    }

    // Get access to the underlying instance
    pub fn instance(&self) -> &Instance {
        &self.0.instance
    }

    // Get access to the underlying surface
    pub fn surface(&self) -> &Surface {
        &self.0.surface
    }
    
    // Get access to the underlying device
    pub fn device(&self) -> &Device {
        &self.0.device
    }
    
    // Get access to the underlying queue
    pub fn queue(&self) -> &Queue {
        &self.0.queue
    }
    
    // Get access to the underlying config
    pub fn config(&self) -> &Mutex<SurfaceConfiguration> {
        &self.0.config
    }
}