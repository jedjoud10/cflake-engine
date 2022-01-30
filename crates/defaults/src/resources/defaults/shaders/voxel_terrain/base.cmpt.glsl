#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
#include_custom {"voxel_include_path"}

const int _CHUNK_SIZE = #constant chunk_size
const int _CSPO = _CHUNK_SIZE + 1; // Chunk size plus one
const int _CSPT = _CHUNK_SIZE + 2; // Chunk size plus two
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) uniform atomic_uint positive_counter;
layout(binding = 0) uniform atomic_uint negative_counter;
layout(std430, binding = 1) writeonly buffer arbitrary_voxels
{   
    Voxel voxels[_CSPT][_CSPT][_CSPT];
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
    if (all(lessThan(pixel_coords, ivec3(_CSPO, _CSPO, _CSPO)))) {        
        // Create the density value
        Voxel voxel = get_voxel(pos);

        // And store the voxel inside our array
        voxels[pc.x][pc.y][pc.z] = voxel;

        // Atomic counter moment    
        if (voxel.density <= 0.0) {
            atomicCounterIncrement(negative_counter);
        } else {
            atomicCounterIncrement(positive_counter);
        }
    }
}