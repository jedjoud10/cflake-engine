#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\shared.func.glsl"

const int _CHUNK_SIZE = #constant chunk_size
const int _CSPO = _CHUNK_SIZE + 1; // Chunk size plus one
const int _CSPT = _CHUNK_SIZE + 2; // Chunk size plus two
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(std430, binding = 3) buffer buffer_data
{   
    Voxel voxels[_CSPT][_CSPT][_CSPT];
    BundledVoxel bundled_voxels[_CSPO][_CSPO][_CSPO];
};
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    ivec3 pc = pixel_coords;

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(_CSPT) - 2.0);
    pos *= size;
    pos += node_pos;       
    // Check if we can actually do calculations or not
    if (all(lessThan(pixel_coords, ivec3(33, 33, 33)))) {        
        // Create the final voxel
        Voxel voxel = voxels[pc.x][pc.y][pc.z];
        FinalVoxel final_voxel = get_final_voxel(pos, voxel);
        BundledVoxel bundled_voxel = BundledVoxel(voxel.density, final_voxel);

        // And store the final voxel inside our array
        bundled_voxels[pc.x][pc.y][pc.z] = bundled_voxel;
    }
}