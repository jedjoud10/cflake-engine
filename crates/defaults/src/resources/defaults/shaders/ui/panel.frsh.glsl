#version 460 core
#load general
out vec4 out_color;
in vec2 v_texture_uvs;
in vec4 v_color;
void main() {
	if (v_color.a <= 0.5) { discard; }
	out_color = vec4(v_texture_uvs, 0.0, 1.0);
}