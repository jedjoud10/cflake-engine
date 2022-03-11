use enum_as_inner::EnumAsInner;
use crate::component::ComponentQuerySet;

#[derive(EnumAsInner)]
pub enum EventKey<'a> {
    Queries(ComponentQuerySet<'a>),
}
