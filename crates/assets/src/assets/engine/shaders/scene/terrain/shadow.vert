#version 460 core

// Vertex scaling factor. n / (n-3)
layout (constant_id = 0) const float scaling_factor = 1.0;

// Main attribute set vertex attributes
layout(location = 0) in vec4 packed;

// Contains position and scale value
layout(std430, set = 2, binding = 0) readonly buffer PositionScaleBuffer {
    vec4 data[];
} position_scale_buffer;

// Push constants for the current lightspace matrix
layout(push_constant) uniform PushConstants {
    mat4 lightspace;
} constants;

void main() {
    // Convert from 4 floats into uints 
    uint packed_cell_position = floatBitsToUint(packed.x);
    uint packed_inner_position = floatBitsToUint(packed.y);
    uint packed_normals = floatBitsToUint(packed.z);

    // Positions only need 16 bits (1 byte for cell coord, 1 byte for inner vertex coord)
    vec4 cell_position = unpackUnorm4x8(packed_cell_position) * 255;
    vec4 inner_position = unpackSnorm4x8(packed_inner_position);
    vec4 normals = unpackSnorm4x8(packed_normals);
    vec4 position = cell_position + inner_position;

	// Model space -> World space -> Clip space
    vec4 position_scale = position_scale_buffer.data[gl_DrawID];
    vec4 world_pos = vec4(((position.xyz * scaling_factor) * position_scale.w + position_scale.xyz) - normals.xyz * 0.5, 1);
    vec4 projected = constants.lightspace * (world_pos); 
    gl_Position = projected;
}