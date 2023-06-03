#version 460 core

// G-Buffer data write
layout(location = 0) out vec4 gbuffer_position;
layout(location = 1) out vec4 gbuffer_albedo;
layout(location = 2) out vec4 gbuffer_normal;
layout(location = 3) out vec4 gbuffer_mask;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_local_position;
layout(location = 2) in vec3 m_normal;
layout(location = 3) in float lod;
layout(location = 4) in flat uint skirts; 

#ifdef submaterials
// Albedo / diffuse map texture array
layout(set = 0, binding = 8) uniform texture2DArray layered_albedo_map;
layout(set = 0, binding = 9) uniform sampler layered_albedo_map_sampler;

// Normal map texture array
layout(set = 0, binding = 10) uniform texture2DArray layered_normal_map;
layout(set = 0, binding = 11) uniform sampler layered_normal_map_sampler;

// Mask map texture array
layout(set = 0, binding = 12) uniform texture2DArray layered_mask_map;
layout(set = 0, binding = 13) uniform sampler layered_mask_map_sampler;

// Triplanar mapping offset and UV scale
const float offset = 0.0;
const vec2 scale = vec2(0.02) * vec2(-1, 1); 
const float normal_strength = 1.0;

// Get the blending offset to be used internally in the triplanar texture
vec3 get_blend(vec3 normal) {
	normal = abs(normal);
	vec3 weights = (max(normal + offset, 0));
	weights /= weights.x + weights.y + weights.z;
	return weights;
}

vec3 triplanar_albedo(float layer, vec3 normal) {
	vec3 blending = get_blend(normalize(normal));

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.zy * scale, layer)).xyz * blending.x;
	vec3 diffusey = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.xz * scale, layer)).xyz * blending.y;
	vec3 diffusez = texture(sampler2DArray(layered_albedo_map, layered_albedo_map_sampler), vec3(m_position.xy * scale, layer)).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;
	return diffuse_final;
}

vec3 triplanar_mask(float layer, vec3 normal) {
	vec3 blending = get_blend(normalize(normal));

	// Sample the diffuse texture three times to make the triplanar texture
	vec3 diffusex = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.zy * scale, layer)).xyz * blending.x;
	vec3 diffusey = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.xz * scale, layer)).xyz * blending.y;
	vec3 diffusez = texture(sampler2DArray(layered_mask_map, layered_mask_map_sampler), vec3(m_position.xy * scale, layer)).xyz * blending.z;
	vec3 diffuse_final = diffusex + diffusey + diffusez;
	return diffuse_final;
}

// https://bgolus.medium.com/normal-mapping-for-a-triplanar-shader-10bf39dca05a
vec3 triplanar_normal(float layer, vec3 normal) {
	vec3 wnormal =  normalize(normal);
	vec3 blending = get_blend(wnormal);

	// Do the same for the normal map
	vec3 normalx = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.zy * scale, layer)).xyz * 2 - 1;
	vec3 normaly = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.xz * scale, layer)).xyz * 2 - 1;
	vec3 normalz = texture(sampler2DArray(layered_normal_map, layered_normal_map_sampler), vec3(m_position.xy * scale, layer)).xyz * 2 - 1;
	normalx = vec3(vec2(normalx.x, normalx.y) * normal_strength + wnormal.zy, wnormal.x) * blending.x;
	normaly = vec3(vec2(normaly.x, normaly.y) * normal_strength + wnormal.xz, wnormal.y) * blending.y;
	normalz = vec3(vec2(normalz.x, normalz.y) * normal_strength + wnormal.xy, wnormal.z) * blending.z;
	vec3 normal_final = normalize(normalx.zyx + normaly.xzy + normalz.xyz);
	return normal_final;
}
#endif

void main() {
	#ifdef lowpoly
	vec3 surface_normal = normalize(cross(dFdy(m_position), dFdx(m_position)));
	#else
	vec3 surface_normal = normalize(m_normal);
	#endif

	// TODO: Splatmap shenanigans
	// We can handle up to 16 materials if we use 1 byte per channel
	// so 4 channels per f32, and 4 f32 per splatmap texture
	// there's probably a way to fit even *more* textures into there too
	#ifdef submaterials
	vec3 albedo1 = triplanar_albedo(float(0), surface_normal);
	vec3 mask1 = triplanar_mask(float(0), surface_normal);
	vec3 normal1 = triplanar_normal(float(0), surface_normal);
	vec3 albedo2 = triplanar_albedo(float(1), surface_normal);
	vec3 mask2 = triplanar_mask(float(1), surface_normal);
	vec3 normal2 = triplanar_normal(float(1), surface_normal);
	float blending_factor = 1 - clamp((surface_normal.y - 0.8) * 8, 0, 1);	
	vec3 albedo = mix(albedo1, albedo2, blending_factor);
	vec3 mask = mix(mask1, mask2, blending_factor);
	vec3 normal = mix(normal1, normal2, blending_factor);
	#else
	vec3 normal = surface_normal;
	vec3 rock = pow(vec3(128, 128, 128) / 255.0, vec3(2.2));
	vec3 dirt = pow(vec3(54, 30, 7) / 255.0, vec3(2.2));
	vec3 grass = pow(vec3(69, 107, 35) / 255.0, vec3(2.2));
	float blending_factor = 1 - clamp((surface_normal.y - 0.90) * 40, 0, 1);
	vec3 albedo = mix(grass, rock, blending_factor);
	vec3 mask = vec3(1.0, 0.9, 0.0);
	#endif

	gbuffer_position = vec4(m_position, 0);
	gbuffer_albedo = vec4(albedo, 1);
	mask *= vec3(pow(mask.r, 1.2), 3.0, 0.3);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(mask, 0);
	

	/*
	gbuffer_position = vec4(m_position, 0);
	gbuffer_albedo = vec4(float(skirts));
	gbuffer_normal = vec4(m_normal, 0);
	gbuffer_mask = vec4(0, 0.9, 0.0, 0);
	*/
}