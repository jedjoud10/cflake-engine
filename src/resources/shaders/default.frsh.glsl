#version 460 core
out vec3 color;
uniform vec3 rgb;
in vec3 debug_color;
in vec3 normal;
void main() {
	float val = dot(normal, vec3(0, 1, 0));
	color = debug_color;
}