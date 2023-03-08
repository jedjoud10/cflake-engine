// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    // Projection & view matices
    mat4 projection;
    mat4 view;

    /*
    // Shadow uniform strength and params
    float strength;
    float spread;
    float
    */
} shadow;

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    in vec3 position,
    in texture2D shadow_map,
    in ShadowUniform ubo,
) {
    return 0.0;
}