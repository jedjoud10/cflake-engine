use std::sync::{RwLock, Arc, mpsc::SendError};

use ecs::{ECSManager};
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::{PipelineStartData, Pipeline};
use ui::UIManager;

use crate::{GameConfig, WorldTask, RefTaskSenderContext, WorldTaskTiming};

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
pub struct RefContext<'a> {
    pub input: &'a InputManager,
    pub time: &'a Time,
    pub ui: &'a UIManager,
    pub ecs: &'a ECSManager<World>,
    pub io: &'a SaverLoader,
    pub config: &'a GameConfig,
    pub pipeline: &'a Pipeline,
}

impl<'a> RefContext<'a> {
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
    // Create a RefTaskSenderContext that we can use to send tasks to the main thread
    pub fn create_sender(&self) -> RefTaskSenderContext {
        RefTaskSenderContext {
            timing: WorldTaskTiming::default(),
        }
    }
}
// Some context that stores a reference to all of the world managers and data
// However, we can mutate this context, so we can mutate the world data
pub struct MutContext<'a> {
    pub input: &'a mut InputManager,
    pub time: &'a mut Time,
    pub ui: &'a mut UIManager,
    pub ecs: &'a mut ECSManager<World>,
    pub io: &'a mut SaverLoader,
    pub config: &'a mut GameConfig,
    pub pipeline: &'a Pipeline,
}

impl<'a> MutContext<'a> {
    // Convert a world into a mutable context. We cannot share this to other threads
    pub fn convert(world: &'a mut World) -> Self {
        Self {
            input: &mut world.input,
            time: &mut world.time,
            ui: &mut world.ui,
            ecs: &mut world.ecs,
            io: &mut world.io,
            config: &mut world.config,
            pipeline: { 
                // We can convert it to a pointer then back to a borrow, since we will only use Context inside the system events, so we are not actually updating any pipeline data
                let pipeline = world.pipeline.read().unwrap();
                let ptr = &*pipeline as *const Pipeline; 
                unsafe { &*ptr }
            }
        }
    }
    // Create a RefTaskSenderContext that we can use to send tasks to the main thread
    pub fn create_sender(&self) -> RefTaskSenderContext {
        RefTaskSenderContext {
            timing: WorldTaskTiming::default(),
        }
    }
}