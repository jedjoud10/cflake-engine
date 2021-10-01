#version 460 core
out vec4 out_color;
in vec2 uvs;
uniform vec4 color;
void main() {
	out_color = vec4(uvs, 0, 1);
}