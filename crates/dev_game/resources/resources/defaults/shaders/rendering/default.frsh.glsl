#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
uniform sampler2D normals_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
uniform vec3 tint;
uniform float normals_strength;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in vec3 m_color;
in vec2 screen_space_pos;
in mat3 tbn;
void main() {
	frag_diffuse = texture(diffuse_tex, m_uv * uv_scale).xyz * tint * m_color;
	vec3 tangent_space_normals = texture(normals_tex, m_uv * uv_scale).xyz * 2.0 - 1.0;
	tangent_space_normals.xy *= normals_strength;
	frag_normal = normalize(tbn * tangent_space_normals);
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}