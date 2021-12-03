use crate::tasks::WorldTask;

// A dispatch group used to dispatch multiple commands to the world at the same time, so we don't have to wait for them on the system threads
pub struct WorldCommandDispatchGroup {
    pub commands: Vec<WorldCommand>
}
// A world command
pub struct WorldCommand {
    pub id: u64,
    pub task: WorldTask,
}

// A task sender that will send tasks from other thread to the main thread, asynchronously.
pub struct WorldTaskSender {

}

