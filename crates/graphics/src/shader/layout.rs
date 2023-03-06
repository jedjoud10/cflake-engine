use crate::{
    BindLayoutUseError, GpuPodRelaxed, Texel, Texture, UniformBuffer,
};

// Bindings that must be defined whenever we create a shader or compute shader
// These bindings contain the type and usages for each binding entry within the shader
pub struct BindLayout {}

impl BindLayout {
    // Create the defitions for zero bindings
    pub fn new() -> Self {
        Self {}
    }

    // Define a uniform buffer type's inner struct type
    pub fn use_ubo<'s, T: GpuPodRelaxed>(
        &mut self,
        name: &'s str,
    ) -> Result<(), BindLayoutUseError<'s>> {
        Ok(())
    }

    // Define a "fill" uniform buffer whose layout is defined at runtime
    pub fn use_fill_ubo<'s>(
        &mut self,
        name: &'s str,
    ) -> Result<(), BindLayoutUseError<'s>> {
        Ok(())
    }

    // Define a uniform texture's type and texel
    pub fn use_texture<'s, T: Texture>(
        &mut self,
        name: &'s str,
    ) -> Result<(), BindLayoutUseError<'s>> {
        Ok(())
    }

    // Define a uniform sampler's type and texel
    /*
    // This is called automatically if the sampler is bound to the texture
    pub fn set_sampler<'s, T: Texture>(
        &mut self,
        name: &'s str,
    ) -> Result<(), BindLayoutUseError<'s>> {
        Ok(())
    }
    */
}
