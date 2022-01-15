use std::{sync::{RwLock, Arc, mpsc::SendError, RwLockReadGuard, RwLockWriteGuard}, marker::PhantomData};

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
    pub ecs: (ECSManager, ecs::EventHandler<Context>),
    pub io: SaverLoader,
    pub config: GameConfig,
    pub pipeline: Arc<RwLock<Pipeline>>,
    pub pipeline_thread: PipelineStartData,
}

// A context that can mutate the world if self is mut
#[derive(Clone)]
pub struct Context {
    pub(crate) world: Arc<RwLock<World>>,
}

impl Context {
    // Convert a world into a context, so we can share it around multiple threads
    // We call this whenever we execute the systems
    pub fn convert(world: Arc<RwLock<World>>) -> Self {
        Self {
            world: world.clone()
        }
    }
    // Create a RefTaskSenderContext that we can use to send tasks to the main thread
    pub fn create_sender(&self) -> RefTaskSenderContext {
        RefTaskSenderContext {
            timing: WorldTaskTiming::default(),
        }
    }
    // Get ref
    pub fn get<'a>(&self) -> RwLockReadGuard<World> {
        self.world.read().unwrap()
    }
    // Get mut
    pub fn get_mut<'a>(&mut self) -> RwLockWriteGuard<World> {
        self.world.write().unwrap()
    }
}