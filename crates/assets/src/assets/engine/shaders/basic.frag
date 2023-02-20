#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in vec3 m_tangent;
layout(location = 3) in vec3 m_bitangent;
layout(location = 4) in vec2 m_tex_coord;

// Camera, scene, and time shared objects
#include <engine/shaders/common/camera.glsl>
/*
//#include <engine/shaders/common/scene.glsl>
//#include <engine/shaders/common/timing.glsl>

// Material scalar data
layout(set = 1, binding = 1) uniform MaterialData {
	vec3 tint;
	float bumpiness;
} material;
*/

// Albedo / diffuse map
layout(set = 1, binding = 0) uniform texture2D albedo_map;
layout(set = 1, binding = 1) uniform sampler albedo_map_sampler;

// Normal map
layout(set = 1, binding = 2) uniform texture2D normal_map;
layout(set = 1, binding = 3) uniform sampler normal_map_sampler;

void main() {
	// Fetch the albedo color and normal map value
	vec4 albedo = texture(sampler2D(albedo_map, albedo_map_sampler), m_tex_coord).rgba;
	vec3 bumps = texture(sampler2D(normal_map, normal_map_sampler), m_tex_coord).rgb * 2.0 - 1.0;
	bumps.xz *= 2.0;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Do some basic light calculations
	vec3 direction = vec3(0, 1, 0);
	float lighting = dot(direction, normal);

	// Calculate diffuse lighting
	frag = lighting * albedo;
}