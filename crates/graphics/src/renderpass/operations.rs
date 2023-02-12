use crate::{Texel, ColorLayout, DepthStencilLayout, ColorTexel, Depth, Stencil, DepthElement, StencilElement};

// What we should do when loading in data from the render target
// Even though WGPU has a LoadOp type, I still decided to implement one myself simply
// due to the fact that we can use type safety to store the texel color type
pub enum LoadOp<T: Texel> {
    Load,
    Clear(T::Storage),
}

// What we should do when writing data to the render target
pub enum StoreOp {
    Ignore,
    Store,
}

// Operation applied to all types of render targets
pub struct Operation<T: Texel> {
    pub load: LoadOp<T>,
    pub store: StoreOp,
}

// Implemented for tuples of color attachment operators
pub trait ColorOperations<C: ColorLayout> {
    // Wgpu operations for each render target
    fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>>;
}

impl<T: ColorTexel> ColorOperations<T> for Operation<T> {
    fn operations(&self) -> Vec<wgpu::Operations<wgpu::Color>> {
        let load = match self.load {
            LoadOp::Load => wgpu::LoadOp::Load,
            LoadOp::Clear(color) => wgpu::LoadOp::Clear(T::into_color(color).unwrap()),
        };

        let store = match self.store {
            StoreOp::Ignore => true,
            StoreOp::Store => true,
        };

        vec![wgpu::Operations { load, store }]
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

impl<D: DepthElement> DepthStencilOperations<Depth<D>> for Depth<D> where Self: Texel + DepthStencilLayout {
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>> {
        Some(todo!())
    }

    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>> {
        None
    }
}

impl<S: StencilElement> DepthStencilOperations<Stencil<S>> for Stencil<S> where Self: Texel + DepthStencilLayout {
    fn depth_operations(&self) -> Option<wgpu::Operations<f32>> {
        None
    }

    fn stencil_operations(&self) -> Option<wgpu::Operations<u32>> {
        Some(todo!())
    }
}