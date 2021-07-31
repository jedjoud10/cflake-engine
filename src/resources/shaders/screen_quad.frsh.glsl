#version 460 core
out vec3 color;
uniform sampler2D color_texture;
in vec2 uv_coordinates;

void main() {
	color = texture(color_texture, uv_coordinates).xyz;
	color = vec3(1, 1, 1);
}