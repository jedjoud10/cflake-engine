// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
float voxel(vec3 position, out uint material) {
    float biome1 = fbm(position.xz * 0.02, 3, 2.0, 0.4) * 5 + position.y;
    vec3 test = erosion(position.xz * 0.002, 0.2);

    float blend = clamp(snoise(position.xz * 0.0002) * 0.5 + 0.5, 0, 1);

    float biome2 = test.x * -1200 + position.y;
    float up = abs(test.y) + abs(test.z);
    uint material2 = 0;
    if (up < 0.4) {
        material2 = 1;
    } else if (up < 0.8) {
        material2 = 2;
    } else {
        material2 = 0;
    }

    uint material3 = 2;
    float rocky = (1 - fbmCellular(position * 0.01 * vec3(1.3, 3, 1), 3, 2.4, 0.5).x) * 220 + position.y - 700;

    float density = mix(biome2, rocky, blend);

    if (blend < 0.5 + snoise(position * 0.005) * 0.4) {
        material = material2;
    } else {
        material = material3;
    }
    
    return density;
}