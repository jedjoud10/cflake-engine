use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use ecs::ECSManager;
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::pipeline::PipelineContext;
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
    pub pipeline: PipelineContext,
}

// A context that can mutate the world if self is mut
pub struct Context {
    world: Rc<RefCell<World>>,
}

impl Context {
    // Convert a world into a context, so we can share it around multiple threads
    // We call this whenever we execute the systems
    pub fn convert(world: &Rc<RefCell<World>>) -> Self {
        Self { world: world.clone() }
    }
    // Read
    pub fn read<'a>(&'a self) -> Option<ReadContext<'a>> {
        Some(ReadContext {
            world: self.world.try_borrow().ok()?,
        })
    }
    // Write
    pub fn write<'a>(&'a mut self) -> Option<WriteContext<'a>> {
        Some(WriteContext {
            world: self.world.try_borrow_mut().ok()?,
        })
    }
}
// A readable world context
pub struct ReadContext<'a> {
    world: Ref<'a, World>,
}

impl<'a> ReadContext<'a> {
    // Create a sender that we can use to send multiple tasks the main thread
    pub fn sender(&self) -> TaskSenderContext {
        TaskSenderContext(())
    }
}

impl<'a> std::ops::Deref for ReadContext<'a> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &*self.world
    }
}

// A writable world context
pub struct WriteContext<'a> {
    world: RefMut<'a, World>,
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
