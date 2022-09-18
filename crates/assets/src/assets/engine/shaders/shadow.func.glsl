// Check if a fragment is in shadow or not
// This will return 1.0 if the fragment in shadow, and 0.0 if the fragment is not in shadow
float is_in_shadow(vec3 position, vec3 light_dir, mat4 lightspace_matrix, sampler2D shadow_map_texture) {
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace_matrix * vec4(position, 1.0); 
    vec3 projected = ndc.xyz / ndc.w;
    if (projected.z > 1.0) {
        return 0.0;
    }

    // Remap from -1, 1 to 0, 1
    vec3 lightspace_uvs = projected * 0.5 + 0.5;

    // Get depths and test
    float shadow_bias = 0.0004;
    float current_depth = lightspace_uvs.z;
    float closest_depth = texture(shadow_map_texture, lightspace_uvs.xy).r;
    float in_shadow = current_depth - shadow_bias > closest_depth ? 1.0 : 0.0; 

    /*
    float accumulated_shadow = 0.0;

    // Sample the depth texture multiple times to smooth it out
    vec2 offset_size = 1.0 / textureSize(shadow_map_texture, 0);
    for(int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 offset = vec2(x, y) * offset_size;
            float closest_depth = texture(shadow_map_texture, lightspace_uvs.xy + offset).r;
            float in_shadow = current_depth - shadow_bias > closest_depth ? 1.0 : 0.0; 
            accumulated_shadow += in_shadow;
        }
    }
    
    // Average
    accumulated_shadow /= 9.0;
    */

    return in_shadow;
}