#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\custom_voronoi.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
void get_voxel(vec3 pos, int depth, out Voxel voxel, out MaterialVoxel material_voxel) {
    int material_id = 0;
    // Write the result
    voxel = Voxel(pos.y + -mountain(pos.xz * 0.008, 0.2).x * 800 + 400);
    material_voxel = MaterialVoxel(material_id);
}