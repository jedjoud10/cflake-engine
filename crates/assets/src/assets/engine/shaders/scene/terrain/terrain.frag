#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

void main() {
	frag = vec4(m_position, 1.0);
}