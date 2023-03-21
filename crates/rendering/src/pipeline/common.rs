use graphics::{ActiveGraphicsPipeline, ColorLayout, DepthStencilLayout, DrawIndirectBuffer, DrawIndexedIndirectBuffer};
use utils::Storage;
use crate::{Material, Surface, Mesh};


// Common draw method that will simply draw a mesh onto the active graphics pipeline
pub fn draw<M: Material, C: ColorLayout, DS: DepthStencilLayout>(
    surface: &Surface<M>,
    indirect: &Storage<DrawIndexedIndirectBuffer>,
    mesh: &Mesh,
    active: &mut ActiveGraphicsPipeline<C, DS>
) {
    // Draw the triangulated mesh
    match surface.indirect.as_ref() {
        // Draw using indirect buffer
        Some(handle) => {
            let buffer = indirect.get(handle);
            //active.draw_indexed_indirect();
        },
    
        // Draw the mesh normally
        None => {
            let indices = 0..(mesh.triangles().buffer().len() as u32 * 3);
            active.draw_indexed(indices, 0..1);
        },
    }
}
