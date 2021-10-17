// Some voxel data that is generated in the first pass of the compute shader
struct VoxelData {
    float density;
    int biome_id;
};
struct ColorVoxel {
    vec3 color;    
};