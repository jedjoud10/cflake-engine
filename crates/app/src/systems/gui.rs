use gui::egui::Widget;

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
    pub tick_total: f32,
}

// Map a button to be able to hide/show the stats
// Also add the default internal resources
fn init(world: &mut World) {
    world.insert(StatsState(false));
    world.insert(EventStatsDurations::default());
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("toggle-stats", KeyboardButton::P);
}

// Render some debug statistical EGUI windows
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let stats = world.get::<GraphicsStats>().unwrap();
    let mut state = world.get_mut::<StatsState>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
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
        compute_pipelines,
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
    frame.shadow = egui::epaint::Shadow::NONE;

    // Function that we will use to chose the color for the colored labels
    fn pick_stats_label_color(percentage: f32) -> vek::Rgb<u8> {
        let green = vek::Rgb::new(48, 150, 58u8).as_::<f32>();
        let red = vek::Rgb::new(150, 21, 21u8).as_::<f32>();

        (vek::Rgb::lerp(green, red, 1.0 - (1.0 - percentage).powf(3.0))).as_::<u8>()
    }

    // Event stats for init, update, and tick
    egui::Window::new("Time taken per Event")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::ZERO)
        .frame(frame)
        .collapsible(true)
        .default_open(false)
        .show(&gui, |ui| {
            ui.heading("Initialization Events Registry");

            egui::Grid::new("init events")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    let mut vec = durations.init.clone();
                    vec.sort_by(|(_, a), (_, b)| f32::total_cmp(b, a));
                    for (stage, duration) in vec.into_iter().take(10) {
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
                    for (stage, duration) in vec.into_iter().take(10) {
                        let color = pick_stats_label_color(duration / durations.update_total);
                        let color = egui::Color32::from_rgb(color.r, color.g, color.b);
                        ui.colored_label(color, stage.system.name);
                        ui.colored_label(color, format!("{duration:.2?}ms"));
                        ui.end_row();
                    }
                });

            ui.heading("Tick Events Registry");
            egui::Grid::new("tick events")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    let mut vec = durations.tick.clone();
                    vec.sort_by(|(_, a), (_, b)| f32::total_cmp(b, a));
                    for (stage, duration) in vec.into_iter().take(10) {
                        let color = pick_stats_label_color(duration / durations.tick_total);
                        let color = egui::Color32::from_rgb(color.r, color.g, color.b);
                        ui.colored_label(color, stage.system.name);
                        ui.colored_label(color, format!("{duration:.2?}ms"));
                        ui.end_row();
                    }
                });
        });

    // Graphics Stats
    egui::Window::new("Graphics Stats")
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .frame(frame)
        .show(&gui, |ui| {
            ui.label(format!("Acquires: {acquires}"));
            ui.label(format!("Submissions: {submissions}"));
            ui.label(format!("Stalls: {stalls}"));
            ui.label(format!("Stg Buffers: {staging_buffers}"));

            ui.heading("Cached Graphics Data");
            ui.label(format!("Shaders: {cached_shaders}"));
            ui.label(format!("Samplers: {cached_samplers}"));
            ui.label(format!("Pipeline Layouts: {cached_pipeline_layouts}"));
            ui.label(format!("Bind Group Layouts: {cached_bind_group_layouts}"));
            ui.label(format!("Bind Group: {cached_bind_groups}"));

            ui.heading("Graphics Processor");
            ui.label(format!("Name: {name}"));
            ui.label(format!("Backend: {backend:#?}"));
            ui.label(format!("Type: {device:#?}"));

            ui.heading("WGPU Raw Data Types");
            ui.label(format!("Adapters: {}", adapters));
            ui.label(format!("Devices: {}", devices));
            ui.label(format!("Pipeline Layouts: {}", pipeline_layouts));
            ui.label(format!("Shader Modules: {}", shader_modules));
            ui.label(format!("Bind Group Layouts: {}", bind_group_layouts));
            ui.label(format!("Bind Groups: {}", bind_groups));
            ui.label(format!("Command Buffers: {}", command_buffers));
            ui.label(format!("Render Pipelines: {}", render_pipelines));
            ui.label(format!("Compute Pipelines: {}", compute_pipelines));
            ui.label(format!("Buffers: {}", buffers));
            ui.label(format!("Textures: {}", textures));
            ui.label(format!("Texture Views: {}", texture_views));
            ui.label(format!("Samplers: {}", samplers));
        });

    // General Performance
    egui::Window::new("General Performance")
        .frame(frame)
        .show(&gui, |ui| {
            let last_delta = time.delta().as_secs_f32();
            let last_ticks_to_exec = time.ticks_to_execute().map(|x| x.get()).unwrap_or(0) as f32;
            let mut out_delta = 0.0;
            let mut out_ticks_to_exec = 0.0;
            ui.memory_mut(|memory| {
                let indeed = memory
                    .data
                    .get_temp_mut_or_insert_with(egui::Id::new(0), || last_delta);
                *indeed = *indeed * 0.99 + last_delta * 0.01;
                out_delta = *indeed;

                let indeed2 = memory
                    .data
                    .get_temp_mut_or_insert_with(egui::Id::new(1), || last_ticks_to_exec);
                *indeed2 = *indeed2 * 0.99 + last_ticks_to_exec * 0.01;
                out_ticks_to_exec = *indeed2;
            });

            let since = time.startup().elapsed().as_secs_f32();
            ui.label(format!("Seconds since startup: {:.3}", since));

            let ms = out_delta * 1000.0;
            ui.label(format!("Delta (ms/f): {:.3}", ms));

            let fps = 1.0 / out_delta;
            ui.label(format!("FPS (f/s): {:.0}", fps));

            let ticks = out_ticks_to_exec;
            ui.label(format!("Ticks to execute: {:.3}", ticks));

            ui.label(format!("Tick-rate: {:.3} t/s", utils::TICKS_PER_SEC))
        });

    // ECS Stats
    egui::Window::new("Entity Components")
        .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::ZERO)
        .frame(frame)
        .collapsible(true)
        .default_open(false)
        .show(&gui, |ui| {
            ui.label(format!("Entities: {}", scene.entities().len()));

            let iter = scene
                .archetypes()
                .iter()
                .map(|x| x.1.entities().len() * (x.1.mask().count_ones() as usize));
            ui.label(format!("Components: {}", iter.sum::<usize>()));
            ui.label(format!("Registered Components: {}", ecs::count()));
            ui.label(format!("Archetypes: {}", scene.archetypes().len()));
            let ratio = scene.entities().len() as f32 / scene.archetypes().len() as f32;
            ui.label(format!("E/A Ratio: {:.1}", ratio));

            ui.collapsing("Registered Components Table", |ui| {
                egui::Grid::new("components")
                    .min_col_width(0f32)
                    .max_col_width(400f32)
                    .striped(true)
                    .show(ui, |ui| {
                        for count in 0..ecs::count() {
                            let mask = ecs::Mask::one() << count;
                            ui.label(format!("Mask: 1 << {count}",));
                            ui.label(format!("Name: {}", ecs::name(mask).unwrap()));
                            ui.end_row();
                        }
                    });
            });

            ui.collapsing("Archetypes Table", |ui| {
                egui::Grid::new("archetypes")
                    .min_col_width(0f32)
                    .max_col_width(400f32)
                    .striped(true)
                    .show(ui, |ui| {
                        for (mask, archetype) in scene.archetypes().iter() {
                            ui.label(format!("Mask: {mask}"));
                            ui.label(format!("Entities: {}", archetype.entities().len()));
                            ui.end_row();
                        }
                    });
            });
            

            ui.collapsing("Prefabs Table", |ui| {
                egui::Grid::new("prefabs")
                    .min_col_width(0f32)
                    .max_col_width(400f32)
                    .striped(true)
                    .show(ui, |ui| {
                        for (name, (_, mask)) in scene.prefabs() {
                            ui.label(format!("Name: {name}"));
                            ui.label(format!("Mask: {mask}"));
                            ui.end_row();
                        }
                    });
            });
        });

    // Terrain stats
    if let Ok(mut terrain) = world.get_mut::<Terrain>() {
        egui::Window::new("Terrain").frame(frame).show(&gui, |ui| {
            let settings = &mut terrain.settings;
            ui.heading("Manager settings");
            ui.label(format!("Chunk resolution: {}", settings.resolution()));
            ui.label(format!("Is Blocky?: {}", settings.blocky()));
            ui.label(format!("Is Low-Poly?: {}", settings.lowpoly()));
            ui.heading("Memory settings");
            ui.label(format!("Allocation count: {}", settings.allocation_count()));
            ui.label(format!(
                "Sub-allocation count: {}",
                settings.sub_allocation_count()
            ));

            let pending = scene
                .query::<&Chunk>()
                .into_iter()
                .filter(|c| c.state() == ChunkState::Pending)
                .count();
            let generated = scene
                .query::<&Chunk>()
                .into_iter()
                .filter(|c| matches!(c.state(), ChunkState::Generated { .. }))
                .count();
            let pending_readback = scene
                .query::<&Chunk>()
                .into_iter()
                .filter(|c| {
                    matches!(c.state(), ChunkState::PendingReadbackStart)
                        | matches!(c.state(), ChunkState::PendingReadbackData)
                })
                .count();
            let removal = scene
                .query::<&Chunk>()
                .into_iter()
                .filter(|c| matches!(c.state(), ChunkState::PendingRemoval))
                .count();
            ui.heading("Real-time stats");
            ui.label(format!("Pending chunks count: {}", pending));
            ui.label(format!("Generated chunks count: {}", generated));
            ui.label(format!("Pending readbacks: {}", pending_readback));
            ui.label(format!("Pending removals: {}", removal));

            ui.horizontal(|ui| {
                ui.label("Active?: ");
                ui.add(egui::Checkbox::new(&mut terrain.active, ""));
            });

            egui::Grid::new("lod-multipliers")
                .min_col_width(0f32)
                .max_col_width(400f32)
                .striped(true)
                .show(ui, |ui| {
                    let mut values = terrain.manager.lod_multipliers.borrow_mut();

                    for value in values.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.label("Multiplier: ");
                            ui.add(egui::DragValue::new(value));
                        });
                        ui.end_row();
                    }
                });

        });
    }

    // Camera controller settings
    if let Some((controller, rotation, position, velocity)) =
        scene.find_mut::<(&mut CameraController, &Rotation, &Position, &Velocity)>()
    {
        egui::Window::new("Camera Controller")
            .frame(frame)
            .show(&gui, |ui| {
                ui.label(format!("Forward vector: {:.2}", rotation.forward()));
                ui.label(format!("Up vector: {:.2}", rotation.up()));
                ui.label(format!("Right vector: {:.2}", rotation.right()));
                ui.label(format!("Position: {:.2}", **position));
                ui.label(format!("Velocity: {:.2}", **velocity));

                ui.horizontal(|ui| {
                    ui.label("Base Speed (m/s): ");
                    ui.add(egui::DragValue::new(&mut controller.base_speed));
                });

                ui.horizontal(|ui| {
                    ui.label("Boost Speed (m/s): ");
                    ui.add(egui::DragValue::new(&mut controller.boost_speed));
                });

                ui.horizontal(|ui| {
                    ui.label("FOV Change Key Speed: ");
                    ui.add(egui::DragValue::new(&mut controller.fov_change_key_speed));
                });

                ui.horizontal(|ui| {
                    ui.label("FOV Change Scroll Speed: ");
                    ui.add(egui::DragValue::new(
                        &mut controller.fov_change_scroll_speed,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label("Sensivity: ");
                    ui.add(egui::DragValue::new(&mut controller.sensivity));
                });

                ui.horizontal(|ui| {
                    ui.label("Smoothness: ");
                    ui.add(egui::DragValue::new(&mut controller.smoothness));
                });

                ui.horizontal(|ui| {
                    ui.label("Active?: ");
                    ui.add(egui::Checkbox::new(&mut controller.active, ""));
                });
            });
    }

    // Shadow mapping settings
    if let Ok(mut shadowmapping) = world.get_mut::<ShadowMapping>() {
        egui::Window::new("Shadow Mapping")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                egui::Grid::new("cascades")
                    .min_col_width(0f32)
                    .max_col_width(400f32)
                    .striped(true)
                    .show(ui, |ui| {
                        for (i, value) in shadowmapping.percents.iter_mut().enumerate() {
                            ui.label(format!("Cascade: {i}"));
                            ui.add(egui::Slider::new(value, 0.0..=1.0).max_decimals(6).trailing_fill(true));
                            ui.end_row();
                        }
                    });

                ui.heading("Shadow Parameters");

                ui.horizontal(|ui| {
                    ui.label("Strengh: ");
                    let value = &mut shadowmapping.parameters.strength;
                    ui.add(egui::Slider::new(value, 0.05f32..=1f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Spread: ");
                    let value = &mut shadowmapping.parameters.spread;
                    ui.add(egui::Slider::new(value, 0.05f32..=1f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Base Bias: ");
                    let value = &mut shadowmapping.parameters.base_bias;
                    ui.add(egui::Slider::new(value, -0.001f32..=0.001f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Bias Bias: ");
                    let value = &mut shadowmapping.parameters.bias_bias;
                    ui.add(egui::Slider::new(value, -0.001f32..=0.001f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Bias Factor Base: ");
                    let value = &mut shadowmapping.parameters.bias_factor_base;
                    ui.add(egui::Slider::new(value, 0.1f32..=3.0f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Distance: ");
                    let value = &mut shadowmapping.distance;
                    ui.add(egui::DragValue::new(value));
                });
            });
    }

    // Forward renderer settings
    if let Ok(renderer) = world.get_mut::<DeferredRenderer>() {
        egui::Window::new("Deferred Rendering")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.label(format!(
                    "Unique material count: {}",
                    renderer.drawn_unique_material_count
                ));
                ui.label(format!(
                    "Drawn material instances: {}",
                    renderer.material_instances_count
                ));
                ui.label(format!(
                    "Drawn sub-surfaces: {}",
                    renderer.rendered_sub_surfaces
                ));
                ui.label(format!(
                    "Vertices draw count: {}k",
                    renderer.rendered_direct_vertices_drawn / 1000
                ));
                ui.label(format!(
                    "Triangles draw count: {}k",
                    renderer.rendered_direct_triangles_drawn / 1000
                ));
            });
    }

    // Compositor settings
    if let Ok(mut compositor) = world.get_mut::<Compositor>() {
        egui::Window::new("Compositor - Post processing")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Exposure: ");
                    ui.add(egui::Slider::new(
                        &mut compositor.post_process.exposure,
                        0.001..=5.0,
                    ).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Gamma: ");
                    ui.add(egui::Slider::new(
                        &mut compositor.post_process.gamma,
                        0.01..=3.0,
                    ).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Vignette Strength: ");
                    ui.add(egui::Slider::new(
                        &mut compositor.post_process.vignette_strength,
                        0.0..=1.0,
                    ).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Vignette Size: ");
                    ui.add(egui::DragValue::new(
                        &mut compositor.post_process.vignette_size,
                    ));
                });

                let mut selected_tonemapping =
                    Tonemapping::from_index(compositor.post_process.tonemapping_mode);

                ui.horizontal(|ui| {
                    ui.label("Tonemapping Mode: ");

                    egui::ComboBox::from_label("tonemapping-mode")
                        .selected_text(format!("{:?}", selected_tonemapping))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected_tonemapping, Tonemapping::Reinhard, "Reinhard");
                            ui.selectable_value(
                                &mut selected_tonemapping,
                                Tonemapping::ReinhardJodie,
                                "ReinhardJodie",
                            );
                            ui.selectable_value(&mut selected_tonemapping, Tonemapping::ACES, "ACES");
                            ui.selectable_value(&mut selected_tonemapping, Tonemapping::ALU, "ALU");
                            ui.selectable_value(&mut selected_tonemapping, Tonemapping::Clamp, "Clamp");
                        });
                });

                
                let mut selected_debug_gbuffer =
                    DebugGBuffer::from_index(compositor.post_process.debug_gbuffer);

                ui.horizontal(|ui| {
                    ui.label("G-Buffer Debug Mode: ");

                    egui::ComboBox::from_label("gbuffer-debug-mode")
                        .selected_text(format!("{:?}", selected_debug_gbuffer))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::None, "None");
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::Position, "Position");
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::Albedo, "Albedo");
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::Normal, "Normal");
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::ReconstructedNormal, "Reconstructed Normal");
                            ui.selectable_value(&mut selected_debug_gbuffer, DebugGBuffer::Mask, "Mask");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Tonemapping Strength: ");
                    ui.add(egui::Slider::new(
                        &mut compositor.post_process.tonemapping_strength,
                        0.0..=1.0,
                    ).trailing_fill(true));
                });

                fn pick_vec4_color(text: &str, ui: &mut egui::Ui, vec: &mut vek::Vec4<f32>) {
                    let mut rgba = egui::Rgba::from_rgb(vec.x, vec.y, vec.z);

                    ui.horizontal(|ui| {
                        ui.label(text);
                        egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                    });

                    vec.x = rgba.r();
                    vec.y = rgba.g();
                    vec.z = rgba.b();
                }                

                pick_vec4_color("Color Correction Gain: ", ui, &mut compositor.post_process.cc_gain);
                pick_vec4_color("Color Correction Lift: ", ui, &mut compositor.post_process.cc_lift);
                pick_vec4_color("Color Correction Gamma: ", ui, &mut compositor.post_process.cc_gamma);


                compositor.post_process.tonemapping_mode = selected_tonemapping.into_index();
                compositor.post_process.debug_gbuffer = selected_debug_gbuffer.into_index();
            });
    }

    // Physics stats
    if let Ok(mut physics) = world.get_mut::<Physics>() {
        let rigidbodies = scene.query::<&RigidBody>();
        let max = rigidbodies.len();
        let sleeping = rigidbodies.into_iter().filter(|x| x.is_sleeping()).count();

        egui::Window::new("Rapier3D Physics")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.label(format!(
                    "Total number of rigid-bodies: {}",
                    max
                ));
                
                ui.label(format!(
                    "Number of sleeping rigid-bodies: {}",
                    sleeping
                ));
            });
    }
}

// Statistics system
pub fn system(system: &mut System) {
    system.insert_init(init);
    system.insert_update(update);
}
