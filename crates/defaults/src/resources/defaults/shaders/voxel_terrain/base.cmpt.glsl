#version 460 core
#include_custom {"voxel_include_path"}
#include "defaults\shaders\voxel_terrain\shared.func.glsl"

const int CHUNK_SIZE = #constant chunk_size
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(std430, binding = 1) writeonly buffer arbitrary_voxels
{   
    Voxel voxels[];
};
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    ivec3 pc = pixel_coords;

    // Get the position
    vec3 pos = vec3(pixel_coords.xyz);    
    float size = float(node_size) / (float(CHUNK_SIZE));
    pos *= size;
    pos += node_pos;
    // Check if we can actually do calculations or not
    if (all(lessThan(pixel_coords, ivec3(CHUNK_SIZE+2, CHUNK_SIZE+2, CHUNK_SIZE+2)))) {        
        // Create the density value
        Voxel voxel = get_voxel(uvec3(pc), pos);

        // And store the voxel inside our array
        voxels[flatten(pc, CHUNK_SIZE+2)] = voxel;
    }
}