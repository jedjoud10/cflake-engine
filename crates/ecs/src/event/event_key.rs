use crate::component::ComponentQuerySet;
use enum_as_inner::EnumAsInner;

#[derive(EnumAsInner)]
pub enum EventKey<'subsystem> {
    Queries(ComponentQuerySet<'subsystem>),
}
