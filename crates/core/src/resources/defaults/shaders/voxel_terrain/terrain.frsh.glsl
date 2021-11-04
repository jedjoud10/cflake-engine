#version 460 core
#include "defaults\shaders\voxel_terrain\material.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
uniform sampler2D normals_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
uniform vec3 tint;
uniform float normals_strength;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec3 m_color;
in vec2 m_uv;
void main() {
	vec3 normal;
	vec3 diffuse;
	get_frag(m_position, m_normal, uv_scale, normals_strength, diffuse, normal);
	frag_diffuse = diffuse * m_color * tint;
	frag_normal = normal;
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}