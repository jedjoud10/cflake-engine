// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;

    // Actual range for this is 65535, since on the CPU we store this as a u16
    uint mat_type;
};

// A final voxel that will be packed, then sent back to the CPU
struct FinalVoxel {
    float density;
    vec3 normal;
    vec3 color;
    uint mat_type; 
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(vec3 pos) {
    const float pi = 3.1415; 
    float density = pos.y + snoise(pos * 0.006) * 100;
    return Voxel(density, vec3(1.0), 0);
}

// Get the final voxel at a specific position (Second Pass)
FinalVoxel get_final_voxel(vec3 pos, vec3 normal, Voxel voxel) {
    return FinalVoxel(voxel.density, normal, voxel.color, voxel.mat_type);
}