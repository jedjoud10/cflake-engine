// Some world commands
pub enum WorldTask {
    // Components
    CreateComponent(),
    DestroyComponent(),
    // Entity
    CreateEntity(),
    DestroyEntity(),
    // Linking
    LinkComponent(),
    UnlinkComponent(),
}
// A task sender that will send tasks from other thread to the main thread, asynchronously.
pub struct WorldTaskSender {

}