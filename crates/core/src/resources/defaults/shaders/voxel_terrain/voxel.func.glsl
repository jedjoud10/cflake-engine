#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\custom_voronoi.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
void get_voxel(vec3 pos, int depth, out Voxel voxel, out MaterialVoxel material_voxel) {
    int shader_id = 0;
    int material_id = 0;

    // Write the result
    voxel = Voxel(pos.y);
    material_voxel = MaterialVoxel(shader_id, material_id);
}