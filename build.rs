
use std::{process::Command};

fn main() {
    Command::new("pack-resources.bat").spawn().unwrap();
}
