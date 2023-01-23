#version 460 core
//#include <engine/shaders/common/camera.glsl>

layout(push_constant) uniform MeshConstants {
	//mat4 model_matrix;
	float test;
	uint test2;
} constants;

// https://vkguide.dev/docs/chapter-2/triangle_walkthrough/
void main() {
    const vec2 positions[6] = vec2[6](
		vec2(1.0f, 1.0f),
		vec2(-1.0f, 1.0f),
		vec2(1.0f, -1.0f),

		vec2(-1.0f, -1.0f),
		vec2(1.0f, -1.0f), 
		vec2(-1.0f, 1.0f)
	);

	gl_Position = vec4(positions[gl_VertexIndex] + vec2(constants.test), 0.0f, 1.0f);

}