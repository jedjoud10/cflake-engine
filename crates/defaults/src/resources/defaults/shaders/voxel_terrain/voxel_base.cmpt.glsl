#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
#include_custom {"voxel_include_path"}

const float _CHUNK_SIZE = #constant chunk_size
const float _CSPO = _CHUNK_SIZE + 1; // Chunk size plus one
const float _CSPT = _CHUNK_SIZE + 1; // Chunk size plus two
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 2) uniform atomic_uint positive_counter;
layout(binding = 2) uniform atomic_uint negative_counter;
layout(std430, binding = 3) buffer buffer_data
{   
    Voxel voxels[_CSPO][_CSPO][_CSPO];
    FinalVoxel final_voxels[_CSPO][_CSPO][_CSPO];
    float densities[_CSPT][_CSPT][_CSPT]
};
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(_CSPT) - 2.0);
    pos *= size;
    pos += node_pos;       
    // Check if we can actually do calculations or not
    if (all(lessThan(pixel_coords, ivec3(33, 33, 33)))) {        
        // Create the density value
        float density = 0.0;
        Voxel voxel = default_voxel();
        get_density(pos, density, voxel);   
        buffer_data

        // Write the voxel pixel
        vec4 pixel = vec4(density, 0.0, 0.0, 0.0);        
        imageStore(density_image, pixel_coords, pixel);   
        
        // Atomic counter moment    
        if (base.density <= 0.0) {
            atomicCounterIncrement(negative_counter);
        } else {
            atomicCounterIncrement(positive_counter);
        }
    }
}