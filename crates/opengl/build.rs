extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(
        Api::Gl,
        (4, 6),
        Profile::Core,
        Fallbacks::All,
        [
            "GL_ARB_bindless_texture",
            "GL_KHR_parallel_shader_compile",
            "GL_EXT_texture_filter_anisotropic",
            "GL_ARB_seamless_cubemap_per_texture",
        ],
    )
    .write_bindings(GlobalGenerator, &mut file)
    .unwrap();
}
