use crate::system::Registries;

/// A plugin allows us to register multiple systems that each hook onto different events
/// The order of execution between plugins does not matter since we solely use them for registering systems
/// Plugins are implemented for all systems that take in an ``&mut Registries`` as argument 
pub trait Plugin {
    /// Register the plugin's resources and systems
    fn register(self, registries: &mut Registries);
}

impl<F: FnOnce(&mut Registries) + 'static> Plugin for F {
    fn register(self, registries: &mut Registries) {
        (self)(registries)
    }
}