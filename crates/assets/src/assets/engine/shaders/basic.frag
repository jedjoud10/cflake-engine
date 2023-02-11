#version 460 core
//#include <engine/shaders/bindless/samplers.glsl>
/*
layout(push_constant) uniform MeshConstants {
	uint albedo_texture;
	uint normal_texture;

	float roughness;
	vec4 color;
} constants;

layout(set = 1, binding = 0) uniform BasicMaterial {
    
} material;
*/
//layout(location = 1) in int SSAO_KERNEL_SIZE;
//layout(constant_id = 0) const int suckmeoff = 0;
layout(location = 0) out vec4 outColor;

// https://vkguide.dev/docs/chapter-2/triangle_walkthrough/
void main() {
	vec4 pos = gl_FragCoord;
	vec2 coords = pos.xy / vec2(1920, 1080);
	outColor = vec4(coords, 0, 0);
}