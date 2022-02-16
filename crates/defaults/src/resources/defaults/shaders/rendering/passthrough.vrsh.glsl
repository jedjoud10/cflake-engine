#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
layout(location = 2) in vec4 model_tangent;
layout(location = 3) in vec2 model_uv;
layout(location = 4) in vec3 model_color;
out vec2 uv_coordinates;

void main() {
	gl_Position = vec4(model_pos, 1);
	uv_coordinates = model_uv;
}