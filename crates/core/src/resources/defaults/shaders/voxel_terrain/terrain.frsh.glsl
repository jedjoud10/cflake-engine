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
uniform float depth;
uniform vec3 tint;
uniform float normals_strength;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec3 m_color;
in vec2 m_uv;
void main() {
	frag_diffuse = vec3(1, 1, 1) * m_color * tint;
	frag_normal = m_normal;
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}