// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// Post-rendering effects
vec3 post(vec2 uvs, vec3 color) {
    // A vignette effect
    float vignette_strength_x = pow(abs(uvs.x - 0.5), 4);
    float vignette_strength_y = pow(abs(uvs.y - 0.5), 4);
    float vignette_strength = (vignette_strength_x + vignette_strength_y) * 2.0; 
    float vignette = (1-vignette_strength);

    // exposure tone mapping
    const float gamma = 2.2;
    float exposure = 0.2;
    vec3 mapped = vec3(1.0) - exp(-color * exposure);
    // gamma correction 
    mapped = pow(mapped, vec3(1.0 / gamma));
    
    return mapped;
}