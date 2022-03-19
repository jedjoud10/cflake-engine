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
    // Ridged noise
    float noise = 0.0;
    float strength = 700.0;
    float scale = 0.0001;
    float lacunarity = 1.63;
    float persistence = 0.5;
    int octaves = 8;
    for (int i = 0; i < octaves; i++) {
        noise += (1-voronoi(pos * scale * vec3(1, 2.0, 1.0) * pow(lacunarity, i) + 4.0595).x) * pow(persistence, i);
    }
    float density = noise * strength + pos.y - 500;
    
    return Voxel(density, 255, vec3(1.0));
}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    // If the material is already set, use it
    if (voxel.material != 255) {
        return;
    }
    /*
    if (dot(normal, vec3(0, 1, 0)) > 0.9) {
        voxel.material = 0;
    } else if (dot(normal, vec3(0, 1, 0)) > 0.8) {
        voxel.material = 1;
    } else {
        voxel.material = 2;
    }
    */
    voxel.material = 0;
}