#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
layout(location = 3) out vec3 frag_emissive;
in vec3 m_position;
uniform float depth_level;
void main() {
	frag_diffuse = vec3(depth_level, depth_level, depth_level);
	frag_normal = vec3(1, 1, 1);
	frag_pos = m_position;
	frag_emissive = vec3(depth_level, depth_level, depth_level);
}