use cflake_engine::prelude::*;

// Custom material example game window
fn main() {
    App::default()
        .set_app_name("cflake engine custom material example")
        .insert_init(init)
        .execute();
}

// Custom material struct and trait implementations
pub struct CustomMaterial;

impl Material for CustomMaterial {
    type Resources<'w> = ();
    type RenderPath = Direct;
    type Settings<'s> = ();
    type Query<'a> = &'a ();

    // Load the respective PBR shader modules and compile them
    fn shader<P: Pass>(
        _settings: &Self::Settings<'_>,
        graphics: &Graphics,
        assets: &Assets,
    ) -> Option<Shader> {
        match P::pass_type() {
            crate::PassType::Deferred => {
                let vert = assets
                    .load::<VertexModule>("user/shaders/custom.vert")
                    .unwrap();

                let frag = assets
                    .load::<FragmentModule>("user/shaders/custom.frag")
                    .unwrap();

                let mut compiler = Compiler::new(assets, graphics);
                compiler.use_uniform_buffer::<CameraUniform>("camera");

                compiler.use_push_constant_layout(
                    PushConstantLayout::vertex(<vek::Vec4<vek::Vec4<f32>> as GpuPod>::size())
                        .unwrap(),
                );

                Some(Shader::new(vert, frag, &compiler).unwrap())
            }

            crate::PassType::Shadow => None,
        }
    }

    fn fetch<P: Pass>(_world: &World) -> Self::Resources<'_> {}

    fn set_global_bindings<'r, P: Pass>(
        _resources: &'r mut Self::Resources<'_>,
        group: &mut BindGroup<'r>,
        default: &DefaultMaterialResources<'r>,
    ) {
        group
            .set_uniform_buffer("camera", default.camera_buffer, ..)
            .unwrap();
    }

    fn set_push_constants<'r, 'w, P: Pass>(
        &self,
        renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        constants: &mut PushConstants<ActiveRenderPipeline<P::C, P::DS>>,
    ) {
        let matrix = renderer.matrix;
        let cols = matrix.cols;
        let bytes = GpuPod::into_bytes(&cols);
        constants.push(bytes, 0, ModuleVisibility::Vertex).unwrap();
    }
}

// Creates a movable camera
fn init(world: &mut World) {
    // Fetch the required resources from the world
    world.insert::<Storage<CustomMaterial>>(Storage::default());
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut materials = world.get_mut::<Storage<CustomMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();

    // Get the material id (also registers the material pipeline)
    asset!(assets, "user/shaders/custom.frag", "/examples/assets/");
    asset!(assets, "user/shaders/custom.vert", "/examples/assets/");
    let id = pipelines
        .register::<CustomMaterial>(&graphics, &assets)
        .unwrap();

    // Get the default meshes from the forward renderer
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let plane = renderer.plane.clone();
    let sphere = renderer.sphere.clone();

    // Create a new material instance
    let material = materials.insert(CustomMaterial);

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

    // Create a prefab that contains the renderer, customized surface, and default position
    let renderer = Renderer::default();
    let position = Position::default();
    let surface = Surface::new(sphere, material, id);
    scene.prefabify("sphere", (renderer, position, surface));

    // ADD THE ENTITIES NOW!!
    for x in 0..25 {
        let mut entry = scene.instantiate("sphere").unwrap();
        let position = entry.get_mut::<Position>().unwrap();
        *position = Position::at_xyz((x / 5) as f32 * 4.0, 1.0, (x % 5) as f32 * 4.0);
    }

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight {
        intensity: 1.0,
        color: vek::Rgb::broadcast(255),
    };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}
