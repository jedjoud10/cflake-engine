#version 460 core
layout(location = 0) out vec4 frag;

// Window bind group buffer (creates a 'window' object)
#include <engine/shaders/common/window.glsl>

// Fetch the scene color data
layout(set = 0, binding = 0) uniform texture2D color_map;
layout(set = 0, binding = 1) uniform sampler color_map_sampler;

void main() {
	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	// Fetch the color data
	vec2 coords = vec2(x, y);
	vec3 color = texture(sampler2D(color_map, color_map_sampler), coords).rgb;
	color = pow(color, vec3(1.0 / 2.2));
	frag = vec4(color, 0);
}