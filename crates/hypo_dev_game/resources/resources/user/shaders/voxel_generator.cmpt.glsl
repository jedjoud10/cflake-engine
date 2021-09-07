#version 460 core
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image3D voxel_image;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    
    // Create the pixel value
    vec4 pixel = vec4(1.0, 1.0, 1.0, 1.0);    
    
    // Write the pixel
    imageStore(voxel_image, pixel_coords, pixel);
}