#version 460 core
#include "defaults\shaders\volumetric\volumetric.func.glsl"
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(rgba8, binding = 1) uniform image2D result_tex;
layout(r32f, binding = 0) uniform image2D depth_tex;
layout(location = 2) uniform sampler3D sdf_tex;
layout(location = 3) uniform vec3 camera_pos;
layout(location = 4) uniform mat4 custom_vp_matrix;
layout(location = 5) uniform mat4 projection_matrix;
layout(location = 6) uniform vec2 nf_planes;

void main() {
    // Get the pixel coord
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);    
    vec2 uvs = vec2(pixel_coords.xy) / vec2(gl_NumWorkGroups.xy);
    vec3 pixel_forward = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	vec3 pixel_forward_projection = normalize((inverse(projection_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	VolumetricResult volumetric_result = volumetric(camera_pos, uvs, pixel_forward, pixel_forward_projection, nf_planes, sdf_tex);
    
    vec4 pixel = vec4(volumetric_result.color, 0.0);
    // Write the pixel
    imageStore(result_tex, pixel_coords, pixel);
    imageStore(depth_tex, pixel_coords, vec4(volumetric_result.depth, 0, 0, 0));
}