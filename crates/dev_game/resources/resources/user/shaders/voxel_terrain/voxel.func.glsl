#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
void get_voxel(vec3 pos, int depth, out Voxel voxel, out MaterialVoxel material_voxel) {
    // Actual function for voxels
    // FBM Invertex billow noise with 8 octaves
    float fd = 0;
    for(int i = 0; i < 5; i++) {
        fd -= ((cellular(pos * vec3(1, 0.3, 1) * 0.001 * pow(2.0, i)).x)) * 200 * pow(0.5, i);
    }

    // Add the noise
    float density = pos.y + fd;

    // Make the terrain flatter
    //density = opSmoothUnion(density + 80, pos.y - 16.0, 30.0);
    density = max(density + 200, pos.y - 60);
    
    int shader_id = (snoise(pos * 0.001) > 0.5) ? 0 : 1;
    int texture_id = 0;
    int biome_id = 0;
    int hardness = 0;

    // Write the result
    voxel = Voxel(density * 20);
    material_voxel = MaterialVoxel(shader_id, texture_id, biome_id, hardness);
}
// Generate the Vertex Color, Smoothness, Metallic and Material ID
void get_color_voxel(vec3 pos, vec3 local_uv, Voxel voxel, MaterialVoxel material_voxel, int depth, out ColorVoxel color_voxel) {
    vec3 color = vec3(1, 1, 1);  
    color_voxel = ColorVoxel(color);
}
/*
// Get the detail data at a specific point3
void get_detail(vec3 pos, Voxel voxel, vec3 voxel_normal, ColorVoxel color, out Detail detail) {
    detail = Detail(ivec3(0, 0, 0), false, vec3(0, 0, 0), 1.0);
} 
*/