use crate::{Settings, WorldState};
use audio::player::AudioPlayer;
use ecs::ECSManager;
use globals::GlobalsCollection;
use gui::GUIManager;
use input::InputManager;
use io::IOManager;
use others::Time;
use physics::PhysicsSimulation;
use rendering::pipeline::Pipeline;
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
    pub pipeline: Pipeline,
    pub state: WorldState,
    pub audio: AudioPlayer,
    pub physics: PhysicsSimulation,
}

// World implementation
impl World {
    // Create a new world
    pub fn new(settings: Settings, io: IOManager, mut pipeline: Pipeline) -> Self {
        let gui = gui::GUIManager::new(&mut pipeline);
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
        // Just set the game settings and we are done
        world.settings = settings;
        println!("World init done!");
        world
    }    
    // Called each frame
    pub fn update(&mut self, delta: f64) {
        self.state = WorldState::Running;
        // Update the timings
        self.time.update(delta);
        // Update game logic (this includes rendering the world)
        self.time.update_current_frame_time();
        let (systems, settings) = self.ecs.ready();
        let systems = systems.borrow();
        ECSManager::<World>::execute_systems(systems, self, settings);
    }
    // We must destroy the world
    pub fn destroy(&mut self) {
        // Quit the saver loader
        self.io.quit();
    }
}
