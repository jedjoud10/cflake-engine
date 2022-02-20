// Convert depth to linear depth
float to_linear_depth(float odepth, vec2 nf_planes) {
    return (nf_planes.x * odepth) / (nf_planes.y - odepth * (nf_planes.y - nf_planes.x));	
}

// Project a world space point to screen space and get it's 3D UV coordinates
vec3 convert_to_screen_space_uvs(vec3 point, mat4x4 pv_matrix) {
    vec4 proj = pv_matrix * vec4(point, 1.0);
    vec3 ndc = proj.xyz / proj.w;
    return ndc * 0.5 + 0.5;
}

// Projects a world coordinates into screen cordinates and checks it's depth with the depth at that pixel
// New depth - old depth
float calculate_depth_difference(vec2 nf_planes, sampler2D depth_texture, vec3 uvs) {    
    // Sample the depth at that point and compare
    float og_depth = texture(depth_texture, uvs.xy).x;
    float new_depth = uvs.z; 
    // Gotta convert this to linear depth
    float og_depth_linear = to_linear_depth(og_depth, nf_planes);
    float new_depth_linear = to_linear_depth(new_depth, nf_planes);
    return new_depth_linear - og_depth_linear;
}

// Raymarching settings
const int MAX_STEPS = 128;
const int MAX_STEPS_FINE = 8;
const float STEP_SIZE = 2.0;

// Calculate the reflected color value for a specific pixel
vec3 calculate_ssr(vec3 pixel_dir, vec3 position, vec2 nf_planes, vec3 normal, sampler2D depth_texture, sampler2D color_texture, mat4x4 pv_matrix) {
    // Calculate the reflected normal
    vec3 reflected_dir = reflect(pixel_dir, normal) * STEP_SIZE;
    if (dot(reflected_dir, pixel_dir) < 0.0) { return vec3(0); } 
    // Raymarch in that direction, until we find a collision
    vec3 point = position;
    for(int i = 0; i < MAX_STEPS; i++) {
        point += reflected_dir;
        // Check if we hit a surface by checking depths
        vec3 projected_uvs = convert_to_screen_space_uvs(point, pv_matrix);
        // Outside the screen, don't care
        if (any(lessThan(projected_uvs, vec3(0))) || any(greaterThan(projected_uvs, vec3(1)))) {
            return vec3(0);
        }
        float depth_diff = calculate_depth_difference(nf_planes, depth_texture, projected_uvs);
        if (depth_diff > 0.0) {
            // Since we hit the surface using a coarse collision check, we must refine it and find the exact collision point
            vec3 point1 = point - reflected_dir;
            vec3 point2 = point;
            vec3 dir = (point2 - point1) / float(MAX_STEPS_FINE);
            for(int j = 0; j < MAX_STEPS_FINE; j++) {
                // Le fine
                vec3 uvs = convert_to_screen_space_uvs(point, pv_matrix);
                float depth_diff_fine = calculate_depth_difference(nf_planes, depth_texture, uvs);

                if (depth_diff_fine > 0.0 && depth_diff_fine < 0.02) {
                    vec3 color = texture(color_texture, uvs.xy).rgb;
                    return color;
                }
                point1 += dir;
            }
        }
    }
    // We didn't hit a surface
    return vec3(0);
}