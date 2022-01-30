#version 460 core
#load renderer
#load general
#include "defaults\shaders\voxel_terrain\terrain_shader.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_emissive;
layout(location = 2) out vec3 frag_normal;
layout(location = 3) out vec3 frag_pos;
uniform sampler2DArray diffuse_tex;
uniform sampler2DArray emissive_tex;
uniform sampler2DArray normals_tex;
uniform vec2 uv_scale;
uniform vec3 tint;
uniform float emissive_strength;
uniform float normals_strength;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec3 m_color;
in vec2 m_uv;
void main() {
	vec3 normal;
	vec3 diffuse;
	get_frag(diffuse_tex, normals_tex, m_position, m_normal, uv_scale, normals_strength, diffuse, normal);
	frag_diffuse = vec3(1, 1, 1) * m_color;
	frag_normal = m_normal;
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}