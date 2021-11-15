#include "defaults\shaders\others\triplanar.func.glsl"
// Get the color at a specific fragment in the fragment shader of the terrain
void get_frag(int material_id, sampler2DArray diffuse_textures, sampler2DArray normals_textures, vec3 m_position, vec3 m_normal, vec2 uv_scale, float normals_strength, out vec3 frag_diffuse, out vec3 frag_normal) {
    triplanar_multitexture(diffuse_textures, normals_textures, 0, uv_scale, m_position, m_normal, normals_strength, 0.3, frag_diffuse, frag_normal);
}