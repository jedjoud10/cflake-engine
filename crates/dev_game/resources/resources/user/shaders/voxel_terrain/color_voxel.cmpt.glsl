#version 460 core
#include "user\shaders\voxel_terrain\color_voxel.func.glsl"
// Load the color voxel function file
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(rg32f, binding = 0) uniform image3D voxel_image;
layout(rgba32f, binding = 1) uniform image3D color_voxel_image;
uniform vec3 node_pos;
uniform int node_size;
uniform int chunk_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;              
    // Create the pixel value
    ColorVoxel voxel = get_color_voxel(pos);
    vec4 pixel = vec4(voxel.color, 0.0);    
    
    // Write the pixel
    imageStore(color_voxel_image, pixel_coords, pixel);
}