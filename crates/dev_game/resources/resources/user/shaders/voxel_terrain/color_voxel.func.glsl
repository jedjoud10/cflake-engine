#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the Vertex Color, Smoothness, Metallic and Material ID
ColorVoxel get_color_voxel(vec3 pos, sampler3D voxel_texture, uvec3 local_pos) {
    vec3 color = vec3(0.1, 0.1, 0.1);
    vec4 rgba = texture(voxel_texture, local_pos);
    int material_id = int(rgba.z * 255);
    return ColorVoxel(vec3(1000, 1000, 1000));
}