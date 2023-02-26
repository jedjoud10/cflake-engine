#version 460 core
layout(location = 0) out vec4 frag;

// Window bind group buffer (creates a 'window' object)
#include <engine/shaders/common/window.glsl>

// Fetch the scene color data
layout(set = 0, binding = 0) uniform texture2D color_map;
layout(set = 0, binding = 1) uniform sampler color_map_sampler;

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
	color *= 1.0;
	color = pow(aces(color), vec3(1.0 / 2.2));
	frag = vec4(color, 0);
}