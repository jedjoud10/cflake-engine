use ecs::Component;

// A synced entity components will allow entities spawned on the server
// to be also spawned on the clients, and vice versa when despawning them
#[derive(Component)]
pub struct SyncedEntity {}

// Owned entities are entitie that are solely owned by a single client
// Owned entities can only be mutated by their respective client
#[derive(Component)]
pub struct OwnedEntity;