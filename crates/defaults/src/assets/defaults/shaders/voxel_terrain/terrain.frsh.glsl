#version 460 core
#load renderer
#load general
#include "defaults/shaders/others/triplanar.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_emissive;
layout(location = 2) out vec3 frag_normal;
layout(location = 3) out vec3 frag_pos;
uniform sampler2DArray diffuse_m;
uniform sampler2DArray normal_m;
uniform vec2 uv_scale;
uniform float bumpiness;
in vec3 m_position;
in vec3 m_normal;
in vec3 m_color;
in flat vec2 m_uv;
void main() {
	// Triplanar settings
	TriplanarSettings settings = TriplanarSettings(uv_scale, 0.1);
	uint m_material_type = uint(m_uv.x * 255);
	frag_diffuse = array_triplanar(diffuse_m, m_position, m_normal, int(m_material_type), settings) * m_color;	
	frag_normal = array_triplanar_normal(normal_m, m_position, m_normal, int(m_material_type), bumpiness, settings);
	frag_pos = m_position;
	frag_emissive = vec3(0.0);
}