use crate::{data::World, GameSettings, WorldTaskReceiver};
use rendering::pipeline::PipelineContext;
use std::sync::Arc;

// World implementation
impl World {
    // Create a new world
    pub fn new(config: GameSettings, io: io::SaverLoader, pipeline: PipelineContext) -> Self {
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            ui: Default::default(),
            ecs: ecs::ECSManager::<Self>::new(),
            globals: Default::default(),
            io,
            settings: Default::default(),
            pipeline,
        };
        println!("Initializing world...");
        // Load the default stuff

        // Create an empty default UI
        let root = ui::Root::default();
        /*
        root.add_element(
            ui::Element::default()
                .with_size(veclib::vec2(200, 200))
                .with_center(veclib::vec2(100, 100))
                .with_color(veclib::vec4(1.0, 0.0, 1.0, 1.0)),
        );
        */
        world.ui.add_root("default", root);
        let pipeline = world.pipeline.read();
        pipeline.window.set_vsync(config.vsync);
        pipeline.window.set_fullscreen(config.fullscreen);
        drop(pipeline);
        world.settings = config;
        println!("World init done!");
        world
    }
    // Resize window event
    pub fn resize_window_event(&mut self, new_dimensions: veclib::Vector2<u16>) {
        let pipeline = self.pipeline.read();
        rendering::pipeline::pipec::update_callback(&pipeline, move |pipeline, renderer| {
            pipeline.update_window_dimensions(renderer, new_dimensions);
        })
        .unwrap();
    }
    // Begin frame update. We also get the Arc<RwLock<World>> so we can pass it to the systems
    pub fn update_start(&mut self, _task_receiver: &mut WorldTaskReceiver) {
        // While we do world logic, start rendering the frame on the other thread
        // Update the timings then we can start rendering
        let handler = self.pipeline.handler.lock().unwrap();
        let mut time = handler.time.lock().unwrap();
        time.0 = self.time.elapsed;
        time.1 = self.time.delta;
        handler.sbarrier.wait();
        drop(time);
        drop(handler);
        let system_count = self.ecs.count_systems();
        // Loop for every system and update it
        for system_index in 0..system_count {
            let execution_data = {
                let system = &self.ecs.get_systems()[system_index];
                system.run_system(&self.ecs)
            };
            // Actually execute the system now
            execution_data.run(self);
            {
                // Flush all the commends that we have dispatched during the system's frame execution
                _task_receiver.flush(self);
                let system = &self.ecs.get_systems()[system_index];
                system.clear::<World>();
            }
        }
        // Finish update
        self.ecs.finish_update();
    }
    // End frame update
    pub fn update_end(&mut self, _task_receiver: &mut WorldTaskReceiver) {
        // End the frame
        {
            let delta = self.time.delta as f32;
            self.input.late_update(delta);
            let handler = &self.pipeline.handler.lock().unwrap();
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
