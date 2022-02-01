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
    float density = pos.y;
    float noise = 0.0;
    for(int i = 0; i < 3; i++) {
        noise += (snoise(pos * 0.001 * pow(2.0, i) * vec3(1.0, 2.0, 1.0))) * pow(0.5, i) * 100;
    }
    return Voxel(max((noise + pos.y), -sdBox(pos, vec3(30.0))), vec3(1.0), 0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    float val = clamp(snoise(pos * 0.0002) * 10.0, 0, 1);
    voxel.color = texture(tex, pos.xz / 128.0).rgb;
}