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

	// Calculate sky color based on sun
	frag = vec4(albedo, 1.0);
}