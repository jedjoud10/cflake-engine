#version 460 core
#include <engine/shaders/common/camera.glsl>

// Vertex attributes from Rust
layout (location = 0) in vec3 vertex_position;

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

	gl_Position = vec4(positions[gl_VertexIndex], 0.0f, 1.0f);

}