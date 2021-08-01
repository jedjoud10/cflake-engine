#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_tangent;
layout(location = 3) out vec2 frag_uv;
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec2 m_uv;
void main() {
	float val = floor(m_position.x) + floor(m_position.z) + floor(m_position.y);
	val = mod(val, 2.0) == 0 ? 0 : 1;
	frag_color = vec3(val, val, val);
	frag_normal = m_normal;
	frag_tangent = m_tangent;
	frag_uv = m_uv;
}