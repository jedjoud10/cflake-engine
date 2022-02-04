use std::{cell::RefCell, rc::Rc, sync::Arc};

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
        let mut root = ui::Root::default();
        root.add_element(
            ui::Element::default()
                .with_size(veclib::vec2(100, 100))
                .with_center(veclib::vec2(0, 0))
                .with_color(veclib::vec4(1.0, 0.0, 1.0, 1.0)),
        );
        self.ui.add_root("default", root);

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
    pub fn update_start(world: &Rc<RefCell<World>>, _task_receiver: &mut WorldTaskReceiver) {
        // While we do world logic, start rendering the frame on the other thread
        {
            let world = world.try_borrow().unwrap();
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
                let world = world.try_borrow().unwrap();
                world.ecs.count_systems()
            };
            // Loop for every system and update it
            for system_index in 0..system_count {
                let execution_data = {
                    let world = world.try_borrow().unwrap();
                    let system = &world.ecs.get_systems()[system_index];
                    system.run_system(&world.ecs)
                };
                // Actually execute the system now
                let mut context = Context::convert(world);
                execution_data.run(&mut context);
                {
                    // Flush all the commends that we have dispatched during the system's frame execution
                    let mut world = world.try_borrow_mut().unwrap();
                    _task_receiver.flush(&mut world);
                }
            }
        }
        {
            // Finish update
            let mut world = world.try_borrow_mut().unwrap();
            world.ecs.finish_update();
        }
    }
    // End frame update
    pub fn update_end(world: &Rc<RefCell<Self>>, _task_receiver: &mut WorldTaskReceiver) {
        // End the frame
        {
            let mut world = world.try_borrow_mut().unwrap();
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
