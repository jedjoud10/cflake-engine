// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    uint material;
    vec4 color;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const uvec3 local_pos, const vec3 pos) {
    // Material type 255 is a reserved default material (air)
    return Voxel(pos.y, 255, vec4(1));
}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // If the material is already set, use it
    if (voxel.material != 255) {
        return;
    }
}