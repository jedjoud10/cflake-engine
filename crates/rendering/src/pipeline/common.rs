use graphics::{ActiveRenderPipeline, ColorLayout, DepthStencilLayout};

use crate::{DefaultMaterialResources, Mesh, MeshAttribute, MeshAttributes, RenderPath};

// Set a mesh binding vertex buffer to the current render pass
pub(crate) fn set_vertex_buffer_attribute<
    'a,
    'r,
    A: MeshAttribute,
    R: RenderPath,
    C: ColorLayout,
    DS: DepthStencilLayout,
>(
    supported: MeshAttributes,
    mesh: &'r Mesh<R>,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveRenderPipeline<'a, 'r, '_, C, DS>,
    index: &mut u32,
    last: &mut Option<&'r R::AttributeBuffer<A>>,
) where
    for<'x> &'x R::AttributeBuffer<A>: PartialEq<&'x R::AttributeBuffer<A>>,
{
    // If the material doesn't support the attribute, no need to set it
    if !supported.contains(A::ATTRIBUTE) {
        return;
    }

    // Check if the mesh contains the attribute, and if it does, render it
    if let Ok(buffer) = mesh.vertices().attribute::<A>() {
        // Only set the buffer if necessary
        if *last != Some(buffer) {
            R::set_vertex_buffer(*index, .., buffer, defaults, active).unwrap();
            *last = Some(buffer);
        }

        *index += 1;
    }
}

// Set a mesh triangle buffer to the current render pass
pub(crate) fn set_index_buffer_attribute<
    'a,
    'r,
    R: RenderPath,
    C: ColorLayout,
    DS: DepthStencilLayout,
>(
    mesh: &'r Mesh<R>,
    defaults: &DefaultMaterialResources<'r>,
    active: &mut ActiveRenderPipeline<'a, 'r, '_, C, DS>,
    last: &mut Option<&R::TriangleBuffer<u32>>,
) where
    for<'x> &'x R::TriangleBuffer<u32>: PartialEq<&'x R::TriangleBuffer<u32>>,
{
    // Get the triangle buffer from the mesh
    let triangles = mesh.triangles();
    let buffer = triangles.buffer();

    // Only set the triangles if necessary
    if *last != Some(buffer) {
        R::set_index_buffer(.., buffer, defaults, active).unwrap();
    }
}
