#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"

uniform sampler2D tex;
// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;
    uint material;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const vec3 pos) {
    float noise = 0.0;
    for(int i = 0; i < 4; i++) {
        noise += (1-voronoi(pos * 0.0005 * pow(1.8, i)).x) * pow(0.4, i) * 300;
    }
    float density = pos.y + noise - 760;
    density = opSmoothUnion(pos.y, density, 40.0);
    return Voxel(density, vec3(1.0), 0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    vec3 color = vec3(0.0);
    // Some colors
    if (dot(normal, vec3(0, 1, 0)) > 0.9) {
        color = vec3(24, 120, 50) / 255;
    } else if (dot(normal, vec3(0, 1, 0)) > 0.7) {
        color = vec3(102, 67, 30) / 255;
    } else {
        color = vec3(0.2);
    }
    voxel.color = color;
}