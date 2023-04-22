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
    // Convert from 4 floats into uints
    uint packed_cell_position = floatBitsToUint(packed.x);
    uint packed_inner_position = floatBitsToUint(packed.y);

    // Positions only need 16 bits (1 byte for cell coord, 1 byte for inner vertex coord)
    vec4 cell_position = unpackUnorm4x8(packed_cell_position) * 255;
    vec4 inner_position = unpackSnorm4x8(packed_inner_position);
    vec4 position = cell_position + inner_position;

	// Model space -> World space -> Clip space
    vec4 world_pos = constants.mesh * vec4(position.xyz * scaling_factor, 1);
    vec4 projected = constants.lightspace * world_pos; 
    gl_Position = projected;
}