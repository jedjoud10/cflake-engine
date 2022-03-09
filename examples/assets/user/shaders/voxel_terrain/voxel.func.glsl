#include "defaults/shaders/others/hashes.func.glsl"
#include "defaults/shaders/noises/simplex.func.glsl"
#include "defaults/shaders/noises/voronoi.func.glsl"
#include "defaults/shaders/others/sdf.func.glsl"

uniform sampler2D tex;
// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    uint material;
    vec3 color;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const uvec3 local_pos, vec3 pos) {
    float noise = 0.0;
    for (int i = 0; i < 6; i++) {
        noise += abs(snoise(pos * 0.0008 * vec3(1, 0.2, 1.0) * pow(1.7, i) + 4.0595)) * pow(0.43, i);
    }
    return Voxel(snoise(pos * 0.002) * 300 + pos.y, 255, vec3(1.0));
}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // If the material is already set, use it
    if (voxel.material != 255) {
        return;
    }
    if (dot(normal, vec3(0, 1, 0)) > 0.9) {
        voxel.material = 0;
    } else if (dot(normal, vec3(0, 1, 0)) > 0.8) {
        voxel.material = 1;
    } else {
        voxel.material = 2;
    }
}