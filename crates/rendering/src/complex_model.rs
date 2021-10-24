use crate::{Model};

// A complex model that has support for Multiple Materials and other fancy things
#[derive(Default)]
pub struct ComplexModel {
    pub sub_models: Vec<Model>,    
}

impl ComplexModel {
    // Add a model to this complex model
    pub fn add_model(&mut self, model: Model) {
        self.sub_models.push(model);
    }
}