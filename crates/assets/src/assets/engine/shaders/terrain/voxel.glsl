// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
// Negative values represent terrain, positive values represent air
float voxel(vec3 position, out uint material) {
    float biome1 = fbm(position.xz * 0.02, 3, 2.0, 0.5) * 1 + position.y;
    float blend = clamp(snoise(position.xz * 0.0002) * 0.5 + 0.5, 0, 1);
    blend = clamp((blend - 0.5) * 1 + 0.5, 0, 1);

    float up = 0.0;
    uint material2 = 0;

    uint material3 = 1;
    float rocky = position.y + fbmCellular(position * 0.001 * vec3(1, 1.5, 1), 4, 0.7, 3.0).y * -330 - 50;
    float density = mix(biome1, rocky, blend);

    if (blend < 0.5 - cellular(position * 0.008).x * 0.3) {
        material = material2;
    } else {
        material = material3;
    }
    
    return max(-density, -sdSphere(position, 10));
}