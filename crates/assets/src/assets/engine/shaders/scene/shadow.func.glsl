// Check if a fragment is in shadow or not
// This will return 1.0 if the fragment in shadow, and 0.0 if the fragment is not in shadow
float is_in_shadow(vec3 position, vec3 light_dir, mat4 lightspace_matrix, sampler2DShadow tex) {
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace_matrix * vec4(position, 1.0); 
    vec3 projected = ndc.xyz / ndc.w;
    if (projected.z > 1.0) {
        return 0.0;
    }

    // Remap from -1, 1 to 0, 1
    vec3 lightspace_uvs = projected * 0.5 + 0.5;

    // Get depths and test
    float shadow_bias = 0.0001;
    float current = lightspace_uvs.z;

    // Number of omni-directional samples to take
    const int SAMPLES = 1;
    const int HALVED_SAMPLES = SAMPLES / 2;

    // Sample the depth texture multiple times to smooth it out
    float sum = 0.0;
    vec2 offset_size = 1.0 / textureSize(tex, 0);
    for(int x = -HALVED_SAMPLES; x <= HALVED_SAMPLES; x++) {
        for (int y = -HALVED_SAMPLES; y <= HALVED_SAMPLES; y++) {
            vec2 offset = vec2(x, y) * offset_size;
            float in_shadow = texture(tex, vec3(lightspace_uvs.xy + offset, current - shadow_bias)).r;
            sum += in_shadow;
        }
    }
    sum /= pow(HALVED_SAMPLES*2 + 1, 2);

    return sum;
}