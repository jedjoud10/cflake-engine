#version 460 core
out vec3 color;
flat in vec3 normal;
uniform vec3 rgb;
void main() {
	float value = pow(max(dot(normal, normalize(vec3(1, 1, 1))), 0.0), 20.0);
	color = vec3(value, value, value);
}