#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"

uniform sampler2D anime_girl;
// A simple voxel that is stored in an array, in a GPU buffer 
// This voxel struct can contain some arbitrary values related to voxel generation
struct Voxel {
    float density;
    vec3 color;
    float hardness;
};

// Get the voxel at a specific position (First Pass)
Voxel get_voxel(const vec3 pos) {
    float density = pos.y;
    float voronoi_density = 0.0;
    for(int i = 0; i < 3; i++) {
        voronoi_density += (snoise(pos * 0.001 * pow(2.0, i) * vec3(1.0, 4.0, 1.0))) * pow(0.5, i) * 200;
    }
    return Voxel(pos.y + voronoi_density * 0.2, vec3(1.0), 1.0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    voxel.color = texture(anime_girl, pos.xz / 300.0).rgb;
}