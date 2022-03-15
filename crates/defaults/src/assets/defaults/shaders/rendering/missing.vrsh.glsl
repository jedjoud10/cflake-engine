#version 460 core
layout(location = 0) in vec3 mesh_pos;
layout(location = 1) in vec3 mesh_normal;
layout(location = 2) in vec4 mesh_tangent;
layout(location = 3) in vec2 mesh_uv;
layout(location = 4) in vec3 mesh_color;
uniform mat4 project_view_matrix;
uniform mat4 mesh_matrix;
out vec3 m_normal;
out vec3 m_position;

void main() {
	vec4 mesh_matrix_pos = (mesh_matrix * vec4(mesh_pos, 1.0));
	vec4 mvp_pos = project_view_matrix * mesh_matrix_pos;
	gl_Position = mvp_pos;

	// Pass the data to the next shader
	m_position = mesh_matrix_pos.xyz;
	m_normal = normalize((mesh_matrix * vec4(mesh_normal, 0.0)).xyz);
}