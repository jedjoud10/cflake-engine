#version 460 core
layout(local_size_x = 8, local_size_y = 8) in;
layout(rgba8, binding = 0) uniform image2D image_stats;
layout(location = 1) uniform sampler1D entities_texture;
layout(location = 2) uniform float time;
layout(location = 3) uniform float fps;

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
    // Write the pixel
    if (pixel_coords.y < (gl_NumWorkGroups.y * gl_WorkGroupSize.y)-64) {
        float t = mod(time*10.0, gl_NumWorkGroups.x * gl_WorkGroupSize.x);
        imageStore(image_stats, ivec2(t, pixel_coords.y), pixel);
    } else {
        // Write the entity pixel
        float valid_entity = texture(entities_texture, uvs.x).r;
        pixel = vec4(valid_entity, valid_entity, valid_entity, 1.0);
        imageStore(image_stats, pixel_coords, pixel);
    }
}