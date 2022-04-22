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
in vec3 m_tangent;
in vec3 m_bitangent;
in vec2 m_uv;
in vec3 m_color;

void main() {
	// Load diffuse/emissive, and check if we must alpha clip
	vec4 diffuse = texture(diffuse_m, (m_uv) * uv_scale);
	vec4 emissive = texture(emissive_m, (m_uv) * uv_scale);
	float alpha1 = diffuse.a; 
	float alpha2 = emissive.a;
	if (alpha1 != 1 || alpha2 != 1) { discard; }

	// Color passthrough
	frag_diffuse = diffuse.xyz * m_color * tint;
	frag_emissive = emissive.xyz * emissivity;

	// Calculate tangent space normals and use that for bump mapping
	vec3 normal = texture(normal_m, m_uv * uv_scale).xyz * 2.0 - 1.0;
	normal.xy *= bumpiness;

	// Construct TBN matrix, then transform
	mat3 tbn = mat3(m_tangent, m_bitangent, m_normal);
	frag_normal = normalize(tbn * normal);
	
	// Other
	frag_pos = m_position;
}