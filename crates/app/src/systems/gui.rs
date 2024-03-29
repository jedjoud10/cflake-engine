use std::time::Duration;

use gui::egui::Ui;

use crate::prelude::*;

// Simple type to check if stats are enabled or not
struct StatsState(bool);

// Event stats for the init, update, and tick events
// Timings are in milliseconds btw
#[derive(Default)]
pub(crate) struct EventStatsDurations {
    pub init: Vec<EventTimings<Init>>,
    pub init_total: Duration,
    pub update: Vec<EventTimings<Update>>,
    pub update_total: Duration,
    pub tick: Vec<EventTimings<Tick>>,
    pub tick_total: Duration,
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
    let mut durations = world.get_mut::<EventStatsDurations>().unwrap();

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
        ..
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
            // Small function that will show a table for specific event timings
            fn show_events_table<C: Caller>(
                ui: &mut Ui,
                total: Duration,
                timings: &mut Vec<EventTimings<C>>,
            ) {
                ui.push_id(fetch_caller_id::<C>().name, |ui| {
                    let mut table = egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .resizable(false)
                        .vscroll(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::initial(800.0))
                        .column(egui_extras::Column::initial(100.0));

                    if C::persistent() {
                        table = table
                            .column(egui_extras::Column::initial(100.0))
                            .column(egui_extras::Column::initial(100.0))
                            .column(egui_extras::Column::initial(100.0));
                    }

                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Name");
                            });
                            header.col(|ui| {
                                ui.strong("Elapsed (ms)");
                            });

                            if C::persistent() {
                                header.col(|ui| {
                                    ui.strong("Avg (ms)");
                                });
                                header.col(|ui| {
                                    ui.strong("Min (ms)");
                                });
                                header.col(|ui| {
                                    ui.strong("Max (ms)");
                                });
                            }
                        })
                        .body(|body| {
                            timings.sort_by_key(|x| x.elapsed().as_micros());
                            body.rows(18.0f32, timings.len().min(10), |row_index, mut row| {
                                let event = &timings[timings.len() - row_index - 1];
                                let elapsed = event.elapsed();
                                let color = pick_stats_label_color(
                                    elapsed.as_secs_f32() / total.as_secs_f32(),
                                );
                                let color = egui::Color32::from_rgb(color.r, color.g, color.b);

                                row.col(|ui| {
                                    ui.colored_label(color, event.id().system.name);
                                });
                                row.col(|ui| {
                                    ui.colored_label(
                                        color,
                                        format!("{:.2?}ms", elapsed.as_secs_f32() * 1000.0),
                                    );
                                });

                                if let Some(persistent) = event.persistent() {
                                    row.col(|ui| {
                                        ui.colored_label(
                                            color,
                                            format!(
                                                "{:.2?}ms",
                                                persistent.average().as_secs_f32() * 1000.0
                                            ),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.colored_label(
                                            color,
                                            format!(
                                                "{:.2?}ms",
                                                persistent.min().as_secs_f32() * 1000.0
                                            ),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.colored_label(
                                            color,
                                            format!(
                                                "{:.2?}ms",
                                                persistent.max().as_secs_f32() * 1000.0
                                            ),
                                        );
                                    });
                                }
                            });
                        })
                });
            }

            // Show event timings for init, update, and tick registries
            show_events_table(ui, durations.init_total, &mut durations.init);
            show_events_table(ui, durations.update_total, &mut durations.update);
            show_events_table(ui, durations.tick_total, &mut durations.tick);
        });

    // Graphics Stats
    egui::Window::new("Graphics Stats")
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .frame(frame)
        .show(&gui, |ui| {
            ui.label(format!("Acquires: {acquires}"));
            ui.label(format!("Submissions: {submissions}"));
            ui.label(format!("Stalls: {stalls}"));

            ui.heading("Cached Graphics Data: ");
            ui.label(format!("Shaders: {cached_shaders}"));
            ui.label(format!("Samplers: {cached_samplers}"));
            ui.label(format!("Pipeline Layouts: {cached_pipeline_layouts}"));
            ui.label(format!("Bind Group Layouts: {cached_bind_group_layouts}"));
            ui.label(format!("Bind Group: {cached_bind_groups}"));

            ui.heading("Graphics Processor: ");
            ui.label(format!("Name: {name}"));
            ui.label(format!("Backend: {backend:#?}"));
            ui.label(format!("Type: {device:#?}"));

            ui.heading("WGPU Raw Data Types: ");
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
            let delta = time.delta().as_secs_f32();
            let ticks_to_exec = time.ticks_to_execute().map(|x| x.get()).unwrap_or(0) as f32;

            let since = time.startup().elapsed().as_secs_f32();
            ui.label(format!("Seconds since startup: {:.3}", since));

            let ms = delta * 1000.0;
            ui.label(format!("Delta (ms/f): {:.3}", ms));

            let fps = 1.0 / delta;
            ui.label(format!("FPS (f/s): {:.0}", fps));

            let ticks = ticks_to_exec;
            ui.label(format!("Ticks to execute: {:.3}", ticks));

            ui.label(format!("Tick-rate: {:.3} t/s", time.tick_rate()))
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
            let _settings = &mut terrain.settings;
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
                    let len = values.len() - 1;

                    for value in values.iter_mut().take(len) {
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
                            ui.add(
                                egui::Slider::new(value, 0.0..=1.0)
                                    .max_decimals(6)
                                    .trailing_fill(true),
                            );
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
                    ui.add(egui::Slider::new(value, -0.0005f32..=0.0005f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Bias Bias: ");
                    let value = &mut shadowmapping.parameters.bias_bias;
                    ui.add(egui::Slider::new(value, -0.0005f32..=0.0005f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Bias Factor Base: ");
                    let value = &mut shadowmapping.parameters.bias_factor_base;
                    ui.add(egui::Slider::new(value, 1.0f32..=1.2f32).trailing_fill(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Distance: ");
                    let value = &mut shadowmapping.distance;
                    ui.add(egui::DragValue::new(value));
                });
            });
    }

    // Renderer settings
    if let Ok(renderer) = world.get_mut::<DeferredRenderer>() {
        fn show_pass_stats(ui: &mut egui::Ui, stats: PassStats) {
            ui.label(format!(
                "Material Instance Swaps: {}",
                stats.material_instance_swap
            ));
            ui.label(format!("Mesh Instance Swaps: {}", stats.mesh_instance_swap));
            ui.label(format!(
                "Drawn sub-surfaces: {}",
                stats.rendered_sub_surfaces
            ));
            ui.label(format!(
                "Culled sub-surfaces: {}",
                stats.culled_sub_surfaces
            ));
            ui.label(format!(
                "Vertices draw count: {}k",
                stats.rendered_direct_vertices_drawn / 1000
            ));
            ui.label(format!(
                "Triangles draw count: {}k",
                stats.rendered_direct_triangles_drawn / 1000
            ));
        }

        egui::Window::new("Rendering")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.heading("Deferred Pass:");
                show_pass_stats(ui, renderer.deferred_pass_stats);
                
                for x in 0..4 {
                    ui.heading(format!("Shadow Pass ({x}):"));
                    show_pass_stats(ui, renderer.shadow_pass_stats[x]);
                }
            });

        /*
        egui::Window::new("Light")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                if let Some((rotation, light)) = scene.find_mut::<(&mut Rotation, &mut DirectionalLight)>() {
                }
            });
        */
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
                    ui.add(
                        egui::Slider::new(&mut compositor.post_process.exposure, 0.001..=5.0)
                            .trailing_fill(true),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Vignette Strength: ");
                    ui.add(
                        egui::Slider::new(
                            &mut compositor.post_process.vignette_strength,
                            0.0..=1.0,
                        )
                        .trailing_fill(true),
                    );
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
                            ui.selectable_value(
                                &mut selected_tonemapping,
                                Tonemapping::Reinhard,
                                "Reinhard",
                            );
                            ui.selectable_value(
                                &mut selected_tonemapping,
                                Tonemapping::ReinhardJodie,
                                "ReinhardJodie",
                            );
                            ui.selectable_value(
                                &mut selected_tonemapping,
                                Tonemapping::ACES,
                                "ACES",
                            );
                            ui.selectable_value(&mut selected_tonemapping, Tonemapping::ALU, "ALU");
                            ui.selectable_value(
                                &mut selected_tonemapping,
                                Tonemapping::Clamp,
                                "Clamp",
                            );
                        });
                });

                let mut selected_debug_gbuffer =
                    DebugGBuffer::from_index(compositor.post_process.debug_gbuffer);

                ui.horizontal(|ui| {
                    ui.label("G-Buffer Debug Mode: ");

                    egui::ComboBox::from_label("gbuffer-debug-mode")
                        .selected_text(format!("{:?}", selected_debug_gbuffer))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::None,
                                "None",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::Position,
                                "Position",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::Albedo,
                                "Albedo",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::Normal,
                                "Normal",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::ReconstructedNormal,
                                "Reconstructed Normal",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::AmbientOcclusionMask,
                                "AO Mask",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::RoughnessMask,
                                "Roughness Mask",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::MetallicMask,
                                "Metallic Mask",
                            );
                            ui.selectable_value(
                                &mut selected_debug_gbuffer,
                                DebugGBuffer::DiffuseIrradiance,
                                "Diffuse Irradiance",
                            );
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Tonemapping Strength: ");
                    ui.add(
                        egui::Slider::new(
                            &mut compositor.post_process.tonemapping_strength,
                            0.0..=1.0,
                        )
                        .trailing_fill(true),
                    );
                });

                fn pick_vec4_color(text: &str, ui: &mut egui::Ui, vec: &mut vek::Vec4<f32>) {
                    let mut rgba = egui::Rgba::from_rgb(vec.x, vec.y, vec.z);

                    ui.horizontal(|ui| {
                        ui.label(text);
                        egui::color_picker::color_edit_button_rgba(
                            ui,
                            &mut rgba,
                            egui::color_picker::Alpha::Opaque,
                        );
                    });

                    vec.x = rgba.r();
                    vec.y = rgba.g();
                    vec.z = rgba.b();
                }

                pick_vec4_color(
                    "Color Correction Gain: ",
                    ui,
                    &mut compositor.post_process.cc_gain,
                );
                pick_vec4_color(
                    "Color Correction Lift: ",
                    ui,
                    &mut compositor.post_process.cc_lift,
                );
                pick_vec4_color(
                    "Color Correction Gamma: ",
                    ui,
                    &mut compositor.post_process.cc_gamma,
                );

                ui.horizontal(|ui| {
                    ui.label("Color Temperature (K): ");
                    ui.add(
                        egui::Slider::new(
                            &mut compositor.post_process.cc_wb_temperature,
                            1000f32..=12000f32,
                        )
                        .trailing_fill(true),
                    );
                });

                compositor.post_process.tonemapping_mode = selected_tonemapping.into_index();
                compositor.post_process.debug_gbuffer = selected_debug_gbuffer.into_index();
            });
    }

    // Physics stats
    if let Ok(_physics) = world.get_mut::<Physics>() {
        let rigidbodies = scene.query::<&RigidBody>();
        let max = rigidbodies.len();
        let sleeping = rigidbodies.into_iter().filter(|x| x.is_sleeping()).count();

        egui::Window::new("Rapier3D Physics")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.label(format!("Total number of rigid-bodies: {}", max));

                ui.label(format!("Number of sleeping rigid-bodies: {}", sleeping));
            });
    }

    // Audio stats
    if let Some((listener, opt_position, opt_rotation)) =
        scene.find::<(&AudioListener, Option<&Position>, Option<&Rotation>)>()
    {
        egui::Window::new("CPAL Audio Listener")
            .frame(frame)
            .collapsible(true)
            .default_open(false)
            .show(&gui, |ui| {
                ui.label(format!(
                    "Audio Device: {:?}",
                    cpal::traits::DeviceTrait::name(&listener.device)
                ));

                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .vscroll(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::initial(100.0))
                    .column(egui_extras::Column::initial(100.0))
                    .column(egui_extras::Column::initial(100.0))
                    .column(egui_extras::Column::initial(100.0));

                let table = table.header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Channels");
                    });
                    header.col(|ui| {
                        ui.strong("Min sample rate");
                    });
                    header.col(|ui| {
                        ui.strong("Max sample rate");
                    });
                    header.col(|ui| {
                        ui.strong("Sample format");
                    });
                });

                table.body(|mut body| {
                    body.rows(
                        20.0,
                        listener.supported_output_configs.len(),
                        |i, mut row| {
                            let config_range = &listener.supported_output_configs[i];

                            row.col(|ui| {
                                ui.label(config_range.channels().to_string());
                            });

                            row.col(|ui| {
                                ui.label(config_range.min_sample_rate().0.to_string());
                            });

                            row.col(|ui| {
                                ui.label(config_range.max_sample_rate().0.to_string());
                            });

                            row.col(|ui| {
                                ui.label(format!("{:?}", config_range.sample_format()));
                            });
                        },
                    )
                })
            });
    }
}

// Statistics system
pub fn system(system: &mut System) {
    system.insert_init(init);
    system.insert_update(update);
}
