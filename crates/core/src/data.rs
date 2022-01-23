use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use ecs::{ECSManager, entity::EntityID, component::ComponentID};
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::pipeline::{Pipeline, PipelineStartData};
use ui::UIManager;

use crate::{GameConfig, TaskSenderContext};

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub ui: UIManager,
    pub ecs: ECSManager<Context>,
    pub io: SaverLoader,
    pub config: GameConfig,
    pub pipeline: Arc<RwLock<Pipeline>>,
    pub pipeline_thread: PipelineStartData,
}

// A context that can mutate the world if self is mut
#[derive(Clone)]
pub struct Context {
    world: Arc<RwLock<World>>,
}

impl !Send for Context {}
impl !Sync for Context {}

impl Context {
    // Convert a world into a context, so we can share it around multiple threads
    // We call this whenever we execute the systems
    pub fn convert(world: &Arc<RwLock<World>>) -> Self {
        Self { world: world.clone() }
    }
    // Create a ShareableContext that we can send to other threads if they need to access the world
    pub fn share(&self) -> ShareableContext {
        ShareableContext { world: self.world.clone() }
    }
    // Read
    pub fn read<'a>(&'a self) -> ReadContext<'a> {
        ReadContext {
            world: self.world.read().unwrap(),
        }
    }
    // Write
    pub fn write<'a>(&'a mut self) -> WriteContext<'a> {
        WriteContext {
            world: self.world.write().unwrap(),
        }
    }
}

// A context that we can share around to other threads if they need to access the world
pub struct ShareableContext {
    world: Arc<RwLock<World>>,
}

impl ShareableContext {
    // Read
    pub fn read(&self) -> ReadContext {
        ReadContext {
            world: self.world.read().unwrap(),
        }
    }
    // Create a sender that we can use to send multiple tasks the main thread
    pub fn sender(&self) -> TaskSenderContext {
        TaskSenderContext(())
    }
}

// A readable world context
pub struct ReadContext<'a> {
    world: RwLockReadGuard<'a, World>,
}

impl<'a> std::ops::Deref for ReadContext<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &*self.world
    }
}

// A writable world context
pub struct WriteContext<'a> {
    world: RwLockWriteGuard<'a, World>,
}

impl<'a> std::ops::Deref for WriteContext<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &*self.world
    }
}

impl<'a> std::ops::DerefMut for WriteContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.world
    }
}
