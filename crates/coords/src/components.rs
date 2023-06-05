use std::marker::PhantomData;

mod position;
mod rotation;
mod scale;
mod relations;

pub use relations::*;
pub use position::Position as UnmarkedPosition;
pub use rotation::Rotation as UnmarkedRotation;
pub use scale::Scale as UnmarkedScale;

// Values are updated from frame to frame
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameToFrame;

// Values are set to be the first interpolation point when doing sub tick interpolation
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LastTick;

// Values are set to be the next (or last) interpolation point when doing sub tick interpolation
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrentTick;

// Either we use the local coordinate system or global coordinate system
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global<TimeFrame>(PhantomData<TimeFrame>);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Relative<TimeFrame>(PhantomData<TimeFrame>);

// Local coordinates updated frame to frame
pub type RelativePosition = position::Position<Relative<FrameToFrame>>;
pub type RelativeRotation = rotation::Rotation<Relative<FrameToFrame>>;
pub type RelativeScale = scale::Scale<Relative<FrameToFrame>>;

// Global coordinates updated frame to frame
pub type Position = position::Position<Global<FrameToFrame>>;
pub type Rotation = rotation::Rotation<Global<FrameToFrame>>;
pub type Scale = scale::Scale<Global<FrameToFrame>>;

// Global coordinates updated during ticks for interpolation (first point)
pub type LastTickedPosition = position::Position<Global<LastTick>>;
pub type LastTickedRotation = rotation::Rotation<Global<LastTick>>;
pub type LastTickedScale = scale::Scale<Global<LastTick>>;

// Global coordinates updated frame to frame
pub type CurrentTickedPosition = position::Position<Global<CurrentTick>>;
pub type CurrentTickedRotation = rotation::Rotation<Global<CurrentTick>>;
pub type CurrentTickedScale = scale::Scale<Global<CurrentTick>>;