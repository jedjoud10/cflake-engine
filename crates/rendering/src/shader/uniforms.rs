use std::collections::HashMap;

use super::{Program, UniformsError};
use crate::{
    object::ToGlName,
    texture::{Texture},
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
// A uniform variable trait that will set a unique uniform within a shader
pub trait SetRawUniform {
    unsafe fn set(self, loc: i32, program: u32);
}

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

// The main struct that will allow us to set the shader uniforms before it's execution
// We must set ALL the uniforms before each shader execution
// Shader uniforms can be fetched from a compute shader using the scheduler() method and from a painter using the uniforms() method
// When we drop the uniforms, we have to assume that we unbind the values that have a specific lifetime, like buffers and samplers
// Since the only way to set the uniforms is to fill them completely, we are sure that the user will never execute a shader with dangling null references to destroyed objects and shit like that
pub struct Uniforms<'uniforms>(
    pub(crate) &'uniforms mut Program,
    pub(crate) Option<UniformsError>,
);

impl<'uniforms> Uniforms<'uniforms> {
    // Make sure the user set all the proper shader variables before executing
    // This will also make sure there are no internal errors stored from within the uniforms
    pub(crate) fn validate(&mut self) -> Result<(), UniformsError> {
        // Extract any internal errors first
        if let Some(err) = self.1.take() {
            return Err(err);
        }

        // Find the first missing uniforms
        let uniforms = &self.0.uniform_locations;
        let missing_uniform = uniforms.iter().find(|(_, (_, set))| !set);
        let bindings = &self.0.binding_points;
        let missing_binding = bindings.iter().find(|(_, (_, set))| !set);

        // If we have a missing uniform or missing binding, the uniform binder is invalid
        let valid = missing_uniform.is_none() && missing_binding.is_none();

        // Le erron throwing
        if !valid {
            // If we have a missing uniform AND a missing binding, prioritize the error for the missing uniform
            Err(match (missing_uniform, missing_binding) {
                (None, Some((name, _))) => UniformsError::IncompleteBinding(name.clone()),
                (Some((name, _)), None) => UniformsError::IncompleteUniform(name.clone()),
                (Some((name, _)), Some(_)) => UniformsError::IncompleteUniform(name.clone()),
                _ => todo!(),
            })
        } else {
            Ok(())
        }
    }

    // Set the type for any object, as long as it implements SetRawUniform
    fn set_raw_uniform<A: SetRawUniform>(&mut self, name: &str, val: A) {
        let locations = &mut self.0.uniform_locations;
        if locations.contains_key(name) {
            // Get the location and set it's "set" flag to true
            let (loc, set) = locations.get_mut(name).unwrap();
            *set = true;
            unsafe { val.set(*loc as i32, self.0.name()) }
        } else {
            // Internally log the missing uniform error
            self.1
                .get_or_insert(UniformsError::InvalidUniformName(name.to_string()));
        }
    }

    // Set a single scalar type using the Scalar trait
    pub fn set_scalar<S: Scalar>(&mut self, name: &str, scalar: S) {
        self.set_raw_uniform(name, scalar);
    }

    // Set an array of values values
    pub fn set_array<S: Array>(&mut self, name: &str, array: S) {
        self.set_raw_uniform(name, array);
    }

    // Set a 2D vector that consists of scalar values
    pub fn set_vec2<V: Vector<2>>(&mut self, name: &str, vec: V) {
        self.set_raw_uniform(name, vec);
    }

    // Set a 3D vector that consists of scalar values
    pub fn set_vec3<V: Vector<3>>(&mut self, name: &str, vec: V) {
        self.set_raw_uniform(name, vec);
    }

    // Set a 4D vector that consists of scalar values
    pub fn set_vec4<V: Vector<4>>(&mut self, name: &str, vec: V) {
        self.set_raw_uniform(name, vec);
    }

    // Set a 4x4 matrix
    pub fn set_mat4x4<M: Matrix<4, 4>>(&mut self, name: &str, mat: M) {
        self.set_raw_uniform(name, mat);
    }

    // Set a 3x3 matrix
    pub fn set_mat3x3<M: Matrix<3, 3>>(&mut self, name: &str, mat: M) {
        self.set_raw_uniform(name, mat);
    }

    // Set a 2x2 matrix
    pub fn set_mat2x2<M: Matrix<2, 2>>(&mut self, name: &str, mat: M) {
        self.set_raw_uniform(name, mat);
    }

    /*

    // Set a texture sampler, assuming that it uses normal texture binding and not bindless textures
    unsafe fn set_normal_sampler_unchecked(
        &mut self,
        name: &'static str,
        target: u32,
        texture: u32,
    ) {

    }

    // Set a texture sampler, assuming that it uses bindless textures
    unsafe fn set_bindless_sampler_unchecked(&mut self, name: &'static str, bindless: &Bindless) {
        // If the texture isn't resident, we have to make it resident
        bindless.last_residency_instant.set(Instant::now());
        if !bindless.resident.get() {
            // Make the bindless texture a resident bindless texture
            bindless.set_residency(true);
        } else {
            // The bindless texture handle is already resident, we just need to set the uniform
            if let Some(loc) = self.location(name) {
                gl::ProgramUniformHandleui64ARB(self.0.name(), loc as i32, bindless.handle);
            }
        }
    }
    */

    // Set a texture sampler uniform
    // Since the lifetime of this sampler *must* outlive the uniforms, we can make sure the program does not contain invalid sampler references
    pub fn set_sampler<T: Texture>(&mut self, name: &str, sampler: &'uniforms T) {
        let count = self.0.texture_units.len() as u32;
        let offset = *self.0.texture_units.entry(name.to_string()).or_insert(count);

        unsafe {
            gl::BindTexture(T::target(), sampler.name());
            gl::ActiveTexture(gl::TEXTURE0 + offset);
            
            // Set the corresponding sampler uniform
            self.set_scalar(name, offset as i32);
        }
    }

    // Apply the uniorms before executing the code
    // This is going to be called internally by the program scheduler
}

impl<'uniforms> Drop for Uniforms<'uniforms> {
    fn drop(&mut self) {
        // This will clear all the "set" states of the user defined inputs
        self.0
            .binding_points
            .iter_mut()
            .chain(self.0.uniform_locations.iter_mut())
            .for_each(|(_, (_, set))| *set = false);
    }
}
