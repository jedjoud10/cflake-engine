#version 460 core
out vec3 out_color;
in vec2 uvs;
uniform vec3 color;
void main() {
	out_color = vec3(uvs, 0);
}