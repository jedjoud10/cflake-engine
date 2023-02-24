#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec3 position;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;

void main() {
	// Model space -> World space -> Clip space
    m_position = position;
    vec4 world_pos = vec4(position * 100.0 + camera.position.xyz, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;    
}