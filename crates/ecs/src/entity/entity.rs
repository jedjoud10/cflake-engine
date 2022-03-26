use slotmap::new_key_type;
new_key_type! {
    pub(super) struct EntityKey;
}

// A simple ID
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Entity(pub(super) EntityKey);
