const float SHADOW_BIAS = #constant shadow_bias
const float NORMAL_OFFSET = 0.2;

// Calculate if a specific fragment is in shadow or not
float calculate_shadows(vec3 position, vec3 normal, vec3 light_dir, mat4 lightspace_matrix, sampler2DShadow shadow_map_texture) {
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
    float in_shadow = texture(shadow_map_texture, vec3(lightspace_uvs.xy, current_depth - (SHADOW_BIAS * 0.0001)));

    return in_shadow;
}