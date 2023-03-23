#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

void main() {
	vec3 normal = cross(dFdy(m_position), dFdx(m_position));
	//float lighting = dot(normal, vec3(0, 1, 0));
	frag = vec4(normalize(normal), 1.0);
}