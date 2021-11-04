#include "defaults\shaders\others\triplanar.func.glsl"
// Get the color at a specific fragment in the fragment shader of the terrain
void get_frag(vec3 m_position, vec3 m_normal, vec2 uv_scale, float normals_strength, out vec3 frag_diffuse, out vec3 frag_normal) {
    frag_diffuse = vec3(1, 1, 1);
    frag_normal = m_normal;
}