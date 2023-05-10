#version 460 core
layout(location = 0) out vec4 frag;

#include <engine/shaders/common/scene.glsl>
#include <engine/shaders/common/sky.glsl>

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

void main() {
	// Calculate elevation
	vec3 normal = normalize(m_position);
	
	// Get background sky color based on elevation
	vec3 albedo = calculate_sky_color(normal, scene.sun_direction.xyz);
	//vec3 albedo = texture(samplerCube(environment_map, environment_map_sampler), normal).rgb;

	// Create a procedural sun with the scene params
	float sun = dot(normal, -scene.sun_direction.xyz);
	float out_sun = pow(max(sun * 0.3, 0), 3) * 3;
	out_sun += pow(clamp(sun - 0.9968, 0, 1.0) * 250, 4) * 16;

	// Calculate sky color based on sun
	frag = vec4(albedo + vec3(out_sun), 1.0);
}