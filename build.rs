use core::panic;
use std::{env, process::Command};

fn main() {
	Command::new("pack-resources.bat").spawn().expect("OH NO");
	//panic!("Test");
}