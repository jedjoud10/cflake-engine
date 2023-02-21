use thiserror::Error;

#[derive(Debug, Error)]
pub enum FillError<'a> {
    #[error("The field {name} does not exist on the filler")]
    MissingField {
        name: &'a str
    }
}

// A value filler is used to fill data inside a UBO or push constants
// This allows the user to set multiple fields, and them uploads them all at the same time in batch
pub trait ValueFiller {
    // Get the size and offset of a specific field
    // Should return None when we don't have the field
    /*
    fn get_field_size(&self, name: &str) -> Option<u32>;
    fn get_field_offset(&self, name: &str) -> Option<u32>;
    */

    // Set any field (not type checked)
    fn set<'s, T>(&mut self, name: &str, value: T) -> Result<(), FillError<'s>> {
        todo!()
    }

    // Set a single scalar type using the Scalar trait
    fn set_scalar<'s, S>(&mut self, name: &str, scalar: S) -> Result<(), FillError<'s>> {
        self.set::<S>(name, scalar)
    }

    // Set an array of values values
    fn set_array<'s, S>(&mut self, name: &str, array: S) -> Result<(), FillError<'s>> {
        self.set::<S>(name, array)
    }

    // Set a 2D vector that consists of scalar values
    fn set_vec2<'s, V>(&mut self, name: &str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 3D vector that consists of scalar values
    fn set_vec3<'s, V>(&mut self, name: &str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 4D vector that consists of scalar values
    fn set_vec4<'s, V>(&mut self, name: &str, vec: V) -> Result<(), FillError<'s>> {
        self.set::<V>(name, vec)
    }

    // Set a 4x4 matrix
    fn set_mat4x4<'s, M>(&mut self, name: &str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }

    // Set a 3x3 matrix
    fn set_mat3x3<'s, M>(&mut self, name: &str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }

    // Set a 2x2 matrix
    fn set_mat2x2<'s, M>(&mut self, name: &str, mat: M) -> Result<(), FillError<'s>> {
        self.set::<M>(name, mat)
    }
}