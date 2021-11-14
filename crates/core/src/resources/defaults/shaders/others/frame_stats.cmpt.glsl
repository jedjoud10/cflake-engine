#version 460 core
layout(local_size_x = 8, local_size_y = 8) in;
layout(rgba8, binding = 0) uniform image2D image_stats;
layout(location = 1) uniform float time;
layout(location = 2) uniform float fps;
layout(location = 3) uniform sampler2D, 

void main() {
    // Get the pixel coord
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    vec2 uvs = pixel_coords / vec2(gl_NumWorkGroups.xy * gl_WorkGroupSize.xy);
    float offset = 1.0 / float(gl_NumWorkGroups.x * gl_WorkGroupSize.x);
    // Create the pixel value
    float v = ((fps/300.0) - (1-uvs.y));
    vec3 color = vec3(1.0, 1.0, 1.0);
    // Color tuning
    if (fps <= 30) {
        color = vec3(1, 0, 0);
    } else if (fps <= 40) {
        color = vec3(1.0, 0.85, 0);
    } else if (fps >= 40) {
        color = vec3(0, 1, 0);
    }
    float valid = v < 0.0 ? 0.0 : 1.0;
    vec4 pixel = vec4(vec3(valid, valid, valid) * color, 1.0);    
    bool test = entities[pixel_coords.x / 4];
    pixel = test ? vec4(1, 1, 1, 1) : vec4(0, 0, 0, 1);
    
    // Write the pixel
    float c_time = (mod((time*0.1), 1.0));
    if (c_time > uvs.x && c_time < uvs.x + offset) {
        imageStore(image_stats, pixel_coords, pixel);
    }
}