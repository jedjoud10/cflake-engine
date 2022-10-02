// Kindly copied from https://learnopengl.com/Advanced-OpenGL/Framebuffers
const float blur_kernel[9] = float[](
    1.0 / 16, 2.0 / 16, 1.0 / 16,
    2.0 / 16, 4.0 / 16, 2.0 / 16,
    1.0 / 16, 2.0 / 16, 1.0 / 16  
);

const float sharpen_kernel[9] = float [](
    -1, -1, -1,
    -1,  9, -1,
    -1, -1, -1
);

const float offsets[9] = vec2[](
    vec2(-1.0, 1.0), // top-left
    vec2(0.0, 1.0), // top-center
    vec2(1.0, 1.0), // top-right
    vec2(-1.0, 0.0f),   // center-left
    vec2( 0.0f, 0.0f),   // center-center
    vec2( 1.0,  0.0f),   // center-right
    vec2(-1.0, -1.0), // bottom-left
    vec2( 0.0f, -1.0), // bottom-center
    vec2( 1.0, -1.0)  // bottom-right    
);

// Sample a texture, but apply some sort of kernel when fetching texels
// This assumes that the kernel is of 9x9 size
vec4 convoluted(sampler2D tex, vec2 uv, float[] kernel, float offset) {
    vec4 color = vec4(0.0); 
    for(int i = 0; i < 9; i++) {
        vec4 sampled = texture(tex, uv + offsets[i] * offset);
        col += sampled * kernel[i];
    }

    return color;
} 