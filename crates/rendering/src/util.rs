use graphics::{pod::GpuPod, context::Graphics, buffer::{BufferUsage, UniformBuffer, BufferMode}, format::Texel, texture::{Texture2D, TextureUsage, TextureViewSettings, Texture, SamplerSettings, TextureMipMaps}};

// Create a new uniform buffer with default contents
pub(crate) fn create_uniform_buffer<T: GpuPod + Default, const COUNT: usize>(
    graphics: &Graphics,
    usages: BufferUsage,
) -> UniformBuffer<T> {
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
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        &[TextureViewSettings::whole::<
            <Texture2D<T> as Texture>::Region,
        >()],
        Some(TextureMipMaps::Disabled)
    )
    .unwrap()
}
