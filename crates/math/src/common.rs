// Rotation type
#[cfg(not(feature = "two-dim"))]
pub type RawRotation = vek::Quaternion<f32>;
#[cfg(feature = "two-dim")]
pub type RawRotation = f32;

// Matrix type
#[cfg(not(feature = "two-dim"))]
pub type RawMatrix = vek::Mat4<f32>;
#[cfg(feature = "two-dim")]
pub type RawMatrix = vek::Mat3<f32>;

// Point type
#[cfg(not(feature = "two-dim"))]
pub type RawPoint = vek::Vec3<f32>;
#[cfg(feature = "two-dim")]
pub type RawPoint = vek::Vec2<f32>;

// Extent type
#[cfg(not(feature = "two-dim"))]
pub type RawExtent = vek::Extent3<f32>;
#[cfg(feature = "two-dim")]
pub type RawExtent = vek::Extent2<f32>;