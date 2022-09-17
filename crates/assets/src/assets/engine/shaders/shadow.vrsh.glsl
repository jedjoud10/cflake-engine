#version 460 core
layout(location = 0) in vec3 position;
uniform mat4 proj_matrix;
uniform mat4 view_matrix;
uniform mat4 world_matrix;


void main() {
    // Model space -> World space -> Clip space
    vec4 world_pos = world_matrix * vec4(position, 1);
    vec4 projected = (proj_matrix * view_matrix) * world_pos; 
	gl_Position = projected;
}