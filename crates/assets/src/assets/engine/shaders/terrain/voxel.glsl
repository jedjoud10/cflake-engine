// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
float voxel(vec3 position, out uint material) {
    float first = (1 - fbmCellular(position * 0.001, int(4.0), 0.5, 2.1).y) * 600 + position.y;
    return fbm(position * 0.001, 9, 0.4, 2.1) * 200.0 + position.y;
}