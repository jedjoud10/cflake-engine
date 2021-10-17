// Some voxel data that is generated in the first pass of the compute shader
struct VoxelData {
    int density;
    int biomeID;
    int materialID;
};
struct ColorVoxel {
    vec3 color;    
};