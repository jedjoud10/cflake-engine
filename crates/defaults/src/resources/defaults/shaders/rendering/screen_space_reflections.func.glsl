
// Raymarching settings
const int MAX_STEPS = 32;
const float STEP_SIZE = 0.02;

// Check for a collision with the surface using the depth texture
bool check_collision(vec3 point, sampler2D depth_texture, mat4x4 vp_matrix) {
    // Convert the point to screen space
    vec4 proj = vp_matrix * vec4(point, 1.0);
    vec3 ndc = proj.xyz / proj.w;
    vec2 uv = ndc.xy * 0.5 + 0.5;
    // Sample the depth at that point and compare
    float og_depth = texture(depth_texture, uv).x;
    float new_depth = ndc.z; 

    return new_depth > og_depth;
}

// Calculate the reflected color value for a specific pixel
vec3 calculate_ssr(vec3 pixel_dir, vec3 position, vec3 normal, sampler2D depth_texture, sampler2D diffuse_texture, mat4x4 vp_matrix) {
    // Calculate the reflected normal
    vec3 reflected_dir = reflect(pixel_dir, normal);
    // Raymarch in that direction, until we find a collision
    vec3 point = position;
    for(int i = 0; i < MAX_STEPS; i++) {
        // Check
        if (check_collision(point, depth_texture, vp_matrix)) {
            return vec3(point);
        }
        point += reflected_dir;
    }

    // Check if we hit a surface by checking depths

    return vec3(1);
}