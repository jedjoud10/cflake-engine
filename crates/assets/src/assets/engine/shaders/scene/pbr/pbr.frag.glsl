#version 460 core
#include "engine/shaders/scene/pbr/models.func.glsl"
#include "engine/shaders/scene/shadow.func.glsl"
#include "engine/shaders/scene/clustered/clustered.func.glsl"
out vec3 frag;

// Main PBR uniforms
uniform float roughness;
uniform float bumpiness;
uniform float metallic;
uniform float ambient_occlusion;
uniform vec3 tint;
uniform sampler2D albedo;
uniform sampler2D normal;
uniform sampler2D mask;
uniform vec2 scale;

// Uniforms coming from the camera
uniform vec3 camera_position;
uniform vec3 camera_forward;

// Uniforms set by the main scene
uniform vec3 sun_dir;
uniform vec3 sun_color;
uniform float sun_strength;

// Environment mapping
uniform samplerCube irradiance_environment_map;
uniform samplerCube specular_environment_map;
uniform uint specular_environment_map_levels;

// BRDF Integration map (generated from https://github.com/HectorMF/BRDFGenerator)
uniform sampler2D brdf_integration_map;

// Directional shadow mapping
uniform sampler2DShadow shadow_map;
uniform mat4 shadow_lightspace_matrix;

// Clustered shading data
uniform uint cluster_size;
uniform uvec2 resolution;

// Point lights that are in the scene
uniform uint point_lights_num;
layout(std430) readonly buffer point_lights
{
    PackedPointLight lights[];
};

// Clustered shading clusters
// Clustered shading indices

// Data given by the vertex shader
in vec3 m_position;
in vec3 m_normal;
in vec3 m_tangent;
in vec3 m_bitangent;
in vec3 m_color;
in vec2 m_tex_coord;

// Light data struct
struct LightData {
	vec3 backward;
	vec3 color;
	float strength;
	bool directional;
};

// Camera data struct
struct CameraData {
	vec3 view;
	vec3 half_view;
	vec3 position;
};

// Surface data struct 
struct SurfaceData {
	vec3 diffuse;
	vec3 normal;
	vec3 position;
	float roughness;
	float metallic;
	float visibility;
	vec3 f0;
};

// Bidirectional reflectance distribution function, aka PBRRRR
vec3 brdf(SurfaceData surface, CameraData camera, LightData light) {
	if (all(equal(light.color * light.strength, vec3(0.0)))) {
		return vec3(0.0);
	}
	
	// Ks and Kd
	vec3 ks = fresnel(surface.f0, camera.half_view, camera.view);
	vec3 kd = (1 - ks) * (1 - surface.metallic);

	// TODO: Add point light shadow mapping
	// Check if the fragment is in shadow
	float shadow = 0.0;
	if (light.directional) {
		shadow = is_in_shadow(
			surface.position,
			light.backward,
			shadow_lightspace_matrix,
			shadow_map
		);
	}

	// Calculate diffuse and specular
	vec3 brdf = kd * (surface.diffuse / PI) + specular(surface.f0, surface.roughness, camera.view, light.backward, surface.normal, camera.half_view);
	brdf = brdf * light.color * light.strength * max(dot(light.backward, surface.normal), 0.0) * (1 - shadow);
	return brdf;
}

void main() {
	// Fetch the textures and their texels
    vec3 diffuse = texture(albedo, m_tex_coord * scale).xyz * tint;
	vec3 bumps = texture(normal, m_tex_coord * scale).xyz * 2.0 - 1.0;
	vec3 mask = (vec3(0.01) + texture(mask, m_tex_coord * scale).xyz) * vec3(1 / ambient_occlusion, roughness, metallic);

    // Calculate the normal mapped bumpiness
	bumps.xy *= bumpiness;

	// Calculate the world space normals
	mat3 tbn = mat3(
		normalize(m_tangent),
		normalize(m_bitangent),
		normalize(m_normal));
	vec3 normal = normalize(tbn * normalize(bumps));

	// Compute PBR values
	float roughness = clamp(mask.g, 0.02, 1.0);
	float metallic = clamp(mask.b, 0.01, 1.0);
	float visibility = clamp(mask.r, 0.0, 1.0);
	vec3 f0 = mix(vec3(0.04), diffuse, metallic);

	// Set the main PBR structs that we shall re-use
	SurfaceData surface = SurfaceData(diffuse, normal, m_position, roughness, metallic, visibility, f0);
	LightData sun = LightData(sun_dir, sun_color, sun_strength, true);
	vec3 view = normalize(camera_position - m_position);
	CameraData camera = CameraData(view, normalize(view + sun_dir), camera_position);	

	// Ambient diffuse lighting
	vec3 ks = fresnelRoughness(f0, surface.normal, camera.view, roughness);
	vec3 kd = (1 - ks) * (1 - metallic);
	vec3 irradiance = texture(irradiance_environment_map, surface.normal).rgb;

	/*
	frag = vec3(is_in_shadow(
		surface.position,
		sun.backward,
		shadow_lightspace_matrix,
		shadow_map
	));

	return;
	*/

	// Ambient specular lighting
	vec3 specular = textureLod(specular_environment_map, reflect(-camera.view, surface.normal), roughness * float(specular_environment_map_levels)).rgb; 
	vec2 integrated = texture(brdf_integration_map, vec2(max(dot(surface.normal, camera.view), 0.0), roughness)).rg;
	specular *= fresnelRoughness(f0, camera.view, surface.normal, roughness) * integrated.x + integrated.y;

	// Ambient color
	vec3 sum = (kd * irradiance * surface.diffuse + specular) * visibility;

	// Main directional light
	sum += brdf(surface, camera, sun);

	// Iterate through all the point light
	for(int i = 0; i < point_lights_num; i++) {
		vec3 light_position = lights[i].position_attenuation.xyz;
		vec3 dir = normalize(light_position - m_position);
        float dist = distance(light_position, m_position);
		float attenuation = calculate_attenuation(dist);
        vec3 radiance = lights[i].color.xyz * attenuation;

		LightData point_light = LightData(dir, radiance, 1.0, false);
		CameraData camera = CameraData(view, normalize(view + dir), camera_position);
		sum += brdf(surface, camera, point_light);
	}

	// Color of the final result
	frag = sum;
}