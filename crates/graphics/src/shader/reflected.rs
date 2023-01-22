use crate::ShaderModule;
use ahash::AHashMap;
use std::marker::PhantomData;

// This is the reflected SPIRV data from the shader
pub struct Reflected<M: ShaderModule> {
    _phantom: PhantomData<M>,
    push_constant: Option<PushConstantBlock>,
}

impl<M: ShaderModule> Reflected<M> {
    // Create new reflected dat from the raw reflected data
    pub unsafe fn from_raw_parts(
        spirv_reflected: spirv_reflect::ShaderModule,
    ) -> Self {
        // Fetch the push constant block from the spirv data
        let mut push_constant_blocks = spirv_reflected.enumerate_push_constant_blocks(None).unwrap();
        assert!(push_constant_blocks.len() <= 1);
        let push_constant = push_constant_blocks
            .pop()
            .map(|block| {
            PushConstantBlock {
                name: block.name,
                variables: block
                    .members
                    .into_iter()
                    .map(|member| {
                        PushConstantVariable::Unit {
                            name: member.name,
                            size: member.size,
                            offset: member.offset,
                            _type: UnitVariableType::Bool,
                        }
                    }).collect(),
                size: block.size,
                offset: block.offset,
            }
        });

        Self {
            _phantom: PhantomData,
            push_constant,
        }
    }

    // Get the reflected push constant for this shader
    pub fn push_constant_block(&self) -> Option<&PushConstantBlock> {
        todo!()
    }

    // Check if a givne push constant template is used within this reflected data
    pub fn contains_push_constant_block_template(
        &self,
        template: PushConstantBlock,
    ) -> bool {
        todo!()
    }
}

// Type of a specific pre-defined type variable
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum UnitVariableType {
    Bool,
    Int {
        size: u32,
        signed: bool,
    },
    Float {
        size: u32,
    },
    Vector {
        components_count: u32,
        component: Box<UnitVariableType>,
    },
    Martrix {
        column_count: u32,
        row_count: u32,
        stride: u32,
        component: Box<UnitVariableType>,
    },
}

// A push constant variable that is fetched from the shader
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PushConstantVariable {
    // A user mad structure variable
    Structure {
        name: String,
        size: u32,
        offset: u32,
        members: Vec<PushConstantVariable>,
    },

    // A default unit variable like a float or int
    Unit {
        name: String,
        size: u32,
        offset: u32,
        _type: UnitVariableType,
    },
}

// A push constant block that is fetched from the shader
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PushConstantBlock {
    pub name: String,
    pub variables: Vec<PushConstantVariable>,
    pub size: u32,
    pub offset: u32,
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
