use std::env;

// Pack the resources
fn main() {
    println!("cargo:rerun-if-changed=resources\\");
    let target_dir = env::var("OUT_DIR").unwrap();
    let target_dir = target_dir.split("build\\dev_game").collect::<Vec<&str>>()[0];
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    resources_packer::pack(target_dir.to_string(), project_dir);
}
