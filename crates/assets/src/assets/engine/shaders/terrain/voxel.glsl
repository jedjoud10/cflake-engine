// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
float voxel(vec3 position) {
    vec2 density1 = fbmCellular(position * 0.005 * vec3(1, 2, 1), 1, 0.4, 2.2) * 460;
    return (300 - density1.x) + position.y;
}