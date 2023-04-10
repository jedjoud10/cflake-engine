// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    float strength;
    float spread;
} shadow_parameters;

// Contains all the lightspace matrices for each cascade
layout(set = 0, binding = 5) uniform ShadowLightSpaceMatrices {
    mat4 matrices[];
} shadow_lightspace_matrices;

#extension GL_EXT_samplerless_texture_functions : require

/*
// Sample a single shadow texel at the specified pixel coords
float sample_shadow_texel(
    in texture2D tex,
    int mip,
    ivec2 pixel,
    float compare
) {
    float bias = 0.007 + (mip * 0.007);
    float closest = texelFetch(tex, pixel, mip).r;
    return compare > (closest+bias) ? 1.0 : 0.0;
}

// Calculate a linearly interpolated shadow value
float shadow_linear(
    in texture2D tex,
    int mip,
    vec2 uvs,
    uint size,
    float compare
) {
    // Get a quad that contains the binary values
    ivec2 pixel = ivec2(uvs.xy * size);
    float uv0 = sample_shadow_texel(tex, mip, pixel, compare);
    float uv1 = sample_shadow_texel(tex, mip, pixel + ivec2(1, 0), compare);
    float uv2 = sample_shadow_texel(tex, mip, pixel + ivec2(0, 1), compare);
    float uv3 = sample_shadow_texel(tex, mip, pixel + ivec2(1, 1), compare);

    // Interpolate results in the x axis
    vec2 frac = fract(uvs * vec2(size));
    float bottom = mix(uv0, uv1, frac.x);
    float top = mix(uv2, uv3, frac.x);

    // Interpolate results in the y axis
    return mix(bottom, top, frac.y);
}
*/

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    vec3 position,
    vec3 camera,
    in texture2DArray shadow_map,
    mat4 lightspace,
    float strength,
    float spread,
    uint size
) {
    return 0.0;
    /*
    //position = floor(position * 5.0) / 5.0;

    vec4 ndc = lightspace * vec4(position, 1.0); 
    // Transform the world coordinates to NDC coordinates 
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
    vec2 inv = vec2(1)/vec2(size);
    float lod = min(floor(distance(position, camera) / 10.0), 3);
    lod = 0;

    // Calculate size scaler
    int scaler = int(pow(2.0, lod)); 
    float shadowed = 0.0;

    inv *= float(scaler);

    /*
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            shadowed += shadow_linear(shadow_map, int(lod), uvs.xy + vec2(x, y) * inv, size / scaler, current);
        }
    }
    shadowed /= 25.0;
    return shadowed;
    */

    //return sample_shadow_texel(shadow_map, int(lod), ivec2(uvs.xy * (size / scaler)), current);
    //return shadow_linear(shadow_map, int(lod), uvs.xy, size / testino, current);
}