use arrayvec::ArrayVec;
use itertools::Itertools;

use crate::{
    GpuPodRelaxed, ReflectedShader, SetFieldError, ValueFiller,
};
use std::{marker::PhantomData, sync::Arc};

// Push constants are tiny bits of memory that are going to get stored directly in a command encoder
// They are mostly used to upload bits of data very rapidly to use within shaders
pub struct PushConstants<'a> {
    pub(crate) reflected: Arc<ReflectedShader>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) data: Vec<Vec<u8>>,
    pub(crate) stages: Vec<wgpu::ShaderStages>,
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl ValueFiller for PushConstants<'_> {
    // Set the value of a push constant field
    fn set<'s, T: GpuPodRelaxed>(
        &mut self,
        name: &'s str,
        value: T,
    ) -> Result<(), SetFieldError<'s>> {
        // Get shader and it's reflected data
        let reflected = &self.reflected;

        // Check the shader's push constant layouts if they contain the field
        let valid = reflected
            .push_constant_layouts
            .iter()
            .filter_map(|range| range.as_ref())
            .filter_map(|range| {
                // Check if the push constant range layout contains the field
                let member = range
                    .members
                    .iter()
                    .find(|members| &members.name == name);
                member.map(|m| (m, range.stages))
            });

        // Collect into an array instead of a vector
        let array = valid.collect::<ArrayVec<_, 2>>();

        // Check if we can exit early (in case the field doesn't exist)
        if array.is_empty() {
            return Err(SetFieldError::MissingField { name });
        }

        // Convert the data to a byte slice
        let value = [value];
        let bytes = bytemuck::cast_slice::<T, u8>(&value);

        // There is a possibility that the field is shared
        if array.len() == 2 && array[0].0 == array[1].0 {
            let entry = &array[1].0;
            self.offsets.push(entry.offset);
            self.data.push(bytes.to_vec());
            self.stages.push(wgpu::ShaderStages::VERTEX_FRAGMENT);
        } else {
            for (layout, stage) in array {
                self.offsets.push(layout.offset);
                self.data.push(bytes.to_vec());
                self.stages.push(stage);
            }
        }

        Ok(())
    }
}
