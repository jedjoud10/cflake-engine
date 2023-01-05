#version 460 core
#include "engine/shaders/test.glsl"
layout(location = 0) out vec4 outColor;

// https://vkguide.dev/docs/chapter-2/triangle_walkthrough/
void main() {
	vec4 pos = gl_FragCoord;
	vec2 coord = pos.xy / vec2(1920, 1080);
	coord.y = 1 - coord.y;
	coord = coord * 2 - 1.0;
	vec3 ray = normalize(vec3(coord, 0.8));

	vec3 point = ray + vec3(0, 1, -5);
	vec3 col = vec3(0);
	float last_value = sdf(point);
	const float EPSILON = 0.01;

	for (int i = 0; i < 256; i++) {
		point += ray * last_value;		

		last_value = sdf(point);
		if (last_value < EPSILON) {
			col = lighting(normal(point, EPSILON));
			break;
		}
	}

	outColor = vec4(col, 0);
}