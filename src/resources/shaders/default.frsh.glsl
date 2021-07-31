#version 460 core
out vec3 color;
uniform vec3 rgb;
in vec3 debug_color;
void main() {
	color = debug_color;
}