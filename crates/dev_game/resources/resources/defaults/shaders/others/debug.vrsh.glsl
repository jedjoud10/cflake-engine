#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_color;
uniform mat4 vp_matrix;
out vec3 m_position;
out vec3 m_color;
void main() {
	vec4 vp_pos = vp_matrix * vec4(model_pos, 1.0);
	gl_Position = vp_pos;

	// Pass the data to the next shader
	m_position = vp_pos.xyz;
	m_color = model_color;
}