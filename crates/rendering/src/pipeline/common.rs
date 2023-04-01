use graphics::{
    ActiveGraphicsPipeline, ColorLayout, DepthStencilLayout,
};

use crate::{
    ActiveScenePipeline, DefaultMaterialResources, Mesh,
    MeshAttribute, MeshAttributes, RenderPath,
};

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
    active: &mut ActiveGraphicsPipeline<'a, 'r, '_, C, DS>,
    index: &mut u32,
) {
    // If the material doesn't support the attribute, no need to set it
    if !supported.contains(A::ATTRIBUTE) {
        return;
    }

    // Check if the mesh contains the attribute, and if it does, render it
    if let Ok(buffer) = mesh.vertices().attribute::<A>() {
        R::set_vertex_buffer(*index, .., buffer, defaults, active)
            .unwrap();
        *index += 1;
    }
}
