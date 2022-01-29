// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
};

// A final voxel that is also stored in an array, but we will read it back eventually on the CPU
// This final voxel should have only the nessecarry values that we need on a vertex to vertex basis
struct FinalVoxel {
    float density;
    vec3 normal;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(vec3 pos) {
    return Voxel(pos.x);
}

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, Voxel voxel) {
    return FinalVoxel(voxel.density, vec3(0.0));
}