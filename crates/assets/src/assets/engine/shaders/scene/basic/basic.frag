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

// Sky gradient texture map
layout(set = 0, binding = 5) uniform texture2D gradient_map;
layout(set = 0, binding = 6) uniform sampler gradient_map_sampler;

// Shadow-map texture map
layout(set = 0, binding = 7) uniform texture2D shadow_map;

// Material scalar data

// Push constants for the material data
layout(push_constant) uniform PushConstants {
	layout(offset = 64) vec3 tint;
	layout(offset = 76) float bumpiness;
} material;

// Albedo / diffuse map
layout(set = 1, binding = 0) uniform texture2D albedo_map;
layout(set = 1, binding = 1) uniform sampler albedo_map_sampler;

// Normal map
layout(set = 1, binding = 2) uniform texture2D normal_map;
layout(set = 1, binding = 3) uniform sampler normal_map_sampler;

void main() {
	// Flip the Y coordinate (dunno why bruv)
	vec2 uv = m_tex_coord;
	uv.y = 1 - m_tex_coord.y;

	// Fetch the albedo color and normal map value
	vec3 albedo = texture(sampler2D(albedo_map, albedo_map_sampler), uv).rgb;
	vec3 bumps = texture(sampler2D(normal_map, normal_map_sampler), uv).rgb * 2.0 - 1.0;
	bumps.xy *= material.bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Calculate ambient sky color
	vec3 ambient = calculate_sky_color(normal, scene.sun_direction.xyz);

	// Calculate light dir 
	vec3 light = normalize(-scene.sun_direction.xyz);
	
	// Check if the fragment is shadowed
	float shadowed = calculate_shadowed(m_position, shadow_map, shadow.lightspace, shadow.strength, shadow.spread, shadow.size);
	
	// Basic dot product light calculation
	float value = clamp(dot(light, normal), 0, 1) * (1-shadowed);
	vec3 lighting = value + (ambient * 0.5 + vec3(0.08)); 

	// Calculate specular reflections
	vec3 view = normalize(camera.position.xyz - m_position);
	vec3 reflected = reflect(-light, normal);
	float specular = pow(max(dot(reflected, view), 0), 256) * (1-shadowed);

	// Calculate diffuse lighting
	frag = vec4(lighting * albedo * material.tint + specular*1.4, 1.0);
}