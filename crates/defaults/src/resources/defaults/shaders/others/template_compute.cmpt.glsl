#version 460 core
layout(local_size_x = 1, local_size_y = 1) in;
layout(std430, binding = 0) buffer bufferName {
    int my_array[4];
};

void main() {
    // Get the pixel coord
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    my_array[pixel_coords.x] += 1;
    
    // Create the pixel value
    vec4 pixel = vec4(1.0, 1.0, 1.0, 1.0);    
}