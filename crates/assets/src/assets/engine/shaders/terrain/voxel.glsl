// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main voxel function that will create the shape of the terrain
float voxel(vec3 position) {
    return position.y + snoise(position * 0.006) * 300;
}