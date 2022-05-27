use crate::{mesh::SubMesh, object::ToGlName};

// A trait that will be implemented for objects that can be drawed onto the screen, like submeshes or clustered submeshes
pub trait Draw: Sized {
    // This will cull any objects that must not be drawn
    fn cull(frustum: f32, filter: &mut Vec<Self>);

    // This will draw a set of unique drawable items
    fn draw(ctx: &mut Context, framebuffer: &mut Framebuffer, objects: Vec<Self>);

    // This will draw a single object onto the framebuffer
    fn draw_unique(&self, ctx: &mut Context, framebuffer: &mut Framebuffer);
}