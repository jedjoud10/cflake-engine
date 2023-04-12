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

#extension GL_EXT_samplerless_texture_functions : require

// Sample a single shadow texel at the specified pixel coords
float sample_shadow_texel(
    uint layer,
    ivec2 pixel,
    float compare
) {
    float bias = 0.00017 * (layer*2+1);
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
    float depth,
    vec3 normal,
    vec3 light_dir,
    vec3 camera
) {
    //return clamp(linearize_depth(depth, 0.01, 5000), 0, 1);

    // Offset more when the normal is perpendicular to the light dir
    float normal_offset_factor = 1 - abs(dot(normal, light_dir));

    // Taken from a comment by Octavius Ace from the same learn OpenGL website 
    vec4 res = step(cascade_plane_distances.distances, vec4(linearize_depth(depth, 0.01, 5000)));
    uint layer = uint(res.x + res.y + res.z + res.w);
    //return float(layer) / 4.0;

    // Get the proper lightspace matrix that we will use
    mat4 lightspace = shadow_lightspace_matrices.matrices[layer];
    
    // Transform the world coordinates to NDC coordinates 
    vec4 ndc = lightspace * vec4(position + normal * normal_offset_factor * 0.03, 1.0); 

    // Project the world point into uv coordinates to read from
    vec3 uvs = ndc.xyz / ndc.w;
    uvs.xy *= 0.5;
    uvs.xy += 0.5;
    uvs.y = 1-uvs.y;
    float current = uvs.z;

    // Get texture size
    uint size = uint(textureSize(shadow_map, 0).x);

    /*
    float shadowed = 0.0;
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            shadowed += shadow_linear(layer, uvs.xy + vec2(x, y) * 0.001, size, current);
        }
    }
    shadowed /= 25.0;
    return shadowed;
    */
    

    return sample_shadow_texel(layer, ivec2(uvs.xy * size), current);
    //return shadow_linear(shadow_map, layer, uvs.xy, current);
}