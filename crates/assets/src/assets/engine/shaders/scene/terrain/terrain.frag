#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in flat vec3 m_color;

// Camera, scene, and shadowmap shared objects
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/common/scene.glsl>
#include <engine/shaders/common/shadow.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/fbm.glsl>
#include <engine/shaders/common/sky.glsl>
#include <engine/shaders/math/models.glsl>
#include <engine/shaders/math/dither.glsl>
#include <engine/shaders/math/triplanar.glsl>

// Push constants for the material data
layout(push_constant) uniform PushConstants {
	layout(offset = 64) float fade;
} material;

// Albedo / diffuse map texture array
layout(set = 1, binding = 0) uniform texture2DArray layered_albedo_map;
layout(set = 1, binding = 1) uniform sampler layered_albedo_map_sampler;

// Normal map texture array
layout(set = 1, binding = 2) uniform texture2DArray layered_normal_map;
layout(set = 1, binding = 3) uniform sampler layered_normal_map_sampler;

// Mask map texture array
layout(set = 1, binding = 4) uniform texture2DArray layered_mask_map;
layout(set = 1, binding = 5) uniform sampler layered_mask_map_sampler;

// Triplanar mapping offset and UV scale
const float offset = 0.0;
const vec2 scale = vec2(0.1) * vec2(-1, 1); 
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

void main() {
	// We do a bit of fading V2
	/*
	if ((1-cellular(floor(m_position) * 0.01).y) > (material.fade-1)) {
		discard;
	}
	*/

	// We do a bit of fading
	/*
	float fade = min(material.fade, 1);
	if (dither(ivec2(gl_FragCoord.xy), fade)) {
		discard;
	}
	*/

	// Get normals either by derivating them or getting them smoothed
	#ifdef lowpoly
	vec3 surface_normal = normalize(cross(dFdy(m_position), dFdx(m_position)));
	#else
	vec3 surface_normal = normalize(m_normal);
	#endif

	// TODO: Splatmap shenanigans
	// We can handle up to 16 materials if we use 1 byte per channel
	// so 4 channels per f32, and 4 f32 per splatmap texture
	// there's probably a way to fit even *more* textures into there too

	vec3 albedo1 = triplanar_albedo(float(0), surface_normal);
	vec3 mask1 = triplanar_mask(float(0), surface_normal);
	vec3 normal1 = triplanar_normal(float(0), surface_normal);

	vec3 albedo2 = triplanar_albedo(float(1), surface_normal);
	vec3 mask2 = triplanar_mask(float(1), surface_normal);
	vec3 normal2 = triplanar_normal(float(1), surface_normal);

	float blending_factor = 1 - clamp((surface_normal.y - 0.7) * 6, 0, 1);

	vec3 albedo = mix(albedo1, albedo2, blending_factor);
	vec3 mask = mix(mask1, mask2, blending_factor);
	vec3 normal = mix(normal1, normal2, blending_factor);

	// Compute PBR values
	mask *= vec3(pow(mask.r, 2), 1.3, 0.4);
	float roughness = clamp(mask.g, 0.02, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	float visibility = clamp(mask.r, 0.0, 1.0);
	vec3 f0 = mix(vec3(0.04), albedo, metallic);

	// Create the data structs
	SunData sun = SunData(-scene.sun_direction.xyz, scene.sun_color.rgb);
	SurfaceData surface = SurfaceData(albedo, normalize(normal), surface_normal, m_position, roughness, metallic, visibility, f0);
	vec3 view = normalize(camera.position.xyz - m_position);
	CameraData camera = CameraData(view, normalize(view - scene.sun_direction.xyz), camera.position.xyz, camera.view, camera.projection);

	// Check if the fragment is shadowed
	vec3 color = brdf(surface, camera, sun);

	// Calculate diffuse lighting
	frag = vec4(color, 0.0);
}