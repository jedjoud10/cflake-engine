#version 460 core
#include_custom {"voxel_include_path"}
#include "defaults\shaders\voxel_terrain\shared.func.glsl"

const int _CHUNK_SIZE = #constant chunk_size
const int _CSPO = _CHUNK_SIZE + 1; // Chunk size plus one
const int _CSPT = _CHUNK_SIZE + 2; // Chunk size plus two
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) uniform atomic_uint positive_counter;
layout(binding = 0) uniform atomic_uint negative_counter;
layout(std140, binding = 1) writeonly coherent restrict buffer arbitrary_voxels
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
    float size = float(node_size) / (float(_CHUNK_SIZE));
    pos *= size;
    pos += node_pos;
    // Check if we can actually do calculations or not
    if (all(lessThan(pixel_coords, ivec3(_CSPT, _CSPT, _CSPT)))) {        
        // Create the density value
        Voxel voxel = get_voxel(pos);

        // And store the voxel inside our array
        voxels[flatten(pc, _CSPT)] = voxel;

        // Atomic counter moment    
        if (voxel.density <= 0.0) {
            atomicCounterIncrement(negative_counter);
        } else {
            atomicCounterIncrement(positive_counter);
        }
    }
}