// An ID for an element
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ElementID(pub(crate) Option<u64>);