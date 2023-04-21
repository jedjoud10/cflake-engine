use std::time::Duration;

use ahash::AHashMap;
use gui::egui::{RichText, Color32};

use crate::prelude::*;

// Simple type to check if stats are enabled or not
struct StatsState(bool);

// Event stats for the init, update, and tick events
// Timings are in milliseconds btw
#[derive(Default)]
pub(crate) struct EventStatsDurations {
    pub init: Vec<(StageId, f32)>,
    pub init_total: f32,
    pub update: Vec<(StageId, f32)>,
    pub update_total: f32,
    pub tick: Vec<(StageId, f32)>,
} 

// Map a button to be able to hide/show the stats
// Also add the default internal resources
fn init(world: &mut World) {
    world.insert(StatsState(false));
    world.insert(EventStatsDurations::default());
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("toggle-stats", Button::P);
}

// Render some debug statistical EGUI windows
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let stats = world.get::<GraphicsStats>().unwrap();
    let mut state = world.get_mut::<StatsState>().unwrap();
    let input = world.get::<Input>().unwrap();
    let scene = world.get::<Scene>().unwrap();
    let gui = world.get_mut::<Interface>().unwrap();
    let time = world.get::<Time>().unwrap();
    let durations = world.get::<EventStatsDurations>().unwrap();

    // Check if stats are enabled at the moment
    match input.get_button("toggle-stats") {
        ButtonState::Pressed => state.0 = !state.0,
        _ => {}
    };

    // If the stats are disabled, then don't continue
    if !state.0 {
        return;
    }

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
        adapters,
        devices,
        pipeline_layouts,
        shader_modules,
        bind_group_layouts,
        bind_groups,
        command_buffers,
        render_pipelines,
        buffers,
        textures,
        texture_views,
        samplers,
        cached_shaders,
    } = *stats;

    // Get the GPU stats
    let gpuinfo = graphics.adapter().get_info();
    let name = gpuinfo.name;
    let backend = gpuinfo.backend;
    let device = gpuinfo.device_type;
    let _driver = gpuinfo.driver;

    let mut frame = egui::containers::Frame::window(&gui.style());
    frame.rounding = egui::epaint::Rounding::none();
    frame.shadow = egui::epaint::Shadow::small_dark();

    // Function that we will use to chose the color for the colored labels
    fn pick_stats_label_color(percentage: f32) -> vek::Rgb<u8> {
        let green = vek::Rgb::new(48, 150, 58u8).as_::<f32>();
        let red = vek::Rgb::new(150, 21, 21u8).as_::<f32>();

        (vek::Rgb::lerp(green, red, 1.0-(1.0-percentage).powf(3.0))).as_::<u8>()
    }

    // Event stats for init, update, and tick
    egui::Window::new("Time taken per Event").anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::ZERO).frame(frame).show(
        &gui,
        |ui| {
            ui.heading("Initialization Events Registry");

            egui::Grid::new("init events")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    let mut vec = durations.init.clone();
                    vec.sort_by(|(_, a), (_, b)| f32::total_cmp(b, a));
                    for (stage, duration) in vec.into_iter().take(10)
                    {
                        let color = pick_stats_label_color(duration / durations.init_total);
                        let color = egui::Color32::from_rgb(color.r, color.g, color.b);
                        ui.colored_label(color, stage.system.name);
                        ui.colored_label(color, format!("{duration:.2?}ms"));
                        ui.end_row();
                    }
                });

            ui.heading("Update Events Registry");
            egui::Grid::new("update events")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    let mut vec = durations.update.clone();
                    vec.sort_by(|(_, a), (_, b)| f32::total_cmp(b, a));
                    for (stage, duration) in vec.into_iter().take(10)
                    {
                        let color = pick_stats_label_color(duration / durations.update_total);
                        let color = egui::Color32::from_rgb(color.r, color.g, color.b);
                        ui.colored_label(color, stage.system.name);
                        ui.colored_label(color, format!("{duration:.2?}ms"));
                        ui.end_row();
                    }
                });
        }
    );

    // Graphics Stats
    egui::Window::new("Graphics Stats").anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO).frame(frame).show(
        &gui,
        |ui| {
            ui.label(format!("Acquires: {acquires}"));
            ui.label(format!("Submissions: {submissions}"));
            ui.label(format!("Stalls: {stalls}"));
            ui.label(format!("Stg Buffers: {staging_buffers}"));

            ui.heading("Cached Graphics Data");
            ui.label(format!("Shaders: {cached_shaders}"));
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

            ui.heading("WGPU Raw Data Types");
            ui.label(format!("Adapters: {}", adapters));
            ui.label(format!("Devices: {}", devices));
            ui.label(format!(
                "Pipeline Layouts: {}",
                pipeline_layouts
            ));
            ui.label(format!("Shader Modules: {}", shader_modules));
            ui.label(format!(
                "Bind Group Layouts: {}",
                bind_group_layouts
            ));
            ui.label(format!("Bind Groups: {}", bind_groups));
            ui.label(format!("Command Buffers: {}", command_buffers));
            ui.label(format!(
                "Graphic Pipelines: {}",
                render_pipelines
            ));
            ui.label(format!("Buffers: {}", buffers));
            ui.label(format!("Textures: {}", textures));
            ui.label(format!("Texture Views: {}", texture_views));
            ui.label(format!("Samplers: {}", samplers));
        },
    );

    // General Performance
    egui::Window::new("General Performance").anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO).frame(frame).show(
        &gui,
        |ui| {
            let last_delta = time.delta().as_secs_f32();
            let last_ticks_to_exec =
                time.ticks_to_execute().map(|x| x.get()).unwrap_or(0)
                    as f32;
            let mut out_delta = 0.0;
            let mut out_ticks_to_exec = 0.0;
            ui.memory_mut(|memory| {
                let indeed = memory.data.get_temp_mut_or_insert_with(
                    egui::Id::new(0),
                    || last_delta,
                );
                *indeed = *indeed * 0.99 + last_delta * 0.01;
                out_delta = *indeed;

                let indeed2 =
                    memory.data.get_temp_mut_or_insert_with(
                        egui::Id::new(1),
                        || last_ticks_to_exec,
                    );
                *indeed2 =
                    *indeed2 * 0.99 + last_ticks_to_exec * 0.01;
                out_ticks_to_exec = *indeed2;
            });

            let ms = out_delta * 1000.0;
            ui.label(format!("Delta (ms/f): {:.3}", ms));

            let fps = 1.0 / out_delta;
            ui.label(format!("FPS (f/s): {:.0}", fps));

            let ticks = out_ticks_to_exec;
            ui.label(format!("Ticks to execute: {:.3}", ticks));
        },
    );

    // ECS Stats
    egui::Window::new("Entity Components").anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::ZERO).frame(frame).show(
        &gui,
        |ui| {
            ui.label(format!("Entities: {}", scene.entities().len()));

            let iter = scene.archetypes().iter().map(|x| {
                x.1.entities().len()
                    * (x.1.mask().count_ones() as usize)
            });
            ui.label(format!("Components: {}", iter.sum::<usize>()));
            ui.label(format!(
                "Registered Components: {}",
                ecs::count()
            ));
            ui.label(format!(
                "Archetypes: {}",
                scene.archetypes().len()
            ));
            let ratio = scene.entities().len() as f32
                / scene.archetypes().len() as f32;
            ui.label(format!("E/A Ratio: {:.1}", ratio));

            ui.heading("Registered Components Table");

            egui::Grid::new("components")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    for count in 0..ecs::count() {
                        let mask = ecs::Mask::one() << count;
                        ui.label(format!("Mask: 1 << {count}",));
                        ui.label(format!(
                            "Name: {}",
                            ecs::name(mask).unwrap()
                        ));
                        ui.end_row();
                    }
                });

            ui.heading("Archetypes Table");

            egui::Grid::new("archetypes")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    for (mask, archetype) in scene.archetypes().iter()
                    {
                        ui.label(format!("Mask: {mask}"));
                        ui.label(format!(
                            "Entities: {}",
                            archetype.entities().len()
                        ));
                        ui.end_row();
                    }
                });
        },
    );

    // Terrain stats
    if let Ok(terrain) = world.get::<Terrain>() {
        egui::Window::new("Terrain").anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO).frame(frame).show(
            &gui,
            |ui| {
                let settings = &terrain.settings;
                ui.heading("Manager settings");
                ui.label(format!("Chunk resolution: {}", settings.resolution()));
                ui.label(format!("Chunk count: {}", settings.chunks_count()));
                ui.label(format!("Is Blocky?: {}", settings.blocky()));
                ui.label(format!("Is Low-Poly?: {}", settings.lowpoly()));
                ui.heading("Memory settings");
                ui.label(format!("Allocation count: {}", settings.allocation_count()));
                ui.label(format!("Sub-allocation count: {}", settings.sub_allocation_count()));

                let pending = scene.query::<&Chunk>().into_iter().filter(|c| c.state() == ChunkState::Pending).count();
                let generated = scene.query::<&Chunk>().into_iter().filter(|c| c.state() == ChunkState::Generated).count();
                let visible = scene.query::<(&Chunk, &Surface<TerrainMaterial>)>().into_iter().filter(|(_, s)| s.visible).count();
                ui.heading("Real-time stats");
                ui.label(format!("Pending chunks count: {}", pending));
                ui.label(format!("Generated chunks count: {}", generated));
                ui.label(format!("Visible chunks count: {}", visible));
            },
        );
    }
}

// Statistics system
pub fn system(system: &mut System) {
    system.insert_init(init);
    system.insert_update(update);
}