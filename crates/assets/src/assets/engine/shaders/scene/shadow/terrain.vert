#version 460 core
layout(location = 0) in vec3 packed;

// Vertex scaling factor. n / (n-3)
layout (constant_id = 0) const float scaling_factor = 1.0;

// Push constants for the mesh matrix and current lightspace matrix
layout(push_constant) uniform PushConstants {
    mat4 mesh;
    mat4 lightspace;
} constants;

void main() {
    // TODO: DONT READ FROM POSITION BUFFER THAT SHIT EMPTY (we moved to UVs remember)


	// Model space -> World space -> Clip space
    vec4 world_pos = constants.mesh * vec4(vec3(0) * scaling_factor, 1);
    vec4 projected = constants.lightspace * world_pos; 
    gl_Position = projected;
}