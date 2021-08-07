#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_tangent;
layout(location = 3) out vec2 frag_uv;
uniform sampler2D diffuse_tex;
in vec3 m_normal;
in vec3 m_tangent;
in vec2 m_uv;
void main() {
	float val = dot(m_normal, vec3(0, 1, 0));
	frag_color = texture(diffuse_tex, m_uv * 10.0).xyz;
	frag_normal = m_normal;
	frag_tangent = m_tangent;
	frag_uv = m_uv;
}