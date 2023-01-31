#version 460 core
#include <engine/shaders/common/camera.glsl>

// Vertex attributes from Rust
layout (location = 0) in vec3 vertex_position;
//layout (location = 1) in vec3 vertex_normal;

/*
// This material stores indices to actual textures and other params
struct BasicMaterial {
	int diffuse_map;
	int normal_map;
	float roughness;
	vec4 tint;
};

// Mesh constants only contain indices and per-draw data
layout(push_constant) uniform MeshConstants {
	// Per-draw data
	vec4 model_matrix;

	// Index to use to fetch material buffer
	int material_index;
} constants;
*/

// https://vkguide.dev/docs/chapter-2/triangle_walkthrough/
void main() {
	gl_Position = vec4(vertex_position.xy, 0.0f, 1.0f);

}