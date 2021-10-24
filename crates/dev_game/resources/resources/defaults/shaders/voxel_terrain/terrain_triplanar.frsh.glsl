#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
layout(location = 3) out vec3 frag_emissive;
uniform sampler2D diffuse_tex;
uniform sampler2D normals_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
uniform float depth;
uniform vec3 tint;
uniform float normals_strength;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec3 m_color;
in vec2 m_uv;
vec3 get_blend(vec3 normal) {
	const float offset = -0.3;
	normal = abs(normal);
	vec3 weights = (max(normal + offset, 0));
	weights /= weights.x + weights.y + weights.z;
	return weights;
}
void main() {
	vec3 world_normal = normalize(m_normal);
	vec3 blending = get_blend(world_normal);

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(diffuse_tex, m_position.zy * uv_scale).xyz * blending.x;
	vec3 diffusey = texture(diffuse_tex, m_position.xz * uv_scale).xyz * blending.y;
	vec3 diffusez = texture(diffuse_tex, m_position.xy * uv_scale).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;

	// Do the same for the normal map
	vec3 normalx = texture(normals_tex, m_position.zy * uv_scale).xyz * 2 - 1;
	vec3 normaly = texture(normals_tex, m_position.xz * uv_scale).xyz * 2 - 1;
	vec3 normalz = texture(normals_tex, m_position.xy * uv_scale).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, -normalx.y) * normals_strength + world_normal.zy, world_normal.x) * blending.x;
	normaly = vec3(vec2(normaly.x, -normaly.y) * normals_strength + world_normal.xz, world_normal.y) * blending.y;
	normalz = vec3(vec2(normalz.x, -normalz.y) * normals_strength + world_normal.xy, world_normal.z) * blending.z;
	vec3 normal_final = normalize(normalx.zyx + normaly.xzy + normalz.xyz);
	frag_diffuse = diffuse_final * m_color * tint;
	frag_normal = normal_final;
	frag_pos = m_position;
	frag_emissive = vec3(0, 0, 0);
}