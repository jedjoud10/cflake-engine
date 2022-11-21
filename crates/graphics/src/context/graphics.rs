use std::ffi::{CStr, CString};
use super::{Window, FrameRateLimit, WindowSettings};
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
pub struct Graphics {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Graphics {
    // Create a new wgpu graphics context based on the window wrapper
    pub(crate) fn new(
        window: &winit::window::Window,
        graphic_settings: &GraphicSettings,
        window_settings: &WindowSettings,
    ) -> Graphics {
        // Create the wgpu instance and main surface
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        // Pick an appopriate adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).unwrap();       
        
        // Create the device and main queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),                
                label: None,
            },
            None,
        )).unwrap();

        // Create the surface that we will render to
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);
        
        Self {
            surface,
            device,
            queue,
            config,
        }
    }


    // Draw the main window swapchain sheize
    pub(crate) unsafe fn draw(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
            })],
            depth_stencil_attachment: None,
        });
        drop(render_pass);


        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    // Destroy the context after we've done using it
    pub(crate) unsafe fn destroy(mut self) {
    }
}