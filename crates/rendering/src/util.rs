use crate::{
    AlbedoMap, CameraBuffer, MaskMap, Mesh, NormalMap, SceneBuffer, TimingBuffer, WindowBuffer,
};

use assets::Assets;

use ecs::Entity;
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, Depth, GpuPod,
    Graphics, LoadOp, Operation, RenderPass, SamplerFilter,
    SamplerMipMaps, SamplerSettings, SamplerWrap, StoreOp, Texel, Texture, Texture2D,
    TextureMipMaps, TextureMode, TextureUsage, UniformBuffer, RGBA, BGRA, SwapchainFormat, RenderPipeline, VertexModule, FragmentModule, Compiler, Shader, VertexConfig, PrimitiveConfig, Normalized,
};
use utils::{Handle, Storage};

// Create a new uniform buffer with default contents
pub(crate) fn create_uniform_buffer<T: GpuPod + Default, const COUNT: usize>(graphics: &Graphics, usages: BufferUsage) -> UniformBuffer<T> {
    UniformBuffer::from_slice(
        graphics,
        &[T::default(); COUNT],
        BufferMode::Dynamic,
        usages,
    )
    .unwrap()
}

// Create a 4x4 texture 2D with the given value
pub(crate) fn create_texture2d<T: Texel>(graphics: &Graphics, value: T::Storage) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        Some(&[value; 16]),
        vek::Extent2::broadcast(4),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}