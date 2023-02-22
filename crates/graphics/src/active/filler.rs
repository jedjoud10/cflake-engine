use thiserror::Error;

use crate::GpuPod;

#[derive(Debug, Error)]
pub enum FillError<'a> {
    #[error("The field {name} does not exist on the filler")]
    MissingField {
        name: &'a str
    },

    #[error("Small dick, no balls, no bitches, many Ls")]
    WrongSize,
}

// A value filler is used to fill data inside a UBO or push constants
// This allows the user to set multiple fields, and them uploads them all at the same time in batch
pub trait ValueFiller {
    // Called when we set any field (not type checked)
    fn set<'s, T: GpuPod>(&mut self, name: &'s str, value: T) -> Result<(), FillError<'s>>;

    // Set a single scalar type using the Scalar trait
    fn set_scalar<'s, S: GpuPod>(&mut self, name: &'s str, scalar: S) -> Result<(), FillError<'s>> {
        self.set::<S>(name, scalar)
    }

    // Set an array of values values
    fn set_array<'s, S: GpuPod>(&mut self, name: &'s str, array: S) -> Result<(), FillError<'s>> {
        self.set::<S>(name, array)
    }

    // Set a 2D vector that consists of scalar values
    fn set_vec2<'s, V: GpuPod>(&mut self, name: &'s str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 3D vector that consists of scalar values
    fn set_vec3<'s, V: GpuPod>(&mut self, name: &'s str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 4D vector that consists of scalar values
    fn set_vec4<'s, V: GpuPod>(&mut self, name: &'s str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 4x4 matrix
    fn set_mat4x4<'s, M: GpuPod>(&mut self, name: &'s str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }

    // Set a 3x3 matrix
    fn set_mat3x3<'s, M: GpuPod>(&mut self, name: &'s str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }

    // Set a 2x2 matrix
    fn set_mat2x2<'s, M: GpuPod>(&mut self, name: &'s str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }
}