#version 460 core
layout(location = 1) out vec3 frag_normal;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
in vec3 normal;
in vec4 gl_FragCoord;
in vec3 m_normal;
in vec2 m_uv;
void main() {
	frag_emissive = vec3(0.95, 0.95, 0.95);
	frag_normal = vec3(0, 0, 0);
}