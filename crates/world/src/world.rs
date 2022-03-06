use crate::{Settings, WorldState};
use audio::player::AudioPlayer;
use ecs::ECSManager;
use globals::GlobalsCollection;
use gui::GUIManager;
use input::InputManager;
use io::IOManager;
use others::Time;
use physics::PhysicsSimulation;
use rendering::pipeline::PipelineContext;
use std::sync::Arc;

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub gui: GUIManager,
    pub ecs: ECSManager<Self>,
    pub globals: GlobalsCollection,
    pub io: IOManager,
    pub settings: Settings,
    pub pipeline: PipelineContext,
    pub state: WorldState,
    pub audio: AudioPlayer,
    pub physics: PhysicsSimulation,
}

// World implementation
impl World {
    // Create a new world
    pub fn new(settings: Settings, io: IOManager, pipeline: PipelineContext) -> Self {
        let gui = gui::GUIManager::new(&pipeline);
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            gui,
            ecs: ECSManager::<Self>::default(),
            globals: Default::default(),
            io,
            settings: Default::default(),
            pipeline,
            state: WorldState::StartingUp,
            audio: Default::default(),
            physics: PhysicsSimulation::new(),
        };
        //others::set_main_thread();
        // Just set the game settings and we are done
        world.settings = settings;
        println!("World init done!");
        world
    }
    // Resize window event
    pub fn resize_window_event(&mut self, new_dimensions: veclib::Vector2<u16>) {
        let pipeline = self.pipeline.read();
        rendering::pipeline::pipec::update_callback(&pipeline, move |pipeline, renderer| {
            pipeline.update_window_dimensions(renderer, new_dimensions);
        });
    }
    // Begin frame update. We also get the Arc<RwLock<World>> so we can pass it to the systems
    pub fn update_start(&mut self) {
        self.state = WorldState::Running;
        // Handle GUI begin frame
        {
            let pipeline = self.pipeline.read();
            let window = &pipeline.window;
            self.gui.begin_frame(window.inner.as_ref().unwrap());
        }
        
        // While we do world logic, start rendering the frame on the other thread
        // Update the timings then we can start rendering
        let handler = self.pipeline.handler.as_ref().unwrap().lock();
        let mut time = handler.time.lock();
        time.0 = self.time.elapsed;
        time.1 = self.time.delta;
        handler.sbarrier.wait();
        drop(time);
        drop(handler);
        self.time.update_current_frame_time();
        let (systems, settings) = self.ecs.ready();
        let systems = systems.borrow();
        ECSManager::<World>::execute_systems(systems, self, settings);
    }
    // End frame update
    pub fn update_end(&mut self) {
        // Handle GUI end frame
        self.gui.end_frame();

        // End the frame
        {
            let delta = self.time.delta as f32;
            self.input.late_update(delta);
            let handler = &self.pipeline.handler.as_ref().unwrap().lock();
            handler.ebarrier.wait();
        }
    }
    // We must destroy the world
    pub fn destroy(&mut self) {
        // We update the pipeline's shutdown atomic, telling it to shutdown
        //let pipeline = self.pipeline.read().unwrap();
        let handler = Arc::try_unwrap(self.pipeline.handler.take().unwrap());
        if let Ok(handler) = handler {
            let handler = handler.into_inner();
            // Run the render thread loop for one last time
            handler.sbarrier.wait();
            handler.eatomic.store(true, std::sync::atomic::Ordering::Relaxed);
            handler.ebarrier.wait();
            // Join the render thread now
            handler.handle.join().unwrap();
        }
        // Quit the saver loader
        self.io.quit();
    }
}
