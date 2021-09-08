#version 460 core
#include "user\shaders\density.func.glsl"
// Load the density function file
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(r32f, binding = 0) uniform image3D voxel_image;
layout(location = 1 ) uniform vec3 node_pos;
layout(location = 2 ) uniform int node_size;
layout(location = 3 ) uniform int chunk_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;              
    // Create the pixel value
    vec4 pixel = vec4(get_density(pos), 0.0, 0.0, 0.0);    
    
    // Write the pixel
    imageStore(voxel_image, pixel_coords, pixel);
}