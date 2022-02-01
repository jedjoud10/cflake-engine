use std::sync::{Arc, RwLock};

use rendering::pipeline::PipelineContext;

use crate::{data::World, Context, GameConfig, WorldTaskReceiver};

// World implementation
impl World {
    // Create a new world
    pub fn new(author_name: &str, app_name: &str, pipeline: PipelineContext) -> Self {
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
            pipeline,
        };
        world.init();
        world
    }
    // Initialize the world
    fn init(&mut self) {
        println!("Initializing world...");
        // Load the default stuff
        
        // Create an empty default UI
        self.ui.add_root("default", ui::Root::default());

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
        let pipeline = self.pipeline.read();
        rendering::pipeline::pipec::task(rendering::object::PipelineTask::SetWindowDimension(new_dimensions), &pipeline);
    }
    // Begin frame update. We also get the Arc<RwLock<World>> so we can pass it to the systems
    pub fn update_start(world: &Arc<RwLock<Self>>, _task_receiver: &mut WorldTaskReceiver) {
        // While we do world logic, start rendering the frame on the other thread
        {
            let world = world.write().unwrap();
            let handler = &world.pipeline.handler.lock().unwrap();

            // Update the timings then we can start rendering
            {
                let mut time = handler.time.lock().unwrap();
                time.0 = world.time.elapsed;
                time.1 = world.time.delta;
                handler.sbarrier.wait();
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
                    let system = &world.ecs.get_systems()[system_index];
                    system.run_system(&world.ecs)
                };
                // Actually execute the system now
                let mut context = Context::convert(world);
                execution_data.run(&mut context);
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
            let delta = world.time.delta as f32;
            world.input.late_update(delta);
            let handler = &world.pipeline.handler.lock().unwrap();
            handler.ebarrier.wait();
        }
    }
    // We must destroy the world
    pub fn destroy(self) {
        // We update the pipeline's shutdown atomic, telling it to shutdown
        //let pipeline = self.pipeline.read().unwrap();
        let handler = Arc::try_unwrap(self.pipeline.handler);
        if let Ok(handler) = handler {
            let handler = handler.into_inner().unwrap();
            // Run the render thread loop for one last time
            handler.sbarrier.wait();
            handler.eatomic.store(true, std::sync::atomic::Ordering::Relaxed);
            handler.ebarrier.wait();
            // Join the render thread now
            handler.handle.join().unwrap();
        }
    }
}
