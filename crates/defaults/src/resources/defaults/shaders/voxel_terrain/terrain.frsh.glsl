#version 460 core
#load renderer
#load general
#include "defaults\shaders\voxel_terrain\terrain_shader.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
uniform sampler2DArray diffuse_textures;
uniform sampler2DArray normals_textures;
uniform vec2 uv_scale;
uniform vec3 view_pos;
uniform vec3 tint;
uniform int node_depth;
uniform int max_depth;
uniform float normals_strength;
uniform int material_id;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec3 m_color;
in vec2 m_uv;
void main() {
	#load renderer_main_start
	#load renderer_life_fade
	vec3 normal;
	vec3 diffuse;
	get_frag(material_id, diffuse_textures, normals_textures, m_position, m_normal, uv_scale, normals_strength, diffuse, normal);
	frag_diffuse = diffuse * m_color * tint * (float(node_depth) / float(max_depth));
	/*
	if (any(lessThan(mod(m_position, 32), vec3(1)))) {
		frag_diffuse *= 0.0;
	} 
	*/
	frag_normal = normal;
	frag_pos = m_position;
}