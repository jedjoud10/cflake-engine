#version 460 core

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