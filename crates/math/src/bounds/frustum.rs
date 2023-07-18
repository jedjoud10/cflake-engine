use arrayvec::ArrayVec;
use num_traits::real::Real;

// A single frustum plane that contains a normal direction and a distance
#[derive(Clone, Copy, PartialEq)]
pub struct FrustumPlane<T: Real> {
    pub normal: vek::Vec3<T>,
    pub distance: T,
}

impl<T: Real> FrustumPlane<T> {
    // Create a new frustum plane with a vec4 that contains the normal and distance
    pub fn new(column: vek::Vec4<T>) -> Self {
        let mag = column.xyz().magnitude();
        Self {
            normal: column.xyz() / mag,
            distance: (column.w / mag),
        }
    }
}

// A multitude of frustum planes that can be created from a camera's projection and view matrix
#[derive(Clone, Copy, PartialEq)]
pub struct Frustum<T: Real>([FrustumPlane<T>; 6]);

impl<T: Real> Frustum<T> {
    // Get an immutable reference to the frustum planes
    pub fn planes(&self) -> &[FrustumPlane<T>; 6] {
        &self.0
    }

    // Get an immutable iterator over the frustum planes
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &FrustumPlane<T>> {
        self.0.iter()
    }

    // Get a mutable iterator over the frustum planes
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut FrustumPlane<T>> {
        self.0.iter_mut()
    }
}

macro_rules! impl_methods {
    ($t:ty) => {
        impl Frustum<$t> {
            // Create a new frustum using a camera's projection matrix and view matrix
            pub fn from_camera_matrices(projection: vek::Mat4<$t>, view: vek::Mat4<$t>) -> Self {
                let columns = (projection * view).transposed().into_col_arrays();
                let columns = columns
                    .into_iter()
                    .map(vek::Vec4::from)
                    .collect::<ArrayVec<vek::Vec4<$t>, 4>>();

                // Magic from https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
                // And also from https://gamedev.stackexchange.com/questions/156743/finding-the-normals-of-the-planes-of-a-view-frustum
                // YAY https://stackoverflow.com/questions/12836967/extracting-view-frustum-planes-gribb-hartmann-method
                let left = FrustumPlane::new(columns[3] + columns[0]);
                let right = FrustumPlane::new(columns[3] - columns[0]);
                let top = FrustumPlane::new(columns[3] - columns[1]);
                let bottom = FrustumPlane::new(columns[3] + columns[1]);
                let near = FrustumPlane::new(columns[3] + columns[2]);
                let far = FrustumPlane::new(columns[3] - columns[2]);
                Self([top, bottom, left, right, near, far])
            }
        }
    };
}

impl_methods!(f32);
impl_methods!(f64);
