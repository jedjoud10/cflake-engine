// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
float voxel(vec3 position) {
    return (1 - fbmCellular(position * 0.002, int(7.0), 0.5, 2.1).y) * 300 + position.y;
}