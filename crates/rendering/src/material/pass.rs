use assets::Assets;
use graphics::{ColorLayout, DepthStencilLayout, RenderPass, BindGroup, Graphics, Shader};
use world::World;

use crate::{RenderPath, DefaultMaterialResources};

// A single render pass that will write to some output resources (images / buffers)
// A render-pass can use some input resources like persistant images/buffers to render 
// the scene from a specific viewpoint onto output resources 
pub trait Pass {
    type ColorLayout: ColorLayout;
    type DepthStencilLayout: DepthStencilLayout;
    type Resources<'w>: 'w;
    type Settings<'s>;

    fn shader(settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Shader;

    fn create_render_pass() -> RenderPass<Self::ColorLayout, Self::DepthStencilLayout>;

    fn fetch(world: &World) -> Self::Resources<'_>;

    fn set_render_pass_bindings<'r>(
        _resources: &'r mut Self::Resources<'_>,
        _group: &mut BindGroup<'r>,
        _default: &DefaultMaterialResources<'r>,
    ) {
    }

    fn render<R: RenderPath>();
}