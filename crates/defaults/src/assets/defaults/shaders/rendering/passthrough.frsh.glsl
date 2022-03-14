#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 2) out vec3 frag_emissive;
layout(location = 3) out vec3 frag_normal;
layout(location = 4) out vec3 frag_pos;
uniform sampler2D diffuse_m;
uniform sampler2D normal_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in mat3 tbn;
void main() {
	frag_diffuse = texture(diffuse_m, m_uv * uv_scale).xyz;
	frag_emissive = frag_diffuse;
	frag_normal = m_normal;
	frag_pos = m_position;
}