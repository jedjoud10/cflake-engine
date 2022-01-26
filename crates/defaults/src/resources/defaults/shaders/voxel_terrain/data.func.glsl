// The density voxel only contains the density at a specific location
struct DensityVoxel {
    float density;
};
// The material voxel can contain the type of material and some tint for a specific voxel
struct MaterialVoxel {
    int material_id;
};