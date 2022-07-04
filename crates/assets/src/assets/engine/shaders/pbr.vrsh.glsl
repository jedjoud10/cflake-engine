#version 460 core
layout(location = 0) in vec3 position;
uniform mat4 _view_matrix;
uniform mat4 _proj_matrix;
uniform mat4 _world_matrix;
out vec3 test;
void main()
{
    vec4 world_pos = _world_matrix * vec4(position, 1);
    vec4 projected = (_proj_matrix * _view_matrix) * world_pos; 
    gl_Position = projected;
    test = world_pos.xyz;
}