use super::Program;
use crate::{
    context::Context,
    object::Active,
    texture::{TexelLayout, Texture, R, Texture2D, Sampler},
};

// This macro will automatically implement the set function for single element types, like basic numbers and scalars
macro_rules! impl_basic_uniform_value {
    ($glfunc:ident, $name:ident, $t:ty) => {
        paste::paste! {
            fn [<set_ $name>] (&mut self, name: &'static str, val: $t) {
                unsafe {
                    let p = self.0.as_ref().program.get();
                    if let Some(loc) = self.0.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 1 $glfunc>](p, loc as i32, val)
                    }
                }
            }
        }
    };
}

// This macro will automatically implement the set function for arrays
macro_rules! impl_arrays_uniform_value {
    ($glfunc:ident, $name:ident, $t:ty) => {
        paste::paste! {
            fn [<set_ $name _array>](&mut self, name: &'static str, array: &[$t]) {
                let p = self.0.as_ref().program.get();
                unsafe {
                    if let Some(loc) = self.0.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 1 $glfunc v>](p, loc as i32, array.len() as i32, array.as_ptr())
                    }
                }
            }            
        }
    };
}
/*



// This macro will automatically implement the trait for math vectors
macro_rules! impl_vectors_uniform_value {
    ($glfunc:ident, $suffix:ident, $t:ty) => {
        paste::paste! {
            fn [<$suffix vec3>](&mut self, name: &'static str, bound: &mut Active<Program>) {
                let p = self.0.as_ref().program.get();
                unsafe {
                    if let Some(loc) = self.0.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 2 $glfunc>](p, loc as i32, self.x, self.y)
                    }
                }
            }
            /*

            impl<'a> UniformValue for &'a vek::Vec3<$t> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 3 $glfunc>](p, loc as i32, self.x, self.y, self.z)
                    }
                }
            }

            impl<'a> UniformValue for &'a vek::Vec4<$t> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 4 $glfunc>](p, loc as i32, self.x, self.y, self.z, self.w)
                    }
                }
            }

            impl<'a> UniformValue for &'a vek::Rgb<$t> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 3 $glfunc>](p, loc as i32, self.r, self.g, self.b)
                    }
                }
            }

            impl<'a> UniformValue for &'a vek::Rgba<$t> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::[<ProgramUniform 4 $glfunc>](p, loc as i32, self.r, self.g, self.b, self.a)
                    }
                }
            }
            */
        }
    };
}

// This macro will automatically implement the trait for matrices
macro_rules! impl_matrices_uniform_value {
    () => {
        paste::paste! {
            impl<'a> UniformValue for &'a vek::Mat4<f32> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix4fv(p, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                    }
                }
            }

            impl<'a> UniformValue for &'a vek::Mat3<f32> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix3fv(p, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                    }
                }
            }

            impl<'a> UniformValue for &'a vek::Mat2<f32> {
                unsafe fn set_raw_uniform_value(self, name: &'static str, bound: &mut Active<Program>) {
                    let p = bound.as_ref().program.get();
                    if let Some(loc) = bound.fetch_uniform_location(name) {
                        gl::ProgramUniformMatrix2fv(p, loc as i32, 1, gl::FALSE, self.as_col_ptr())
                    }
                }
            }
        }
    };
}

*/


// The main struct that will allow us to set the uniforms
pub struct Uniforms<'a>(Active<'a, Program>);

impl<'a> Uniforms<'a> {
    // Basic scalar types
    impl_basic_uniform_value!(f, float, f32);
    impl_basic_uniform_value!(i, int, i32);
    impl_basic_uniform_value!(ui, uint, u32);

    // Vectors make use of generic traits
    impl_arrays_uniform_value!(f, float, f32);
    impl_arrays_uniform_value!(i, int, i32);
    impl_arrays_uniform_value!(ui, uint, u32);

    // 
}

fn test() {
    let u: Uniforms<'static> = todo!();
    u.set_array::<u32>();
    u.set_scalar::<u32>();
    u.set_vec3::<u32>()
}