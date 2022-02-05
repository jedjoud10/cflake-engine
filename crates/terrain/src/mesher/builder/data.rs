use rendering::basics::model::{CustomVertexDataBuffer, Model};

// Data that regroups the model with it's custom vdata
pub(crate) struct BuilderModelData {
    pub(crate) model: Model,
    pub(crate) vdata: CustomVertexDataBuffer<u32>,
}
