use world::World;

pub(crate) fn update(world: &mut World) {
    let stats = world.get::<graphics::GraphicsStats>().unwrap();
    let mut gui = world.get_mut::<gui::Interface>().unwrap();

    let graphics::GraphicsStats {
        acquires,
        submissions,
        stalls,
        staging_buffers,
        cached_samplers,
        cached_bind_group_layouts,
        cached_pipeline_layouts,
        cached_bind_groups,
    } = *stats;

    gui::egui::Window::new("Graphics Stats").show(&gui, |ui| {
        ui.label(format!("Acquires: {acquires}"));
        ui.label(format!("Submissions: {submissions}"));
        ui.label(format!("Stalls: {stalls}"));
        ui.label(format!("Stg Buffers: {staging_buffers}"));
        
        ui.heading("Cached Graphics Data");
        ui.label(format!("Samplers: {cached_samplers}"));
        ui.label(format!("Pipeline Layouts: {cached_pipeline_layouts}"));
        ui.label(format!("Bind Group Layouts: {cached_bind_group_layouts}"));
        ui.label(format!("Bind Group: {cached_bind_groups}"));

    });
}