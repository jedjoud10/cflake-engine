#version 460 core
layout(location = 0) out vec4 frag;
layout(location = 0) in vec3 m_position;

void main() {
	vec3 normal = normalize(m_position);
	frag = vec4(0);
}