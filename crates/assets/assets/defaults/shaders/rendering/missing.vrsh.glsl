#version 460 core
#load model
#load camera

// Mesh data given by the CPU
layout(location = 0) in vec3 mesh_pos;

void main() {
	// Only calculate the world position, since debug shader moment
	vec4 world = (_model_matrix * vec4(mesh_pos, 1.0));
	gl_Position = _pv_matrix * world;
}