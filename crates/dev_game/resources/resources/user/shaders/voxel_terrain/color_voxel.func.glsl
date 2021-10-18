#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the Vertex Color, Smoothness, Metallic and Material ID
ColorVoxel get_color_voxel(vec3 pos, sampler3D voxel_texture, vec3 coords) {
    vec3 color = vec3(1, 1, 1);    
    vec4 rgba = texture(voxel_texture, coords);
    int material_id = int(rgba.w * 255);
    if (material_id == 1) {
        color = vec3(0, 0, 0);
    }
    return ColorVoxel(vec3(1, 1, 1));
}