#version 460 core

// Load the PBR shading function and basic renderer code
#load pbr

// Pixel color
out vec3 frag_color;

// Texture maps
uniform sampler2D diffuse_m;
uniform sampler2D emissive_m;
uniform sampler2D normal_m;
uniform sampler2D mask_m;

// Attributes
uniform vec2 uv_scale;
uniform vec3 tint;
uniform float emissivity;
uniform float bumpiness;
uniform float roughness;
uniform float metallic;
uniform float ao_strength;

// Data given from the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec2 m_uv;
in vec3 m_color;

void main() {
	// Load diffuse/emissive, and check if we must alpha clip
	vec2 uv = m_uv * uv_scale;
	vec4 diffuse = texture(diffuse_m, uv);
	vec4 emissive = texture(emissive_m, uv);
	float alpha1 = diffuse.a; 
	float alpha2 = emissive.a;
	if (alpha1 != 1 || alpha2 != 1) { discard; }

	// Calculate tangent space normals and use that for bump mapping
	vec3 normal = texture(normal_m, uv).xyz * 2.0 - 1.0;
	normal.xy *= bumpiness;

	// Transform the tangent space normal into world space using a TBN matrix
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	normal = normalize(tbn * normalize(normal));	
	
	// This is a certified PBR moment
	vec3 mask = texture(mask_m, uv).rgb;
	mask.r = pow(mask.r, ao_strength);
	mask.g *= roughness;
	mask.b *= metallic;

	// Apply shading
	SunData sun = SunData(vec3(1, 0, 0), 5.0, vec3(1));
	PixelData pixel = PixelData(diffuse.rgb, normal, emissive.rgb, m_position, mask.r, mask.g, mask.b, 0.0);

	// PBR moment
	frag_color = compute_lighting_pbr(sun, pixel);
}