const float SHADOW_BIAS = #constant shadow_bias
const float NORMAL_OFFSET = 0.0;

// Calculate if a specific fragment is in shadow or not
float calculate_shadows(vec3 position, vec3 normal, vec3 light_dir, mat4 lightspace_matrix, sampler2D shadow_map_texture) {
    // Offset more when the normal is perpendicular to the light dir
    float normal_offset_factor = 1 - abs(dot(normal, light_dir));
    
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace_matrix * vec4(position + normal * normal_offset_factor * NORMAL_OFFSET, 1.0); 
    vec3 projected = ndc.xyz / ndc.w;
    if (projected.z > 1.0) {
        return 0.0;
    }
    // Remap from -1, 1 to 0, 1
    vec3 lightspace_uvs = projected * 0.5 + 0.5;

    // Get depths and test
    float current_depth = lightspace_uvs.z;
    float accumulated_shadow = 0.0;
    // Sample the depth texture multiple times to smooth it out
    vec2 offset_size = 1.0 / textureSize(shadow_map_texture, 0);
    const int samples = 1;
    for(int x = -samples; x <= samples; x++) {
        for (int y = -samples; y <= samples; y++) {
            vec2 offset = vec2(x, y) * offset_size;
            float in_shadow = texture(shadow_map_texture, vec2(lightspace_uvs.xy + offset)).r > current_depth - (SHADOW_BIAS * 0.0001) ? 0 : 1;
            accumulated_shadow += in_shadow;
        }
    }
    // Average
    accumulated_shadow /= float((samples*2+1) * (samples*2+1));
    return accumulated_shadow;
}