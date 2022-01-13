use std::sync::{RwLock, Arc, mpsc::SendError};

use ecs::{ECSManager};
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::{PipelineStartData, Pipeline};
use ui::UIManager;

use crate::{GameConfig, WorldTask};

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub ui: UIManager,
    pub ecs: ECSManager<Self>,
    pub io: SaverLoader,
    pub config: GameConfig,
    pub pipeline: Arc<RwLock<Pipeline>>,
    pub pipeline_thread: PipelineStartData,
}
// Some context that stores a reference to all of the world managers and data
pub struct Context<'a> {
    pub input: &'a InputManager,
    pub time: &'a Time,
    pub ui: &'a UIManager,
    pub ecs: &'a ECSManager<World>,
    pub io: &'a SaverLoader,
    pub config: &'a GameConfig,
    pub pipeline: &'a Pipeline,
}

impl<'a> Context<'a> {
    // Convert a world into a context, so we can share it around multiple threads
    // We call this whenever we execute the systems
    pub fn convert(world: &'a World) -> Self {
        Self {
            input: &world.input,
            time: &world.time,
            ui: &world.ui,
            ecs: &world.ecs,
            io: &world.io,
            config: &world.config,
            pipeline: { 
                // We can convert it to a pointer then back to a borrow, since we will only use Context inside the system events, so we are not actually updating any pipeline data
                let pipeline = world.pipeline.read().unwrap();
                let ptr = &*pipeline as *const Pipeline; 
                unsafe { &*ptr }
            }
        }
    }
}