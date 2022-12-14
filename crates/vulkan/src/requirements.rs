use std::ffi::CString;
use ash::vk::{
    self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
use vk::PhysicalDeviceType;

// Check wether or not a physical device is suitable for rendering
// This checks the minimum requirements that we need to achieve to be able to render
pub(super) fn is_physical_device_suitable(
    name: &str,
    _type: vk::PhysicalDeviceType,
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    features: vk::PhysicalDeviceFeatures,
    modes: &[vk::PresentModeKHR],
    formats: &[vk::SurfaceFormatKHR]
) -> bool {
    log::debug!(
        "Checking if adapter {} is suitable...",
        name
    );

    // Check all the requirements that are needed for us to use this Adapter
    let double_buffering_supported =
        is_double_buffering_supported(surface_capabilities);
    let format_supported = is_surface_format_supported(formats);
    let features_supported = is_feature_list_supported(features);
    let present_supported = is_present_mode_supported(modes);
    let device_type_okay = is_device_type_optimal(_type);

    // All the checks must pass
    double_buffering_supported
        && features_supported
        && format_supported
        && present_supported
        && device_type_okay
}

// Check if the Adapter is optimal (dGPU)
fn is_device_type_optimal(_type: PhysicalDeviceType) -> bool {
    let device_type_okay = _type == PhysicalDeviceType::DISCRETE_GPU;
    log::debug!("Adapter Device Type: {:?}", _type);
    device_type_okay
}

// Check if the Adapter supports the given features
fn is_feature_list_supported(given: vk::PhysicalDeviceFeatures) -> bool {
    // Convert a vk::PhysicalDeviceFeatures to some bitset
    fn convert(value: vk::PhysicalDeviceFeatures) -> u64 {
        value.robust_buffer_access as u64 * 1
        | value.full_draw_index_uint32 as u64 * 2
        | value.image_cube_array as u64 * 4
        | value.independent_blend as u64 * 8
        | value.geometry_shader as u64 * 16
        | value.tessellation_shader as u64 * 32
        | value.sample_rate_shading as u64 * 64
        | value.dual_src_blend as u64 * 128
        | value.logic_op as u64 * 256
        | value.multi_draw_indirect as u64 * 512
        | value.draw_indirect_first_instance as u64 * 1024
        | value.depth_clamp as u64 * 2048
        | value.depth_bias_clamp as u64 * 4096
        | value.fill_mode_non_solid as u64 * 8192
        | value.depth_bounds as u64 * 16384
        | value.wide_lines as u64 * 32768
        | value.large_points as u64 * 65536
        | value.alpha_to_one as u64 * 131072
        | value.multi_viewport as u64 * 262144
        | value.sampler_anisotropy as u64 * 524288
        | value.texture_compression_etc2 as u64 * 1048576
        | value.texture_compression_astc_ldr as u64 * 2097152
        | value.texture_compression_bc as u64 * 4194304
        | value.occlusion_query_precise as u64 * 8388608
        | value.pipeline_statistics_query as u64 * 16777216
        | value.vertex_pipeline_stores_and_atomics as u64 * 33554432
        | value.fragment_stores_and_atomics as u64 * 67108864
        | value.shader_tessellation_and_geometry_point_size as u64 * 134217728
        | value.shader_image_gather_extended as u64 * 268435456
        | value.shader_storage_image_extended_formats as u64 * 536870912
        | value.shader_storage_image_multisample as u64 * 1073741824
        | value.shader_storage_image_read_without_format as u64 * 2147483648
        | value.shader_storage_image_write_without_format as u64 * 4294967296
        | value.shader_uniform_buffer_array_dynamic_indexing as u64 * 8589934592
        | value.shader_sampled_image_array_dynamic_indexing as u64 * 17179869184
        | value.shader_storage_buffer_array_dynamic_indexing as u64 * 34359738368
        | value.shader_storage_image_array_dynamic_indexing as u64 * 68719476736
        | value.shader_clip_distance as u64 * 137438953472
        | value.shader_cull_distance as u64 * 274877906944
        | value.shader_float64 as u64 * 549755813888
        | value.shader_int64 as u64 * 1099511627776
        | value.shader_int16 as u64 * 2199023255552
        | value.shader_resource_residency as u64 * 4398046511104
        | value.shader_resource_min_lod as u64 * 8796093022208
        | value.sparse_binding as u64 * 17592186044416
        | value.sparse_residency_buffer as u64 * 35184372088832
        | value.sparse_residency_image2_d as u64 * 70368744177664
        | value.sparse_residency_image3_d as u64 * 140737488355328
        | value.sparse_residency2_samples as u64 * 281474976710656
        | value.sparse_residency4_samples as u64 * 562949953421312
        | value.sparse_residency8_samples as u64 * 1125899906842624
        | value.sparse_residency16_samples as u64 * 2251799813685248
        | value.sparse_residency_aliased as u64 * 4503599627370496
        | value.variable_multisample_rate as u64 * 9007199254740992
        | value.inherited_queries as u64 * 18014398509481984
    }

    let required = convert(required_features());
    println!("{:b}", required);
    let given = convert(given);
    println!("{:b}", given);

    let supported = required & given == required; 
    log::debug!("Adapter Supports Required Features: {supported}");
    supported
}

// Check if the Adapter supports a min image count of 2
fn is_double_buffering_supported(
    surface: SurfaceCapabilitiesKHR,
) -> bool {
    let double_buffering_supported = surface.min_image_count == 2;
    log::debug!(
        "Adapter Double Buffering: {}",
        double_buffering_supported
    );
    double_buffering_supported
}

// Check if the Adapter present modes support FIFO_RELAXED and IMMEDIATE
fn is_present_mode_supported(modes: &[PresentModeKHR]) -> bool {
    let present_supported = modes
        .iter()
        .find(|&&present| {
            let relaxed = present == vk::PresentModeKHR::FIFO_RELAXED;
            let immediate = present == vk::PresentModeKHR::IMMEDIATE;
            relaxed || immediate
        })
        .is_some();

    present_supported
}

// Check if the Adapter formats supportB8G8R8A8_SRGB and SRGB_NONLINEAR
fn is_surface_format_supported(formats: &[SurfaceFormatKHR]) -> bool {
    let format_supported = formats
        .iter()
        .find(|format| {
            let format_ = format.format == vk::Format::B8G8R8A8_SRGB;
            let color_space_ = format.color_space
                == vk::ColorSpaceKHR::SRGB_NONLINEAR;
            format_ && color_space_
        })
        .is_some();
    log::debug!(
        "Adapter Swapchain Format Supported: {}",
        format_supported
    );
    format_supported
}

// Get the required validation layers
pub fn required_validation_layers() -> Vec<CString> {
    #[cfg(debug_assertions)]
    return vec![CString::new("VK_LAYER_KHRONOS_validation".to_owned())
        .unwrap()];

    #[cfg(not(debug_assertions))]
    return vec![];
}

// Get the required instance extensions
pub fn required_instance_extensions() -> Vec<CString> {
    vec![
        ash::extensions::ext::DebugUtils::name().to_owned(),
        ash::extensions::khr::Surface::name().to_owned(),
    ]
}
    
// Get the reqwuired logical device extensions
pub fn required_device_extensions() -> Vec<CString> {
    vec![
        ash::extensions::khr::Swapchain::name().to_owned(),
        ash::extensions::khr::Synchronization2::name().to_owned(),
    ]
}

// Get the features that we will use for the device
pub fn required_features() -> vk::PhysicalDeviceFeatures {
    *vk::PhysicalDeviceFeatures::builder()
        .tessellation_shader(true)
        .multi_draw_indirect(true)
        .draw_indirect_first_instance(true)
        .sampler_anisotropy(true)
        .shader_float64(true)
        .shader_int16(true)
        .shader_int64(true)
}

// The required Vulkan API version
pub fn required_api_version() -> u32 {
    vk::API_VERSION_1_3
}