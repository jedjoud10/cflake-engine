// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    // Projection & view matices, and their mult
    mat4 lightspace;

    // Shadow uniform strength and params
    float strength;
    float spread;
    uint size;
} shadow;

#extension GL_EXT_samplerless_texture_functions : require

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    in vec3 position,
    in texture2D shadow_map,
    in mat4 lightspace,
    in float strength,
    in float spread,
    in uint size
) {
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace * vec4(position, 1.0); 
    if(abs(ndc.x) > 1.0 ||
       abs(ndc.y) > 1.0 ||
       ndc.z > 1.0 || ndc.z < 0.0) {
        return 0.0;
    }

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;
    float bias = 0.005;

    float shadowed = 0.0;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            // Compare the greatest depth (from the shadowmap) and current depth
            float closest = texelFetch(shadow_map, ivec2((uvs.xy + vec2(x * 0.002, y * 0.002)) * size), 0).r;
            shadowed += current > (closest+bias) ? 1.0 : 0.0;
        }
    }
    shadowed /= 9.0;



    return shadowed;
}