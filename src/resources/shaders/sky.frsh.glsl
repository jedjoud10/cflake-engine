#version 460 core
out vec3 color;
in vec3 normal;
void main() {
	color = normal;
}