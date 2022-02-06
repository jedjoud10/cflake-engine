// Some shared code between the normal shaders and the compute shaders

use std::{collections::HashSet, ffi::CString, ptr::null_mut};

use ahash::AHashMap;

use crate::{basics::transfer::Transfer, object::GlTracker, pipeline::Pipeline};

use super::{
    info::{QueryParameter, QueryResource, Resource, ShaderInfo, ShaderInfoQuerySettings, UpdatedParameter},
    IncludeExpansionError, ShaderSettings,
};

// Load the files that need to be included for this specific shader and return the included lines
pub(crate) fn load_includes(settings: &ShaderSettings, source: &mut String, included_paths: &mut HashSet<String>) -> Result<bool, IncludeExpansionError> {
    // Turn the string into lines
    let mut lines = source.lines().into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
    for (_i, line) in lines.iter_mut().enumerate() {
        // Check if this is an include statement
        if line.starts_with("#include ") {
            // Get the local path of the include file
            let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace('"', "");
            let local_path = local_path.trim_start();

            // Load the include function text
            let text = if !included_paths.contains(&local_path.to_string()) {
                // Load the function shader text
                included_paths.insert(local_path.to_string());
                assets::assetc::load_text(local_path)
                    .map_err(|_| IncludeExpansionError::new(format!("Tried to include function shader '{}' and it was not pre-loaded!.", local_path)))?
            } else {
                String::new()
            };

            // Update the original line
            *line = text;
            break;
        }
        // External shader code
        if line.trim().starts_with("#include_custom ") {
            // Get the source
            let c = line.split("#include_custom ").collect::<Vec<&str>>()[1];
            let source_name = &c[2..(c.len() - 2)].to_string();
            let source = settings
                .external_code
                .get(source_name)
                .unwrap_or_else(|| panic!("Tried to expand #include_custom, but the given source name '{}' is not valid!", source_name));
            *line = source.clone();
            break;
        }
        // Impl default types
        if line.trim().starts_with("#load") {
            let x = match line.split("#load ").collect::<Vec<&str>>()[1] {
                // Refactor this
                "renderer" => {
                    *line = "#include defaults\\shaders\\others\\default_impls\\renderer.func.glsl".to_string();
                    Ok(())
                }
                "general" => {
                    *line = "#include defaults\\shaders\\others\\default_impls\\general.func.glsl".to_string();
                    Ok(())
                }
                x => Err(IncludeExpansionError::new(format!("Tried to expand #load, but the given type '{}' is not valid!", x))),
            };
            x?;
            break;
        }
        // Constants
        if line.trim().contains("#constant ") {
            fn format(line: &String, val: &String) -> String {
                format!("{}{};", line.trim().split("#constant").next().unwrap(), val)
            }
            let const_name = line.split("#constant ").collect::<Vec<&str>>()[1];
            let x = settings.consts.get(const_name);
            if let Some(x) = x {
                *line = format(line, x);
                Ok(())
            } else {
                Err(IncludeExpansionError::new(format!(
                    "Tried to expand #constant, but the given const name '{}' is not valid!",
                    const_name
                )))
            }?;
            break;
        }
    }
    // Update the source
    *source = lines.join("\n");
    // Check if we need to continue expanding the includes
    let need_to_continue = lines
        .iter()
        .any(|x| x.trim().starts_with("#include ") || x.trim().starts_with("#include_custom ") || x.trim().starts_with("#load ") || x.trim().contains("#constant "));
    Ok(need_to_continue)
}

// Query some information about a shader
pub(crate) fn query_shader_info(pipeline: &Pipeline, oid: u32, settings: ShaderInfoQuerySettings, read: Transfer<ShaderInfo>) -> GlTracker {
    GlTracker::fake(
        move |_| unsafe {
            // Get the query info

            // Gotta count the number of unique resource types
            let mut unique_count = AHashMap::<QueryResource, usize>::new();
            let mut indexed_resources = AHashMap::<Resource, (Vec<QueryParameter>, usize)>::new();
            for (x, parameters) in settings.res.iter() {
                let count = unique_count.entry(x.res.clone()).or_default();
                indexed_resources.insert(x.clone(), (parameters.clone(), *count));
                *count += 1;
            }

            // First we gotta get how many resources of a single type we have, and their respective max name len
            let _types_and_counts = unique_count
                .iter()
                .map(|(res, _)| {
                    let mut max_resources = 0_i32;
                    let mut max_name_len = 0_i32;
                    gl::GetProgramInterfaceiv(oid, res.convert(), gl::ACTIVE_RESOURCES, &mut max_resources);
                    gl::GetProgramInterfaceiv(oid, res.convert(), gl::MAX_NAME_LENGTH, &mut max_name_len);
                    (res.clone(), (max_resources, max_name_len as usize))
                })
                .collect::<AHashMap<_, _>>();

            // Now we can actually query the parameters
            let mut output_queried_resources = AHashMap::<Resource, Vec<UpdatedParameter>>::new();
            for (res, (parameters, _i)) in indexed_resources {
                let cstring = CString::new(res.name.clone()).unwrap();
                // Get the resource's index
                let resource_index = gl::GetProgramResourceIndex(oid, res.convert(), cstring.as_ptr());
                if resource_index == gl::INVALID_INDEX {
                    panic!()
                }

                // Now we can finally access the resource's parameters
                let converted_params = parameters.iter().map(|x| x.convert()).collect::<Vec<_>>();
                let max_len = converted_params.len();
                let mut output = vec![-1; max_len];
                gl::GetProgramResourceiv(
                    oid,
                    res.convert(),
                    resource_index,
                    max_len as i32,
                    converted_params.as_ptr(),
                    output.len() as i32,
                    null_mut(),
                    output.as_mut_ptr(),
                );

                // Check for negative numbers, because if we fine some, that means that we failed to retrieve a specific parameter
                for maybe in output.iter() {
                    if *maybe == -1 {
                        panic!()
                    }
                }

                let converted_outputs = parameters
                    .iter()
                    .zip(output)
                    .map(|(x, opengl_val)| x.convert_output(opengl_val))
                    .collect::<Vec<UpdatedParameter>>();

                // After reading everything, we can add convert the output values into their respective Rust enums and store them
                output_queried_resources.insert(res, converted_outputs);
            }

            // Finally update the mutex that holds the queried resources
            let mut lock = read.0.res.lock().unwrap();
            *lock = output_queried_resources;
        },
        pipeline,
    )
}
