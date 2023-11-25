pub use wgpu::StoreOp;
use crate::format::{Texel, Conversion, ColorTexel, DepthElement, Depth, StencilElement, Stencil};
use super::{ColorLayout, DepthStencilLayout};

// What we should do when loading in data from the render target
// Even though WGPU has a LoadOp type, I still decided to implement one myself simply
// due to the fact that we can use type safety to store the texel color type
pub enum LoadOp<T: Texel> {
    Load,
    Clear(T::Storage),
}


// Operation applied to all types of render targets
pub struct Operation<T: Texel> {
    pub load: LoadOp<T>,
    pub store: StoreOp,
}

// Converts the Operation to a wgpu::Operations with the valid target type
fn convert<C: Conversion>(input: &Operation<C>) -> wgpu::Operations<C::Target> {
    let load = match input.load {
        LoadOp::Load => wgpu::LoadOp::Load,
        LoadOp::Clear(storage) => wgpu::LoadOp::Clear(C::into_target(storage)),
    };

    wgpu::Operations { load, store: input.store }
}

// Implemented for tuples of color attachment operators
pub trait ColorOperations<C: ColorLayout> {
    // Wgpu operations for each render target
    fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>>;
}

impl ColorOperations<()> for () {
    fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>> {
        Vec::new()
    }
}

impl<T: ColorTexel> ColorOperations<T> for Operation<T> {
    fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>> {
        let load = match self.load {
            LoadOp::Load => wgpu::LoadOp::Load,
            LoadOp::Clear(color) => wgpu::LoadOp::Clear(T::try_into_color(color).unwrap()),
        };

        vec![wgpu::Operations { load, store: self.store }]
    }
}

// Implemented for depth stencil attachment operators
pub trait DepthStencilOperations<DS: DepthStencilLayout> {
    // Load/store operations for the depth target (if it exists)
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>>;

    // Load/store operations for the stencil target (if it exists)
    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>>;
}

impl DepthStencilOperations<()> for () {
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>> {
        None
    }

    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>> {
        None
    }
}

impl<D: DepthElement> DepthStencilOperations<Depth<D>> for Operation<Depth<D>>
where
    Depth<D>: Texel + DepthStencilLayout + Conversion<Target = f32>,
{
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>> {
        Some(convert(self))
    }

    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>> {
        None
    }
}

impl<S: StencilElement> DepthStencilOperations<Stencil<S>> for Operation<Stencil<S>>
where
    Stencil<S>: Texel + DepthStencilLayout + Conversion<Target = u32>,
{
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>> {
        None
    }

    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>> {
        Some(convert(self))
    }
}
