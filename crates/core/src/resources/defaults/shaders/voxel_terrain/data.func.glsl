// Some voxel data that is generated in the first pass of the compute shader
struct Voxel {
    float density;
};
struct MaterialVoxel {
    int shader_id;
    int material_id;
};