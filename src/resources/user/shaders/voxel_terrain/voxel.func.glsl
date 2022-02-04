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
Voxel get_voxel(const uvec3 local_pos, const vec3 pos) {
    float noise = 0.0;
    float density = pos.y + snoise(pos * 0.0002) * 1000.0;
    return Voxel(density, vec3(1.0), 0);
}

// Modify the voxel after we get it's normal
void modify_voxel(const uvec3 local_pos, const vec3 pos, inout vec3 normal, inout Voxel voxel) {
    vec3 color = vec3(0.0);
    // Some colors
    if (dot(normal, vec3(0, 1, 0)) > 0.9) {
        color = vec3(94, 128, 25) / 255;
    } else if (dot(normal, vec3(0, 1, 0)) > 0.7) {
        color = vec3(43, 27, 5) / 255;
    } else {
        color = vec3(0.2);
    }
    voxel.color = color;
    voxel.color *= mix(snoise(pos * 0.03 + 502.0), 1.0, 0.95);
    if (any(lessThan(local_pos.xz, uvec2(2, 2)))) {
        //voxel.color = vec3(0);
    }
}