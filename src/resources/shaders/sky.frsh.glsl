#version 460 core
out vec3 color;
in vec3 normal;
in vec4 gl_FragCoord;
void main() {
	color = normal;
}