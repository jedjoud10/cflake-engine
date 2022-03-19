use world::{
    ecs::component::Component,
    terrain::editing::{Edit, EditKey},
};

// A dynamic terrain edit
#[derive(Component)]
pub struct DynamicEdit {
    // Terrain edit data
    pub edit: Edit,

    // Terrain edit index
    pub(crate) key: EditKey,
}

impl DynamicEdit {
    pub fn new(edit: Edit) -> Self {
        Self { edit, key: EditKey::default() }
    }
}
