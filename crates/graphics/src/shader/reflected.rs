use crate::ShaderModule;
use ahash::AHashMap;
use spirv_reflect::types::ReflectTypeFlags;
use std::marker::PhantomData;

// This is the reflected SPIRV data from the shader
pub struct Reflected {
    push_constant: Option<PushConstantBlock>,
}

impl Reflected {
    // Fetch the appropriate unit variable type from a ReflectBlockVariable
    fn reflect_variable_type(member: &spirv_reflect::types::ReflectBlockVariable) -> Option<VariableType> {
        if member.members.len() > 0 {
            return None;
        }
        let _type = member.type_description.as_ref().unwrap().type_flags;

        let value = if _type.contains(ReflectTypeFlags::BOOL) {
            VariableType::Bool
        } else if _type.contains(ReflectTypeFlags::FLOAT) {
            VariableType::Float {
                size: member.numeric.scalar.width
            }
        } else if _type.contains(ReflectTypeFlags::INT) {
            VariableType::Int {
                size: member.numeric.scalar.width,
                signed: member.numeric.scalar.signedness != 0
            }
        } else {
            panic!()
        };

        Some(value)
    }

    // Convert a ReflectBlockVariable (spirv_reflect) to a ReflectedVariable (Rust)
    fn reflect_variable(member: spirv_reflect::types::ReflectBlockVariable) -> (String, BlockVariable) {
        let name = member.name.clone();
        let _type = Self::reflect_variable_type(&member).unwrap();

        // Only unit variables supported for now 
        let variable = BlockVariable::Unit {
            name: member.name,
            size: member.size,
            offset: member.offset,
            _type,
        };

        dbg!(&variable);

        (name, variable)
    }

    // Create new reflected data from the raw reflected data
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
                    .map(Self::reflect_variable).collect(),
                size: block.size,
                offset: block.offset,
            }
        });

        Self {
            push_constant,
        }
    }

    // Get the reflected push constant for this shader
    pub fn push_constant_block(&self) -> Option<&PushConstantBlock> {
        self.push_constant.as_ref()
    }
}

// Type of a specific pre-defined type variable
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum VariableType {
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
        component: Box<VariableType>,
    },
    Martrix {
        column_count: u32,
        row_count: u32,
        stride: u32,
        component: Box<VariableType>,
    },
}

// A block variable that is fetched from the shader
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BlockVariable {
    // A user mad structure variable
    Structure {
        name: String,
        size: u32,
        offset: u32,
        members: AHashMap<String, BlockVariable>,
    },

    // A default unit variable like a float or int
    Unit {
        name: String,
        size: u32,
        offset: u32,
        _type: VariableType,
    },
}

impl BlockVariable {
    // Get the name of the variable
    pub fn name(&self) -> &str {
        match self {
            BlockVariable::Structure { name, .. } => name,
            BlockVariable::Unit { name, .. } => name,
        }
    }

    // Get the size of the variable
    pub fn size(&self) -> u32 {
        *match self {
            BlockVariable::Structure { size, .. } => size,
            BlockVariable::Unit { size, .. } => size,
        }
    }
    
    // Get the offset of the variable
    pub fn offset(&self) -> u32 {
        *match self {
            BlockVariable::Structure { offset, .. } => offset,
            BlockVariable::Unit { offset, .. } => offset,
        }
    }
}

// A push constant block that is fetched from the shader
#[derive(Clone, PartialEq, Eq)]
pub struct PushConstantBlock {
    pub name: String,
    pub variables: AHashMap<String, BlockVariable>,
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
