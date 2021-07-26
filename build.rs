use core::panic;
use std::{env, process::Command};

fn main() {
	Command::new("pack-resources.bat").spawn().unwrap();
}