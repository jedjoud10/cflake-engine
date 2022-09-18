use crate::{
    material::{Material},
};





use world::{World};

// Render to the main global shadow map if the material is a shadow caster
pub(crate) fn render_shadows<M: for<'w> Material<'w>>(_world: &mut World) {
    if !M::shadow_caster() {
        
    }

    /*
    let mut property_block_resources = M::fetch_resources(world);
    let ecs = world.get::<Scene>().unwrap();
    let materials = world.get::<Storage<M>>().unwrap();
    let meshes = world.get::<Storage<Mesh>>().unwrap();
    let window = world.get::<Window>().unwrap();
    let mut _shading = world.get_mut::<ClusteredShading>().unwrap();
    let shading = &mut *_shading;
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut stats = world.get_mut::<RenderedFrameStats>().unwrap();
    stats.unique_materials += 1;    
    */
}