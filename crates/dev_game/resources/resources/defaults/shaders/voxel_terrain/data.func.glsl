// Some voxel data that is generated in the first pass of the compute shader
struct VoxelData {
    float density;
    int biome_id;
}
// Effects voxel data that is generated in a second compute shader pass
struct EffectsVoxelData {
    vec3 color;    
}