// Triplanar setting that we can use
struct TriplanarSettings {
	vec2 scale;
	float offset;
}
// Get the blending offset to be used internally in the triplanar texture
vec3 get_blend(vec3 normal, float offset) {
	normal = abs(normal);
	vec3 weights = (max(normal + offset, 0));
	weights /= weights.x + weights.y + weights.z;
	return weights;
}
// Sample a triplanar texture (PS: This does not work on normal maps; Use triplanar_normal instead)
vec3 triplanar(sampler2D tex, vec3 position, vec3 normal, TriplanarSettings settings) {
	vec3 blending = get_blend(normalize(normal), settings.offset);

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(diffuse_tex, position.zy * settings.scale).xyz * blending.x;
	vec3 diffusey = texture(diffuse_tex, position.xz * settings.scale).xyz * blending.y;
	vec3 diffusez = texture(diffuse_tex, position.xy * settings.scale).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;
	return diffuse_final;
}
// Sample a triplanar normal map texture 
vec3 triplanar_normal(sampler2D normal_tex, vec3 position, vec3 normal, float strength, TriplanarSettings settings) {
	vec3 wnormal =  normalize(normal);
	vec3 blending = get_blend(wnormal, settings.offset);

	// Do the same for the normal map
	vec3 normalx = texture(normal_tex, position.zy * settings.scale).xyz * 2 - 1;
	vec3 normaly = texture(normal_tex, position.xz * settings.scale).xyz * 2 - 1;
	vec3 normalz = texture(normal_tex, position.xy * settings.scale).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, normalx.y) * strength + wnormal.zy, wnormal.x) * blending.x;
	normaly = vec3(vec2(normaly.x, normaly.y) * strength + wnormal.xz, wnormal.y) * blending.y;
	normalz = vec3(vec2(normalz.x, normalz.y) * strength + wnormal.xy, wnormal.z) * blending.z;
	vec3 normal_final = normalize(normalx.zyx + normaly.xzy + normalz.xyz);
	return normal_final;
}