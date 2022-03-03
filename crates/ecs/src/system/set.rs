use super::{System, SystemBuilder};
use getset::Getters;


// System set
#[derive(Getters)]
pub struct SystemSet<World> {
    #[getset(get = "pub")]
    pub(crate) inner: Vec<System<World>>,
}

impl<World> Default for SystemSet<World> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<World> SystemSet<World> {
    // Create a new system build
    pub fn builder(&mut self) -> SystemBuilder<World> {
        SystemBuilder::new(self)
    }
    // Add a system to our current systems
    pub(crate) fn add(&mut self, system: System<World>) {
        self.inner.push(system)
    }
}