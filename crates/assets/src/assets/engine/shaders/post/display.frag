#version 460 core
layout(location = 0) out vec4 frag;

// Include common GLSL extensions
#include <engine/shaders/common/extensions.glsl>

// Window bind group buffer (creates a 'window' object)
#include <engine/shaders/common/window.glsl>

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>

// Fetch the scene color data
layout(set = 1, binding = 0) uniform texture2D color_map;
layout(set = 1, binding = 1) uniform sampler color_map_sampler;

// Fetch the scene depth data
layout(set = 1, binding = 2) uniform texture2D depth_map;
layout(set = 1, binding = 3) uniform sampler depth_map_sampler;

// Turn the 0-1 depth value to the zNear - zFar range
float linearize_depth(float d,float zNear,float zFar)
{
	d = 2.0 * d - 1.0;
    return zNear * zFar / (zFar + d * (zNear - zFar));
}

// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

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

	// Le fog
	color = mix(color, vec3(1), depth * 0.01);

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