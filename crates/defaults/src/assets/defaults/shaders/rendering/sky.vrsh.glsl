#version 460 core
#load model
#load camera

// Mesh data given by the CPU
layout(location = 0) in vec3 mesh_pos;

// Data that will be given to the sky fragment shader
out vec3 m_position;

void main() {
	// Calculate world position first
	vec4 world = (_model_matrix * vec4(mesh_pos, 1.0));
	gl_Position = _pv_matrix * world;
    m_position = world.xyz;
}