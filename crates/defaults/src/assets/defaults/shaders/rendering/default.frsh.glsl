#version 460 core
#load renderer
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_emissive;
layout(location = 2) out vec3 frag_normal;
layout(location = 3) out vec3 frag_pos;
uniform sampler2D diffuse_m;
uniform sampler2D emissive_m;
uniform sampler2D normal_m;
uniform vec2 uv_scale;
uniform vec3 tint;
uniform float emissivity;
uniform float bumpiness;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in vec3 m_color;
in mat3 tbn;
void main() {
	vec4 texture_vals = texture(diffuse_m, (m_uv) * uv_scale); 
	vec4 emissive_vals = texture(emissive_m, (m_uv) * uv_scale); 
	// Alpha clipping
	if (texture_vals.a != 1 || emissive_vals.a != 1) { discard; }

	frag_emissive = emissive_vals.xyz * emissivity;
	frag_diffuse = texture_vals.xyz * m_color * tint;
	vec3 tangent_space_normals = texture(normal_m, (m_uv) * uv_scale).xyz * 2.0 - 1.0;
	tangent_space_normals.xy *= bumpiness;
	frag_normal = normalize(tbn * tangent_space_normals);
	frag_pos = m_position;
}