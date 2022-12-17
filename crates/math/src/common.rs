// Scalar type, f32 or f64 based on precision
#[cfg(not(feature = "double-precision"))]
pub type Scalar = f32;
#[cfg(feature = "double-precision")]
pub type Scalar = f64;

// Rotation type
#[cfg(not(feature = "two-dim"))]
pub type RawRotation = vek::Quaternion<Scalar>;
#[cfg(feature = "two-dim")]
pub type RawRotation = Scalar;

// Matrix type
#[cfg(not(feature = "two-dim"))]
pub type RawMatrix = vek::Mat4<Scalar>;
#[cfg(feature = "two-dim")]
pub type RawMatrix = vek::Mat3<Scalar>;

// Point type
#[cfg(not(feature = "two-dim"))]
pub type RawPoint = vek::Vec3<Scalar>;
#[cfg(feature = "two-dim")]
pub type RawPoint = vek::Vec2<Scalar>;

// Extent type
#[cfg(not(feature = "two-dim"))]
pub type RawExtent = vek::Extent3<Scalar>;
#[cfg(feature = "two-dim")]
pub type RawExtent = vek::Extent2<Scalar>;
