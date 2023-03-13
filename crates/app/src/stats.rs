use ecs::Scene;
use graphics::{Graphics, GraphicsStats};
use gui::{egui, Interface};
use rendering::Renderer;
use utils::Time;
use world::World;

// Render some debug statistical EGUI windows
pub(crate) fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let stats = world.get::<GraphicsStats>().unwrap();
    let scene = world.get::<Scene>().unwrap();
    let gui = world.get_mut::<Interface>().unwrap();
    let time = world.get::<Time>().unwrap();

    // Get the graphics stats
    let GraphicsStats {
        acquires,
        submissions,
        stalls,
        staging_buffers,
        cached_samplers,
        cached_bind_group_layouts,
        cached_pipeline_layouts,
        cached_bind_groups,
    } = *stats;

    // Get the GPU stats
    let gpuinfo = graphics.adapter().get_info();
    let name = gpuinfo.name;
    let backend = gpuinfo.backend;
    let device = gpuinfo.device_type;
    let _driver = gpuinfo.driver;

    // Graphics Stats
    egui::Window::new("Graphics Stats").show(&gui, |ui| {
        ui.label(format!("Acquires: {acquires}"));
        ui.label(format!("Submissions: {submissions}"));
        ui.label(format!("Stalls: {stalls}"));
        ui.label(format!("Stg Buffers: {staging_buffers}"));

        ui.heading("Cached Graphics Data");
        ui.label(format!("Samplers: {cached_samplers}"));
        ui.label(format!(
            "Pipeline Layouts: {cached_pipeline_layouts}"
        ));
        ui.label(format!(
            "Bind Group Layouts: {cached_bind_group_layouts}"
        ));
        ui.label(format!("Bind Group: {cached_bind_groups}"));

        ui.heading("Graphics Processor");
        ui.label(format!("Name: {name}"));
        ui.label(format!("Backend: {backend:#?}"));
        ui.label(format!("Type: {device:#?}"));
    });

    // General Performance
    egui::Window::new("General Performance").show(&gui, |ui| {
        let last = time.delta().as_secs_f32();
        let mut out = 0.0;
        ui.memory_mut(|memory| {
            let indeed = memory.data.get_temp_mut_or_insert_with(egui::Id::new(0), || last);
            *indeed = *indeed * 0.99 + last * 0.01;
            out = *indeed;
        });
        
        let ms = out * 1000.0;
        ui.label(format!("Delta (ms/f): {:.3}", ms));
        
        let fps = 1.0 / out;
        ui.label(format!("FPS (f/s): {:.0}", fps));
    });

    // ECS Stats
    egui::Window::new("Entity Components").show(&gui, |ui| {
        ui.label(format!("Entities: {}", scene.entities().len().to_string()));
        
        let iter = scene
            .archetypes()
            .iter()
            .map(|x| 
                x.1.entities().len() * (x.1.mask().count_ones() as usize)
            );
        ui.label(format!("Components: {}", iter.sum::<usize>()));

        ui.label(format!("Registered Components: {}", ecs::count()));

        ui.label(format!("Archetypes: {}", 
            scene
                .archetypes()
                .len())
        );
    });
}
