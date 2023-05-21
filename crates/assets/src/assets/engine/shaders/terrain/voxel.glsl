// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>
    
// A voxel is a 3D pixel in the world that can contain multiple parameters
// Each voxel contains a "density". 
// Density allows us to represent either full terrain or air, and everything in between
// Main voxel function that will create the shape of the terrain
float smooth_floor(float x) {
    return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
}

float voxel(vec3 position, float quality) {
    position *= 0.5;
    float density = smooth_floor(position.y / 100) * 100 + (1-fbmCellular(position * 0.002 * vec3(1, 0.1, 1), uint(10.0 * quality), 0.3, 2.1).x) * 440;
    return density; 
}

// Post-process voxel step that gets executed after we generate the main voxel texture
void post(vec3 position, inout vec3 normal, int lod) {
}

// Terrain detail are basically props that we can generate on top of the terrain at close distances
struct Detail {
    vec3 offset;
    vec3 rotation;
    float scale;
    uint type;
    bool spawn;
};

// Checks if we should generate a detail at a specific vertex point
Detail detail(vec3 position, vec3 normal) {
    return Detail(vec3(0), vec3(0), 1.0, 0, false);
}