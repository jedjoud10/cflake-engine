#version 460 core
out vec3 color;
in vec2 uvs;
void main() {
	color = vec3(uvs, 0);
}