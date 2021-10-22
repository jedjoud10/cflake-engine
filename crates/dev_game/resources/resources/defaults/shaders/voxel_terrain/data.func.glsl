// Some voxel data that is generated in the first pass of the compute shader
struct Voxel {
    float density;
};
struct MaterialVoxel {
    int material_id;
    int biome_id;
    int hardness;
    int texture_id;
};
struct ColorVoxel {
    vec3 color;    
};
// Detail data for the detail spawner
struct Detail {
    // For the first texture
    // Position offset from the current pixel, the offset is actually calculated by dividing this value with 255
    ivec3 position_offset;
    bool spawn;
    // For the second texture
    vec3 rotation;
    float scale;
};