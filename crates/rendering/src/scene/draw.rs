use crate::{mesh::SubMesh, object::ToGlName, framebuffer::Canvas, context::Context};

// A trait that will be implemented for objects that can be drawed onto the screen, like submeshes or clustered submeshes
pub trait Draw: Sized {
    // This will cull any objects that must not be drawn
    fn cull(frustum: f32, filter: &mut Vec<Self>);

    // This will draw a set of unique drawable items
    fn draw(ctx: &mut Context, canvas: &mut Canvas, objects: Vec<Self>);

    // This will draw a single object onto the framebuffer
    fn draw_unique(&self, ctx: &mut Context, canvas: &mut Canvas);
}