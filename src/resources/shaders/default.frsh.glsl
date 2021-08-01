#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
in vec3 m_normal;
void main() {
	float val = dot(m_normal, vec3(0, 1, 0));
	frag_color = vec3(val, val, val);
	frag_normal = m_normal;
}