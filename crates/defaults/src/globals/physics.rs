use world::globals::Global;

// Physics global that contains some simulation settings
#[derive(Default, Global)]
pub struct Physics {
    // The last time we executed a physics step
    pub(crate) last_execution_time: f32,
}
