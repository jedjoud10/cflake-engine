#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

// Sky gradient texture map
layout(set = 1, binding = 0) uniform texture2D gradient_map;
layout(set = 1, binding = 1) uniform sampler gradient_map_sampler;

void main() {
	// Calculate elevation
	float y = normalize(m_position).y;
	y = clamp(y, 0, 1);
	
	// Get background sky color based on elevation
	vec3 albedo = texture(sampler2D(gradient_map, gradient_map_sampler), vec2(0.5, 1-y)).rgb;

	// Calculate sky color based on sun
	frag = vec4(albedo, 1.0);
}