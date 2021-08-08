#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;
uniform vec3 view_pos;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in mat3 tbn;
void main() {
	float val = dot(m_normal, vec3(0, 1, 0));
	frag_color = texture(diffuse_tex, m_uv).xyz;
	vec3 tangent_space_normals = texture(normal_tex, m_uv).xyz * 2.0 - 1.0;
	frag_normal = normalize(tbn * tangent_space_normals);
	frag_pos = m_position;
}