#version 460 core

// Load the PBR shading function and basic renderer code
#load pbr
#load model

// Pixel color
out vec4 frag_color;

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
in VertexData vert;

void main() {
	// Decompose
	vec3 position = vert.position;
	vec3 normal = vert.normal;
	vec3 tangent = vert.tangent;
	vec3 bitangent = vert.bitangent;
	vec2 uv = vert.uv;
	vec3 color = vert.color;

	// Load diffuse/emissive, and check if we must alpha clip
	vec2 tex_coords = uv * uv_scale;
	vec4 diffuse = texture(diffuse_m, tex_coords);
	vec4 emissive = texture(emissive_m, tex_coords);
	float alpha1 = diffuse.a; 
	float alpha2 = emissive.a;
	if (alpha1 != 1 || alpha2 != 1) { discard; }

	// Calculate tangent space normals and use that for bump mapping
	normal = texture(normal_m, tex_coords).xyz * 2.0 - 1.0;
	normal.xy *= bumpiness;

	// Transform the tangent space normal into world space using a TBN matrix
	mat3 tbn = mat3(
		normalize(tangent),
		normalize(bitangent),
		normalize(normal));
	normal = normalize(tbn * normalize(normal));	
	
	// This is a certified PBR moment
	vec3 mask = texture(mask_m, tex_coords).rgb;
	mask.r = pow(mask.r, ao_strength);
	mask.g *= roughness;
	mask.b *= metallic;

	// Apply shading
	SunData sun = SunData(vec3(1, 0, 0), 5.0, vec3(1));
	PixelData pixel = PixelData(diffuse.rgb, normal, emissive.rgb, position, mask.r, mask.g, mask.b, 0.0);

	// PBR moment
	const float gamma = 2.2;
    vec3 mapped = shade_pbr(sun, pixel);
    // gamma correction 
    mapped = pow(mapped, vec3(1.0 / gamma));
	frag_color = vec4(mapped, 1);
}