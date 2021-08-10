#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in mat3 tbn;
void main() {
	frag_diffuse = texture(diffuse_tex, m_uv * uv_scale).xyz;
	vec3 tangent_space_normals = texture(normal_tex, m_uv * uv_scale).xyz * 2.0 - 1.0;
	frag_normal = normalize(tbn * tangent_space_normals);
	frag_pos = m_position;
}