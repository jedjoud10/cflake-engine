#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
in vec3 m_position;
in vec3 m_normal;
in vec2 screen_position;
uniform vec3 tint;
uniform vec3 view_pos;
void main() {
	// Create a checkerboard pattern
	float value = mod(floor(m_position.x*0.2) + floor(m_position.y*0.2) + floor(m_position.z*0.2), 2.0) == 0 ? 1.0 : 0.0;
	float d = abs(dot(m_normal, normalize(view_pos - m_position)));
	if (d > 0.2) { discard; }
	frag_diffuse = tint*2;
	// Some cool blending effect
	frag_normal = vec3(1, 1, 1);
	frag_pos = m_position;
}