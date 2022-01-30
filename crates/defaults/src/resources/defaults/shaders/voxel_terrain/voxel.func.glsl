// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;

    // Actual range for this is 65535, since on the CPU we store this as a u16
    uint mat_type;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(vec3 pos) {
    return Voxel(pos.y + snoise(pos * 0.006 * vec3(1.0, 2.0 + sin(pos.y * 0.2), 1.0)) * 20.0, vec3(sin(pos.x), sin(pos.y-15.21), sin(pos.z+123)), 0);
}