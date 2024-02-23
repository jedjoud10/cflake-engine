use crate::pipeline::{DynPipeline, MaterialId, Pipeline};
use crate::material::Material;
use ahash::AHashMap;
use assets::Assets;

use graphics::{Graphics, PipelineInitializationError};
use std::{any::TypeId, marker::PhantomData, rc::Rc};

// A pipeline manager will store and manager multiple material pipelines and their IDs
pub struct Pipelines {
    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynPipeline>>,
}

impl Pipelines {
    // Create a new pipeline manager with no stored pipelines
    pub fn new() -> Self {
        Self {
            pipelines: Default::default(),
        }
    }

    // Register a new material pipeline within the renderer
    // Assumes that the material does not use any custom settings
    pub fn register<M: Material>(
        &mut self,
        graphics: &Graphics,
        assets: &Assets,
    ) -> Result<MaterialId<M>, PipelineInitializationError>
    where
        for<'x> M::Settings<'x>: Default,
    {
        self.register_with(graphics, Default::default(), assets)
    }

    // Register a new material pipeline within the renderer
    // using the given custom material settings
    pub fn register_with<M: Material>(
        &mut self,
        graphics: &Graphics,
        settings: M::Settings<'_>,
        assets: &Assets,
    ) -> Result<MaterialId<M>, PipelineInitializationError> {
        // Initialize the pipeline and register it if needed
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            log::debug!(
                "Creating pipeline for material {}...",
                std::any::type_name::<M>()
            );
            let pipeline = Pipeline::<M>::new(settings, graphics, assets)?;
            self.pipelines.insert(key, Rc::new(pipeline));
            log::debug!(
                "Registered pipeline for material {}",
                std::any::type_name::<M>()
            );
        }

        // Material ID is just a marker type for safety
        Ok(MaterialId(PhantomData))
    }

    // Get a MaterialID from a pre-initialized pipeline
    pub fn get<M: Material>(&self) -> Option<MaterialId<M>> {
        let key = TypeId::of::<M>();
        self.pipelines.get(&key).map(|_| MaterialId(PhantomData))
    }

    // Extract the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn DynPipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }
}
