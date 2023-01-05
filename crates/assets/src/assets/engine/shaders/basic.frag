#version 460 core
layout(location = 0) out vec4 outColor;

// https://vkguide.dev/docs/chapter-2/triangle_walkthrough/
void main() {
	vec4 pos = gl_FragCoord;
	vec2 coords = pos.xy / vec2(1920, 1080);
	outColor = vec4(coords, 0, 0);
}