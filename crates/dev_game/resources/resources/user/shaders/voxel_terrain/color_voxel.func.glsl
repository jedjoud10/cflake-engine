#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the Vertex Color, Smoothness, Metallic and Material ID
ColorVoxel get_color_voxel(vec3 pos, sampler3D voxel_texture, uvec3 local_pos) {
    vec3 color = vec3(1, 1, 1);    
    vec4 rgba = texture(voxel_texture, vec3(local_pos) / 34);
    float material_id = rgba.w;
    return ColorVoxel(color * material_id);
}