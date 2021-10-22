#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
Voxel get_voxel(vec3 pos) {
    // Actual function for voxels
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
    return Voxel(density * 20.0, 0, material_id);
}
// Generate the Vertex Color, Smoothness, Metallic and Material ID
ColorVoxel get_color_voxel(vec3 pos, sampler3D voxel_texture, Voxel voxel, vec3 voxel_normal, vec3 coords) {
    vec3 color = vec3(1, 1, 1);  
    return ColorVoxel(color * voxel.materialID);
}
// Get the detail data at a specific point3
Detail get_detail(vec3 pos, Voxel voxel, vec3 voxel_normal, ColorVoxel color) {
    return Detail(ivec3(0, 0, 0), false, vec3(0, 0, 0), 1.0);
} 
