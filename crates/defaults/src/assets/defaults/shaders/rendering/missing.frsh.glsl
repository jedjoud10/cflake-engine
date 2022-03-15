#version 460 core
#load renderer
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_emissive;
layout(location = 2) out vec3 frag_normal;
layout(location = 3) out vec3 frag_pos;
in vec3 m_position;
in vec3 m_normal;
void main() {
	frag_emissive = vec3(1, 0, 1);
	frag_diffuse = vec3(1, 0, 1);
	frag_normal = m_normal;
	frag_pos = m_position;
}