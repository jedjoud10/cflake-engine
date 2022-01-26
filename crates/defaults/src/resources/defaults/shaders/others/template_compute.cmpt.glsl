#version 460 core
layout(local_size_x = 1, local_size_y = 1) in;
layout(rgba8, binding = 0) uniform image2D output_img;

void main() {
    // Get the pixel coord
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    
    // Create the pixel value
    vec4 pixel = vec4(1.0, 1.0, 1.0, 1.0);    
    
    // Write the pixel
    imageStore(output_img, pixel_coords, pixel);
}