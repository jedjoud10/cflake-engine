use crate::GpuPod;
use thiserror::Error;

// Type of error whenever we try to set a field on a ValueFiller
#[derive(Debug, Error)]
pub enum SetFieldError<'a> {
    #[error("The field {name} does not exist in the layout")]
    MissingField { name: &'a str },

    #[error("The given type does not have the same size as the one defined in the shader source")]
    WrongSize,
}

// A value filler is used to fill data inside a UBO or push constants
// This allows the user to set multiple fields, and them uploads them all at the same time in batch
pub trait ValueFiller {
    fn set<'s, T: GpuPod>(
        &mut self,
        name: &'s str,
        value: T,
    ) -> Result<(), SetFieldError<'s>>;
}
