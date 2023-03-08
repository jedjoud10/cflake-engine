// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    // Projection & view matices, and their mult
    mat4 lightspace;
    mat4 test;

    /*
    // Shadow uniform strength and params
    float strength;
    float spread;
    float
    */
} shadow;

#extension GL_EXT_samplerless_texture_functions : require

// Check if a pixel is obscured by the shadow map
vec3 calculate_shadowed(
    in vec3 position,
    in texture2D shadow_map,
    in mat4 lightspace,
    in mat4 test
) {
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = test * vec4(position, 1.0); 
    if(abs(ndc.x) > 1.0 ||
       abs(ndc.y) > 1.0 ||
       ndc.z > 1.0 || ndc.z < 0.0) {
        return vec3(0.0);
    }

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float closest = texelFetch(shadow_map, ivec2(uvs.xy * 4096), 0).r;
    float current = uvs.z;
    float bias = 0.001;

    // Compare the greatest depth (from the shadowmap) and current depth
    return vec3(closest);
    //return current > (closest+bias) ? 1.0 : 0.0;
}