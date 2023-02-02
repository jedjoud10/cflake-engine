
pub struct Features {
    pub robust_buffer_access: bool,
    pub full_draw_index_uint32: bool,
    pub image_cube_array: bool,
    pub independent_blend: bool,
    pub geometry_shader: bool,
    pub tessellation_shader: bool,
    pub sample_rate_shading: bool,
    pub dual_src_blend: bool,
    pub logic_op: bool,
    pub multi_draw_indirect: bool,
    pub draw_indirect_first_instance: bool,
    pub depth_clamp: bool,
    pub depth_bias_clamp: bool,
    pub fill_mode_non_solid: bool,
    pub depth_bounds: bool,
    pub wide_lines: bool,
    pub large_points: bool,
    pub alpha_to_one: bool,
    pub multi_viewport: bool,
    pub sampler_anisotropy: bool,
    pub texture_compression_etc2: bool,
    pub texture_compression_astc_ldr: bool,
    pub texture_compression_bc: bool,
    pub occlusion_query_precise: bool,
    pub pipeline_statistics_query: bool,
    pub vertex_pipeline_stores_and_atomics: bool,
    pub fragment_stores_and_atomics: bool,
    pub shader_tessellation_and_geometry_point_size: bool,
    pub shader_image_gather_extended: bool,
    pub shader_storage_image_extended_formats: bool,
    pub shader_storage_image_multisample: bool,
    pub shader_storage_image_read_without_format: bool,
    pub shader_storage_image_write_without_format: bool,
    pub shader_uniform_buffer_array_dynamic_indexing: bool,
    pub shader_sampled_image_array_dynamic_indexing: bool,
    pub shader_storage_buffer_array_dynamic_indexing: bool,
    pub shader_storage_image_array_dynamic_indexing: bool,
    pub shader_clip_distance: bool,
    pub shader_cull_distance: bool,
    pub shader_float64: bool,
    pub shader_int64: bool,
    pub shader_int16: bool,
    pub shader_resource_residency: bool,
    pub shader_resource_min_lod: bool,
    pub sparse_binding: bool,
    pub sparse_residency_buffer: bool,
    pub sparse_residency_image2_d: bool,
    pub sparse_residency_image3_d: bool,
    pub sparse_residency2_samples: bool,
    pub sparse_residency4_samples: bool,
    pub sparse_residency8_samples: bool,
    pub sparse_residency16_samples: bool,
    pub sparse_residency_aliased: bool,
    pub variable_multisample_rate: bool,
    pub inherited_queries: bool,

    pub storage_buffer16_bit_access: bool,
    pub uniform_and_storage_buffer16_bit_access: bool,
    pub storage_push_constant16: bool,
    pub storage_input_output16: bool,
    pub multiview: bool,
    pub multiview_geometry_shader: bool,
    pub multiview_tessellation_shader: bool,
    pub variable_pointers_storage_buffer: bool,
    pub variable_pointers: bool,
    pub protected_memory: bool,
    pub sampler_ycbcr_conversion: bool,
    pub shader_draw_parameters: bool,

    pub sampler_mirror_clamp_to_edge: bool,
    pub draw_indirect_count: bool,
    pub storage_buffer8_bit_access: bool,
    pub uniform_and_storage_buffer8_bit_access: bool,
    pub storage_push_constant8: bool,
    pub shader_buffer_int64_atomics: bool,
    pub shader_shared_int64_atomics: bool,
    pub shader_float16: bool,
    pub shader_int8: bool,
    pub descriptor_indexing: bool,
    pub shader_input_attachment_array_dynamic_indexing: bool,
    pub shader_uniform_texel_buffer_array_dynamic_indexing: bool,
    pub shader_storage_texel_buffer_array_dynamic_indexing: bool,
    pub shader_uniform_buffer_array_non_uniform_indexing: bool,
    pub shader_sampled_image_array_non_uniform_indexing: bool,
    pub shader_storage_buffer_array_non_uniform_indexing: bool,
    pub shader_storage_image_array_non_uniform_indexing: bool,
    pub shader_input_attachment_array_non_uniform_indexing: bool,
    pub shader_uniform_texel_buffer_array_non_uniform_indexing: bool,
    pub shader_storage_texel_buffer_array_non_uniform_indexing: bool,
    pub descriptor_binding_uniform_buffer_update_after_bind: bool,
    pub descriptor_binding_sampled_image_update_after_bind: bool,
    pub descriptor_binding_storage_image_update_after_bind: bool,
    pub descriptor_binding_storage_buffer_update_after_bind: bool,
    pub descriptor_binding_uniform_texel_buffer_update_after_bind: bool,
    pub descriptor_binding_storage_texel_buffer_update_after_bind: bool,
    pub descriptor_binding_update_unused_while_pending: bool,
    pub descriptor_binding_partially_bound: bool,
    pub descriptor_binding_variable_descriptor_count: bool,
    pub runtime_descriptor_array: bool,
    pub sampler_filter_minmax: bool,
    pub scalar_block_layout: bool,
    pub imageless_framebuffer: bool,
    pub uniform_buffer_standard_layout: bool,
    pub shader_subgroup_extended_types: bool,
    pub separate_depth_stencil_layouts: bool,
    pub host_query_reset: bool,
    pub timeline_semaphore: bool,
    pub buffer_device_address: bool,
    pub buffer_device_address_capture_replay: bool,
    pub buffer_device_address_multi_device: bool,
    pub vulkan_memory_model: bool,
    pub vulkan_memory_model_device_scope: bool,
    pub vulkan_memory_model_availability_visibility_chains: bool,
    pub shader_output_viewport_index: bool,
    pub shader_output_layer: bool,
    pub subgroup_broadcast_dynamic_id: bool,

    pub robust_image_access: bool,
    pub inline_uniform_block: bool,
    pub descriptor_binding_inline_uniform_block_update_after_bind: bool,
    pub pipeline_creation_cache_control: bool,
    pub private_data: bool,
    pub shader_demote_to_helper_invocation: bool,
    pub shader_terminate_invocation: bool,
    pub subgroup_size_control: bool,
    pub compute_full_subgroups: bool,
    pub synchronization2: bool,
    pub texture_compression_astc_hdr: bool,
    pub shader_zero_initialize_workgroup_memory: bool,
    pub dynamic_rendering: bool,
    pub shader_integer_dot_product: bool,
    pub maintenance4: bool,

    pub robust_buffer_access2: bool,
    pub robust_image_access2: bool,
    pub null_descriptor: bool,
}


// Get the features that we will use for the device
pub fn required_features() -> Features {
    let features = *vk::PhysicalDeviceFeatures::builder()
        .tessellation_shader(true)
        .multi_draw_indirect(true)
        .draw_indirect_first_instance(true)
        .sampler_anisotropy(true)
        .shader_float64(true)
        .robust_buffer_access(true)
        .shader_sampled_image_array_dynamic_indexing(true)
        .shader_storage_image_array_dynamic_indexing(true)
        .shader_storage_buffer_array_dynamic_indexing(true)
        .shader_uniform_buffer_array_dynamic_indexing(true)
        .shader_int64(true);

    let features11 = *vk::PhysicalDeviceVulkan11Features::builder();

    let features12 = *vk::PhysicalDeviceVulkan12Features::builder()
        .imageless_framebuffer(true)
        .buffer_device_address(true)
        .draw_indirect_count(true)
        .timeline_semaphore(true)
        .buffer_device_address_capture_replay(true)
        .descriptor_binding_partially_bound(true)
        .runtime_descriptor_array(true)
        .descriptor_indexing(true);

    let robustness12 = *vk::PhysicalDeviceRobustness2FeaturesEXT::builder()
        .null_descriptor(true);

    let features13 = *vk::PhysicalDeviceVulkan13Features::builder()
        .dynamic_rendering(true)
        .robust_image_access(true)
        .synchronization2(true);

    AdapterFeatures {
        features,
        features11,
        features12,
        features13,
        robustness12,
    }
}


// Check if the features from an Adapter are valid for required features
pub fn supports_required_features() -> bool {
    
} 