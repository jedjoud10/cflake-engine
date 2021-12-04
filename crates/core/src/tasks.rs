// Some world commands
pub enum WorldTask {
    // Components
    CreateComponent(WorldTaskReturn<usize>),
    DestroyComponent(),
    // Entity
    CreateEntity(WorldTaskReturn<usize>),
    DestroyEntity(),
    // Linking
    LinkComponent(),
    UnlinkComponent(),
}
// The return type for a world task, we can wait for the return or just not care lol
pub struct WorldTaskReturn<T> {
    pub id: u64,
    pub val: T,
}

impl<T> WorldTaskReturn<T> {
    // Wait for the main thread to finish this specific task
    pub fn wait() {
        
    }
}