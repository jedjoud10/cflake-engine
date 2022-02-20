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
    float 
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const uvec3 local_pos, const vec3 pos) {
    float noise = 0.0;
    float density = pos.y + snoise(pos * 0.01 * vec3(1, 2, 1)) * 130.0;
    float density2 = 0.0;
    for (int i = 0; i < 8; i++) {
        density2 += pos.y + (1-voronoi(pos * 0.001 * vec3(1, 1.5, 1) * pow(1.6, i)).x) * 500 * pow(0.423, i);
    }
    float factor = 1.0;
    return Voxel(density2, 0, false);
}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // Some colors
    if (!voxel.artificial) {
        if (dot(normal, vec3(0, 1, 0)) > 0.9) {
            voxel.material = 0;
        } else if (dot(normal, vec3(0, 1, 0)) > 0.8) {
            voxel.material = 1;
        } else {
            voxel.material = 2;
        }
    } else {
        voxel.material = 2;
    }
}