use vulkan::{vk, Device};

use crate::ModuleKind;

// Translate GLSL data into SPIRV byte code
pub unsafe fn translate_glsl_spirv(device: &Device, file_name: &str, code: &str, kind: ModuleKind) -> Vec<u32> {
    let kind = match kind {
        ModuleKind::Vertex => vulkan::ShaderKind::Vertex,
        ModuleKind::Fragment => vulkan::ShaderKind::Fragment,
        ModuleKind::Compute => vulkan::ShaderKind::Compute,
    };
    
    device.translate_glsl_spirv(code, file_name, "main", kind)
}

// Compile an actual SPIRV module into a Vulkan module
pub unsafe fn compile_shader_module(device: &Device, bytecode: Vec<u32>) -> vk::ShaderModule {
    device.compile_shader_module(&bytecode)
}