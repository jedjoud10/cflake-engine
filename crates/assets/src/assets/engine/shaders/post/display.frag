#version 460 core
layout(location = 0) out vec4 frag;

#include <engine/shaders/common/window.glsl>
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/math/conversions.glsl>

// Fetch the scene color data
layout(set = 1, binding = 0) uniform texture2D color_map;
layout(set = 1, binding = 1) uniform texture2D depth_map;

// Post processing settings
layout(set = 0, binding = 1) uniform PostProcessUniform {
    float exposure;
	float gamma;
	float vignette_strength;
	float vignette_size;
	uint tonemapping_mode;
	float tonemapping_strength;
} post_processing;


void main() {
	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	// Fetch the color data
	vec2 coords = vec2(x, y);
	vec3 color = texelFetch(color_map, ivec2(gl_FragCoord.xy), 0).rgb;

	// Increase exposure
	color *= post_processing.exposure;
	color = max(color, vec3(0));

	/*
	float non_linear_depth = texelFetch(depth_map, ivec2(gl_FragCoord.xy), 0).r;
	float depth = linearize_depth(non_linear_depth, 0.01, 5000);
	vec3 fog = mix(color, vec3(1), pow(clamp(depth / 100.0, 0, 1), 3));
	color += fog * (non_linear_depth > 0.99999 ? 0.0 : 1.0);
	*/

	// Apply tonemapping based on settings
	vec3 tonemapped = color;

	/*
	Reinhard,
	ReinhardJodie,
	ACES,
	Clamp,
	*/

	// Handle tonemapping mode
	switch(post_processing.tonemapping_mode) {
		case 0:
			tonemapped = reinhard(color);
			break;
		case 1:
			tonemapped = reinhard_jodie(color);
			break;
		case 2:
			tonemapped = aces(color);
			break;
		case 3:
			tonemapped = min(color, vec3(1));
			break;
	}
	
	// Apply gamma correction
	tonemapped = mix(color, tonemapped, post_processing.tonemapping_strength);
	color = pow(tonemapped, vec3(1.0 / post_processing.gamma));

	// Create a simple vignette
	vec2 uv = vec2(x, y);
	float vignette = length(abs(uv - 0.5));
	vignette += post_processing.vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, 4.0) * clamp(post_processing.vignette_strength, 0.0, 2.0);
	color = mix(color, vec3(0), vignette);
	frag = vec4(color, 0);
}