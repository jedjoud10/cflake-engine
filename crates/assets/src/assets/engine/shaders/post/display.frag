#version 460 core
layout(location = 0) out vec4 frag;

#include <engine/shaders/common/window.glsl>
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/math/conversions.glsl>

// Fetch the scene color data
layout(set = 1, binding = 0) uniform texture2D color_map;
layout(set = 1, binding = 1) uniform texture2D depth_map;

void main() {
	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	// Fetch the color data
	vec2 coords = vec2(x, y);
	vec3 color = texelFetch(color_map, ivec2(gl_FragCoord.xy), 0).rgb;

	// Increase exposure
	color *= 2.0;
	color = max(color, vec3(0));

	/*
	float non_linear_depth = texelFetch(depth_map, ivec2(gl_FragCoord.xy), 0).r;
	float depth = linearize_depth(non_linear_depth, 0.01, 5000);
	vec3 fog = mix(color, vec3(1), pow(clamp(depth / 100.0, 0, 1), 3));
	color += fog * (non_linear_depth > 0.99999 ? 0.0 : 1.0);
	*/

	// Apply tonemapping and gamma mapping
	//color = pow(color, vec3(1.0 / 2.2));
	color = pow(aces(color), vec3(1.0 / 2.2));

	// Create a simple vignette
	float vignette_size = 0.1;
	float vignette_strength = 0.4;
	vec2 uv = vec2(x, y);
	float vignette = length(abs(uv - 0.5));
	vignette += vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, 4.0) * clamp(vignette_strength, 0.0, 2.0);
	color = mix(color, vec3(0), vignette);
	frag = vec4(color, 0);
}