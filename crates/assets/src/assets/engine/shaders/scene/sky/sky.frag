#version 460 core
layout(location = 0) out vec4 frag;

#include <engine/shaders/common/scene.glsl>

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

// Sky gradient texture map
layout(set = 1, binding = 0) uniform texture2D gradient_map;
layout(set = 1, binding = 1) uniform sampler gradient_map_sampler;

void main() {
	// Calculate elevation
	vec3 normal = normalize(m_position);
	float y = normal.y;
	y = clamp(y, 0, 1);
	
	// Get background sky color based on elevation
	vec3 albedo = texture(sampler2D(gradient_map, gradient_map_sampler), vec2(0.5, 1-y)).rgb;

	// Create a procedural sun with the scene params
	float sun = dot(normal, -scene.sun_direction.xyz);
	sun = pow(max(sun, 0), 800) * 3;

	// Calculate sky color based on sun
	frag = vec4(albedo + vec3(sun), 1.0);
}