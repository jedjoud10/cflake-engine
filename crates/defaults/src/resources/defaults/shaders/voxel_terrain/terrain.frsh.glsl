#version 460 core
#load renderer
#load general
#include "defaults\shaders\others\triplanar.func.glsl"
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
in flat uint m_material_type;
void main() {
	// Triplanar settings
	TriplanarSettings settings = TriplanarSettings(vec2(1, 1) * 0.02, 0.3);

	frag_diffuse = vec3(1, 1, 1) * m_color * 0.8;
	frag_diffuse = array_triplanar(diffuse_tex, m_position, m_normal, int(m_material_type), settings);
	frag_normal = array_triplanar_normal(normals_tex, m_position, m_normal, int(m_material_type), 1.0, settings);
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}