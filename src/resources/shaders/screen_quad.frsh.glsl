#version 460 core
out vec3 color;
uniform sampler2D color_texture;
uniform sampler2D normals_texture;
uniform sampler2D tangents_texture;
uniform sampler2D uvs_texture;

in vec2 uv_coordinates;

void main() {
	color = texture(uvs_texture, uv_coordinates).xyz;
}