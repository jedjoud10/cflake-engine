use crate::prelude::{Depth, DepthTexel, Element, Stencil, StencilTexel};
use super::Attachment;

// This trait is implemented for color texels exclusively and the unit tuple
pub trait PainterColorLayout {}
impl PainterColorLayout for () {}

// This trait is implemented for depth texels exclusively and the unit tuple
pub trait PainterDepthTexel {}
impl PainterDepthTexel for () {}
impl<E: Element> PainterDepthTexel for Depth<E> where Self: DepthTexel {}

// This trait is implemented for stencil texels exclusively and the unit tuple
pub trait PainterStencilTexel {}
impl PainterStencilTexel for () {}
impl<E: Element> PainterStencilTexel for Stencil<E> where Self: StencilTexel {}
