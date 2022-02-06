#version 460 core
layout(location = 0) in vec3 model_pos;
uniform mat4 lightspace_matrix;

void main() {
	gl_Position = lightspace_matrix * vec4(model_pos, 1.0);
}