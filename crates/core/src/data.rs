use std::{sync::{RwLock, Arc, mpsc::SendError, RwLockReadGuard, RwLockWriteGuard}, marker::PhantomData};

use ecs::{ECSManager};
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::{PipelineStartData, Pipeline};
use ui::UIManager;

use crate::{GameConfig, WorldTask, TaskSenderContext, WorldTaskTiming};

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
    // Create a TaskSenderContext that we can use to send tasks to the main thread
    pub fn new_task_sender(&self) -> TaskSenderContext {
        TaskSenderContext {
            timing: WorldTaskTiming::default(),
        }
    }
    // Read
    pub fn read<'a>(&'a self) -> ReadableContext<'a> {
        ReadableContext { world: self.world.read().unwrap() }
    }
    // Write
    pub fn write<'a>(&'a mut self) -> WritableContext<'a> {
        WritableContext { world: self.world.write().unwrap() }
    }
}

// A readable world context
pub struct ReadableContext<'a> {
    pub(crate) world: RwLockReadGuard<'a, World>
}

impl<'a> std::ops::Deref for ReadableContext<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &*self.world
    }
}

// A writable world context
pub struct WritableContext<'a> {
    pub(crate) world: RwLockWriteGuard<'a, World>
}

impl<'a> std::ops::Deref for WritableContext<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &*self.world
    }
}

impl<'a> std::ops::DerefMut for WritableContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.world
    }
}