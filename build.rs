use core::panic;
use std::process::Command;

fn main() {
	let command = Command::new("target\\debug\\hypothermia").args(&["--pack-resources"]).output().unwrap();
}