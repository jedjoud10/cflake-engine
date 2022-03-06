use crate::component::DanglingComponentsToRemove;

// Settings that tell us how we should execute a system
#[derive(Clone)]
pub struct SystemSettings {
    pub(crate) to_remove: DanglingComponentsToRemove,
}