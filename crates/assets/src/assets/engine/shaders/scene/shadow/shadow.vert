#version 460 core
layout(location = 0) in vec3 position;

// Push constants for the mesh matrix and current lightspace matrix
layout(push_constant) uniform PushConstants {
    mat4 mesh;
    mat4 lightspace;
} constants;

void main() {
    vec4 world_pos = constants.mesh * vec4(position, 1);
    vec4 projected = constants.lightspace * world_pos; 
}