// Calculate a linearly interpolated shadow type
float shadow_linear(sampler2DShadow tex, vec2 uv, float compare) {
    return texture(tex, vec3(uv, compare)).r;

    /*
    float offset = 1.0 / textureSize(tex, 0).x;
    float uv0 = sample_shadow_texel(tex, uv, compare);
    float uv1 = sample_shadow_texel(tex, uv + vec2(offset, 0.0), compare);
    float uv2 = sample_shadow_texel(tex, uv + vec2(0.0, offset), compare);
    float uv3 = sample_shadow_texel(tex, uv + vec2(offset, offset), compare);

    // Interpolate results in the x axis
    vec2 frac = fract(uv * textureSize(tex, 0).xy);
    float bottom = mix(uv0, uv1, frac.x);
    float top = mix(uv2, uv3, frac.x);

    // Interpolate results in the y axis
    return mix(bottom, top, frac.y);
    */
}

// Sample the shadow texture multiple times with a specific offset
// This will sample the depth texture multiple times to smooth it out
float shadow_pcf(sampler2DShadow tex, vec3 uv, float offset) {
    // Get depths and test
    const float bias = 0.0001;
    float main = shadow_linear(tex, uv.xy, uv.z + bias);
    
    if (main > 1.0 || main < 0.0) {
        return main;
    }

    const int SAMPLES = 2;
    const int HALVED_SAMPLES = SAMPLES / 2;

    float sum = 0.0;
    vec2 offset_size = 1.0 / textureSize(tex, 0) * (1.0 / float(SAMPLES+1));
    for(int x = -HALVED_SAMPLES; x <= HALVED_SAMPLES; x++) {
        for (int y = -HALVED_SAMPLES; y <= HALVED_SAMPLES; y++) {
            vec2 local_offset = vec2(x, y) * offset_size * offset;
            sum += shadow_linear(tex, uv.xy + local_offset, uv.z + bias);
        }
    }

    sum /= pow(HALVED_SAMPLES*2 + 1, 2);
    return sum;
}

// Check if a fragment is in shadow or not
// This will return 1.0 if the fragment in shadow, and 0.0 if the fragment is not in shadow
float is_in_shadow(vec3 position, vec3 light_dir, mat4 lightspace_matrix, sampler2DShadow tex) {
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace_matrix * vec4(position, 1.0); 
    if(abs(ndc.x) > 1.0 ||
       abs(ndc.y) > 1.0 ||
       abs(ndc.z) > 1.0) {
        return 0.0;
    }

    // Project the world point into uv coordinates to read from
    vec3 projected = ndc.xyz / ndc.w;
    vec3 lightspace_uvs = projected * 0.5 + 0.5;

    // PCF shadow sampling (hardware)
    return shadow_pcf(tex, lightspace_uvs, 4.5);
}