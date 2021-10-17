#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the Vertex Color, Smoothness, Metallic and Material ID
ColorVoxel get_color_voxel(vec3 pos) {
    return ColorVoxel(pos);
}