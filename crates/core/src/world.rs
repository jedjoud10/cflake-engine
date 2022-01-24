use std::sync::{Arc, RwLock};

use ecs::{entity::EntityID, component::ComponentID};
use rendering::pipeline::PipelineStartData;

use crate::{data::World, Context, GameConfig, WorldTaskReceiver};

// World implementation
impl World {
    // Create a new world
    pub fn new(author_name: &str, app_name: &str, pipeline_data: PipelineStartData) -> Self {
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            ui: Default::default(),
            ecs: ecs::ECSManager::new(|| {
                // This is ran on every thread in the ECS thread pool
                rendering::pipeline::init_coms();
                crate::sender::init_coms();
            }),
            io: io::SaverLoader::new(author_name, app_name),
            config: Default::default(),
            pipeline: pipeline_data.pipeline.clone(),
            pipeline_thread: pipeline_data,
        };
        world.init();
        world
    }
    // Initialize the world
    fn init(&mut self) {
        println!("Initializing world...");
        // Load the default stuff

        // Create some default UI that prints some default info to the screen
        let mut root = ui::Root::new(1);
        // ----Add the elements here----

        // Create a text element
        for x in 0..2 {
            let text_element_1 = ui::Element::new()
                .set_coordinate_system(ui::CoordinateType::Pixel)
                .set_position(veclib::Vector2::Y * 40.0 * x as f32)
                .set_text("", 40.0);
            root.add_element(text_element_1);
        }

        // Set this as the default root
        self.ui.add_root("default", root);

        // Create the default root for the console
        let mut console_root = ui::Root::new(64);
        let console_panel = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Factor)
            .set_color(veclib::Vector4::new(0.0, 0.0, 0.0, 0.7));
        let console_panel_id = console_root.add_element(console_panel);
        let console_text = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::ZERO)
            .set_size(veclib::Vector2::ONE)
            .set_text("text", 30.0);
        let console_text_id = console_root.add_element(console_text);
        ui::Element::attach(&mut console_root, console_panel_id, vec![console_text_id]);
        console_root.visible = false;
        self.ui.add_root("console", console_root);

        // Load the config file (create it if it doesn't exist already)
        self.io.create_default("config\\game_config.json", &crate::GameConfig::default());
        // Then load
        let config: GameConfig = self.io.load("config\\game_config.json");
        self.config = config;
        // TODO: Apply the config file's data to the rendering window
        println!("World init done!");

    }
    // Resize window event
    pub fn resize_window_event(&mut self, new_dimensions: veclib::Vector2<u16>) {
        let pipeline = self.pipeline.read().unwrap();
        rendering::pipeline::pipec::task(rendering::object::PipelineTask::SetWindowDimension(new_dimensions), &*pipeline);
    }
    // Begin frame update. We also get the Arc<RwLock<World>> so we can pass it to the systems
    pub fn update_start(world: &Arc<RwLock<Self>>, _task_receiver: &mut WorldTaskReceiver) {
        // While we do world logic, start rendering the frame on the other thread
        {
            let world = world.write().unwrap();
            let start_data = &world.pipeline_thread;

            // Update the timings then we can start rendering
            {
                let time = &world.pipeline_thread.time;
                let mut time = time.lock().unwrap();
                time.0 = world.time.elapsed;
                time.1 = world.time.delta;
                start_data.sbarrier.wait();
            }
        }
        {
            let system_count = {
                let world = world.read().unwrap();
                world.ecs.count_systems()
            };
            // Loop for every system and update it
            for system_index in 0..system_count {
                let execution_data = {
                    let world = world.read().unwrap();
                    let system = &world.ecs.systems()[system_index];
                    system.run_system(&world.ecs)
                };
                // Actually execute the system now
                let context = Context::convert(world);
                execution_data.run(context);
                {
                    // Run the callback after executing a single system
                    let mut world = world.write().unwrap();
                    _task_receiver.flush(&mut world);
                }
            }
        }
        {
            // Finish update
            let mut world = world.write().unwrap();
            world.ecs.finish_update();
        }
    }
    // End frame update
    pub fn update_end(world: &Arc<RwLock<Self>>, _task_receiver: &mut WorldTaskReceiver) {
        // End the frame
        {
            let mut world = world.write().unwrap();
            let start_data = &world.pipeline_thread;
            start_data.ebarrier.wait();
            let delta = world.time.delta as f32;
            world.input.late_update(delta);
        }
    }
    // We must destroy the world
    pub fn destroy(self) {
        // We update the pipeline's shutdown atomic, telling it to shutdown
        //let pipeline = self.pipeline.read().unwrap();
        let start_data = self.pipeline_thread;
        // Run the render thread loop for one last time
        start_data.sbarrier.wait();
        start_data.eatomic.store(true, std::sync::atomic::Ordering::Relaxed);
        start_data.ebarrier.wait();
        // Join the render thread now
        start_data.handle.join().unwrap();
    }
}
