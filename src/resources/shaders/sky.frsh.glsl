#version 460 core
layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec3 frag_normal;
in vec3 normal;
in vec4 gl_FragCoord;
in vec3 m_normal;
void main() {
	frag_color = m_normal;
	frag_normal = m_normal;
}