#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in flat vec3 m_color;

// Camera, scene, and shadowmap shared objects
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/common/scene.glsl>
#include <engine/shaders/common/extensions.glsl>
#include <engine/shaders/common/shadow.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/fbm.glsl>
#include <engine/shaders/common/sky.glsl>
#include <engine/shaders/math/models.glsl>
#include <engine/shaders/math/dither.glsl>

// Push constants for the material data
layout(push_constant) uniform PushConstants {
	layout(offset = 64) float bumpiness;
    layout(offset = 68) float metallic;
    layout(offset = 72) float ambient_occlusion;
    layout(offset = 76) float roughness;
	layout(offset = 80) float fade;
} material;

void main() {
	// We do a bit of fading
	if (dither(ivec2(gl_FragCoord.xy), pow(material.fade, 4))) {
		//discard;
	}

	// Fetch the albedo color, normal map value, and mask values
    vec3 mask = vec3(1 / material.ambient_occlusion, material.roughness, material.metallic);

	// Assume world space normals
	//vec3 normal = normalize(m_normal);
	vec3 normal = normalize(cross(dFdy(m_position), dFdx(m_position)));
	vec3 albedo = vec3(1);
	vec3 rock = vec3(128, 128, 128) / 255.0;
	vec3 dirt = vec3(54, 30, 7) / 255.0;
	vec3 grass = vec3(69, 107, 35) / 255.0;
	albedo = m_color;
	/*
	albedo = grass;

	if (normal.y < 0.85) {
		albedo = rock;
	}
	*/

	// Compute PBR values
	float roughness = clamp(mask.g, 0.02, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	float visibility = clamp(mask.r, 0.0, 1.0);

	vec3 f0 = mix(vec3(0.04), albedo, metallic);

	// Create the data structs
	SunData sun = SunData(scene.sun_direction.xyz, scene.sun_color.rgb, 2.6);
	SurfaceData surface = SurfaceData(albedo, -normal, m_position, gl_FragCoord.z, roughness, metallic, visibility, f0);
	vec3 view = normalize(-camera.position.xyz + m_position);
	CameraData camera = CameraData(view, normalize(view + scene.sun_direction.xyz), camera.position.xyz, camera.view, camera.projection);

	// Check if the fragment is shadowed
	vec3 color = brdf(surface, camera, sun);

	// Calculate diffuse lighting
	frag = vec4(color, 0.0);
}