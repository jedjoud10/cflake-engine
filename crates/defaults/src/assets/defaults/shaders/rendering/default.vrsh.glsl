#version 460 core
#load model
#load camera

// Mesh data given by the CPU
layout(location = 0) in vec3 mesh_pos;
layout(location = 1) in vec3 mesh_normal;
layout(location = 2) in vec4 mesh_tangent;
layout(location = 3) in vec2 mesh_uv;
layout(location = 4) in vec3 mesh_color;

// Data that will be passed to the next shader (the preivous load already defines the renderer matrices)
out vec3 m_normal;
out vec3 m_tangent;
out vec3 m_bitangent;
out vec2 m_uv;
out vec3 m_position;
out vec3 m_color;

void main() {
	// Calculate world position first
	vec4 world = (_model_matrix * vec4(mesh_pos, 1.0));
	gl_Position = _pv_matrix * world;

	// Calculate world normal
	vec3 normal = (_model_matrix * vec4(mesh_normal, 0.0)).xyz;

	// Pass the data to the next shader
	m_position = world.xyz;
	m_normal = normal;
	m_uv = mesh_uv;
	m_color = mesh_color;

	// Calculate the world tangent
	m_tangent = (_model_matrix * vec4(mesh_tangent.xyz, 0.0)).xyz;
	float _sign = mesh_tangent.w;
	vec3 bitangent = cross(normalize(m_normal), normalize(mesh_tangent.xyz)) * _sign;
	m_bitangent = normalize((_model_matrix * vec4(bitangent, 0.0)).xyz);
}