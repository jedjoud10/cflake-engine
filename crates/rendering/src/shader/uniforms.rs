use crate::{context::Context, object::Active};
use rendering_derive::Uniform;
use super::Program;

// A uniform value that can be stored within some uniforms
pub trait UniformValue {
    // Update the uniform within the currentlty bound program
    unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>);
}

// This macro will automatically implement the trait for single element types, like basic numbers
macro_rules! impl_basic_uniform_value {
    ($glfunc:ident, $t:ty) => {     
        paste::paste! {
            impl UniformValue for $t {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 1 $glfunc>](p, loc as i32, *self)
                    }
                }
            }
        }
    };
}

// This macro will automatically implement the trait for arrays
macro_rules! impl_arrays_uniform_value {
    ($glfunc:ident, $t:ty) => {     
        paste::paste! {
            impl UniformValue for [$t] {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 1 $glfunc v>](p, loc as i32, self.len() as i32, self.as_ptr())
                    }
                }
            }
        }
    };
}

// This macro will automatically implement the trait for math vectors
macro_rules! impl_vectors_uniform_value {
    ($glfunc:ident, $t:ty) => {     
        paste::paste! {
            impl UniformValue for vek::Vec2<$t> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 2 $glfunc>](p, loc as i32, self.x, self.y)
                    }
                }
            }

            impl UniformValue for vek::Vec3<$t> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 3 $glfunc>](p, loc as i32, self.x, self.y, self.z)
                    }
                }
            }

            impl UniformValue for vek::Vec4<$t> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 4 $glfunc>](p, loc as i32, self.x, self.y, self.z, self.w)
                    }
                }
            }

            impl UniformValue for vek::Rgb<$t> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 3 $glfunc>](p, loc as i32, self.r, self.g, self.b)
                    }
                }
            }

            impl UniformValue for vek::Rgba<$t> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 4 $glfunc>](p, loc as i32, self.r, self.g, self.b, self.a)
                    }
                }
            }
        }
    };
}

// This macro will automatically implement the trait for matrices
macro_rules! impl_matrices_uniform_value {
    () => {     
        paste::paste! {
            impl UniformValue for vek::Mat4<f32> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix4fv(p, loc as i32, 1, Self::GL_SHOULD_TRANSPOSE as _, self.as_col_ptr())
                    }
                }
            }

            impl UniformValue for vek::Mat3<f32> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix3fv(p, loc as i32, 1, Self::GL_SHOULD_TRANSPOSE as _, self.as_col_ptr())
                    }
                }
            }

            impl UniformValue for vek::Mat2<f32> {
                unsafe fn set_raw_uniform_value(&self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix2fv(p, loc as i32, 1, Self::GL_SHOULD_TRANSPOSE as _, self.as_col_ptr())
                    }
                }
            }
        }
    };
}


// Automatic implementations for basic types
impl_basic_uniform_value!(f, f32);
impl_basic_uniform_value!(ui, u32);
impl_basic_uniform_value!(i, i32);

// Arrays
impl_arrays_uniform_value!(f, f32);
impl_arrays_uniform_value!(ui, u32);
impl_arrays_uniform_value!(i, i32);

// Vectors
impl_vectors_uniform_value!(f, f32);
impl_vectors_uniform_value!(ui, u32);
impl_vectors_uniform_value!(i, i32);

// Matrices
impl_matrices_uniform_value!();



// A uniform struct will set multiple uniform values at once
pub unsafe trait UniformStruct {
    // Set multiple uniform values at once
    unsafe fn set_uniform_values(&self, bound: &mut Active<Program>);
}