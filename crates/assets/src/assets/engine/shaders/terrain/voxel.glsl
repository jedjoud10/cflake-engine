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
// quality: Integer between 0-4, 4 being best quality and 0 being worse
    float smooth_floor(float x) {
        return x - (sin(2 * 3.1415 * x) / (2 * 3.1415));
    }

float voxel(vec3 position, uint quality) {
    //return position.y;
    //return position.y + (1-fbmCellular(position * 0.002 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 440;
    return position.y + snoise(position * 0.01) * 20;
    /*
    position *= 0.1;

    float density = 0.0;
    float d2 = (1-fbmCellular(position * 0.008 * vec3(1, 0.1, 1), 8, 0.3, 2.1).x) * 140;
    d2 = smooth_floor(d2 / 50) * 50;

    d2 += position.y;
    d2 = opSmoothUnion(d2, position.y + 140, 10);
    d2 = opSmoothSubtraction(-d2, position.y + 100, 50);
    density += d2 - 140;
    density = opUnion(density, sdSphere(position, 20));
    return density;
    */

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