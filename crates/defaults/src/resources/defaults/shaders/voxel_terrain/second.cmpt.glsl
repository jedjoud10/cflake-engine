#version 460 core
#include_custom {"voxel_include_path"}
#include "defaults\shaders\voxel_terrain\shared.func.glsl"

const int CHUNK_SIZE = #constant chunk_size
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) uniform atomic_uint chunk_subregion_positive_check;
layout(binding = 0) uniform atomic_uint chunk_subregion_negative_check;
layout(std430, binding = 0) readonly buffer arbitrary_voxels
{   
    Voxel voxels[];
};
layout(std430, binding = 1) writeonly buffer output_voxels
{   
    PackedVoxel packed_voxels[];
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
    if (all(lessThan(pixel_coords, ivec3(CHUNK_SIZE+1, CHUNK_SIZE+1, CHUNK_SIZE+1)))) {        
        // Create the final voxel
        Voxel voxel = voxels[flatten(pc, CHUNK_SIZE+2)];
        Voxel vx = voxels[flatten(pc+ivec3(1, 0, 0), CHUNK_SIZE+2)];
        Voxel vy = voxels[flatten(pc+ivec3(0, 1, 0), CHUNK_SIZE+2)];
        Voxel vz = voxels[flatten(pc+ivec3(0, 0, 1), CHUNK_SIZE+2)];

        // Calculate the normal for a voxel using the neighboring normals
        vec3 normal = normalize(vec3(vx.density-voxel.density, vy.density-voxel.density, vz.density-voxel.density));
        modify_voxel(pos, normal, voxel);
        FinalVoxel final_voxel = get_final_voxel(pos, normalize(normal), voxel);
        // Pack the voxel
        PackedVoxel packed_voxel = get_packed_voxel(final_voxel);

        // And store the final voxel inside our array
        packed_voxels[flatten(pc.xzy, CHUNK_SIZE+1)] = packed_voxel;

        // Atomic counter moment   
        
        // Get the chunk sub region that we are in
        ivec3 chunk_subregion = clamp(pc.xyz / (CHUNK_SIZE/2), ivec3(0), ivec3(1));
        uint chunk_subregion_index = clamp(flatten(chunk_subregion, 2), 0, 7); 
        if (final_voxel.density <= 0.0) {
            atomicCounterOr(chunk_subregion_negative_check, 1 << chunk_subregion_index);
        } else {
            atomicCounterOr(chunk_subregion_positive_check, 1 << chunk_subregion_index);
        }
    }
}