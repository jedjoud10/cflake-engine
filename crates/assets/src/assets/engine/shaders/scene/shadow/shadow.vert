#version 460 core
layout(location = 0) in vec3 position;

#include <engine/shaders/common/shadow.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

void main() {
    vec4 world_pos = mesh.matrix * vec4(position, 1);
    vec4 projected = shadow.lightspace * world_pos; 
	gl_Position = projected;
}