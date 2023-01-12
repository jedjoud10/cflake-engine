#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_nonuniform_qualifier : require

// Images bindings
layout(set = 2, binding = 1, rgba8) uniform image2D global_images_2d_rgba8[];
layout(set = 2, binding = 1, rgba16f) uniform image2D global_images_2d_rgba16f[];
layout(set = 2, binding = 1, rgba32f) uniform image2D global_images_2d_rgba32f[];
layout(set = 2, binding = 1, r32f) uniform image2D global_images_2d_r32f[];

// Textures samplers bindings
layout(set = 2, binding = 0) uniform sampler2D global_textures[];
layout(set = 2, binding = 0) uniform usampler2D global_textures_uint[];
layout(set = 2, binding = 0) uniform sampler3D global_textures_3d[];
layout(set = 2, binding = 0) uniform usampler3D global_textures_3d_uint[];