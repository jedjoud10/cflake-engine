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

// Chunk definition
struct Chunk {
    uint depth;
    uint size;
};


// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const Chunk chunk, vec3 pos) {
    float noise = 0.0;
    for (int i = 0; i < 6; i++) {
        noise += abs(snoise(pos * 0.0009 * vec3(1, 1.3, 1.0) * pow(1.7, i) + hash11(4.0595 * i))) * pow(0.43, i);
    }
    return Voxel(pos.y + noise * 200.0, 255, vec3(1.0));
}

// Modify the voxel after we get it's normal
void modify_voxel(const Chunk chunk, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // If the material is already set, use it
    if (voxel.material != 255) {
        return;
    }
    if (dot(normal, vec3(0, 1, 0)) > 0.9) {
        voxel.material = 0;
    } else {
        voxel.material = 1;
    }
}