use std::sync::Arc;

use ahash::AHashMap;
use enum_as_inner::EnumAsInner;
use parking_lot::Mutex;

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
#[derive(Clone, Debug, EnumAsInner)]
pub enum UpdatedParameter {
    ByteSize(usize),
    Location(usize),
}

// Some settings to help us query shader info
#[derive(Default)]
pub struct ShaderInfoQuerySettings {
    // Collection of resources and some query parameters we want to query for said resources
    pub res: AHashMap<Resource, Vec<QueryParameter>>,
    pub res_all: AHashMap<QueryResource, Vec<QueryParameter>>,
}

impl ShaderInfoQuerySettings {
    // Add a resource that will get it's parameters queried
    pub fn query(&mut self, res: Resource, params: Vec<QueryParameter>) {
        self.res.insert(res, params);
    }
    // Query the parameters for all resources
    pub fn query_all(&mut self, unique_resource: QueryResource, params: Vec<QueryParameter>) {
        self.res_all.insert(unique_resource, params);
    }
}

// Shader info that we queried from the pipeline and that we must return to the calling thread
#[derive(Default, Clone)]
pub struct ShaderInfoRead {
    pub(crate) inner: Arc<Mutex<ShaderInfo>>,
}

impl ShaderInfoRead {
    // Get the updated query parameters of a specific resource
    pub fn get(&self, res: &Resource) -> Option<Vec<UpdatedParameter>> {
        let lock_ = self.inner.lock();
        let lock = &lock_.res;
        lock.get(res).cloned()
    }
    // Get all the updated query parameter
    pub fn get_all(
        &self,
        unique_resource: &QueryResource,
    ) -> Option<Vec<(String, Vec<UpdatedParameter>)>> {
        let lock_ = self.inner.lock();
        let lock = &lock_.res_all;
        lock.get(unique_resource).cloned()
    }
}

// Some shader info that we queried from the pipeline
#[derive(Default, Clone)]
pub(crate) struct ShaderInfo {
    // The updated resources
    pub(crate) res: AHashMap<Resource, Vec<UpdatedParameter>>,
    // The updated unique resources
    pub(crate) res_all: AHashMap<QueryResource, Vec<(String, Vec<UpdatedParameter>)>>,
}

impl ShaderInfo {
    // Get the updated query parameters of a specific resource
    #[allow(dead_code)]
    pub(crate) fn get(&self, res: &Resource) -> Option<&Vec<UpdatedParameter>> {
        self.res.get(res)
    }
    // Get all the updated query parameter
    pub(crate) fn get_all(
        &self,
        unique_resource: &QueryResource,
    ) -> Option<&Vec<(String, Vec<UpdatedParameter>)>> {
        self.res_all.get(unique_resource)
    }
}
