#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
void get_voxel(vec3 pos, out Voxel voxel, out MaterialVoxel material_voxel) {
    // Actual function for voxels
    // FBM Invertex billow noise with 8 octaves
    float fd = 0;
    for(int i = 0; i < 8; i++) {
        fd -= (1-abs(snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)))) * 100 * pow(0.43, i);
    }

    // Add the noise
    float density = pos.y + fd;

    // Make the terrain flatter
    density = opSmoothUnion(density + 80, pos.y - 16.0, 30.0);
    density = max(density, pos.y - 60);
    //density = abs(density) - 30;
    
    int material_id = 0;
    if (snoise(pos * 0.002) > -0.5) {
        material_id = 1;
    }
    int biome_id = 0;
    int hardness = 0;
    int texture_id = 0;

    // Write the result
    voxel = Voxel(density * 20);
    material_voxel = MaterialVoxel(material_id, biome_id, hardness, texture_id);
}
// Generate the Vertex Color, Smoothness, Metallic and Material ID
void get_color_voxel(vec3 pos, Voxel voxel, MaterialVoxel material_voxel, out ColorVoxel color_voxel) {
    vec3 color = vec3(1, 1, 1);  
    color_voxel = ColorVoxel(color * material_voxel.material_id);
}
/*
// Get the detail data at a specific point3
void get_detail(vec3 pos, Voxel voxel, vec3 voxel_normal, ColorVoxel color, out Detail detail) {
    detail = Detail(ivec3(0, 0, 0), false, vec3(0, 0, 0), 1.0);
} 
*/