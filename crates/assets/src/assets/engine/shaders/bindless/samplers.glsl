#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_nonuniform_qualifier : require

layout(set = 0, binding = 0) uniform sampler2D global_textures[];
layout(set = 0, binding = 0) uniform usampler2D global_textures_uint[];
layout(set = 0, binding = 0) uniform sampler3D global_textures_3d[];
layout(set = 0, binding = 0) uniform usampler3D global_textures_3d_uint[];