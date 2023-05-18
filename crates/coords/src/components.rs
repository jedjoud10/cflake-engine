mod position;
mod rotation;
mod scale;

// Either we use the local coordinate system or global coordinate system
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local;

// Global coordinates
pub type Position = position::Position<Global>;
pub type Rotation = rotation::Rotation<Global>;
pub type Scale = scale::Scale<Global>;

// Local coordinates
pub type LocalPosition = position::Position<Local>;
pub type LocalRotation = rotation::Rotation<Local>;
pub type LocalScale = scale::Scale<Local>;