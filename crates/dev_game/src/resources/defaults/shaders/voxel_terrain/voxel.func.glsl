#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
void get_voxel(vec3 pos, int depth, out Voxel voxel, out MaterialVoxel material_voxel) {
    // Actual function for voxels
    int shader_id = 0;
    int texture_id = 0;
    int biome_id = 0;
    int hardness = 0;
    float sphere = sdSphere(pos, 5);
    float box = sdBox(pos + vec3(60, 0, 0), vec3(10, 10, 10));
    float rbox = sdRoundBox(pos - vec3(60, 0, 0), vec3(10, 10, 10), 3);
    float p = pos.y - 0.5 + snoise(pos * 0.004) * 20;
    float d = min(sphere, min(box, min(rbox, p)));
    // Write the result
    voxel = Voxel(d * 300);
    material_voxel = MaterialVoxel(shader_id, texture_id, biome_id, hardness);
}
// Generate the Vertex Color, Smoothness, Metallic and Material ID
void get_color_voxel(vec3 pos, vec3 local_uv, Voxel voxel, MaterialVoxel material_voxel, int depth, out ColorVoxel color_voxel) {
    vec3 color = vec3(1, 1, 1);  
    color_voxel = ColorVoxel(color);
}