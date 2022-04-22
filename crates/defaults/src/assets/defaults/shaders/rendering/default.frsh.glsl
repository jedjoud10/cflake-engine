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
in vec3 test;

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

	vec3 n = m_normal;
	vec3 t = m_tangent;
	vec3 b = m_bitangent;

	mat3 tbn = mat3(
		normalize(t),
		normalize(b),
		normalize(n));
	/*
	frag_normal = normalize(tbn * normalize(normal));
	vec3 vNout = normalize(normal.x * m_tangent + normal.y * m_bitangent + normal.z * m_normal);
	*/
	frag_normal = normalize(tbn * normalize(normal));
	/*
	// Construct TBN matrix, then transform
	*/
	
	//frag_normal = -cross(normalize(m_bitangent), normalize(m_tangent));
	//frag_normal = normalize(m_normal);
	
	// Other
	frag_pos = m_position;
}