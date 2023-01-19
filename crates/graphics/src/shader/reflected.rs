use std::marker::PhantomData;
use ahash::AHashMap;
use crate::ShaderModule;



// This is the reflected SPIRV data from the shader
pub struct Reflected<M: ShaderModule> {
    _phantom: PhantomData<M>,
    push_constant: Option<PushConstantBlock>,
}

impl<M: ShaderModule> Reflected<M> {
    // Create new reflected dat from the raw reflected data
    pub unsafe fn from_raw_parts(spirv_reflected: spirv_reflect::ShaderModule) -> Self {
        todo!()
        /*
        let push_constant_blocks = spirv_reflected.enumerate_push_constant_blocks(None).unwrap();
        
        let mut pconst = None;
        if let Some(block) = push_constant_blocks.first() {
            let variables = block.members.iter().map(|x| {
                PushConstantVarLayout {
                    name: x.name.clone(),
                    memory: MemoryLayout {
                        size: x.size,
                        offset: x.offset,
                        absolute_offset: x.absolute_offset,
                    },
                }
            });

            pconst = Some(PushConstantBlockLayout {
                name: block.name.clone(),
                variables: variables.collect(),
                memory: MemoryLayout {
                    size: block.size,
                    offset: block.offset,
                    absolute_offset: block.absolute_offset
                },
            })
        }

        Self {
            _phantom: PhantomData,
            push_constant: pconst,
        }
        */
    }

    /*
    // Get all the reflected descriptor sets for this shader
    pub fn descriptor_sets(&self) -> &AHashMap<u32, ReflectedDescriptorSet> {
        todo!()
    } 
    */

    // Get the reflected push constant for this shader
    pub fn push_constant(&self) -> Option<&PushConstantBlock> {
        todo!()
    }
}

pub enum ReflectedVar<T> {
    DontCare,
    Reflected(T),
    MustMatch(T),
}

// A push constant variable that is fetched from the shader
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PushConstantVariable {
    pub name: String,
    pub _type: (),
    pub size: u32,
    pub offset: u32,
    pub global_offset: ReflectedVar<u32>,
}

// A push constant block that is fetched from the shader
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PushConstantBlock {
    pub name: String,
    pub variables: Vec<PushConstantVariable>,
    pub size: u32,
    pub offset: u32,
    pub global_offset: u32,
}

/*
// This is a descriptor set layout stored within a shader stage
pub struct ReflectedDescriptorSet {
    name: String,
    bindings: AHashMap<u32, ReflectedVar>,
}

impl ReflectedDescriptorSet {
    // Get the name of the descriptor set
    pub fn name(&self) -> &str {
        &self.name
    }

    // Get the bindings of the descriptor set
    pub fn bindings(&self) -> &AHashMap<u32, ReflectedVar> {
        &self.bindings
    }
}
*/