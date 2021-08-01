#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_tangent;
layout(location = 3) out vec2 frag_uv;
in vec3 m_normal;
in vec3 m_tangent;
in vec2 m_uv;
void main() {
	float val = dot(m_normal, vec3(0, 1, 0));
	frag_color = vec3(val, val, val);
	frag_normal = m_normal;
	frag_tangent = m_tangent;
	frag_uv = m_uv;
}