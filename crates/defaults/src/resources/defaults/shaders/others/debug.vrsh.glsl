#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
out vec3 m_position;
out vec2 screen_position;
out vec3 m_normal;
void main() {
	vec4 mvp_pos = mvp_matrix * vec4(model_pos, 1.0);
	vec3 model_matrix_pos = (model_matrix * vec4(model_pos, 1.0)).xyz;
	gl_Position = mvp_pos;
	m_normal = model_normal;
	screen_position = mvp_pos.xy;
	// Pass the data to the next shader
	m_position = model_matrix_pos;
}