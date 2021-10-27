#version 460 core
layout(location = 1) out vec3 frag_normal;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
uniform float time;
uniform vec3 light_dir;
in vec3 normal;
in vec3 m_normal;
in vec2 m_uv;
void main() {
	// Use the diffuse texture as the sky gradient
	float light_val = (dot(m_normal, vec3(0, 1, 0)));
	vec3 color = texture(diffuse_tex, vec2(1.0, light_val)).xyz;
	frag_emissive = dot(m_normal, normalize(light_dir)) * vec3(1, 1, 1);
	frag_normal = vec3(0, 0, 0);
}