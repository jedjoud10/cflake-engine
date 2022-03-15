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
        noise += abs(snoise(pos * 0.0009 * vec3(1, 2.0, 1.0) * pow(1.7, i) + 4.0595)) * pow(0.43, i);
    }

    vec3 offset = vec3(snoise(pos * 0.001 + 41.2), snoise(pos * 0.001 - 12.41), snoise(pos * 0.001 + 13.41));

    float cave = ((1-voronoi(pos * 0.0008 * vec3(1, 2, 1) + offset * 0.01).x) * 2 - 1.0) * 20.0;

    float mixing = snoise(pos * 0.0001 * vec3(1, 0, 1)) / 2 + 0.5;
    float mountains = opSmoothUnion(pos.y + noise * 200.0, pos.y + 110.0, 45.0);
    return Voxel(mix(mountains, cave, mixing), 255, vec3(1.0));

}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // If the material is already set, use it
    if (voxel.material != 255) {
        return;
    }
    if (dot(normal, vec3(0, 1, 0)) > 0.0) {
        voxel.material = 0;
    } else {
        voxel.material = 1;
    }
}