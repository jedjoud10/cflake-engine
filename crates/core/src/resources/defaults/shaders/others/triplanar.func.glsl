vec3 get_blend(vec3 normal, float offset) {
	normal = abs(normal);
	vec3 weights = (max(normal + offset, 0));
	weights /= weights.x + weights.y + weights.z;
	return weights;
}
void triplanar(sampler2D diffuse_tex, sampler2D normals_tex, vec2 uv_scale, vec3 position, vec3 normal, float normals_strength, float blending_offset, out vec3 frag_diffuse, out vec3 frag_normal) {
	vec3 world_normal = normalize(normal);
	vec3 blending = get_blend(world_normal, blending_offset);

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(diffuse_tex, position.zy * uv_scale).xyz * blending.x;
	vec3 diffusey = texture(diffuse_tex, position.xz * uv_scale).xyz * blending.y;
	vec3 diffusez = texture(diffuse_tex, position.xy * uv_scale).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;

	// Do the same for the normal map
	vec3 normalx = texture(normals_tex, position.zy * uv_scale).xyz * 2 - 1;
	vec3 normaly = texture(normals_tex, position.xz * uv_scale).xyz * 2 - 1;
	vec3 normalz = texture(normals_tex, position.xy * uv_scale).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, normalx.y) * normals_strength + world_normal.zy, world_normal.x) * blending.x;
	normaly = vec3(vec2(normaly.x, normaly.y) * normals_strength + world_normal.xz, world_normal.y) * blending.y;
	normalz = vec3(vec2(normalz.x, normalz.y) * normals_strength + world_normal.xy, world_normal.z) * blending.z;
	vec3 normal_final = normalize(normalx.zyx + normaly.xzy + normalz.xyz);
	frag_diffuse = diffuse_final;
	frag_normal = normal_final;
}