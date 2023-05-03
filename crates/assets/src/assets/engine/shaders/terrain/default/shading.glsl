#ifdef submaterials
// Albedo / diffuse map texture array
layout(set = 1, binding = 0) uniform texture2DArray layered_albedo_map;
layout(set = 1, binding = 1) uniform sampler layered_albedo_map_sampler;

// Normal map texture array
layout(set = 1, binding = 2) uniform texture2DArray layered_normal_map;
layout(set = 1, binding = 3) uniform sampler layered_normal_map_sampler;

// Mask map texture array
layout(set = 1, binding = 4) uniform texture2DArray layered_mask_map;
layout(set = 1, binding = 5) uniform sampler layered_mask_map_sampler;

// Triplanar mapping offset and UV scale
const float offset = 0.0;
const vec2 scale = vec2(0.1) * vec2(-1, 1); 
const float normal_strength = 1.0;
#endif

// Returned by all shading functions
struct TerrainSurfaceData {
    // Color of the fragment
    vec4 albedo;

    // AO, Roughness, Metallic
    vec4 mask;

    // World space normal
    vec3 normal;
};


#ifdef submaterials

// with sub-materials
TerrainSurfaceData shade(
    vec3 position,
    vec3 normal,
) {
    return TerrainSurfaceData(vec4(0), vec4(0), normal);
}
#else

// no material
TerrainSurfaceData shade(
    vec3 position,
    vec3 normal,
) {
    return TerrainSurfaceData(vec4(0), vec4(0), normal);
}

#endif
