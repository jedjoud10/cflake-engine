#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in vec3 m_tangent;
layout(location = 3) in vec3 m_bitangent;
layout(location = 4) in vec2 m_tex_coord;

// Camera, scene, and shadowmap shared objects
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/common/scene.glsl>
#include <engine/shaders/common/extensions.glsl>
#include <engine/shaders/common/shadow.glsl>
#include <engine/shaders/common/sky.glsl>
#include <engine/shaders/math/models.glsl>

// Shadow-map texture map
layout(set = 0, binding = 7) uniform texture2D shadow_map;

// Push constants for the material data
layout(push_constant) uniform PushConstants {
	layout(offset = 64) float bumpiness;
    layout(offset = 68) float metallic;
    layout(offset = 72) float ambient_occlusion;
    layout(offset = 76) float roughness;
	layout(offset = 80) vec4 tint;
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
	// Flip the Y coordinate (dunno why bruv)
	vec2 uv = m_tex_coord;
	uv.y = 1 - m_tex_coord.y;

	// Fetch the albedo color, normal map value, and mask values
	vec3 albedo = texture(sampler2D(albedo_map, albedo_map_sampler), uv).rgb * material.tint.rgb;
	vec3 bumps = texture(sampler2D(normal_map, normal_map_sampler), uv).rgb * 2.0 - 1.0;
    vec3 mask = texture(sampler2D(mask_map, mask_map_sampler), uv).rgb;
    mask *= vec3(1 / material.ambient_occlusion, material.roughness, material.metallic);
	bumps.xy *= material.bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Compute PBR values
	mask = pow(mask, vec3(2));
	float roughness = clamp(mask.g, 0.02, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	metallic = roughness;
	float visibility = clamp(mask.r, 0.0, 1.0);
	vec3 f0 = mix(vec3(0.04), albedo, metallic);

	// Create the data structs
	SunData sun = SunData(scene.sun_direction.xyz, scene.sun_color.rgb, 1.6);
	SurfaceData surface = SurfaceData(albedo, normal, m_position, roughness, metallic, visibility, f0);
	vec3 view = normalize(-camera.position.xyz + m_position);
	CameraData camera = CameraData(view, normalize(view + scene.sun_direction.xyz), camera.position.xyz);

	// Check if the fragment is shadowed
	vec3 color = brdf(shadow_map, surface, camera, sun);

	// Calculate diffuse lighting
	frag = vec4(color, 0.0);
}