// Stolen from https://learnopengl.com/Guest-Articles/2021/CSM
// Thanks

#include <engine/shaders/math/conversions.glsl>

// UBO set globally at the start of the frame
layout(set = 0, binding = 4) uniform ShadowUniform {
    float strength;
    float spread;
} shadow_parameters;

// Contains all the lightspace matrices for each cascade
layout(set = 0, binding = 5) uniform ShadowLightSpaceMatrices {
    mat4 matrices[4];
} shadow_lightspace_matrices;

// Contains all the cascade plane distances
layout(set = 0, binding = 6) uniform ShadowPlaneDistances {
    vec4 distances;
} cascade_plane_distances;

// Shadow-map texture map
layout(set = 0, binding = 7) uniform texture2DArray shadow_map;

// Sample a single shadow texel at the specified pixel coords
float sample_shadow_texel(
    uint layer,
    ivec2 pixel,
    float compare
) {
    float closest = texelFetch(shadow_map, ivec3(pixel, int(layer)), 0).r;
    return (compare > closest) ? 1.0 : 0.0;
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
    float depth,
    vec3 normal,
    vec3 light_dir,
    vec3 camera
) {
    // TODO: FUCKING FIX SHADOWS FFS
    // AAAAAAAAAAAAAAAAAAAAAAAAAAA
    return 0.0;

    // Taken from a comment by Octavius Ace from the same learn OpenGL website 
    vec4 res = step(cascade_plane_distances.distances, vec4(depth));
    uint layer = uint(res.x + res.y + res.z + res.w);
    
    // Get the proper lightspace matrix that we will use
    mat4 lightspace = shadow_lightspace_matrices.matrices[layer];
    
    // Transform the world coordinates to NDC coordinates 
    float perpendicularity = 1 - abs(dot(normal, light_dir));
    vec4 ndc = lightspace * vec4(position + normal, 1.0); 
    float factor = pow(1.35, layer);
    float bias = -0.001 - (perpendicularity * 0.0003);
    bias *= factor;

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz / ndc.w;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;

    // Get texture size
    uint size = uint(textureSize(shadow_map, 0).x);

    // TODO: Spread size is calculated based on distance
    float spread = 0.0004;

    float shadowed = 0.0;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            shadowed += shadow_linear(layer, uvs.xy + vec2(x, y) * spread, size, current + bias);
        }
    }
    shadowed /= 9.0;
    return shadowed;   


    //return sample_shadow_texel(layer, ivec2(uvs.xy * size), current + bias);
    //return shadow_linear(shadow_map, layer, uvs.xy, current);
}