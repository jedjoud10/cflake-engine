#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) writeonly uniform image3D density_image;
layout(binding = 1) writeonly uniform image3D material_image;
layout(binding = 2) uniform atomic_uint positive_counter;
layout(binding = 2) uniform atomic_uint negative_counter;
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;
layout(location = 4) uniform int chunk_size;
layout(location = 5) uniform float isoline;

void main() {
    // Get the pixel coord#include_custom {"voxel_interpreter"}
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;              
    // Create the pixel value    
    DensityVoxel density = DensityVoxel(0.0);
    MaterialVoxel material = MaterialVoxel(0);
    get_voxel(pos, density, material);   

    // Write the voxel pixel
    vec4 pixel = vec4(density.density, 0.0, 0.0, 0.0);        
    imageStore(density_image, pixel_coords, pixel);        
    
    // Write the material pixel
    vec4 material_pixel = vec4(material.material_id/255.0, 0, 0, 0);
    imageStore(material_image, pixel_coords, material_pixel);  

    // Add to the atomic counters
    if (all(lessThan(pixel_coords, ivec3(33, 33, 33)))) {
        if (density.density <= 0.0) {
            atomicCounterIncrement(negative_counter);
        } else {
            atomicCounterIncrement(positive_counter);
        }
    }
}