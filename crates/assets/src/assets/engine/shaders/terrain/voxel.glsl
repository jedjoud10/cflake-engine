// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
// Negative values represent terrain, positive values represent air
float voxel(vec3 position, out uint material) {
    float biome1 = fbm(position.xz * 0.02, 3, 2.0, 0.4) * 5 + position.y;
    vec3 test = erosion(position.xz * 0.002, 0.2);

    float blend = clamp(snoise(position.xz * 0.0002) * 0.5 + 0.5, 0, 1);

    float biome2 = test.x * -1200 + position.y;
    float up = abs(test.y) + abs(test.z);
    uint material2 = 0;
    if (up < 0.68) {
        material2 = 1;
    } else if (up < 0.8) {
        material2 = 2;
    } else {
        material2 = 0;
    }

    uint material3 = 2;
    float rocky = position.y + fbmCellular(position * 0.001, 4, 0.7, 3.0).y * -90 + 10;
    rocky = opSmoothUnion(rocky, position.y - 40, 10);
    float density = mix(biome2, rocky, blend);

    if (blend < 0.5 - cellular(position * 0.008).x * 0.1) {
        material = material2;
    } else {
        material = material3;
    }
    
    return -density;
}