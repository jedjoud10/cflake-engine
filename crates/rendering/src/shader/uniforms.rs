use std::num::NonZeroU32;

use super::Program;
use crate::{
    context::Context,
    object::{Active, ToGlName, ToGlType},
    texture::{Bindless, Sampler, TexelLayout, Texture, Texture2D, R},
};

// IMplement the scalar trait for single, scalar uniform types
macro_rules! impl_scalars {
    ($glfunc:ident, $t:ty) => {
        paste::paste! {
            impl SetRawUniform for $t {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 1 $glfunc>](program, loc, self)
                }
            }

            impl Scalar for $t {}
        }
    };
}

// Implement the array trait for arrays of scalar types
macro_rules! impl_scalar_arrays {
    ($glfunc:ident, $t:ty) => {
        paste::paste! {
            // Scalar arrays
            impl<'a> SetRawUniform for &'a [$t] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 1 $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr())
                }
            }

            impl<'a, const SIZE: usize> SetRawUniform for &'a [$t; SIZE] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 1 $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr())
                }
            }

            impl<const SIZE: usize> SetRawUniform for [$t; SIZE] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 1 $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr())
                }
            }

            impl<'a> Array for &'a [$t] {}
            impl<'a, const SIZE: usize> Array for &'a [$t; SIZE] {}
            impl<'a, const SIZE: usize> Array for [$t; SIZE] {}
        }
    };
}

// Simply macro wrapper around impl_vector_arrays_unique to automate it even more
macro_rules! impl_vector_arrays {
    ($glfunc:ident, $t:ty) => {
        impl_vector_arrays_unique!($glfunc, 2, vek::Vec2<$t>);
        impl_vector_arrays_unique!($glfunc, 3, vek::Vec3<$t>);
        impl_vector_arrays_unique!($glfunc, 4, vek::Vec4<$t>);
    };
}

// Implement the array trait for arrays of vector types ($t being the vector type directly)
macro_rules! impl_vector_arrays_unique {
    ($glfunc:ident, $count:expr, $t:ty) => {
        paste::paste! {
            // Vec2 arrays
            impl<'a> SetRawUniform for &'a [$t] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform $count $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr() as _)
                }
            }
            impl<'a, const SIZE: usize> SetRawUniform for &'a [$t; SIZE] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform $count $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr() as _)
                }
            }

            impl<const SIZE: usize> SetRawUniform for [$t; SIZE] {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform $count $glfunc v>](program, loc as i32, self.len() as i32, self.as_ptr() as _)
                }
            }

            impl<'a> Array for &'a [$t] {}
            impl<'a, const SIZE: usize> Array for &'a [$t; SIZE] {}
            impl<'a, const SIZE: usize> Array for [$t; SIZE] {}
        }
    };
}

// Implement the vector trait for mathetmatical vectors that consist of scalar types
macro_rules! impl_math_vectors {
    ($glfunc:ident, $t:ty) => {
        paste::paste! {
            impl SetRawUniform for vek::Vec2<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 2 $glfunc>](program, loc, self.x, self.y)
                }
            }

            impl Vector<2> for vek::Vec2<$t> {}

            impl SetRawUniform for vek::Vec3<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 3 $glfunc>](program, loc, self.x, self.y, self.z)
                }
            }

            impl Vector<3> for vek::Vec3<$t> {}

            impl SetRawUniform for vek::Rgb<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 3 $glfunc>](program, loc, self.r, self.g, self.b)
                }
            }

            impl Vector<3> for vek::Rgb<$t> {}

            impl SetRawUniform for vek::Vec4<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 4 $glfunc>](program, loc, self.x, self.y, self.z, self.w)
                }
            }

            impl Vector<4> for vek::Vec4<$t> {}


            impl SetRawUniform for vek::Rgba<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 4 $glfunc>](program, loc, self.r, self.g, self.b, self.a)
                }
            }

            impl Vector<4> for vek::Rgb<$t> {}
        }
    };
}

// Implement the matrix trait for 4x4, 3x3, and 2x2 floating point matrices
macro_rules! impl_matrices {
    () => {
        paste::paste! {
            impl<'a> SetRawUniform for &'a vek::Mat4<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix4fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> SetRawUniform for &'a vek::Mat3<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix3fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> SetRawUniform for &'a vek::Mat2<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix2fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> Matrix<4, 4> for &'a vek::Mat4<f32> {}
            impl<'a> Matrix<3, 3> for &'a vek::Mat3<f32> {}
            impl<'a> Matrix<2, 2> for &'a vek::Mat2<f32> {}
        }
    };
}

// Insanity
mod raw {
    // A uniform variable trait that will set a unique uniform within a shader
    pub trait SetRawUniform {
        unsafe fn set(self, loc: i32, program: u32);
    }
}
use raw::SetRawUniform;

// Wrapper traits
pub trait Scalar: SetRawUniform {}
pub trait Array: SetRawUniform {}
pub trait Vector<const SIZE: u32>: SetRawUniform {}
pub trait Matrix<const HEIGHT: u32, const WIDTH: u32>: SetRawUniform {}

// Scalar implementations
impl_scalars!(f, f32);
impl_scalars!(i, i32);
impl_scalars!(ui, u32);

// Array implementations (scalars)
impl_scalar_arrays!(f, f32);
impl_scalar_arrays!(i, i32);
impl_scalar_arrays!(ui, u32);

// Array implementations (vectors)
impl_vector_arrays!(f, f32);
impl_vector_arrays!(i, i32);
impl_vector_arrays!(ui, u32);

// Vector implementations
impl_math_vectors!(f, f32);
impl_math_vectors!(i, i32);
impl_math_vectors!(ui, u32);

// Matrix implementations
impl_matrices!();

// Normal implementations now
impl<'a> SetRawUniform for &'a Sampler {
    unsafe fn set(self, loc: i32, program: u32) {
        todo!()
    }
}

// The main struct that will allow us to set the uniforms
pub struct Uniforms<'a>(Active<'a, Program>);

impl<'a> Uniforms<'a> {
    // Set the type for any object, as long as it implements SetRawUniform
    fn set_raw<A: SetRawUniform>(&mut self, name: &'static str, val: A) {
        let p = self.0.as_ref().program.get();
        if let Some(loc) = self.0.uniform_location(name) {
            unsafe { val.set(loc as i32, p) }
        }
    }

    // Set a single scalar type using the Scalar trait
    pub fn set_scalar<S: Scalar>(&mut self, name: &'static str, scalar: S) {
        self.set_raw(name, scalar);
    }

    // Set an array of values values
    pub fn set_array<S: Array>(&mut self, name: &'static str, array: S) {
        self.set_raw(name, array);
    }

    // Set a 2D vector that consists of scalar values
    pub fn set_vec2<V: Vector<2>>(&mut self, name: &'static str, vec: V) {
        self.set_raw(name, vec);
    }

    // Set a 3D vector that consists of scalar values
    pub fn set_vec3<V: Vector<3>>(&mut self, name: &'static str, vec: V) {
        self.set_raw(name, vec);
    }

    // Set a 4D vector that consists of scalar values
    pub fn set_vec4<V: Vector<4>>(&mut self, name: &'static str, vec: V) {
        self.set_raw(name, vec);
    }

    // Set a 4x4 matrix
    pub fn set_mat4x4<M: Matrix<4, 4>>(&mut self, name: &'static str, mat: M) {
        self.set_raw(name, mat);
    }

    // Set a 3x3 matrix
    pub fn set_mat3x3<M: Matrix<3, 3>>(&mut self, name: &'static str, mat: M) {
        self.set_raw(name, mat);
    }

    // Set a 2x2 matrix
    pub fn set_mat2x2<M: Matrix<2, 2>>(&mut self, name: &'static str, mat: M) {
        self.set_raw(name, mat);
    }

    // Set a texture sampler, assuming that it uses normal texture binding and not bindless textures
    unsafe fn set_normal_sampler_unchecked(&mut self, name: &'static str, target: u32, texture: NonZeroU32) {
        // First of all, we must get the texture unit offset
        let count = self.0.as_mut().texture_units.len() as u32;
        let offset = *self.0.as_mut().texture_units.entry(name).or_insert(count);

        // Set the uniforms properly now
        let p = self.0.as_ref().program.get();
        if let Some(loc) = self.0.uniform_location(name) {
            gl::ActiveTexture(gl::TEXTURE0 + offset);
            gl::BindTexture(target, texture.get());
            gl::ProgramUniform1i(p, loc as i32, offset as i32);
        }
    }

    // Set a texture sampler, assuming that it uses bindless textures
    unsafe fn set_bindless_sampler_unchecked(&mut self, name: &'static str, bindless: Bindless) {
        // If the texture isn't resident, we have to make it resident
        if !bindless.resident.get() {
            // TODO: Convert the texture into a resident texture
        } else {
            // The bindless texture handle is already resident, we just need to set the uniform
            let p = self.0.as_ref().program.get();
            if let Some(loc) = self.0.uniform_location(name) {
                gl::ProgramUniformHandleui64ARB(p, loc as i32, bindless.handle);
            }
        }
    }

    // Set a texture sampler, switching between the bindless and normal methods
    pub fn set_sampler(&mut self, name: &'static str, sampler: &Sampler) {
        unsafe {
            match sampler.bindless {
                Some(bindless) => self.set_bindless_sampler_unchecked(name, bindless),
                None => self.set_normal_sampler_unchecked(name, sampler.target, sampler.texture),
            }
        }
    }
}