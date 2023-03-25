#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;

void main() {
	vec3 normal = normalize(-m_normal);
	//vec3 normal = cross(dFdy(m_position), dFdx(m_position));
	float value = clamp(dot(normalize(normal), vec3(0, 1, 0)), 0, 1);
	frag = vec4(value);
}