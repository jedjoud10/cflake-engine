// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    uint global_chunk_index;
    uint local_allocation_index;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>
    
// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz;
    
    //position *= (position * 0.0045) + 1 + sin(position.x * 0.1) * 0.1;

    float density1 = (1-fbmCellular(position * 0.01 * vec3(1, 2, 1), 20, 0.4, 2.0).y) * 20;
    float density2 = opSmoothUnion(-erosion(position.xz * 0.04, 0.112).x * 420 + position.y + 200, position.y, 40) + 5;

    //return position.y; 
    return mix(density1, density2, clamp(snoise(position.xz * 0.003) * 0.5 + 0.5, 0, 1)) + position.y;
}

// TODO: Implement voxel struct and color function