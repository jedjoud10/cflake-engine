#include_custom {"voxel_include_path"}
// A voxel type that contains FinalVoxel, but also the density of said voxel
struct BundledVoxel {
    float density;
    FinalVoxel fvoxel;
}