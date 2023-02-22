use std::{marker::PhantomData, sync::Arc};
use crate::{ReflectedShader, ValueFiller, FillError, GpuPodRelaxed};

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
    fn set<'s, T: GpuPodRelaxed>(&mut self, name: &'s str, value: T) -> Result<(), FillError<'s>> {
        // Get shader and it's reflected data
        let reflected = &self.reflected;

        // Check the shader's push constant layouts if they contain the field
        let valid = reflected
            .push_constant_layouts
            .iter()
            .filter_map(|range| range.as_ref())
            .filter_map(|range| {
                // Check if the push constant range layout contains the field
                let member = range.members.iter().find(|members| {
                    &members.name == name
                });

                member.map(|m| (m, range.stages))
            });

        // Get the modules that contain the field
        let valid = valid.collect::<Vec<_>>();

        // Return early if we don't have a field to set
        if valid.is_empty() {
            return Err(FillError::MissingField { name: name });
        };

        // Conver the data to a byte slice
        let value = [value];
        let bytes = bytemuck::cast_slice::<T, u8>(&value);

        // There is a possibility that the field is shared
        if valid.len() == 2 && valid[0].0 == valid[1].0 {
            let entry = &valid[1].0;  
            self.offsets.push(entry.offset);
            self.data.push(bytes.to_vec());
            self.stages.push(wgpu::ShaderStages::VERTEX_FRAGMENT);
        } else {
            // Set the field separately
            for (layout, stage) in valid {
                // Return an error if the layout doesn't match the given data
                if layout.size as usize != bytes.len() {
                    return Err(FillError::WrongSize);
                }

                self.offsets.push(layout.offset);
                self.data.push(bytes.to_vec());
                self.stages.push(stage);
            }
        }

        Ok(())
    }
}