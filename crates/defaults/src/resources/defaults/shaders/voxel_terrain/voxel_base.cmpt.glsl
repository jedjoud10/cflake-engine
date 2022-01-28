#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
// VoxelArbitraryData
#include_custom {"voxel_arbitrary_data_struct"}

const float _CHUNK_SIZE = #constant chunk_size
const float _CHUNK_SIZE_PLUS_ONE = _CHUNK_SIZE + 1;
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) writeonly uniform image3D density_image;
layout(binding = 2) uniform atomic_uint positive_counter;
layout(binding = 2) uniform atomic_uint negative_counter;
layout(std430, binding = 3) buffer arbitrary_data
{   
    VoxelArbitraryData datas[_CHUNK_SIZE_PLUS_ONE][_CHUNK_SIZE_PLUS_ONE][_CHUNK_SIZE_PLUS_ONE];
};
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;
layout(location = 4) uniform int chunk_size;

VoxelArbitraryData get_density_and_data(vec3 pos, out float density) {
    #include_custom {"voxel_base_function"}
}

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;             

    // Create the density value
    float density = 0.0;
    VoxelArbitraryData data = get_density_and_data(pos, density);   
    datas[pixel_coords.x][pixel_coords.y][pixel_coords.z] = data;
    // Write the voxel pixel
    vec4 pixel = vec4(density, 0.0, 0.0, 0.0);        
    imageStore(density_image, pixel_coords, pixel);        

    // Add to the atomic counters
    if (all(lessThan(pixel_coords, ivec3(33, 33, 33)))) {
        if (base.density <= 0.0) {
            atomicCounterIncrement(negative_counter);
        } else {
            atomicCounterIncrement(positive_counter);
        }
    }
}