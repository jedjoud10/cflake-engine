use world::World;


// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the uniforms
    type Resources<'w>: 'w;

    // Get the depth comparison setting
    fn depth_comparison() -> Option<Comparison> {
        Some(Comparison::Less)
    }


    // Get the transparency setting
    fn blend_mode() -> Option<BlendMode> {
        None
    }

    // Get the rasterizer primitive mode
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Triangles {
            cull: Some(FaceCullMode::Back(true)),
        }
    }

    // Fetch the property block resources
    fn fetch_resources<'w>(world: &'w World) -> Self::Resources<'w>;

    /*
    // Set the global and static instance properties when we start batch rendering
    fn set_static_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
    );

    // Set the uniforms for this property block right before we render our surface
    fn set_surface_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        renderer: &Renderer,
    );

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties(
        uniforms: &mut Uniforms,
        main: &DefaultMaterialResources,
        resources: &mut Self::Resources,
        instance: &Self,
    );
    */
}