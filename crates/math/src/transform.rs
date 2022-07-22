mod location;
mod rotation;
mod scale;

pub use location::*;
pub use rotation::*;
pub use scale::*;

pub trait IntoMatrix {
    fn into_matrix(self) -> vek::Mat4<f32>;
}