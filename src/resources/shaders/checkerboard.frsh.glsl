#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
void main() {
	float val = floor(m_position.x) + floor(m_position.z) + floor(m_position.y);
	val = mod(val, 2.0) == 0 ? 0 : 1;
	frag_diffuse = vec3(val, val, val);
	frag_normal = m_normal;
}