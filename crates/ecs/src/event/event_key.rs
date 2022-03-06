use enum_as_inner::EnumAsInner;

use crate::component::ComponentQuery;

// Some data that will be passed to each of the systems' events whenever we execute them
// This can be used as a "key" to access global components, since it will take reference to this EventQuery instead of the manager
#[derive(EnumAsInner)]
pub enum EventKey<'a> {
    Query(ComponentQuery<'a>),
}
