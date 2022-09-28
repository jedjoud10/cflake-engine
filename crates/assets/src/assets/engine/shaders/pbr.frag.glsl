#version 460 core
#include "engine/shaders/models.func.glsl"
#include "engine/shaders/shadow.func.glsl"
#include "engine/shaders/clustered.func.glsl"
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
uniform vec3 camera;
uniform vec3 camera_forward;

// Uniforms set by the main scene
uniform vec3 sun_dir;
uniform vec3 sun_color;
uniform float sun_strength;

// Environment mapping
uniform samplerCube environment;

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
	vec3 mask;
	vec3 normal;
	vec3 position;
};

// Bidirectional reflectance distribution function, aka PBRRRR
vec3 brdf(SurfaceData surface, CameraData camera, LightData light, samplerCube enviro) {
	if (all(equal(light.color * light.strength, vec3(0.0)))) {
		return vec3(0.0);
	}

	float roughness = clamp(surface.mask.g, 0.06, 1.0);
	float metallic = clamp(surface.mask.b, 0.0, 1.0);
	float visibility = clamp(surface.mask.r, 0.0, 1.0);
	vec3 f0 = mix(vec3(0.04), surface.diffuse, metallic);
	
	// Ks and Kd
	vec3 ks = fresnel(f0, camera.view, camera.half_view, surface.normal);
	vec3 kd = (1 - ks) * (1 - metallic);

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
	vec3 brdf = kd * (surface.diffuse / PI) + specular(f0, roughness, camera.view, light.backward, surface.normal, camera.half_view);
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

	// Set the main PBR structs that we shall re-use
	SurfaceData surface = SurfaceData(diffuse, mask, normal, m_position);
	vec3 view = normalize(camera - m_position);

	// Main directional light
	vec3 ambient = 0.1 * diffuse * mask.r;
	vec3 sum = ambient;
	LightData sun = LightData(sun_dir, sun_color, sun_strength, true);
	CameraData _camera = CameraData(view, normalize(view + sun_dir), camera);	
	sum += brdf(surface, _camera, sun, environment);

	// Iterate through all the point light
	for(int i = 0; i < point_lights_num; i++) {
		vec3 light_position = lights[i].position_attenuation.xyz;
		vec3 dir = normalize(light_position - m_position);
        float dist = distance(light_position, m_position);
		float attenuation = calculate_attenuation(dist);
        vec3 radiance = lights[i].color.xyz * attenuation;

		LightData point_light = LightData(dir, radiance, 1.0, false);
		CameraData camera = CameraData(view, normalize(view + dir), camera);
		sum += brdf(surface, camera, point_light, environment);
	}

	// Color of the final result
	frag = sum;
}