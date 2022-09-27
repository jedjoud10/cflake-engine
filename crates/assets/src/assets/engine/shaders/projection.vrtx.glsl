#version 460 core
layout(location = 0) in vec3 position;
uniform mat4 matrix;
out vec3 l_position;

void main() {
    l_position = position;
    vec4 projected = matrix * vec4(position, 1);
	gl_Position = projected;
}