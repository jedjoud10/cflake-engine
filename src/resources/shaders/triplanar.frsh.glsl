#version 460 core
layout(location = 0) out vec3 frag_diffuse;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_pos;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;
uniform vec2 uv_scale;
uniform vec3 view_pos;
in vec3 m_position;
in vec3 m_normal;
in vec4 m_tangent;
in vec2 m_uv;
in mat3 tbn;
void main() {
	vec3 world_normal = normalize(m_normal);
	const float sharpening = 5.0;

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(diffuse_tex, m_position.zy * uv_scale).xyz * pow(abs(world_normal.x), sharpening);
	vec3 diffusey = texture(diffuse_tex, m_position.xz * uv_scale).xyz * pow(abs(world_normal.y), sharpening);
	vec3 diffusez = texture(diffuse_tex, m_position.xy * uv_scale).xyz * pow(abs(world_normal.z), sharpening);
	vec3 diffuse_final = diffusex + diffusey + diffusez;

	// Do the same for the normal map
	vec3 normalx = texture(normal_tex, m_position.zy * uv_scale).xyz * 2 - 1;
	vec3 normaly = texture(normal_tex, m_position.xz * uv_scale).xyz * 2 - 1;
	vec3 normalz = texture(normal_tex, m_position.xy * uv_scale).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, -normalx.y) + world_normal.zy, world_normal.x) * pow(abs(world_normal.x), sharpening);
	normaly = vec3(vec2(normaly.x, -normaly.y) + world_normal.xz, world_normal.y) * pow(abs(world_normal.y), sharpening);
	normalz = vec3(vec2(normalz.x, -normalz.y) + world_normal.xy, world_normal.z) * pow(abs(world_normal.z), sharpening);

	vec3 normal_final = normalize(normalx.zyx + normaly.zxy + normalz.xyz);

	frag_diffuse = diffuse_final;
	frag_normal = m_normal;
	frag_pos = m_position;
}