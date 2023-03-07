#version 460 core
layout(location = 0) out vec4 frag;

#include <engine/shaders/common/extensions.glsl>
#include <engine/shaders/common/window.glsl>
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/math/conversions.glsl>

// Fetch the scene color data
layout(set = 1, binding = 0) uniform texture2D color_map;
layout(set = 1, binding = 1) uniform sampler color_map_sampler;

// Fetch the scene depth data
layout(set = 1, binding = 2) uniform texture2D depth_map;
layout(set = 1, binding = 3) uniform sampler depth_map_sampler;

void main() {
	// Get the scaled down coordinates
	float x = gl_FragCoord.x / float(window.width);
	float y = gl_FragCoord.y / float(window.height);

	// Fetch the color data
	vec2 coords = vec2(x, y);
	vec3 color = texture(sampler2D(color_map, color_map_sampler), coords).rgb;

	// Fetch the depth data
	float non_linear_depth = texelFetch(depth_map, ivec2(gl_FragCoord.xy), 0).r;
	float depth = linearize_depth(non_linear_depth, 0.01, 5000);

	// Increase exposure
	color *= 1.2;

	// Apply tonemapping and gamma mapping
	color = pow(aces(color), vec3(1.0 / 2.2));

	// Create a simple vignette
	float vignette_size = 0.1;
	float vignette_strength = 1.2;
	vec2 uv = vec2(x, y);
	float vignette = length(abs(uv - 0.5));
	vignette += vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, 4.0) * clamp(vignette_strength, 0.0, 2.0);
	color = mix(color, vec3(0), vignette);

	frag = vec4(color, 0);
}