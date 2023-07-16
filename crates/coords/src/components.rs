use std::marker::PhantomData;

mod position;
mod rotation;
mod scale;
mod relations;

pub use relations::*;
pub use position::Position as UnmarkedPosition;
pub use rotation::Rotation as UnmarkedRotation;
pub use scale::Scale as UnmarkedScale;

/// Values are updated from frame to frame.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameToFrame;

/// Values are set to be the first interpolation point when doing sub tick interpolation.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LastTick;

/// Values are set to be the next (or last) interpolation point when doing sub tick interpolation.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrentTick;

/// Global reference frame. Relative to world origin.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global<TimeFrame>(PhantomData<TimeFrame>);

/// Local reference frame. Relative to parent transform.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local<TimeFrame>(PhantomData<TimeFrame>);

/// Local position updated frame to frame.
pub type LocalPosition = position::Position<Local<FrameToFrame>>;

/// Local rotation updated frame to frame.
pub type LocalRotation = rotation::Rotation<Local<FrameToFrame>>;

/// Local scale updated frame to frame.
pub type LocalScale = scale::Scale<Local<FrameToFrame>>;

/// Global position updated frame to frame.
pub type Position = position::Position<Global<FrameToFrame>>;

/// Global rotation updated frame to frame.
pub type Rotation = rotation::Rotation<Global<FrameToFrame>>;

/// Global scale updated frame to frame.
pub type Scale = scale::Scale<Global<FrameToFrame>>;

/// Global position updated during ticks for interpolation (last point).
pub type LastTickedPosition = position::Position<Global<LastTick>>;

/// Global rotation updated during ticks for interpolation (last point).
pub type LastTickedRotation = rotation::Rotation<Global<LastTick>>;

/// Global scale updated during ticks for interpolation (last point).
pub type LastTickedScale = scale::Scale<Global<LastTick>>;

/// Global position updated during ticks for interpolation (current point).
pub type CurrentTickedPosition = position::Position<Global<CurrentTick>>;

/// Global rotation updated during ticks for interpolation (current point).
pub type CurrentTickedRotation = rotation::Rotation<Global<CurrentTick>>;

/// Global scale updated during ticks for interpolation (current point).
pub type CurrentTickedScale = scale::Scale<Global<CurrentTick>>;