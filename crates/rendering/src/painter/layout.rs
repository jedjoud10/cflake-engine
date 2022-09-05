use crate::prelude::{Depth, DepthTexel, Element, Stencil, StencilTexel};
use super::Attachment;

// This is implemented for every texel type that will be used withint the painter
pub trait PainterBitmask { const BITMASK: u32; }
impl PainterBitmask for () { const BITMASK: u32 = 0; }

// This trait is implemented for color texels exclusively and the unit tuple
pub trait PainterColorLayout: PainterBitmask {}
impl PainterColorLayout for () {}

// This trait is implemented for depth texels exclusively and the unit tuple
pub trait PainterDepthTexel: PainterBitmask {}
impl PainterDepthTexel for () {}
impl<E: Element> PainterBitmask for Depth<E> where Self: DepthTexel, { const BITMASK: u32 = 1; }
impl<E: Element> PainterDepthTexel for Depth<E> where Self: DepthTexel, Self: PainterBitmask, {}

// This trait is implemented for stencil texels exclusively and the unit tuple
pub trait PainterStencilTexel: PainterBitmask {}
impl PainterStencilTexel for () {}
impl<E: Element> PainterBitmask for Stencil<E> where Self: StencilTexel { const BITMASK: u32 = 2; }
impl<E: Element> PainterStencilTexel for Stencil<E> where Self: StencilTexel, Self: PainterBitmask {}
