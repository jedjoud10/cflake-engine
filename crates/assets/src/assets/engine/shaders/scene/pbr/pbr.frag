#version 460 core

// G-Buffer data write
layout(location = 0) out vec4 gbuffer_position;
layout(location = 1) out vec4 gbuffer_albedo;
layout(location = 2) out vec4 gbuffer_normal;
layout(location = 3) out vec4 gbuffer_mask;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in vec3 m_tangent;
layout(location = 3) in vec3 m_bitangent;
layout(location = 4) in vec2 m_tex_coord;

// Push constants for the material data
layout(push_constant) uniform PushConstants {
	layout(offset = 64) float bumpiness;
    layout(offset = 68) float metallic;
    layout(offset = 72) float ambient_occlusion;
    layout(offset = 76) float roughness;
	layout(offset = 80) vec4 tint;
	layout(offset = 96) vec2 scale;
} material;

// Albedo / diffuse map
layout(set = 1, binding = 0) uniform texture2D albedo_map;
layout(set = 1, binding = 1) uniform sampler albedo_map_sampler;

// Normal map
layout(set = 1, binding = 2) uniform texture2D normal_map;
layout(set = 1, binding = 3) uniform sampler normal_map_sampler;

// Mask map
layout(set = 1, binding = 4) uniform texture2D mask_map;
layout(set = 1, binding = 5) uniform sampler mask_map_sampler;

void main() {
	// Certified moment
	vec2 uv = m_tex_coord;
	uv *= material.scale;

	// Fetch the albedo color, normal map value, and mask values
	vec3 albedo = texture(sampler2D(albedo_map, albedo_map_sampler), uv).rgb * material.tint.rgb;
	vec3 bumps = texture(sampler2D(normal_map, normal_map_sampler), uv).rgb * 2.0 - 1.0;
    vec3 mask = texture(sampler2D(mask_map, mask_map_sampler), uv).rgb;
    mask *= vec3(1.0, material.roughness, material.metallic);
	mask.r = clamp((mask.r - 0.5) * material.ambient_occlusion + 0.5, 0, 1);
	bumps.z = -sqrt(1 - (bumps.x*bumps.x + bumps.y*bumps.y));
	bumps.xy *= material.bumpiness;
	bumps.y = -bumps.y;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(-m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Clamp PBR values
	float roughness = clamp(mask.g, 0.01, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	float visibility = clamp(mask.r, 0.0, 1.0);

	// Set the G-buffer values
	gbuffer_position = vec4(m_position, 0);
	gbuffer_albedo = vec4(albedo, 1);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(mask, 0);
}