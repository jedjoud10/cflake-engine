// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"

struct Voxel {
    float density;
    vec3 color;
    float hardness;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const vec3 pos) {
    return Voxel(pos.y + snoise(pos * 0.002) * 200.0, vec3(1.0), 1.0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const vec3 pos, inout vec3 normal, inout Voxel voxel) {
}