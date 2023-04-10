// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    float strength;
    float spread;
} shadow_parameters;

// Contains all the lightspace matrices for each cascade
layout(set = 0, binding = 5) uniform ShadowLightSpaceMatrices {
    mat4 matrices[3];
} shadow_lightspace_matrices;

// Shadow-map texture map
layout(set = 0, binding = 6) uniform texture2DArray shadow_map;

#extension GL_EXT_samplerless_texture_functions : require

// Sample a single shadow texel at the specified pixel coords
float sample_shadow_texel(
    uint layer,
    ivec2 pixel,
    float compare
) {
    float bias = 0.0010 * (layer+1);
    float closest = texelFetch(shadow_map, ivec3(pixel, int(layer)), 0).r;
    return compare > (closest+bias) ? 1.0 : 0.0;
}

// Calculate a linearly interpolated shadow value
float shadow_linear(
    uint layer,
    vec2 uvs,
    uint size,
    float compare
) {
    // Get a quad that contains the binary values
    ivec2 pixel = ivec2(uvs.xy * size);
    float uv0 = sample_shadow_texel(layer, pixel, compare);
    float uv1 = sample_shadow_texel(layer, pixel + ivec2(1, 0), compare);
    float uv2 = sample_shadow_texel(layer, pixel + ivec2(0, 1), compare);
    float uv3 = sample_shadow_texel(layer, pixel + ivec2(1, 1), compare);

    // Interpolate results in the x axis
    vec2 frac = fract(uvs * vec2(size));
    float bottom = mix(uv0, uv1, frac.x);
    float top = mix(uv2, uv3, frac.x);

    // Interpolate results in the y axis
    return mix(bottom, top, frac.y);
}

// Check if a pixel is obscured by the shadow map
float calculate_shadowed(
    vec3 position,
    vec3 camera
) {
    // Calculates what shadow layer we must use
    uint index = 2;
    for(int i = 0; i < 2; i++) {
        mat4 lightspace = shadow_lightspace_matrices.matrices[i];
        vec4 ndc = lightspace * vec4(position, 1.0); 

        // TODO: pls fix coed
        if(!(abs(ndc.x) > 1.0 ||
       abs(ndc.y) > 1.0 ||
       ndc.z > 1.0 || ndc.z < 0.0)) {
            index = i;
            break;
       }
    }
    uint layer = index;
    //uint layer = uint(min(floor(distance(position, camera) / 10.0), 2));

    // Get the proper lightspace matrix that we will use
    mat4 lightspace = shadow_lightspace_matrices.matrices[layer];
    
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace * vec4(position, 1.0); 
    
    /*
    if(abs(ndc.x) > 1.0 ||
       abs(ndc.y) > 1.0 ||
       ndc.z > 1.0 || ndc.z < 0.0) {
        return 0.0;
    }
    */

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;

    // Get texture size
    uint size = uint(textureSize(shadow_map, 0).x);

    float shadowed = 0.0;
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            shadowed += shadow_linear(layer, uvs.xy + vec2(x, y) * 0.001, size, current);
        }
    }
    shadowed /= 25.0;
    return shadowed;
    

    //return sample_shadow_texel(layer, ivec2(uvs.xy * size), current);
    //return shadow_linear(shadow_map, layer, uvs.xy, current);
}