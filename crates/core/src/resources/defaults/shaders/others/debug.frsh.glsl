#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
in vec3 m_position;
in vec3 m_normal;
in vec2 screen_position;
uniform vec3 tint;
void main() {
	// Create a checkerboard pattern
	float value = mod(floor(screen_position.x) + floor(screen_position.y), 2.0) == 0 ? 1.0 : 0.0;
	frag_diffuse = tint*99.0* value;
	// 
	frag_normal = vec3(1, 1, 1);
	frag_pos = m_position;
}