#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(r16f, binding = 0) uniform image3D sdf_tex;
#define CELL_SIZE 16
#define SPHERE_SIZE 0.2


struct SDFSphere {
    bool enabled;
    vec3 location;
};

// Get the random point at a specific cell
SDFSphere random_point(vec3 pixel, vec3 point_offset) {
    vec3 cell_coords = floor(pixel);
    vec3 point_coords = hash33(cell_coords);
    return SDFSphere(hash13(cell_coords) > 0.0, point_coords + point_offset);
}

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    // Dafuq happened here with the terrain gen compute?
    // Get the cell coordinates
    vec3 coords = pixel_coords / float(CELL_SIZE);
    vec3 cell_coords = floor(coords);
    float base_d = 99999;
    for(int x = -1; x < 2; x++) {
        for(int y = -1; y < 2; y++) {
            for(int z = -1; z < 2; z++) {
                // Check if the neighboring coordinates are at the min/max, and if they are, swap them
                vec3 neighbor_coords = coords + vec3(x, y, z); 
                // The see that will be used to get the random point
                vec3 k = mod(neighbor_coords, gl_NumWorkGroups.x / CELL_SIZE);
                // Get the neighboring points
                SDFSphere neighbor_point = random_point(k, floor(neighbor_coords));
                if (neighbor_point.enabled) {
                    float neighbor_d = distance(coords, neighbor_point.location) - SPHERE_SIZE;
                    base_d = min(base_d, neighbor_d);
                }
                // Keep track of the min distance
            }
        }
    }
    vec4 pixel = vec4(base_d, 0, 0, 0);
    // Write the pixel
    imageStore(sdf_tex, pixel_coords, pixel);
}