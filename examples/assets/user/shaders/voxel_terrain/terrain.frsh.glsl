#version 460 core
#load renderer
#load general
#include "defaults/shaders/others/triplanar.func.glsl"
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_emissive;
layout(location = 2) out vec3 frag_normal;
layout(location = 3) out vec3 frag_pos;
layout(location = 4) out vec3 frag_mask;
uniform sampler2D diffuse_m;
uniform sampler2D normal_m;
uniform sampler2D mask_m;
uniform vec2 uv_scale;
uniform float bumpiness;
in vec3 m_position;
in vec3 m_normal;
in flat vec2 m_uv;
void main() {
	// Triplanar settings
	TriplanarSettings settings = TriplanarSettings(uv_scale, 0.0);
	uint m_material_type = uint(m_uv.x * 255);
	frag_diffuse = triplanar(diffuse_m, m_position, m_normal, settings);	
	frag_normal = triplanar_normal(normal_m, m_position, m_normal, bumpiness, settings);
	frag_mask = triplanar(mask_m, m_position, m_normal, settings);
	frag_pos = m_position;
	frag_emissive = vec3(0.0);
}