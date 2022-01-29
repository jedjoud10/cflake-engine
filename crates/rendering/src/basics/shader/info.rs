use std::sync::{Arc, Mutex};

use ahash::AHashMap;

use crate::basics::transfer::{Transfer, Transferable};

// Resource
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum QueryResource {
    ShaderStorageBlock,
    Uniform,
}

impl QueryResource {
    // Convert to the OpenGL enum
    pub fn convert(&self) -> u32 {
        match self {
            QueryResource::ShaderStorageBlock => gl::SHADER_STORAGE_BLOCK,
            QueryResource::Uniform => gl::UNIFORM,
        }
    }
}

// A resource and it's unique resource name
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Resource {
    pub res: QueryResource,
    pub name: String,
}

impl Resource {
    // Convert to the OpenGL enum
    pub fn convert(&self) -> u32 {
        self.res.convert()
    }
}

// Parameters we want to query
#[derive(Clone, Debug)]
pub enum QueryParameter {
    ByteSize,
    Location,
}

impl QueryParameter {
    // Convert to the OpenGL enum
    pub fn convert(&self) -> u32 {
        match self {
            QueryParameter::ByteSize => gl::BUFFER_DATA_SIZE,
            QueryParameter::Location => gl::LOCATION,
        }
    }
    // Convert an OpenGL return value to an UpdatedParameter enum
    pub fn convert_output(&self, opengl_val: i32) -> UpdatedParameter {
        match self {
            QueryParameter::ByteSize => UpdatedParameter::ByteSize(opengl_val as usize),
            QueryParameter::Location => UpdatedParameter::Location(opengl_val as usize),
        }
    }
}

// And their updated counter part
#[derive(Clone, Debug)]
pub enum UpdatedParameter {
    ByteSize(usize),
    Location(usize),
}

// Some settings to help us query shader info
#[derive(Default)]
pub struct ShaderInfoQuerySettings {
    // Collection of resources and some query parameters we want to query for said resources
    pub res: AHashMap<Resource, Vec<QueryParameter>>,
}

impl ShaderInfoQuerySettings {
    // Add a resource that will get it's parameters queried
    pub fn query(&mut self, res: Resource, params: Vec<QueryParameter>) {
        self.res.insert(res, params);
    }
}

// Some shader info that we queried from the pipeline
#[derive(Default, Clone)]
pub struct ShaderInfo {
    // The updated resources
    pub res: Arc<Mutex<AHashMap<Resource, Vec<UpdatedParameter>>>>,
}

impl ShaderInfo {
    // Get the updated query parameters of a specific resource
    pub fn get(&self, res: &Resource) -> Option<Vec<UpdatedParameter>> {
        let lock = self.res.lock().ok()?;
        lock.get(res).map(|x| x.clone())
    }
}

impl Transferable for ShaderInfo {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(self.clone())
    }
}
