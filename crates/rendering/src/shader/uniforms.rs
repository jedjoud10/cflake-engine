use super::{BlockIndex, Program, UniformsError};
use crate::{
    buffer::{Buffer, ShaderBuffer, UniformBuffer},
    context::{Shared, ToGlName},
    prelude::{MipLevelMut, Texel},
    texture::Texture,
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

            impl<'a> SetRawUniform for &'a $t {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 1 $glfunc>](program, loc, *self)
                }
            }

            impl<'a> Scalar for &'a $t {}
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


            impl<'a> SetRawUniform for &'a vek::Vec2<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 2 $glfunc>](program, loc, self.x, self.y)
                }
            }

            impl<'a> Vector<2> for &'a vek::Vec2<$t> {}

            impl<'a> SetRawUniform for &'a vek::Vec3<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 3 $glfunc>](program, loc, self.x, self.y, self.z)
                }
            }

            impl<'a> Vector<3> for &'a vek::Vec3<$t> {}

            impl<'a> SetRawUniform for &'a vek::Rgb<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 3 $glfunc>](program, loc, self.r, self.g, self.b)
                }
            }

            impl<'a> Vector<3> for &'a vek::Rgb<$t> {}

            impl<'a> SetRawUniform for &'a vek::Vec4<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 4 $glfunc>](program, loc, self.x, self.y, self.z, self.w)
                }
            }

            impl<'a> Vector<4> for &'a vek::Vec4<$t> {}


            impl<'a> SetRawUniform for &'a vek::Rgba<$t> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::[<ProgramUniform 4 $glfunc>](program, loc, self.r, self.g, self.b, self.a)
                }
            }

            impl<'a> Vector<4> for &'a vek::Rgb<$t> {}
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

            impl<'a> SetRawUniform for vek::Mat4<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix4fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> SetRawUniform for vek::Mat3<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix3fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> SetRawUniform for vek::Mat2<f32> {
                unsafe fn set(self, loc: i32, program: u32) {
                    gl::ProgramUniformMatrix2fv(program, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                }
            }

            impl<'a> Matrix<4, 4> for vek::Mat4<f32> {}
            impl<'a> Matrix<3, 3> for vek::Mat3<f32> {}
            impl<'a> Matrix<2, 2> for vek::Mat2<f32> {}
        }
    };
}

mod raw {
    // A uniform variable trait that will set a unique uniform within a shader
    pub trait SetRawUniform {
        unsafe fn set(self, loc: i32, program: u32);
    }

    // Wrapper traits
    pub trait Scalar: SetRawUniform {}
    pub trait Array: SetRawUniform {}
    pub trait Vector<const SIZE: u32>: SetRawUniform {}
    pub trait Matrix<const HEIGHT: u32, const WIDTH: u32>: SetRawUniform {}
}

use ahash::{AHashMap, AHashSet};
use raw::*;

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

struct TextureUnit {
    unit: u32,
    texture: u32,
}

enum BufferBindingRange {}

struct BufferBindingId {
    name: String,
    range: BufferBindingRange,
}

struct BufferBinding {
    buffer: u32,
    binding: u32,
}

// The main struct that will allow us to set the shader uniforms before it's execution
// If debug assertions are enabled, the safe execution functions will check if the uniforms are valid
// You can always use the "_unchecked" variant of the execution functions to override this behavior
// debug_assertions on -> verify uniform completion
// debug_assertions off -> assume uniforms are valid
pub struct Uniforms<'uniforms> {
    program: &'uniforms mut Program,
    texture_units: AHashMap<String, TextureUnit>,
    buffer_bindings: AHashMap<String, BufferBinding>,

    #[cfg(debug_assertions)]
    dbg_bound_uniforms: AHashSet<String>,

    #[cfg(debug_assertions)]
    dbg_bound_buffer_bindings: AHashSet<String>,
}

// Valid uniforms are the only way we can render the uniforms normally
pub struct ValidUniforms<'validated>(pub(crate) &'validated mut Program);

impl<'uniforms> Uniforms<'uniforms> {
    // Create a temporary uniforms wrapper using a program and it's inner introspection data
    pub(crate) fn new(program: &'uniforms mut Program) -> Self {
        // Bind the program to the global state
        unsafe {
            gl::UseProgram(program.name());
        }

        Self {
            texture_units: AHashMap::with_capacity(program.uniform_locations.len()),
            buffer_bindings: AHashMap::with_capacity(program.buffer_block_locations.len()),

            #[cfg(debug_assertions)]
            dbg_bound_uniforms: AHashSet::with_capacity(program.uniform_locations.len()),

            #[cfg(debug_assertions)]
            dbg_bound_buffer_bindings: AHashSet::with_capacity(
                program.buffer_block_locations.len(),
            ),
            program,
        }
    }

    // Check for any missing / invalid uniforms and panic if we find any
    #[cfg(debug_assertions)]
    fn check_completion(&mut self) -> Result<(), UniformsError> {
        let missing_uniform = self
            .program
            .uniform_locations
            .keys()
            .find(|name| !self.dbg_bound_uniforms.contains(*name));

        let missing_buffer_binding = self
            .program
            .buffer_block_locations
            .keys()
            .find(|name| !self.dbg_bound_buffer_bindings.contains(*name));

        if let Some(name) = missing_uniform {
            return Err(UniformsError::IncompleteUniform(name.clone()));
        }

        if let Some(name) = missing_buffer_binding {
            return Err(UniformsError::IncompleteBufferBinding(name.clone()));
        }

        let deleted_texture = self
            .texture_units
            .iter()
            .find(|(_, unit)| unsafe { gl::IsTexture(unit.texture) == 0 })
            .map(|(name, _)| name);
        let deleted_buffer = self
            .buffer_bindings
            .iter()
            .find(|(_, binding)| unsafe { gl::IsBuffer(binding.buffer) == 0 })
            .map(|(name, _)| name);

        if let Some(name) = deleted_texture {
            return Err(UniformsError::DeletedTextureUnit(name.clone()));
        }

        if let Some(name) = deleted_buffer {
            return Err(UniformsError::DeletedBufferBinding(name.clone()));
        }

        Ok(())
    }

    // Validate the underlying uniforms
    // If debug assertions are off, this function will always return Ok
    pub fn validate(&mut self) -> Result<ValidUniforms, UniformsError> {
        #[cfg(debug_assertions)]
        self.check_completion()?;

        Ok(unsafe { self.assume_valid() })
    }

    // Assume that the underlying uniforms are valid without checking
    pub unsafe fn assume_valid(&mut self) -> ValidUniforms {
        ValidUniforms(self.program)
    }

    // Set the type for any uniform object, as long as it implements SetRawUniform
    fn set_raw_uniform<A: SetRawUniform>(&mut self, name: &str, val: A) {
        let location = self.program.uniform_locations.get(name);
        if let Some(location) = location {
            #[cfg(debug_assertions)]
            self.dbg_bound_uniforms.insert(name.to_string());

            unsafe { val.set(*location as i32, self.program.name()) }
        }
    }
    // Set a texture and reutrn it's new texture unit location
    fn set_raw_texture(&mut self, name: &str, texture_name: u32) -> u32 {
        let count = self.texture_units.len() as u32;
        let offset = self
            .texture_units
            .entry(name.to_string())
            .or_insert(TextureUnit {
                texture: u32::MAX,
                unit: count,
            })
            .unit;

        self.texture_units.get_mut(name).unwrap().texture = texture_name;
        self.set_scalar(name, offset as i32);
        offset
    }

    // Set a buffer and return it's new binding point location
    fn set_raw_buffer<T: Shared, const TARGET: u32>(
        &mut self,
        name: &str,
        buffer: &Buffer<T, TARGET>,
    ) -> u32 {
        #[cfg(debug_assertions)]
        self.dbg_bound_buffer_bindings.insert(name.to_string());

        let count = self.buffer_bindings.len() as u32;
        self.buffer_bindings
            .entry(name.to_string())
            .or_insert(BufferBinding {
                buffer: u32::MAX,
                binding: count,
            })
            .binding
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

    // Set a sampler uniform (that accepts any type of texture)
    pub fn set_sampler<T: Texture>(&mut self, name: &str, sampler: &T) {
        assert!(!sampler.is_user_writing(), "Cannot read from a texture that is currently being written to");

        let offset = self.set_raw_texture(name, sampler.name());

        unsafe {
            gl::BindTextureUnit(offset, sampler.name());
        }
    }

    // Set an image uniform (a texture that we can modify)
    pub fn set_image<T: Texture>(&mut self, name: &str, sampler: &mut MipLevelMut<T>) {
        let offset = self.set_raw_texture(name, sampler.texture().name());

        unsafe {
            // TODO: What the fuk do we do when we have a depth texture here?
            // TODO: Handle layered textures bozo
            let format_ = <T::T as Texel>::INTERNAL_FORMAT;
            gl::BindImageTexture(
                offset,
                sampler.texture().name(),
                sampler.level() as i32,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                format_,
            );
        }
    }

    // Set a uniform buffer (read only)
    pub fn set_uniform_buffer<T: Shared>(&mut self, name: &str, buffer: &UniformBuffer<T>) {
        let binding = self.set_raw_buffer(name, buffer);

        unsafe {
            self.buffer_bindings.get_mut(name).unwrap().buffer = buffer.name();
            if let BlockIndex::UniformBlock(index) =
                self.program.buffer_block_locations.get(name).unwrap()
            {
                gl::UniformBlockBinding(self.program.name, *index, binding);
                gl::BindBuffer(gl::UNIFORM_BUFFER, buffer.name());
                gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, buffer.name());
            }
        }
    }

    // Set a shader storage buffer (only for reading)
    pub fn set_shader_storage_buffer<T: Shared>(&mut self, name: &str, buffer: &ShaderBuffer<T>) {
        let binding = self.set_raw_buffer(name, buffer);

        unsafe {
            self.buffer_bindings.get_mut(name).unwrap().buffer = buffer.name();
            if let BlockIndex::ShaderStorageBlock(index) =
                self.program.buffer_block_locations.get(name).unwrap()
            {
                gl::ShaderStorageBlockBinding(self.program.name, *index, binding);
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer.name());
                gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, buffer.name());
            }
        }
    }

    // Set a shader storage buffer (read and write)
    pub fn set_shader_storage_buffer_mut<T: Shared>(
        &mut self,
        name: &str,
        buffer: &mut ShaderBuffer<T>,
    ) {
        // TODO: Check shader introspection to make sure the shader is valid for writing into it (not readonly)
        // Custom shading language?
        self.set_shader_storage_buffer(name, buffer);
    }
}
