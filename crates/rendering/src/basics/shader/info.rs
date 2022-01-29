use crate::basics::transfer::{Transfer, Transferable};

// Some settings to help us query shader info
pub struct ShaderInfoQuerySettings {}

// Some shader info that we queried from the pipeline
#[derive(Clone)]
pub struct ShaderInfo {}

impl Transferable for ShaderInfo {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(self.clone())
    }
}
