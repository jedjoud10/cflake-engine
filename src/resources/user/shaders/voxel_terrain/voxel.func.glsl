// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;
    float hardness;
    float blend;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const vec3 pos) {
    return Voxel(pos.y, vec3(1.0), 1.0, 1.0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    voxel.color *= normal * voxel.blend;
}