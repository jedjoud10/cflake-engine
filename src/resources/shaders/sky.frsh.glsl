#version 460 core
layout(location = 1) out vec3 frag_normal;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
in vec3 normal;
in vec4 gl_FragCoord;
in vec3 m_normal;
in vec2 m_uv;
void main() {
	// Use the diffuse texture as the sky gradient
	float light_dir = dot(m_normal, vec3(0, 1, 0)) / 2 + 1;
	vec3 color = texture(diffuse_tex, vec2(0, light_dir)).xyz;
	frag_emissive = color;
	frag_normal = vec3(0, 0, 0);
}