// The density voxel only contains the density at a specific location
struct DensityVoxel {
    float density;
};
// The material voxel can contain the type of material and some tint for a specific voxel
struct MaterialVoxel {
    int material_id;
};

// Generate the voxel data at the specified position
void get_voxel(vec3 pos, out DensityVoxel density, out MaterialVoxel material) {
    density = DensityVoxel(pos.y + sin(pos.x / 10) * 10.0);
    material = MaterialVoxel(0);
}
