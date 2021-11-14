// My custom implementation of Worley (Voronoi noise)
#include "defaults\shaders\others\hashes.func.glsl"
// Get a random sphere location using a cell coordinate
vec3 get_sphere_location(ivec3 cell_coord) {
    vec3 location = hash33(cell_coord);
    return location + cell_coord;
}
// My own implementation of cellular noise
// X: F1 distance
// Y: F2 distance
// Z: cellular value
vec3 voronoi(vec3 pos) {
    // Gotta remap this since it differs from normal simplex noise too much
    pos *= 7.0;
    ivec3 cell_coord = ivec3(floor(pos)); 
    float min_distance = 99999;
    vec3 value = vec3(1, 1, 1);
    for(int x = -1; x < 2; x++) {
        for(int y = -1; y < 2; y++) {
            for(int z = -1; z < 2; z++) {
                // Keep track of the minimum distance
                ivec3 neighbor_coord = cell_coord + ivec3(x, y, z);
                float d = distance(get_sphere_location(neighbor_coord), pos);
                if (d < min_distance) {
                    min_distance = d;
                    value = vec3((d/2.23) * 2.0 - 1.0, 0.0, hash13(neighbor_coord));
                }
            }
        }
    }
    return value;
}